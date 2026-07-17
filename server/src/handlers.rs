// Handlers and utilities for the virtual gamepad server.

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, ConnectInfo, State},
    response::Response,
    Json,
};
use futures_util::SinkExt;
use futures_util::StreamExt;
use get_if_addrs::get_if_addrs;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::state::{AppState, ClientConnection};
use crate::device::{spawn_device_thread, DeviceCmd};
use crate::persistence::save_trusted_devices;

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

// ─── Message types ────────────────────────────────────────────────────────
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

// ─── Input mapping ────────────────────────────────────────────────────────
fn get_key_from_code(code: &str) -> Option<evdev::KeyCode> {
    match code {
        "A" => Some(evdev::KeyCode::BTN_SOUTH),
        "B" => Some(evdev::KeyCode::BTN_EAST),
        "X" => Some(evdev::KeyCode::BTN_NORTH),
        "Y" => Some(evdev::KeyCode::BTN_WEST),
        "LB" => Some(evdev::KeyCode::BTN_TL),
        "RB" => Some(evdev::KeyCode::BTN_TR),
        "START" => Some(evdev::KeyCode::BTN_START),
        "SELECT" => Some(evdev::KeyCode::BTN_SELECT),
        "LS" => Some(evdev::KeyCode::BTN_THUMBL),
        "RS" => Some(evdev::KeyCode::BTN_THUMBR),
        "MODE" => Some(evdev::KeyCode::BTN_MODE),
        _ => None,
    }
}

fn get_key_from_id(id: u8) -> Option<evdev::KeyCode> {
    match id {
        0 => Some(evdev::KeyCode::BTN_SOUTH),
        1 => Some(evdev::KeyCode::BTN_EAST),
        2 => Some(evdev::KeyCode::BTN_NORTH),
        3 => Some(evdev::KeyCode::BTN_WEST),
        4 => Some(evdev::KeyCode::BTN_TL),
        5 => Some(evdev::KeyCode::BTN_TR),
        6 => Some(evdev::KeyCode::BTN_START),
        7 => Some(evdev::KeyCode::BTN_SELECT),
        8 => Some(evdev::KeyCode::BTN_THUMBL),
        9 => Some(evdev::KeyCode::BTN_THUMBR),
        10 => Some(evdev::KeyCode::BTN_MODE),
        _ => None,
    }
}

fn get_axis_from_code(code: &str) -> Option<evdev::AbsoluteAxisCode> {
    match code {
        "LX" => Some(evdev::AbsoluteAxisCode::ABS_X),
        "LY" => Some(evdev::AbsoluteAxisCode::ABS_Y),
        "RX" => Some(evdev::AbsoluteAxisCode::ABS_RX),
        "RY" => Some(evdev::AbsoluteAxisCode::ABS_RY),
        "LT_ABS" => Some(evdev::AbsoluteAxisCode::ABS_Z),
        "RT_ABS" => Some(evdev::AbsoluteAxisCode::ABS_RZ),
        "DX" => Some(evdev::AbsoluteAxisCode::ABS_HAT0X),
        "DY" => Some(evdev::AbsoluteAxisCode::ABS_HAT0Y),
        _ => None,
    }
}

fn get_axis_from_id(id: u8) -> Option<evdev::AbsoluteAxisCode> {
    match id {
        0 => Some(evdev::AbsoluteAxisCode::ABS_X),
        1 => Some(evdev::AbsoluteAxisCode::ABS_Y),
        2 => Some(evdev::AbsoluteAxisCode::ABS_RX),
        3 => Some(evdev::AbsoluteAxisCode::ABS_RY),
        4 => Some(evdev::AbsoluteAxisCode::ABS_Z),
        5 => Some(evdev::AbsoluteAxisCode::ABS_RZ),
        6 => Some(evdev::AbsoluteAxisCode::ABS_HAT0X),
        7 => Some(evdev::AbsoluteAxisCode::ABS_HAT0Y),
        _ => None,
    }
}

// ─── Player slot helper ───────────────────────────────────────────────────
fn next_free_slot(clients: &HashMap<SocketAddr, ClientConnection>) -> i32 {
    let used: HashSet<i32> = clients.values().filter_map(|c| c.player_slot).collect();
    (1..=4).find(|s| !used.contains(s)).unwrap_or(4)
}

