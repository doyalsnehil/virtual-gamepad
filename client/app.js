const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${wsProtocol}//${window.location.host}/ws`;
let ws = null;
let isConnected = false;
let isApproved = false;
let isRejected = false;

// Generate or retrieve a random device ID (alphanumeric + underscore only)
let deviceId = localStorage.getItem('device-id');
if (!deviceId) {
    deviceId = 'phone_' + Math.random().toString(36).substring(2, 9);
    localStorage.setItem('device-id', deviceId);
}

let editMode = false;

// UI Elements
const overlay = document.getElementById('overlay');
const statusIndicator = document.getElementById('status');
const editBtn = document.getElementById('edit-btn');
const saveBtn = document.getElementById('save-btn');
const resetBtn = document.getElementById('reset-btn');
const controlItems = document.querySelectorAll('.control-item');

// ─── Binary Protocol ────────────────────────────────────────────────────────────
// Maps strings to 1-byte IDs for the Rust server
const BTN_MAP = { 'A':0, 'B':1, 'X':2, 'Y':3, 'LB':4, 'RB':5, 'START':6, 'SELECT':7, 'LS':8, 'RS':9, 'MODE':10 };
const AXIS_MAP = { 'LX':0, 'LY':1, 'RX':2, 'RY':3, 'LT_ABS':4, 'RT_ABS':5, 'DX':6, 'DY':7 };

// Pre-allocate a single buffer and reuse it for every send (zero GC pressure)
const _sendBuf = new ArrayBuffer(4);
const _sendView = new DataView(_sendBuf);

function sendBinary(type, codeString, value) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    
    const typeByte = type === 'b' ? 0 : 1;
    const codeByte = type === 'b' ? BTN_MAP[codeString] : AXIS_MAP[codeString];
    
    if (codeByte === undefined) return;

    _sendView.setUint8(0, typeByte);
    _sendView.setUint8(1, codeByte);
    _sendView.setInt16(2, value, true); // Little endian

    try {
        ws.send(_sendBuf);
    } catch (e) {
        console.error('[gamepad] send error:', e);
    }
}

// ─── WebSocket connection ─────────────────────────────────────────────────────
function connect() {
    ws = new WebSocket(wsUrl);
    ws.binaryType = 'arraybuffer';

    ws.onopen = () => {
        isConnected = true;
        document.getElementById('overlay-title').innerText = "Waiting for Approval";
        document.getElementById('overlay-desc').innerText = "Please approve this device on the laptop dashboard.";
        ws.send(JSON.stringify([{ t: "req_conn", id: deviceId }]));
    };

    ws.onclose = () => {
        isConnected = false;
        isApproved = false;
        if (isRejected) return;
        document.getElementById('overlay-title').innerText = "Connection Lost";
        document.getElementById('overlay-desc').innerText = "Reconnecting...";
        overlay.classList.remove('hidden');
        statusIndicator.classList.remove('connected');
        setTimeout(connect, 1500);
    };

    ws.onmessage = (event) => {
        try {
            const msgs = JSON.parse(event.data);
            for (const msg of msgs) {
                if (msg.t === "approved") {
                    isApproved = true;
                    overlay.classList.add('hidden');
                    statusIndicator.classList.add('connected');
                } else if (msg.t === "rejected") {
                    isRejected = true;
                    document.getElementById('overlay-title').innerText = "Connection Rejected";
                    document.getElementById('overlay-desc').innerText = "You were denied access. Restart the app to try again.";
                    document.getElementById('overlay-loader').style.display = 'none';
                    overlay.classList.remove('hidden');
                    ws.close();
                } else if (msg.t === "player") {
                    for (let i = 1; i <= 4; i++) {
                        const led = document.getElementById(`led-${i}`);
                        if (led) {
                            if (i === msg.num) led.classList.add('active');
                            else led.classList.remove('active');
                        }
                    }
                } else if (msg.t === "rumble") {
                    if (navigator.vibrate) {
                        navigator.vibrate(msg.v || 200);
                    }
                }
            }
        } catch (e) {
            console.error('[gamepad] message parse error:', e);
        }
    };

    ws.onerror = (err) => {
        console.error("[gamepad] WebSocket error", err);
        // onclose will fire after onerror
    };
}
connect();

// ─── Fullscreen & Orientation Lock ────────────────────────────────────────────
const fsBtn = document.getElementById('fs-btn');
const fsOverlay = document.getElementById('fs-overlay');

