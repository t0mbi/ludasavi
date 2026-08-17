//! Registers (or unregisters) Ludusavi to launch automatically when the user logs
//! into Windows, via the standard per-user `Run` registry key.

use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Ludusavi";

pub fn set_enabled(enabled: bool) -> std::io::Result<()> {
    let hive = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hive.create_subkey(RUN_KEY_PATH)?;

    if enabled {
        let exe = std::env::current_exe()?;
        let command = format!("\"{}\" --minimized", exe.display());
        key.set_value(VALUE_NAME, &command)?;
    } else {
        match key.delete_value(VALUE_NAME) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}
