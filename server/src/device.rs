use evdev::{
    AttributeSet, KeyCode, AbsoluteAxisCode, UinputAbsSetup, AbsInfo,
    BusType, InputId,
};
use evdev::uinput::VirtualDevice;
use std::os::fd::AsRawFd;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread;
use std::time::Duration;
use libc;
use tokio::sync::mpsc;
use axum::extract::ws::Message;

/// Commands sent to the device thread.
pub enum DeviceCmd {
    /// Emit input events.
    Emit(Vec<evdev::InputEvent>),
    /// Shut down the device thread.
    Stop,
}

/// Create the virtual uinput device.
pub fn create_virtual_device(name: &str) -> std::io::Result<VirtualDevice> {
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

    let abs_x = UinputAbsSetup::new(AbsoluteAxisCode::ABS_X, AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_y = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y, AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_rx = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RX, AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_ry = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RY, AbsInfo::new(0, -32768, 32767, 16, 128, 0));
    let abs_z = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Z, AbsInfo::new(0, 0, 255, 0, 0, 0));
    let abs_rz = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RZ, AbsInfo::new(0, 0, 255, 0, 0, 0));
    let abs_hx = UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0X, AbsInfo::new(0, -1, 1, 0, 0, 0));
    let abs_hy = UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0Y, AbsInfo::new(0, -1, 1, 0, 0, 0));

    let mut ff_bits = AttributeSet::<evdev::FFEffectCode>::new();
    ff_bits.insert(evdev::FFEffectCode::FF_RUMBLE);

    // Xbox 360 controller IDs used by Steam.
    let input_id = InputId::new(BusType::BUS_USB, 0x045e, 0x028e, 0x0114);

    VirtualDevice::builder()?
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

/// Spawn the device thread handling input emission and force‑feedback.
pub fn spawn_device_thread(
    mut dev: VirtualDevice,
    phone_tx: mpsc::Sender<Message>,
) -> (SyncSender<DeviceCmd>, tokio::task::JoinHandle<()>) {
    let (device_tx, device_rx) = sync_channel::<DeviceCmd>(512);
    let (ff_tx, mut ff_rx) = mpsc::channel::<u32>(32);

    // Set O_NONBLOCK on the uinput fd to avoid blocking on fetch_events.
    let raw_fd = dev.as_raw_fd();
    unsafe {
        let flags = libc::fcntl(raw_fd, libc::F_GETFL);
        if flags >= 0 {
            libc::fcntl(raw_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
    }

    let _handle = thread::spawn(move || {
        loop {
            // Emit pending commands.
            while let Ok(cmd) = device_rx.try_recv() {
                match cmd {
                    DeviceCmd::Emit(events) => {
                        if let Err(e) = dev.emit(&events) {
                            eprintln!("[device] emit error: {}", e);
                        }
                    }
                    DeviceCmd::Stop => return,
                }
            }

            // Non‑blocking poll for FF events.
            let pending: Vec<_> = match dev.fetch_events() {
                Ok(it) => it.collect(),
                Err(_) => vec![],
            };
            for ev in pending {
                if ev.event_type() == evdev::EventType::UINPUT {
                    match ev.code() {
                        1 => { // UI_FF_UPLOAD
                            let uinput_ev = evdev::UInputEvent::new(evdev::UInputCode::UI_FF_UPLOAD, ev.value());
                            if let Ok(mut upload) = dev.process_ff_upload(uinput_ev) {
                                let duration_ms = match upload.effect().kind {
                                    evdev::FFEffectKind::Rumble { strong_magnitude, weak_magnitude } => {
                                        let mag = strong_magnitude.max(weak_magnitude);
                                        if mag == 0 { 0 } else { ((mag as u32 * 2000) / 65535).max(50) }
                                    }
                                    _ => 0,
                                };
                                upload.set_retval(0);
                                if duration_ms > 0 {
                                    let _ = ff_tx.blocking_send(duration_ms);
                                }
                            }
                        }
                        2 => { // UI_FF_ERASE
                            let uinput_ev = evdev::UInputEvent::new(evdev::UInputCode::UI_FF_ERASE, ev.value());
                            if let Ok(mut erase) = dev.process_ff_erase(uinput_ev) {
                                erase.set_retval(0);
                            }
                        }
                        _ => {}
                    }
                }
            }

            thread::sleep(Duration::from_millis(1));
        }
    });

    // Forward rumble durations to the phone.
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