fsBtn.addEventListener('click', async () => {
    try {
        if (document.documentElement.requestFullscreen) {
            await document.documentElement.requestFullscreen();
        } else if (document.documentElement.webkitRequestFullscreen) {
            await document.documentElement.webkitRequestFullscreen();
        }
        if (screen.orientation && screen.orientation.lock) {
            await screen.orientation.lock('landscape').catch(() => {});
        }
    } catch (e) {
        console.warn("Fullscreen/Orientation failed", e);
    }
    fsOverlay.style.display = 'none';
});

// ─── Joysticks (NippleJS) ─────────────────────────────────────────────────────
let lsManager, rsManager;

function initJoysticks() {
    if (lsManager) lsManager.destroy();
    if (rsManager) rsManager.destroy();

    lsManager = nipplejs.create({
        zone: document.getElementById('ls-zone'),
        mode: 'static',
        position: { left: '50%', top: '50%' },
        color: 'white',
        size: 100
    });

    rsManager = nipplejs.create({
        zone: document.getElementById('rs-zone'),
        mode: 'static',
        position: { left: '50%', top: '50%' },
        color: 'white',
        size: 100
    });

    // Left Stick
    lsManager.on('move', (evt, data) => {
        if (editMode) return;
        const radian = data.angle ? data.angle.radian : 0;
        const distance = data.distance || 0;
        const x = Math.round(Math.cos(radian) * distance * (32767 / 50));
        const y = Math.round(Math.sin(radian) * distance * (32767 / 50) * -1);
        sendBinary('a', 'LX', Math.max(-32768, Math.min(32767, x)));
        sendBinary('a', 'LY', Math.max(-32768, Math.min(32767, y)));
    });
    lsManager.on('end', () => {
        if (editMode) return;
        sendBinary('a', 'LX', 0);
        sendBinary('a', 'LY', 0);
    });

    // Right Stick
    rsManager.on('move', (evt, data) => {
        if (editMode) return;
        const radian = data.angle ? data.angle.radian : 0;
        const distance = data.distance || 0;
        const x = Math.round(Math.cos(radian) * distance * (32767 / 50));
        const y = Math.round(Math.sin(radian) * distance * (32767 / 50) * -1);
        sendBinary('a', 'RX', Math.max(-32768, Math.min(32767, x)));
        sendBinary('a', 'RY', Math.max(-32768, Math.min(32767, y)));
    });
    rsManager.on('end', () => {
        if (editMode) return;
        sendBinary('a', 'RX', 0);
        sendBinary('a', 'RY', 0);
    });
}

// ─── Buttons ──────────────────────────────────────────────────────────────────
const allButtons = document.querySelectorAll('button[data-key]');

allButtons.forEach(btn => {
    const onPress = (e) => {
        if (editMode) return;
        e.preventDefault();
        btn.classList.add('active');
        const key = btn.getAttribute('data-key');
        
        // True analog axes mapping for triggers and d-pad
        if (key === 'LT') { sendBinary('a', 'LT_ABS', 255); return; }
        if (key === 'RT') { sendBinary('a', 'RT_ABS', 255); return; }
        if (key === 'UP') { sendBinary('a', 'DY', -32768); return; }
        if (key === 'DOWN') { sendBinary('a', 'DY', 32767); return; }
        if (key === 'LEFT') { sendBinary('a', 'DX', -32768); return; }
        if (key === 'RIGHT') { sendBinary('a', 'DX', 32767); return; }

        sendBinary('b', key, 1);
    };

    const onRelease = (e) => {
        if (editMode) return;
        e.preventDefault();
        btn.classList.remove('active');
        const key = btn.getAttribute('data-key');
        
        if (key === 'LT') { sendBinary('a', 'LT_ABS', 0); return; }
        if (key === 'RT') { sendBinary('a', 'RT_ABS', 0); return; }
        if (key === 'UP' || key === 'DOWN') { sendBinary('a', 'DY', 0); return; }
        if (key === 'LEFT' || key === 'RIGHT') { sendBinary('a', 'DX', 0); return; }

        sendBinary('b', key, 0);
    };

    btn.addEventListener('touchstart', onPress, { passive: false });
    btn.addEventListener('touchend', onRelease, { passive: false });
    btn.addEventListener('touchcancel', onRelease, { passive: false });
    
    // Mouse support for desktop testing
    btn.addEventListener('mousedown', (e) => { e.preventDefault(); onPress(e); });
    btn.addEventListener('mouseup', (e) => { e.preventDefault(); onRelease(e); });
    btn.addEventListener('mouseleave', (e) => { if (btn.classList.contains('active')) onRelease(e); });
});

