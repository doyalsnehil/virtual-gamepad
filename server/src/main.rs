use axum::{Router, routing::get};
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use std::sync::Arc;

mod state;
mod persistence;
mod device;
mod handlers;
mod udp_handler;

#[tokio::main]
async fn main() {
    // Initialize shared application state.
    let state = Arc::new(state::AppState {
        clients: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        dashboards: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        trusted_devices: tokio::sync::Mutex::new(persistence::load_trusted_devices()),
    });

    // Build the Axum router, delegating HTTP and WS handlers to the handlers module.
    let app = Router::new()
        .route("/api/ip", get(handlers::get_ips))
        .route("/ws", get(handlers::ws_handler))
        .fallback_service(ServeDir::new("../client"))
        .with_state(state.clone());

    // Spawn the UDP listener for the Android app
    tokio::spawn(udp_handler::start_udp_server(state.clone()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Server running on http://0.0.0.0:8000");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("FATAL: Could not bind port 8000: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await {
        eprintln!("FATAL: Server error: {}", e);
    }
}
