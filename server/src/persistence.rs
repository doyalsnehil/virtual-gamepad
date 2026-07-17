use std::fs;
use std::collections::HashSet;
use serde_json;

/// Load trusted device IDs from `trusted_devices.json`.
pub fn load_trusted_devices() -> HashSet<String> {
    if let Ok(data) = fs::read_to_string("trusted_devices.json") {
        if let Ok(set) = serde_json::from_str::<HashSet<String>>(&data) {
            return set;
        }
    }
    HashSet::new()
}

/// Persist trusted device IDs to `trusted_devices.json` atomically.
pub fn save_trusted_devices(trusted: &HashSet<String>) {
    match serde_json::to_string(trusted) {
        Ok(data) => {
            let tmp = "trusted_devices.json.tmp";
            if let Err(e) = fs::write(tmp, &data) {
                eprintln!("Error writing trusted devices: {}", e);
                return;
            }
            if let Err(e) = fs::rename(tmp, "trusted_devices.json") {
                eprintln!("Error renaming trusted devices file: {}", e);
            }
        }
        Err(e) => eprintln!("Error serializing trusted devices: {}", e),
    }
}
