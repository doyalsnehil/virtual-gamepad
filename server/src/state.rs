use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;
use axum::extract::ws::Message;
use crate::device::DeviceCmd;

/// Shared application state.

pub struct AppState {
    /// Connected client sockets.
    pub clients: Mutex<HashMap<std::net::SocketAddr, ClientConnection>>, // will be re-exported
    /// Dashboard sockets.
    pub dashboards: Mutex<HashMap<std::net::SocketAddr, tokio::sync::mpsc::Sender<Message>>>,
    /// Trusted device IDs persisted across runs.
    pub trusted_devices: Mutex<HashSet<String>>,
}

/// Connection information for a client socket.
pub struct ClientConnection {
    pub id: String,
    pub approved: bool,
    pub player_slot: Option<i32>,
    pub sender: tokio::sync::mpsc::Sender<Message>,
    /// Channel to send input commands to the blocking device thread.
    pub device_tx: Option<std::sync::mpsc::SyncSender<DeviceCmd>>,
    /// Handle to abort the async FF-forwarding task on disconnect.
    pub ff_task: Option<tokio::task::JoinHandle<()>>,
}
