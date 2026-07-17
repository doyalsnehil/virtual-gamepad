use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    extract::ConnectInfo,
    routing::get,
    Router,
};
use evdev::{
    AttributeSet, KeyCode, AbsoluteAxisCode, UinputAbsSetup, AbsInfo,
    BusType, InputId,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::os::unix::io::AsRawFd;
use tokio::sync::{Mutex, mpsc};
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use get_if_addrs::get_if_addrs;

// ─── Validation ──────────────────────────────────────────────────────────────

/// Accept only alphanumeric, underscore, hyphen; max 64 chars
fn validate_device_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn sanitize_device_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(64)
        .collect()
}

// ─── Message types ────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct InputMessage {
    t: String,
    c: Option<String>,
    v: Option<i32>,
    id: Option<String>,
    permanent: Option<bool>,
}

#[derive(Serialize, Clone, Debug)]
struct DeviceState {
    id: String,
    approved: bool,
    player: Option<i32>,
}

#[derive(Serialize)]
struct DashboardMessage {
    t: String,
    devices: Vec<DeviceState>,
}

#[derive(Serialize)]
struct ClientMessage {
    t: String,
}

// ─── Shared state ─────────────────────────────────────────────────────────────

/// Events sent from the async handler into the blocking device thread
enum DeviceCmd {
    /// Emit a single input event (key press / axis move)
    Emit(Vec<evdev::InputEvent>),
    /// Shut down the thread
    Stop,
}

struct ClientConnection {
    id: String,
    approved: bool,
    player_slot: Option<i32>,
    sender: mpsc::Sender<Message>,
    /// Channel to send input commands to the blocking device thread.
    /// None until the device is created (i.e. connection is approved).
    device_tx: Option<std::sync::mpsc::SyncSender<DeviceCmd>>,
    /// Handle to abort the async FF-forwarding task on disconnect.
    ff_task: Option<tokio::task::JoinHandle<()>>,
}

struct AppState {
    // Lock order: clients → dashboards → trusted_devices
    clients: Mutex<HashMap<SocketAddr, ClientConnection>>,
    dashboards: Mutex<HashMap<SocketAddr, mpsc::Sender<Message>>>,
    trusted_devices: Mutex<HashSet<String>>,
}

// ─── Persistence ─────────────────────────────────────────────────────────────

fn load_trusted_devices() -> HashSet<String> {
    if let Ok(data) = std::fs::read_to_string("trusted_devices.json") {
        if let Ok(trusted) = serde_json::from_str::<HashSet<String>>(&data) {
            return trusted;
        }
    }
    HashSet::new()
}

fn save_trusted_devices(trusted: &HashSet<String>) {
    match serde_json::to_string(trusted) {
        Ok(data) => {
            let tmp = "trusted_devices.json.tmp";
            if let Err(e) = std::fs::write(tmp, &data) {
                eprintln!("Error writing trusted devices: {}", e);
                return;
            }
            if let Err(e) = std::fs::rename(tmp, "trusted_devices.json") {
                eprintln!("Error renaming trusted devices file: {}", e);
            }
        }
        Err(e) => eprintln!("Error serializing trusted devices: {}", e),
    }
}

// ─── Input mapping ────────────────────────────────────────────────────────────

fn get_key_from_code(code: &str) -> Option<KeyCode> {
    match code {
        "A"      => Some(KeyCode::BTN_SOUTH),
        "B"      => Some(KeyCode::BTN_EAST),
        "X"      => Some(KeyCode::BTN_NORTH),
        "Y"      => Some(KeyCode::BTN_WEST),
        "LB"     => Some(KeyCode::BTN_TL),
        "RB"     => Some(KeyCode::BTN_TR),
        "START"  => Some(KeyCode::BTN_START),
        "SELECT" => Some(KeyCode::BTN_SELECT),
        "LS"     => Some(KeyCode::BTN_THUMBL),
        "RS"     => Some(KeyCode::BTN_THUMBR),
        "MODE"   => Some(KeyCode::BTN_MODE),
        _ => None,
    }
}

