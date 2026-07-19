# Virtual Gamepad Controller

> ⚠️ **WARNING**: **This application is highly experimental on Windows.** The primary target operating system is Linux (using the `uinput` kernel module). At this point, Windows support is not guaranteed or officially supported.

Turn your Android phone into a wireless gamepad for your Linux PC. This project features a Rust backend server that creates a virtual controller via `uinput` and an Android native client application (Kotlin/Jetpack Compose) for low-latency input streaming.

## Project Structure

```text
controller/
├── android/                 # Android Native Client (Kotlin/Compose)
│   ├── app/                 # App module
│   │   ├── src/             # Source code
│   │   │   ├── main/java    # UI, Networking, and Controller Logic
│   │   │   └── main/res     # Resources, drawables, layouts
│   │   └── build.gradle.kts # Android App build script
│   └── ...                  # Gradle configuration
├── server/                  # Rust backend
│   ├── Cargo.toml           # Dependencies
│   └── src/
│       ├── main.rs          # Server entry point
│       ├── udp_handler.rs   # Low-latency UDP packet processing
│       ├── handlers.rs      # HTTP/WebSocket handlers
│       ├── device.rs        # Virtual gamepad interface (uinput/evdev)
│       └── ...              
├── .gitignore
└── README.md
```

## Requirements

- **Linux OS** (Requires `uinput` kernel module to emulate gamepads)
- **Rust** (1.70+) installed on the PC
- **Android Device** (Android 8.0+ recommended)
- Phone and PC connected to the **same local network** (Wi-Fi/LAN)

## Required Ports

You must ensure the following ports are open on your Linux PC's firewall to allow the Android app to connect:

- **TCP Port `8000`**: Used for the initial connection, device approval, and WebSocket handshakes.
- **UDP Port `8001`**: Used for the ultra-low latency continuous controller input stream.

## Detailed Installation Process

### 1. Starting the Server (Linux PC)

1. Open your terminal and navigate to the server directory:
   ```bash
   cd controller/server
   ```
2. Build the Rust project in release mode:
   ```bash
   cargo build --release
   ```
3. Run the server. **Note**: Because the server needs to create a virtual input device using `uinput`, it must be run with root privileges:
   ```bash
   sudo ./target/release/server
   ```
4. Note the IP address of your Linux PC (e.g., `192.168.1.100`).

### 2. Installing the Android Client

1. Open the project in **Android Studio** or use the Gradle wrapper in the `android/` directory.
2. Build the Release APK:
   ```bash
   cd controller/android
   ./gradlew assembleRelease
   ```
3. Transfer the compiled APK from `android/app/build/outputs/apk/release/app-release.apk` to your Android device and install it.
4. Launch the app on your phone, and enter the IP address of your Linux PC when prompted.

## Features

- **Binary protocol** — 4-byte UDP packets for minimal latency.
- **Independent Triggers & Bumpers** — Full control over independent shoulder button placement.
- **Editable layout** — Drag, resize, and save button and joystick positions on your Android screen.
- **Force feedback** — Rumble/vibration support directly passed from Steam/games to the Android device.
- **Multi-device** — Dashboard to approve/reject connecting phones.
- **Player LEDs** — Visual indicator for player number assignment.
