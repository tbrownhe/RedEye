//! Per-user display preference and opt-in Windows startup registration.

use std::{os::windows::ffi::OsStrExt, path::Path, ptr};

use windows_sys::{
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            HKEY_CURRENT_USER, REG_DWORD, REG_SZ, RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
            RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
        },
    },
    core::w,
};

const SETTINGS_KEY: *const u16 = w!("Software\\RedEye");
const DISPLAY_VALUE: *const u16 = w!("KeepDisplayOn");
const RUN_KEY: *const u16 = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const STARTUP_VALUE: *const u16 = w!("RedEye");

pub fn load_display() -> Result<bool, String> {
    let mut value = 0u32;
    let mut bytes = size_of::<u32>() as u32;
    // SAFETY: Predefined HKCU is valid, strings are static/NUL-terminated,
    // output points to an aligned DWORD with exactly its available byte size.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            SETTINGS_KEY,
            DISPLAY_VALUE,
            RRF_RT_REG_DWORD,
            ptr::null_mut(),
            (&mut value as *mut u32).cast(),
            &mut bytes,
        )
    };
    if missing(status) {
        return Ok(false);
    }
    check(status, "Read the saved display preference")?;
    parse_display(value, bytes)
}

fn parse_display(value: u32, bytes: u32) -> Result<bool, String> {
    match (value, bytes) {
        (0, 4) => Ok(false),
        (1, 4) => Ok(true),
        _ => Err("The saved KeepDisplayOn value is invalid. It must be a DWORD containing 0 or 1. RedEye will allow the display to turn off until you change the option.".into()),
    }
}

pub fn save_display(enabled: bool) -> Result<(), String> {
    let value = u32::from(enabled);
    // SAFETY: Valid static key/value names and readable DWORD. RegSetKeyValue
    // creates the per-user subkey when absent and copies value synchronously.
    let status = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            SETTINGS_KEY,
            DISPLAY_VALUE,
            REG_DWORD,
            (&value as *const u32).cast(),
            size_of::<u32>() as u32,
        )
    };
    check(status, "Save the display preference")
}

/// Any nonempty RedEye Run value counts as registered, including another copy.
/// This allows a user to disable a stale registration from the current copy.
pub fn startup_enabled() -> Result<bool, String> {
    let mut command = [0u16; 261];
    let mut bytes = size_of_val(&command) as u32;
    // SAFETY: Output is an aligned UTF-16 array and bytes describes its exact
    // capacity. Restrict to REG_SZ; Windows guarantees NUL termination on success.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            RUN_KEY,
            STARTUP_VALUE,
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            command.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    if missing(status) {
        return Ok(false);
    }
    check(status, "Read RedEye's Launch at sign-in registration")?;
    parse_startup(&command, bytes)
}

fn parse_startup(command: &[u16], bytes: u32) -> Result<bool, String> {
    let invalid = || {
        "RedEye's sign-in registration is malformed. Inspect HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\\RedEye and remove or correct that value.".to_string()
    };
    if !bytes.is_multiple_of(2) || bytes == 0 {
        return Err(invalid());
    }
    let command = command.get(..bytes as usize / 2).ok_or_else(invalid)?;
    if command.last() != Some(&0) {
        return Err(invalid());
    }
    let text = &command[..command.len() - 1];
    if text.contains(&0) || String::from_utf16(text).is_err() {
        return Err(invalid());
    }
    Ok(!text.is_empty())
}

pub fn set_startup(enabled: bool) -> Result<(), String> {
    if enabled {
        let executable = std::env::current_exe()
            .map_err(|e| format!("Cannot locate RedEye's executable: {e}"))?;
        let temp = std::env::temp_dir();
        // Resolve aliases for the disposable-location check, but register the
        // exact executable path Windows reported, preserving Unicode/quoting.
        let resolved_executable = executable
            .canonicalize()
            .map_err(|e| format!("Cannot verify RedEye's stable executable location: {e}"))?;
        let resolved_temp = temp
            .canonicalize()
            .map_err(|e| format!("Cannot verify the Windows temporary folder: {e}"))?;
        reject_disposable(&resolved_executable, &resolved_temp)?;
        let command = startup_command(&executable, &temp)?;
        // SAFETY: command is valid NUL-terminated UTF-16. Its complete byte
        // count includes the NUL. Only HKCU Run's RedEye value is changed.
        let status = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                RUN_KEY,
                STARTUP_VALUE,
                REG_SZ,
                command.as_ptr().cast(),
                size_of_val(command.as_slice()) as u32,
            )
        };
        check(status, "Enable Launch at sign-in")
    } else {
        // SAFETY: Only the named RedEye value is removed; no key or other value
        // is deleted. Missing registration already satisfies disable.
        let status = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, STARTUP_VALUE) };
        if missing(status) {
            Ok(())
        } else {
            check(status, "Disable Launch at sign-in")
        }
    }
}

