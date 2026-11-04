//! Opens the settings app, `settings\SeokdokSettings.exe` beside the DLL, for
//! the input indicator's menu and the Options button of the language settings.
//!
//! The DLL runs inside whatever application is being typed in, so the app is
//! its own process: it inherits nothing from the host and outlives it, and this
//! code never touches the window focus. A settings change reaches the host
//! through config.toml, at its next key.

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED};
use windows::Win32::Globalization::GetUserDefaultUILanguage;
use windows::Win32::System::Threading::{
    CreateProcessW, CREATE_BREAKAWAY_FROM_JOB, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};

/// LANG_KOREAN.
const KOREAN: u16 = 0x12;

fn wide(text: &std::ffi::OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}

fn app_path() -> Option<PathBuf> {
    Some(crate::dll::dll_dir()?.join("settings").join("SeokdokSettings.exe"))
}

fn start(exe: &Path, flags: PROCESS_CREATION_FLAGS) -> windows::core::Result<()> {
    let application = wide(exe.as_os_str());
    let mut command = wide(format!("\"{}\"", exe.display()).as_ref());
    let folder = wide(exe.parent().unwrap_or(exe).as_os_str());
    let startup = STARTUPINFOW { cb: std::mem::size_of::<STARTUPINFOW>() as u32, ..Default::default() };
    let mut process = PROCESS_INFORMATION::default();
    // SAFETY: the strings are NUL-terminated and outlive the call; `command` is
    // writable, as CreateProcessW asks; `process` is closed below.
    unsafe {
        CreateProcessW(
            PCWSTR(application.as_ptr()),
            Some(PWSTR(command.as_mut_ptr())),
            None,
            None,
            false,
            flags,
            None,
            PCWSTR(folder.as_ptr()),
            &startup,
            &mut process,
        )?;
        let _ = CloseHandle(process.hThread);
        let _ = CloseHandle(process.hProcess);
    }
    Ok(())
}

/// Open the app. A problem is reported from a thread of its own, so the host's
/// window keeps its message loop.
pub fn open() {
    let Some(exe) = app_path() else { return report() };
    if !exe.exists() {
        crate::debug_log::log(|| format!("settings app not found: {}", exe.display()));
        return report();
    }
    // A job that closes with the host would take the window with it. Some jobs
    // refuse to let a process leave; it then runs inside.
    let started = match start(&exe, CREATE_BREAKAWAY_FROM_JOB) {
        Err(e) if e.code() == ERROR_ACCESS_DENIED.to_hresult() => start(&exe, PROCESS_CREATION_FLAGS(0)),
        other => other,
    };
    if let Err(e) = started {
        crate::debug_log::log(|| format!("settings app did not start: {e}"));
        report();
    }
}

fn report() {
    crate::lock_module();
    std::thread::spawn(|| {
        // SAFETY: no pointers; asks for the language of the user interface.
        let korean = unsafe { GetUserDefaultUILanguage() } & 0x3ff == KOREAN;
        let (title, text) = if korean {
            ("釋讀 設定", "設定 앱을 찾지 못했습니다. 釋讀을 다시 設置해 주세요.")
        } else {
            ("Seokdok settings", "The settings app could not be opened. Please reinstall Seokdok.")
        };
        let (title, text) = (wide(title.as_ref()), wide(text.as_ref()));
        // SAFETY: the strings are NUL-terminated and outlive the call.
        unsafe { MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONWARNING) };
        crate::unlock_module();
    });
}