// ── Binary Protocol ID Mappings ──
fn get_key_from_id(id: u8) -> Option<KeyCode> {
    match id {
        0  => Some(KeyCode::BTN_SOUTH),
        1  => Some(KeyCode::BTN_EAST),
        2  => Some(KeyCode::BTN_NORTH),
        3  => Some(KeyCode::BTN_WEST),
        4  => Some(KeyCode::BTN_TL),
        5  => Some(KeyCode::BTN_TR),
        6  => Some(KeyCode::BTN_START),
        7  => Some(KeyCode::BTN_SELECT),
        8  => Some(KeyCode::BTN_THUMBL),
        9  => Some(KeyCode::BTN_THUMBR),
        10 => Some(KeyCode::BTN_MODE),
        _  => None,
    }
}

fn get_axis_from_code(code: &str) -> Option<AbsoluteAxisCode> {
    match code {
        "LX"     => Some(AbsoluteAxisCode::ABS_X),
        "LY"     => Some(AbsoluteAxisCode::ABS_Y),
        "RX"     => Some(AbsoluteAxisCode::ABS_RX),
        "RY"     => Some(AbsoluteAxisCode::ABS_RY),
        "LT_ABS" => Some(AbsoluteAxisCode::ABS_Z),
        "RT_ABS" => Some(AbsoluteAxisCode::ABS_RZ),
        "DX"     => Some(AbsoluteAxisCode::ABS_HAT0X),
        "DY"     => Some(AbsoluteAxisCode::ABS_HAT0Y),
        _ => None,
    }
}

fn get_axis_from_id(id: u8) -> Option<AbsoluteAxisCode> {
    match id {
        0 => Some(AbsoluteAxisCode::ABS_X),
        1 => Some(AbsoluteAxisCode::ABS_Y),
        2 => Some(AbsoluteAxisCode::ABS_RX),
        3 => Some(AbsoluteAxisCode::ABS_RY),
        4 => Some(AbsoluteAxisCode::ABS_Z),
        5 => Some(AbsoluteAxisCode::ABS_RZ),
        6 => Some(AbsoluteAxisCode::ABS_HAT0X),
        7 => Some(AbsoluteAxisCode::ABS_HAT0Y),
        _ => None,
    }
}

// ─── Virtual device ───────────────────────────────────────────────────────────

fn create_virtual_device(name: &str) -> std::io::Result<evdev::uinput::VirtualDevice> {
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_SOUTH);
    keys.insert(KeyCode::BTN_EAST);
    keys.insert(KeyCode::BTN_NORTH);
    keys.insert(KeyCode::BTN_WEST);
    keys.insert(KeyCode::BTN_TL);
    keys.insert(KeyCode::BTN_TR);
    keys.insert(KeyCode::BTN_START);
    keys.insert(KeyCode::BTN_SELECT);
    keys.insert(KeyCode::BTN_THUMBL);
    keys.insert(KeyCode::BTN_THUMBR);
    keys.insert(KeyCode::BTN_MODE);

    let abs_x  = UinputAbsSetup::new(AbsoluteAxisCode::ABS_X,     AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_y  = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y,     AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_rx = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RX,    AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_ry = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RY,    AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_z  = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Z,     AbsInfo::new(0, 0, 255, 0, 0, 0));
    let abs_rz = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RZ,    AbsInfo::new(0, 0, 255, 0, 0, 0));
    let abs_hx = UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0X, AbsInfo::new(0, -1, 1, 0, 0, 0));
    let abs_hy = UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0Y, AbsInfo::new(0, -1, 1, 0, 0, 0));

    let mut ff_bits = AttributeSet::<evdev::FFEffectCode>::new();
    ff_bits.insert(evdev::FFEffectCode::FF_RUMBLE);

    // Xbox 360 controller USB IDs — Steam uses these to identify a real gamepad
    let input_id = InputId::new(BusType::BUS_USB, 0x045e, 0x028e, 0x0114);

    evdev::uinput::VirtualDevice::builder()?
        .name(name)
        .input_id(input_id)
        .with_keys(&keys)?
        .with_absolute_axis(&abs_x)?
        .with_absolute_axis(&abs_y)?
        .with_absolute_axis(&abs_rx)?
        .with_absolute_axis(&abs_ry)?
        .with_absolute_axis(&abs_z)?
        .with_absolute_axis(&abs_rz)?
        .with_absolute_axis(&abs_hx)?
        .with_absolute_axis(&abs_hy)?
        .with_ff(&ff_bits)?
        .with_ff_effects_max(16)
        .build()
}