fn startup_command(executable: &Path, temp: &Path) -> Result<Vec<u16>, String> {
    let path: Vec<u16> = executable.as_os_str().encode_wide().collect();
    if !executable.is_absolute() || path.contains(&0) || path.contains(&(b'"' as u16)) {
        return Err("RedEye's executable path cannot be registered safely. Copy redeye.exe to an absolute, stable location and launch that copy.".into());
    }
    reject_disposable(executable, temp)?;
    let mut command = vec![b'"' as u16];
    command.extend(path);
    command.push(b'"' as u16);
    if command.len() > 260 {
        return Err("The quoted RedEye path exceeds Windows Run's 260-character command limit. Move redeye.exe to a shorter stable path, launch it, then enable Launch at sign-in.".into());
    }
    command.push(0);
    Ok(command)
}

fn reject_disposable(executable: &Path, temp: &Path) -> Result<(), String> {
    let components: Vec<String> = executable
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect();
    let cargo_output = components.iter().enumerate().any(|(index, component)| {
        component == "target"
            && [index + 1, index + 2].into_iter().any(|next| {
                components
                    .get(next)
                    .is_some_and(|part| part == "debug" || part == "release")
            })
    });
    if cargo_output || path_is_within(executable, temp) {
        return Err("Launch at sign-in is unavailable from a Cargo build or temporary folder. Copy redeye.exe to a stable folder, for example %LOCALAPPDATA%\\RedEye, launch that copy, then enable Launch at sign-in.".into());
    }
    Ok(())
}

fn path_is_within(path: &Path, directory: &Path) -> bool {
    // Known Cargo/temp components are ASCII; keep the rest as UTF-16 so a
    // Unicode executable name is never altered in the written command line.
    let normalize = |path: &Path| -> Vec<u16> {
        let mut path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .map(|c| match c {
                65..=90 => c + 32,
                47 => 92,
                _ => c,
            })
            .collect();
        // canonicalize returns extended-length paths. Normalize their prefix
        // before comparing with a regular spelling of the same temp folder.
        if path.starts_with(&[92, 92, 63, 92]) {
            path.drain(..4);
            if path.starts_with(&[117, 110, 99, 92]) {
                path.splice(..4, [92, 92]);
            }
        }
        path
    };
    let path = normalize(path);
    let mut directory = normalize(directory);
    while directory.last() == Some(&92) {
        directory.pop();
    }
    !directory.is_empty()
        && path.starts_with(&directory)
        && (path.len() == directory.len() || path.get(directory.len()) == Some(&92))
}

fn missing(status: u32) -> bool {
    status == ERROR_FILE_NOT_FOUND || status == ERROR_PATH_NOT_FOUND
}

fn check(status: u32, action: &str) -> Result<(), String> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!(
            "Cannot {}: {}. Check your per-user registry permissions.",
            action.to_ascii_lowercase(),
            std::io::Error::from_raw_os_error(status as i32)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_parser_accepts_only_boolean_dword_values() {
        assert!(!parse_display(0, 4).unwrap());
        assert!(parse_display(1, 4).unwrap());
        assert!(parse_display(2, 4).is_err());
        assert!(parse_display(1, 3).is_err());
    }

    #[test]
    fn startup_parser_handles_missing_text_and_rejects_malformed_strings() {
        assert!(!parse_startup(&[0], 2).unwrap());
        assert!(parse_startup(&[65, 0], 4).unwrap());
        assert!(parse_startup(&[65], 2).is_err());
        assert!(parse_startup(&[65, 0, 66, 0], 8).is_err());
        assert!(parse_startup(&[0xd800, 0], 4).is_err());
        assert!(parse_startup(&[0], 1).is_err());
    }

    #[test]
    fn startup_quotes_unicode_and_spaces_and_rejects_disposable_paths() {
        let temp = Path::new(r"C:\Users\Demo\AppData\Local\Temp");
        let command = startup_command(Path::new(r"C:\Tools\Red Eye\rédẹye.exe"), temp).unwrap();
        assert_eq!(
            String::from_utf16(&command[..command.len() - 1]).unwrap(),
            "\"C:\\Tools\\Red Eye\\rédẹye.exe\""
        );
        for path in [
            r"C:\repo\target\debug\redeye.exe",
            r"C:\repo\target\x86_64-pc-windows-msvc\release\redeye.exe",
            r"c:\users\demo\appdata\local\temp\redeye.exe",
            r"\\?\C:\Users\Demo\AppData\Local\Temp\redeye.exe",
            r"redeye.exe",
        ] {
            assert!(startup_command(Path::new(path), temp).is_err());
        }
        assert!(
            startup_command(
                Path::new(r"C:\Users\Demo\AppData\Local\Temporary\redeye.exe"),
                temp
            )
            .is_ok()
        );
        let long_path = format!("C:\\{}\\redeye.exe", "a".repeat(250));
        assert!(startup_command(Path::new(&long_path), temp).is_err());
    }
}