// ─── Edit Mode ────────────────────────────────────────────────────────────────
editBtn.addEventListener('click', () => {
    editMode = true;
    document.body.classList.add('edit-mode');
    editBtn.classList.add('hidden');
    document.getElementById('edit-toolbar').classList.remove('hidden');
    // Destroy joysticks so their touch zones don't interfere with dragging
    if (lsManager) { lsManager.destroy(); lsManager = null; }
    if (rsManager) { rsManager.destroy(); rsManager = null; }
});

saveBtn.addEventListener('click', () => {
    editMode = false;
    document.body.classList.remove('edit-mode');
    document.getElementById('edit-toolbar').classList.add('hidden');
    editBtn.classList.remove('hidden');
    document.querySelectorAll('.control-item').forEach(el => el.classList.remove('selected-drag'));

    const layout = {};
    controlItems.forEach(item => {
        layout[item.id] = {
            top: item.style.top,
            left: item.style.left,
            right: item.style.right,
            bottom: item.style.bottom,
            scale: item.dataset.scale || 1
        };
    });
    localStorage.setItem('gamepad-layout', JSON.stringify(layout));
    // Re-init joysticks after layout is saved
    initJoysticks();
});

resetBtn.addEventListener('click', () => {
    localStorage.removeItem('gamepad-layout');
    location.reload();
});

// Load saved layout
function loadLayout() {
    const saved = localStorage.getItem('gamepad-layout');
    if (saved) {
        try {
            const layout = JSON.parse(saved);
            controlItems.forEach(item => {
                if (layout[item.id]) {
                    if (layout[item.id].top) item.style.top = layout[item.id].top;
                    if (layout[item.id].left) item.style.left = layout[item.id].left;
                    if (layout[item.id].right) item.style.right = layout[item.id].right;
                    if (layout[item.id].bottom) item.style.bottom = layout[item.id].bottom;
                    if (layout[item.id].scale) {
                        item.dataset.scale = layout[item.id].scale;
                        item.style.setProperty('--scale', layout[item.id].scale);
                    }
                }
            });
        } catch (e) {
            console.error('[gamepad] layout parse error:', e);
        }
    }
}
loadLayout();
// Wait for the browser to finish painting the loaded layout positions
// before initializing NippleJS, so it calculates correct touch zones.
requestAnimationFrame(() => {
    requestAnimationFrame(() => {
        initJoysticks();
    });
});

// ─── Drag & Resize ─────────────────────────────────────────────────────────────
let activeDrag = null;
let isDragging = false;
let dragOffset = { x: 0, y: 0 };
const sizeSlider = document.getElementById('size-slider');

sizeSlider.addEventListener('input', (e) => {
    if (!activeDrag) return;
    activeDrag.dataset.scale = e.target.value;
    activeDrag.style.setProperty('--scale', e.target.value);
});

document.addEventListener('touchstart', (e) => {
    if (!editMode) return;
    
    // Allow interacting with the toolbar without triggering drag/deselect
    if (e.target.closest('#edit-toolbar')) return;

    const target = e.target.closest('.control-item');
    if (target) {
        activeDrag = target;
        document.querySelectorAll('.control-item').forEach(el => el.classList.remove('selected-drag'));
        target.classList.add('selected-drag');
        sizeSlider.value = target.dataset.scale || 1;
        // Use getComputedStyle to ignore CSS transforms, preventing jump on drag start
        const computed = window.getComputedStyle(target);
        const leftPx = parseFloat(computed.left) || 0;
        const topPx = parseFloat(computed.top) || 0;
        
        dragOffset.x = e.touches[0].clientX - leftPx;
        dragOffset.y = e.touches[0].clientY - topPx;
        
        target.style.left = leftPx + 'px';
        target.style.top = topPx + 'px';
        target.style.right = 'auto';
        target.style.bottom = 'auto';
        isDragging = true;
    } else {
        activeDrag = null;
        isDragging = false;
        document.querySelectorAll('.control-item').forEach(el => el.classList.remove('selected-drag'));
    }
}, { passive: false });

document.addEventListener('touchmove', (e) => {
    if (!editMode || !activeDrag || !isDragging) return;
    e.preventDefault();
    activeDrag.style.left = (e.touches[0].clientX - dragOffset.x) + 'px';
    activeDrag.style.top = (e.touches[0].clientY - dragOffset.y) + 'px';
}, { passive: false });

document.addEventListener('touchend', () => {
    isDragging = false;
});
