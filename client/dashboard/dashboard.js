const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${wsProtocol}//${window.location.host}/ws`;

let qr = null;
let ws = null;

function generateQR(url) {
    document.getElementById('url-display').innerText = url;
    if (qr) {
        qr.clear();
        qr.makeCode(url);
    } else {
        qr = new QRCode(document.getElementById("qrcode"), {
            text: url,
            width: 200,
            height: 200,
            colorDark: "#000000",
            colorLight: "#ffffff",
            correctLevel: QRCode.CorrectLevel.H
        });
    }
}

async function fetchIPs() {
    try {
        const res = await fetch('/api/ip');
        const ips = await res.json();
        const select = document.getElementById('ip-select');

        if (ips.length === 0) {
            const defaultUrl = `${window.location.protocol}//${window.location.host}/`;
            const option = document.createElement('option');
            option.text = "Localhost";
            option.value = defaultUrl;
            select.appendChild(option);
            generateQR(defaultUrl);
            return;
        }

        ips.forEach(ipStr => {
            const parts = ipStr.split(': ');
            if (parts.length === 2) {
                const iface = parts[0];
                const ip = parts[1];
                const url = `http://${ip}:${window.location.port || 8000}/`;
                const option = document.createElement('option');
                option.text = `${iface} (${ip})`;
                option.value = url;
                select.appendChild(option);
            }
        });

        generateQR(select.value);

        select.addEventListener('change', (e) => {
            generateQR(e.target.value);
        });
    } catch (e) {
        console.error("Failed to fetch IPs", e);
        generateQR(`${window.location.protocol}//${window.location.host}/`);
    }
}

fetchIPs();

// Escape user-controlled strings before inserting into HTML to prevent XSS
function escapeHtml(str) {
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}

function connectWebSocket() {
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        // Always re-register as dashboard on every fresh connection
        ws.send(JSON.stringify([{ t: "dashboard_reg" }]));
    };

    ws.onmessage = (event) => {
        try {
            const msg = JSON.parse(event.data);
            if (msg.t === "state") {
                updateDevices(msg.devices);
            }
        } catch (e) {
            console.error("Dashboard parse error:", e);
        }
    };

    ws.onclose = () => {
        // Re-attach full new WebSocket with all handlers
        setTimeout(connectWebSocket, 1500);
    };

    ws.onerror = (err) => {
        console.error("Dashboard WebSocket error:", err);
        // onclose will fire automatically after onerror
    };
}

connectWebSocket();

function approveDevice(id, permanent) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(JSON.stringify([{ t: "approve", id: id, permanent: !!permanent }]));
}

function rejectDevice(id) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(JSON.stringify([{ t: "reject", id: id }]));
}

function updateDevices(devices) {
    const container = document.getElementById('devices-container');
    container.innerHTML = '';

    document.getElementById('device-count').innerText = devices.length;

    if (devices.length === 0) {
        container.innerHTML = '<p style="color: gray;">No devices connected.</p>';
        return;
    }

    devices.forEach(dev => {
        // Sanitize device ID for safe display and safe use as onclick argument
        const safeId = escapeHtml(dev.id);
        const playerLabel = dev.player ? ` — Player ${dev.player}` : '';

        const div = document.createElement('div');
        div.className = 'device-item';

        const statusBadge = dev.approved
            ? `<span class="badge connected">Active Controller${playerLabel}</span>`
            : '<span class="badge">Pending Approval</span>';

        // Build action buttons safely using DOM methods, not innerHTML for interactive parts
        div.innerHTML = `
            <div>
                <strong>Device: ${safeId}</strong>
                ${statusBadge}
            </div>
            <div class="device-actions"></div>
        `;

        const actionsEl = div.querySelector('.device-actions');
        if (dev.approved) {
            const disconnectBtn = document.createElement('button');
            disconnectBtn.className = 'btn btn-reject';
            disconnectBtn.textContent = 'Disconnect';
            disconnectBtn.addEventListener('click', () => rejectDevice(dev.id));
            actionsEl.appendChild(disconnectBtn);
        } else {
            const approveOnceBtn = document.createElement('button');
            approveOnceBtn.className = 'btn btn-approve';
            approveOnceBtn.textContent = 'Approve Once';
            approveOnceBtn.addEventListener('click', () => approveDevice(dev.id, false));

            const trustBtn = document.createElement('button');
            trustBtn.className = 'btn btn-approve';
            trustBtn.textContent = 'Trust Permanently';
            trustBtn.style.cssText = 'background: #2196F3; margin-left: 10px;';
            trustBtn.addEventListener('click', () => approveDevice(dev.id, true));

            const denyBtn = document.createElement('button');
            denyBtn.className = 'btn btn-reject';
            denyBtn.textContent = 'Deny';
            denyBtn.style.marginLeft = '10px';
            denyBtn.addEventListener('click', () => rejectDevice(dev.id));

            actionsEl.appendChild(approveOnceBtn);
            actionsEl.appendChild(trustBtn);
            actionsEl.appendChild(denyBtn);
        }

        container.appendChild(div);
    });
}
