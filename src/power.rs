use windows::Win32::Foundation::{GetLastError, ERROR_SUCCESS, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, TOKEN_PRIVILEGES,
};
use windows::Win32::System::Power::SetSuspendState;
use windows::Win32::System::Shutdown::{
    AbortSystemShutdownW, ExitWindowsEx, LockWorkStation, EWX_FORCEIFHUNG, EWX_POWEROFF, EWX_REBOOT,
    SHTDN_REASON_FLAG_PLANNED, SHTDN_REASON_MAJOR_APPLICATION, SHTDN_REASON_MINOR_OTHER,
};
use std::os::windows::process::CommandExt;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn shutdown_privilege_available() -> bool {
    unsafe { enable_shutdown_privilege() }
}

pub fn execute(action: &str) -> Result<(), String> {
    if std::env::var("LUMO_POWER_DRYRUN")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        tracing::info!("power: dry-run {action}");
        return Ok(());
    }
    tracing::info!("power: executing {action}");
    match action {
        "lock" => {
            let api = unsafe { LockWorkStation() };
            match api {
                Ok(()) => {
                    tracing::info!("power: LockWorkStation succeeded");
                    Ok(())
                }
                Err(e) => {
                    tracing::warn!("LockWorkStation failed: {e}; sending Win+L");
                    crate::input::lock_workstation_hotkey();
                    Ok(())
                }
            }
        }
        "sleep" => {
            let privileged = unsafe { enable_shutdown_privilege() };
            tracing::info!("power: shutdown privilege={privileged}");
            let res = unsafe { SetSuspendState(false, false, false) };
            if res.as_bool() {
                tracing::info!("power: SetSuspendState succeeded");
                return Ok(());
            }
            tracing::warn!("SetSuspendState returned false; trying rundll32");
            fallback("powrprof.dll,SetSuspendState", &["0,1,0"])
                .map_err(|e| format!("sleep failed: {e}"))
        }
        "restart" | "shutdown" => shutdown_or_restart(action),
        "cancel" => {
            if let Ok(status) = std::process::Command::new("shutdown.exe")
                .arg("/a")
                .creation_flags(CREATE_NO_WINDOW)
                .status()
            {
                if status.success() {
                    tracing::info!("power: shutdown.exe /a cancelled the timer");
                    return Ok(());
                }
            }
            unsafe {
                AbortSystemShutdownW(windows::core::PCWSTR::null())
                    .map_err(|e| format!("nothing to cancel: {e}"))
            }
        }
        other => Err(format!("unknown power action: {other}")),
    }
}

fn shutdown_or_restart(action: &str) -> Result<(), String> {
    let flag = if action == "restart" { "/r" } else { "/s" };
    // shutdown.exe obtains the shutdown privilege itself and works in contexts where
    // a direct ExitWindowsEx call is refused. It returns immediately after scheduling.
    match std::process::Command::new("shutdown.exe")
        .args([flag, "/t", "0", "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
    {
        Ok(status) if status.success() => {
            tracing::info!("power: shutdown.exe {flag} accepted");
            return Ok(());
        }
        Ok(status) => tracing::warn!("power: shutdown.exe {flag} exited with {status}"),
        Err(e) => tracing::warn!("power: could not run shutdown.exe: {e}"),
    }

    let privileged = unsafe { enable_shutdown_privilege() };
    tracing::info!("power: falling back to ExitWindowsEx, privilege={privileged}");
    let reason =
        SHTDN_REASON_MAJOR_APPLICATION | SHTDN_REASON_MINOR_OTHER | SHTDN_REASON_FLAG_PLANNED;
    let flags = if action == "restart" {
        EWX_REBOOT | EWX_FORCEIFHUNG
    } else {
        EWX_POWEROFF | EWX_FORCEIFHUNG
    };
    unsafe {
        ExitWindowsEx(flags, reason).map_err(|e| format!("{action} failed: {e}"))
    }
}

unsafe fn enable_shutdown_privilege() -> bool {
    let mut token = HANDLE::default();
    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
        &mut token,
    )
    .is_err()
    {
        return false;
    }
    let mut luid = LUID::default();
    if LookupPrivilegeValueW(
        windows::core::PCWSTR::null(),
        windows::core::w!("SeShutdownPrivilege"),
        &mut luid,
    )
    .is_err()
    {
        return false;
    }
    let privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    let result = AdjustTokenPrivileges(token, false, Some(&privileges), 0, None, None);
    let last_error = GetLastError();
    if last_error != ERROR_SUCCESS {
        tracing::warn!("power: AdjustTokenPrivileges error={last_error:?}");
    }
    result.is_ok() && last_error == ERROR_SUCCESS
}

fn fallback(dll_entry: &str, args: &[&str]) -> std::io::Result<()> {
    std::process::Command::new("rundll32.exe")
        .arg(dll_entry)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
}
