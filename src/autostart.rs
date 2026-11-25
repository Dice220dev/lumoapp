use anyhow::{Context, Result};
use winreg::enums::{HKEY_CURRENT_USER, KEY_QUERY_VALUE, REG_BINARY};
use winreg::{RegKey, RegValue};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const VALUE_NAME: &str = "Lumo";

pub fn enabled() -> bool {
    command().is_some()
}

pub fn command() -> Option<String> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_QUERY_VALUE)
        .and_then(|key| key.get_value::<String, _>(VALUE_NAME))
        .ok()
}

pub fn set(enabled: bool, start_minimized: bool) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(RUN_KEY)
        .context("open Run registry key")?;
    if enabled {
        let exe = std::env::current_exe().context("current exe")?;
        let command = if start_minimized {
            format!("\"{}\" --minimized", exe.display())
        } else {
            format!("\"{}\"", exe.display())
        };
        key.set_value(VALUE_NAME, &command)
            .context("write autostart value")?;
        // Make the entry explicitly enabled in Task Manager's Startup tab. Without this,
        // an entry that was once disabled there can stay disabled despite the Run value.
        if let Ok((approved, _)) = hkcu.create_subkey(APPROVED_KEY) {
            let value = RegValue {
                bytes: vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                vtype: REG_BINARY,
            };
            let _ = approved.set_raw_value(VALUE_NAME, &value);
        }
    } else {
        match key.delete_value(VALUE_NAME) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).context("remove autostart value"),
        }
        if let Ok(approved) = hkcu.open_subkey_with_flags(APPROVED_KEY, winreg::enums::KEY_SET_VALUE)
        {
            let _ = approved.delete_value(VALUE_NAME);
        }
    }
    Ok(())
}