/// Spawn the device infrastructure for one connected phone.
///
/// **Root cause of the "inputs stop after rumble" bug:**
/// `fetch_events()` on a uinput fd is a *blocking* read by default (confirmed
/// in evdev source, line 373: "Without O_NONBLOCK, this will block.").
/// When Steam sends a rumble command the kernel writes a `UI_FF_UPLOAD` event
/// to the fd.  If we never read it the buffer fills, and the *next* call to
/// `dev.emit()` also blocks — because emit and fetch share the same fd.
///
/// **Fix:** set `O_NONBLOCK` on the fd with `fcntl` before entering the loop.
/// `fetch_events()` then returns immediately (empty / EAGAIN) when no FF
/// events are pending, so the emit path is never starved.
fn spawn_device_thread(
    mut dev: evdev::uinput::VirtualDevice,
    phone_tx: mpsc::Sender<Message>,
) -> (std::sync::mpsc::SyncSender<DeviceCmd>, tokio::task::JoinHandle<()>) {
    let (device_tx, device_rx) = std::sync::mpsc::sync_channel::<DeviceCmd>(512);
    let (ff_tx, mut ff_rx) = mpsc::channel::<u32>(32);

    std::thread::spawn(move || {
        // ── Set O_NONBLOCK on the uinput fd ──────────────────────────────────
        // Without this, fetch_events() blocks forever when no FF events are
        // pending, which starves the emit path that shares the same fd.
        let raw_fd = dev.as_raw_fd();
        unsafe {
            let flags = libc::fcntl(raw_fd, libc::F_GETFL);
            if flags >= 0 {
                libc::fcntl(raw_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
        }

        loop {
            // ── 1. Drain all pending emit commands (non-blocking) ─────────────
            // Use try_recv() so we never block here; we fall through to the
            // FF poll below once the channel is empty.
            loop {
                match device_rx.try_recv() {
                    Ok(DeviceCmd::Emit(events)) => {
                        if let Err(e) = dev.emit(&events) {
                            eprintln!("[device] emit error: {}", e);
                        }
                    }
                    Ok(DeviceCmd::Stop) => return,
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                }
            }

            // ── 2. Non-blocking poll for FF events from kernel ────────────────
            // O_NONBLOCK ensures this returns immediately when empty.
            // Must collect to release the iterator borrow before calling
            // process_ff_upload on the same device.
            let pending: Vec<evdev::InputEvent> = match dev.fetch_events() {
                Ok(it) => it.collect(),
                Err(_) => vec![], // EAGAIN or other error — just continue
            };
            for ev in pending {
                if ev.event_type() == evdev::EventType::UINPUT {
                    // EV_UINPUT (0x0101)
                    if ev.code() == 1 {
                        // UI_FF_UPLOAD
                        let uinput_ev = evdev::UInputEvent::new(evdev::UInputCode::UI_FF_UPLOAD, ev.value());
                        if let Ok(mut upload) = dev.process_ff_upload(uinput_ev) {
                            let duration_ms = match upload.effect().kind {
                                evdev::FFEffectKind::Rumble { strong_magnitude, weak_magnitude } => {
                                    let mag = strong_magnitude.max(weak_magnitude);
                                    if mag == 0 { 0 } else { ((mag as u32 * 2000) / 65535).max(50) }
                                }
                                _ => 0,
                            };
                            upload.set_retval(0); // acknowledge to kernel — MUST do this
                            if duration_ms > 0 {
                                let _ = ff_tx.blocking_send(duration_ms);
                            }
                        }
                    } else if ev.code() == 2 {
                        // UI_FF_ERASE
                        let uinput_ev = evdev::UInputEvent::new(evdev::UInputCode::UI_FF_ERASE, ev.value());
                        if let Ok(mut erase) = dev.process_ff_erase(uinput_ev) {
                            erase.set_retval(0); // ack erase
                        }
                    }
                }
            }

            // ── 3. Yield for 1ms ─────────────────────────────────────────────
            // Keeps CPU near 0% when idle. At 1ms granularity the maximum
            // added latency is 1ms — well within the 16ms frame budget.
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    // Async bridge: rumble durations → phone WebSocket
    let ff_task = tokio::spawn(async move {
        while let Some(duration_ms) = ff_rx.recv().await {
            let payload = serde_json::json!([{"t": "rumble", "v": duration_ms}]);
            if let Ok(json) = serde_json::to_string(&payload) {
                if phone_tx.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    (device_tx, ff_task)
}


// ─── Player slot ─────────────────────────────────────────────────────────────

fn next_free_slot(clients: &HashMap<SocketAddr, ClientConnection>) -> i32 {
    let used: HashSet<i32> = clients.values().filter_map(|c| c.player_slot).collect();
    (1..=4).find(|s| !used.contains(s)).unwrap_or(4)
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    println!("Initializing server...");

    let state = Arc::new(AppState {
        clients: Mutex::new(HashMap::new()),
        dashboards: Mutex::new(HashMap::new()),
        trusted_devices: Mutex::new(load_trusted_devices()),
    });

    let app = Router::new()
        .route("/api/ip", get(get_ips))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("../client"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Server running on http://0.0.0.0:8000");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => { eprintln!("FATAL: Could not bind port 8000: {}", e); std::process::exit(1); }
    };
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await {
        eprintln!("FATAL: Server error: {}", e);
    }
}

async fn get_ips() -> axum::Json<Vec<String>> {
    let mut ips = Vec::new();
    if let Ok(interfaces) = get_if_addrs() {
        for iface in interfaces {
            if !iface.is_loopback() && iface.ip().is_ipv4() {
                ips.push(format!("{}: {}", iface.name, iface.ip()));
            }
        }
    }
    axum::Json(ips)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

// ─── Dashboard broadcast ──────────────────────────────────────────────────────

async fn broadcast_dashboard_state(state: &Arc<AppState>) {
    let devices: Vec<DeviceState> = {
        let clients = state.clients.lock().await;
        clients.values().map(|c| DeviceState {
            id: c.id.clone(),
            approved: c.approved,
            player: c.player_slot,
        }).collect()
    };

    let msg = DashboardMessage { t: "state".to_string(), devices };
    let json = match serde_json::to_string(&msg) {
        Ok(j) => j,
        Err(e) => { eprintln!("Serialize error: {}", e); return; }
    };

    let dashboards = state.dashboards.lock().await;
    for tx in dashboards.values() {
        let _ = tx.try_send(Message::Text(json.clone().into()));
    }
}

// ─── WebSocket handler ────────────────────────────────────────────────────────

async fn handle_socket(socket: WebSocket, addr: SocketAddr, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    // Forward outgoing messages to the WebSocket sink
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() { break; }
        }
    });

    let mut is_dashboard = false;
    let mut is_rejected = false;

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Binary(bytes) => {
                if bytes.len() == 4 {
                    let msg_type = bytes[0];
                    let code_id = bytes[1];
                    let value = i16::from_le_bytes([bytes[2], bytes[3]]) as i32;

                    let clients = state.clients.lock().await;
                    if let Some(client) = clients.get(&addr) {
                        if client.approved {
                            if let Some(ref device_tx) = client.device_tx {
                                let mut events = Vec::new();
                                if msg_type == 0 {
                                    if let Some(key_code) = get_key_from_id(code_id) {
                                        events.push(evdev::InputEvent::new(evdev::EventType::KEY.0, key_code.code(), value));
                                        println!("[Binary Input] Type: Button, CodeID: {}, Value: {}", code_id, value);
                                    }
                                } else if msg_type == 1 {
                                    if let Some(axis_code) = get_axis_from_id(code_id) {
                                        events.push(evdev::InputEvent::new(evdev::EventType::ABSOLUTE.0, axis_code.0, value));
                                        // To prevent spamming the console at 120Hz, only log axis events occasionally or just let them spam.
                                        // For full debug, we'll log everything:
                                        println!("[Binary Input] Type: Axis, CodeID: {}, Value: {}", code_id, value);
                                    }
                                }
                                if !events.is_empty() {
                                    events.push(evdev::InputEvent::new(evdev::EventType::SYNCHRONIZATION.0, evdev::SynchronizationCode::SYN_REPORT.0, 0));
                                    let _ = device_tx.send(DeviceCmd::Emit(events));
                                }
                            }
                        }
                    }
                }
            }
            Message::Text(text) => {
                let events = match serde_json::from_str::<Vec<InputMessage>>(&text) {
                    Ok(e) => e,
                    Err(e) => { eprintln!("JSON parse error from {}: {}", addr, e); continue; }
                };

                for event in events {
                    match event.t.as_str() {
                        "dashboard_reg" => {
                            is_dashboard = true;
                            state.dashboards.lock().await.insert(addr, tx.clone());
                            broadcast_dashboard_state(&state).await;
                        }
                        "req_conn" => {
                            if is_rejected { continue; }
                            let raw_id = match event.id {
                                Some(id) => id,
                                None => continue,
                            };
                            if !validate_device_id(&raw_id) {
                                eprintln!("Rejected invalid device ID from {}: {:?}", addr, raw_id);
                                if let Ok(j) = serde_json::to_string(&vec![ClientMessage { t: "rejected".to_string() }]) {
                                    let _ = tx.try_send(Message::Text(j.into()));
                                }
                                break;
                            }
                            let id = sanitize_device_id(&raw_id);
                            println!("Device '{}' requesting connection from {}", id, addr);

                            let mut clients = state.clients.lock().await;
                            let is_trusted = state.trusted_devices.lock().await.contains(&id);

                            let mut device_tx = None;
                            let mut ff_task = None;
                            let mut slot = None;

                            if is_trusted {
                                let free_slot = next_free_slot(&clients);
                                let dev_name = format!("Virtual Smartphone Gamepad ({})", id);
                                match create_virtual_device(&dev_name) {
                                    Ok(dev) => {
                                        let (dtx, ft) = spawn_device_thread(dev, tx.clone());
                                        device_tx = Some(dtx);
                                        ff_task = Some(ft);
                                        slot = Some(free_slot);
                                        println!("Auto-approved '{}' as Player {}", id, free_slot);
                                    }
                                    Err(e) => eprintln!("Failed to create device for {}: {}", id, e),
                                }
                            }

                            clients.insert(addr, ClientConnection {
                                id: id.clone(),
                                approved: is_trusted,
                                player_slot: slot,
                                sender: tx.clone(),
                                device_tx,
                                ff_task,
                            });
                            drop(clients);

                            if is_trusted {
                                let out = serde_json::json!([
                                    {"t": "approved"},
                                    {"t": "player", "num": slot.unwrap_or(1)}
                                ]);
                                if let Ok(j) = serde_json::to_string(&out) {
                                    let _ = tx.try_send(Message::Text(j.into()));
                                }
                            }

                            broadcast_dashboard_state(&state).await;
                        }
                        "approve" => {
                            if !is_dashboard { continue; }
                            let target_id = match event.id.map(|id| sanitize_device_id(&id)) {
                                Some(id) if !id.is_empty() => id,
                                _ => continue,
                            };

                            if event.permanent == Some(true) {
                                let mut trusted = state.trusted_devices.lock().await;
                                trusted.insert(target_id.clone());
                                save_trusted_devices(&trusted);
                            }

                            let mut clients = state.clients.lock().await;
                            let free_slot = next_free_slot(&clients);

                            for client in clients.values_mut() {
                                if client.id == target_id && !client.approved {
                                    let dev_name = format!("Virtual Smartphone Gamepad ({})", target_id);
                                    match create_virtual_device(&dev_name) {
                                        Ok(dev) => {
                                            let (dtx, ft) = spawn_device_thread(dev, client.sender.clone());
                                            client.device_tx = Some(dtx);
                                            client.ff_task = Some(ft);
                                            client.approved = true;
                                            client.player_slot = Some(free_slot);
                                            println!("Approved '{}' as Player {}", target_id, free_slot);
                                        }
                                        Err(e) => eprintln!("Failed to create device: {}", e),
                                    }

                                    let out = serde_json::json!([
                                        {"t": "approved"},
                                        {"t": "player", "num": free_slot}
                                    ]);
                                    if let Ok(j) = serde_json::to_string(&out) {
                                        let _ = client.sender.try_send(Message::Text(j.into()));
                                    }
                                }
                            }
                            drop(clients);
                            broadcast_dashboard_state(&state).await;
                        }
                        "reject" => {
                            if !is_dashboard { continue; }
                            let target_id = match event.id.map(|id| sanitize_device_id(&id)) {
                                Some(id) if !id.is_empty() => id,
                                _ => continue,
                            };

                            let mut clients = state.clients.lock().await;
                            for client in clients.values() {
                                if client.id == target_id {
                                    if let Ok(j) = serde_json::to_string(&vec![ClientMessage { t: "rejected".to_string() }]) {
                                        let _ = client.sender.try_send(Message::Text(j.into()));
                                    }
                                }
                            }
                            clients.retain(|_, c| {
                                if c.id == target_id {
                                    if let Some(task) = &c.ff_task { task.abort(); }
                                }
                                c.id != target_id
                            });
                            drop(clients);
                            broadcast_dashboard_state(&state).await;
                        }
                        "b" | "a" => {
                            // Handled by binary parser now for performance.
                            // Left in case old clients reconnect before sw updates.
                            if is_rejected { continue; }
                            let (Some(c_code), Some(v_val)) = (event.c.clone(), event.v) else { continue; };
                            let clients = state.clients.lock().await;
                            if let Some(client) = clients.get(&addr) {
                                if client.approved {
                                    if let Some(dev_tx) = &client.device_tx {
                                        let mut evts = Vec::new();
                                        if event.t == "b" {
                                            if let Some(key) = get_key_from_code(&c_code) {
                                                evts.push(evdev::InputEvent::new(evdev::EventType::KEY.0, key.code(), v_val));
                                            }
                                        } else {
                                            if let Some(axis) = get_axis_from_code(&c_code) {
                                                evts.push(evdev::InputEvent::new(evdev::EventType::ABSOLUTE.0, axis.0, v_val));
                                            }
                                        }
                                        if !evts.is_empty() {
                                            evts.push(evdev::InputEvent::new(evdev::EventType::SYNCHRONIZATION.0, evdev::SynchronizationCode::SYN_REPORT.0, 0));
                                            let _ = dev_tx.send(DeviceCmd::Emit(evts));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    send_task.abort();

    // Cleanup on disconnect
    if is_dashboard {
        state.dashboards.lock().await.remove(&addr);
    } else {
        let mut clients = state.clients.lock().await;
        if let Some(conn) = clients.remove(&addr) {
            // Signal device thread to stop (also drops device_tx → thread unblocks)
            if let Some(dtx) = conn.device_tx {
                let _ = dtx.send(DeviceCmd::Stop);
            }
            if let Some(task) = conn.ff_task {
                task.abort();
            }
        }
        drop(clients);
        broadcast_dashboard_state(&state).await;
        println!("Client {} disconnected. Virtual device destroyed.", addr);
    }
}
