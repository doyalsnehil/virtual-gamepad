# Virtual Gamepad Controller

Turn your phone into a wireless gamepad for your Linux PC. Uses a Rust WebSocket server that creates a virtual controller via `uinput`, and a mobile-first web client that runs in your phone's browser.

## Project Structure

```
controller/
├── client/                  # Web client (served by the Rust server)
│   ├── index.html           # Gamepad UI
│   ├── app.js               # Gamepad logic (WebSocket, input handling, layout editor)
│   ├── style.css            # Gamepad styling
│   ├── sw.js                # Service worker for offline/PWA support
│   ├── manifest.json        # PWA manifest
│   └── dashboard/           # Admin dashboard for approving/rejecting devices
│       ├── index.html
│       └── dashboard.js
├── server/                  # Rust backend
│   ├── Cargo.toml           # Dependencies
│   └── src/
│       └── main.rs          # WebSocket server + uinput virtual device
├── .gitignore
└── README.md
```

## Requirements

- **Linux** (uses `uinput` kernel module)
- **Rust** (1.70+)
- Phone and PC on the same network

## Quick Start

```bash
# Build the server
cd server
cargo build --release

# Run (needs root for uinput access)
sudo ./target/release/server

# Open on your phone browser
# http://<your-pc-ip>:8000
```

## Features

- **Binary protocol** — 4-byte packets for minimal latency
- **NippleJS analog sticks** — smooth joystick input
- **Editable layout** — drag, resize, and save button positions
- **PWA installable** — add to home screen for native app feel
- **Force feedback** — rumble/vibration support via Steam
- **Multi-device** — dashboard to approve/reject connecting phones
- **Player LEDs** — visual indicator for player number assignment
