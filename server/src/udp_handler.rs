use std::sync::Arc;
use tokio::net::UdpSocket;
use std::net::SocketAddr;
use evdev::{EventType, SynchronizationCode, InputEvent};
use crate::state::AppState;
use crate::device::DeviceCmd;

// Helper to map code_id to evdev key
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

// Helper to map code_id to evdev axis
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

pub async fn start_udp_server(state: Arc<AppState>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    let socket = match UdpSocket::bind(addr).await {
        Ok(s) => {
            println!("UDP Server running on 0.0.0.0:8001");
            s
        },
        Err(e) => {
            eprintln!("FATAL: Could not bind UDP port 8001: {}", e);
            return;
        }
    };

    let mut buf = [0u8; 128]; // Max packet size should be small

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, _src_addr)) => {
                if len < 6 {
                    continue; // Too small to be a valid packet
                }

                // Parse packet: 
                // [id_len (1 byte)] 
                // [device_id (id_len bytes)]
                // [msg_type (1 byte)]
                // [code_id (1 byte)]
                // [value_lo (1 byte), value_hi (1 byte)]
                
                let id_len = buf[0] as usize;
                if len < 1 + id_len + 4 {
                    continue; // Malformed packet
                }

                let id_bytes = &buf[1..1 + id_len];
                let device_id = match std::str::from_utf8(id_bytes) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let offset = 1 + id_len;
                let msg_type = buf[offset];
                let code_id = buf[offset + 1];
                let value = i16::from_le_bytes([buf[offset + 2], buf[offset + 3]]) as i32;

                // Lookup client by device_id in shared state
                let clients = state.clients.lock().await;
                
                // Find the approved client with this device ID
                if let Some(client) = clients.values().find(|c| c.id == device_id && c.approved) {
                    if let Some(ref device_tx) = client.device_tx {
                        let mut events = Vec::new();
                        
                        if msg_type == 0 {
                            if let Some(k) = get_key_from_id(code_id) {
                                events.push(InputEvent::new(EventType::KEY.0, k.code(), value));
                            }
                        } else if msg_type == 1 {
                            if let Some(a) = get_axis_from_id(code_id) {
                                events.push(InputEvent::new(EventType::ABSOLUTE.0, a.0, value));
                            }
                        }

                        if !events.is_empty() {
                            events.push(InputEvent::new(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0));
                            let _ = device_tx.send(DeviceCmd::Emit(events));
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("UDP receive error: {}", e);
            }
        }
    }
}