// ─── Dashboard broadcasting ────────────────────────────────────────────────
pub async fn broadcast_dashboard_state(state: &Arc<AppState>) {
    let devices: Vec<DeviceState> = {
        let clients = state.clients.lock().await;
        clients
            .values()
            .map(|c| DeviceState {
                id: c.id.clone(),
                approved: c.approved,
                player: c.player_slot,
            })
            .collect()
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

// ─── HTTP handlers ────────────────────────────────────────────────────────
pub async fn get_ips() -> Json<Vec<String>> {
    let mut ips = Vec::new();
    if let Ok(ifaces) = get_if_addrs() {
        for iface in ifaces {
            if !iface.is_loopback() && iface.ip().is_ipv4() {
                ips.push(format!("{}: {}", iface.name, iface.ip()));
            }
        }
    }
    Json(ips)
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

// ─── Core WebSocket handling ───────────────────────────────────────────────
pub async fn handle_socket(socket: WebSocket, addr: SocketAddr, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    // Forward outgoing messages to the client.
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() { break; }
        }
    });

    let mut is_dashboard = false;
    let is_rejected = false;

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
                                    if let Some(k) = get_key_from_id(code_id) {
                                        events.push(evdev::InputEvent::new(evdev::EventType::KEY.0, k.code(), value));
                                    }
                                } else if msg_type == 1 {
                                    if let Some(a) = get_axis_from_id(code_id) {
                                        events.push(evdev::InputEvent::new(evdev::EventType::ABSOLUTE.0, a.0, value));
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
                let events: Vec<InputMessage> = match serde_json::from_str(&text) {
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
                            let raw_id = match event.id { Some(id) => id, None => continue };
                            if !validate_device_id(&raw_id) {
                                eprintln!("Rejected invalid device ID from {}: {:?}", addr, raw_id);
                                let rej = serde_json::to_string(&vec![ClientMessage { t: "rejected".to_string() }]).ok();
                                if let Some(j) = rej { let _ = tx.try_send(Message::Text(j.into())); }
                                break;
                            }
                            let id = sanitize_device_id(&raw_id);
                            let mut clients = state.clients.lock().await;
                            let is_trusted = state.trusted_devices.lock().await.contains(&id);
                            let mut device_tx = None;
                            let mut ff_task = None;
                            let mut slot = None;
                            if is_trusted {
                                let free_slot = next_free_slot(&clients);
                                let dev_name = format!("Virtual Smartphone Gamepad ({})", id);
                                match crate::device::create_virtual_device(&dev_name) {
                                    Ok(dev) => {
                                        let (dtx, ft) = spawn_device_thread(dev, tx.clone());
                                        device_tx = Some(dtx);
                                        ff_task = Some(ft);
                                        slot = Some(free_slot);
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
                                if let Ok(j) = serde_json::to_string(&out) { let _ = tx.try_send(Message::Text(j.into())); }
                            }
                            broadcast_dashboard_state(&state).await;
                        }
                        "approve" => {
                            if !is_dashboard { continue; }
                            let target_id = match event.id.map(|i| sanitize_device_id(&i)) { Some(id) if !id.is_empty() => id, _ => continue };
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
                                    match crate::device::create_virtual_device(&dev_name) {
                                        Ok(dev) => {
                                            let (dtx, ft) = spawn_device_thread(dev, client.sender.clone());
                                            client.device_tx = Some(dtx);
                                            client.ff_task = Some(ft);
                                            client.approved = true;
                                            client.player_slot = Some(free_slot);
                                        }
                                        Err(e) => eprintln!("Failed to create device: {}", e),
                                    }
                                    let out = serde_json::json!([
                                        {"t": "approved"},
                                        {"t": "player", "num": free_slot}
                                    ]);
                                    if let Ok(j) = serde_json::to_string(&out) { let _ = client.sender.try_send(Message::Text(j.into())); }
                                }
                            }
                            drop(clients);
                            broadcast_dashboard_state(&state).await;
                        }
                        "reject" => {
                            if !is_dashboard { continue; }
                            let target_id = match event.id.map(|i| sanitize_device_id(&i)) { Some(id) if !id.is_empty() => id, _ => continue };
                            let mut clients = state.clients.lock().await;
                            for client in clients.values() {
                                if client.id == target_id {
                                    let rej = serde_json::to_string(&vec![ClientMessage { t: "rejected".to_string() }]).ok();
                                    if let Some(j) = rej { let _ = client.sender.try_send(Message::Text(j.into())); }
                                }
                            }
                            clients.retain(|_, c| {
                                if c.id == target_id {
                                    // Stop the device thread so the virtual uinput device is destroyed
                                    if let Some(dtx) = &c.device_tx { let _ = dtx.send(DeviceCmd::Stop); }
                                    if let Some(task) = &c.ff_task { task.abort(); }
                                }
                                c.id != target_id
                            });
                            drop(clients);
                            // Remove from trusted so reconnect shows as "requested"
                            let mut trusted = state.trusted_devices.lock().await;
                            if trusted.remove(&target_id) {
                                save_trusted_devices(&trusted);
                            }
                            drop(trusted);
                            broadcast_dashboard_state(&state).await;
                        }
                        "b" | "a" => {
                            if is_rejected { continue; }
                            let c_code = match event.c.clone() { Some(c) => c, None => continue };
                            let v_val = match event.v { Some(v) => v, None => continue };
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

    // Cleanup
    send_task.abort();
    if is_dashboard {
        state.dashboards.lock().await.remove(&addr);
    } else {
        let mut clients = state.clients.lock().await;
        if let Some(conn) = clients.remove(&addr) {
            if let Some(dtx) = conn.device_tx { let _ = dtx.send(DeviceCmd::Stop); }
            if let Some(task) = conn.ff_task { task.abort(); }
        }
        drop(clients);
        broadcast_dashboard_state(&state).await;
    }
}
