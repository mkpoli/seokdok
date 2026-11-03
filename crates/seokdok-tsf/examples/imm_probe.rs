//! Reproduce how an IMM32 application (Qt, as in Telegram Desktop) sees the
//! input method: a plain window that reads compositions with ImmGetCompositionString
//! the way Qt does, never touching TSF itself.
//!
//! Run on Windows with seokdok installed:
//!   imm_probe.exe [keys]        default keys "eocnd" (대충), log to imm_probe.log
//! It activates the seokdok profile for this process, types the keys with
//! SendInput while its own window is in the foreground (checked before every
//! key) and logs every IME message it receives. Options: `--native` (Hangul
//! mode first), `--ms` (Microsoft's Korean IME, as a control), `--dll=PATH`
//! (serve the text service from a build, for this process only). In the keys,
//! `_` is Space, `~` is Enter and `.` waits one tick.

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    probe::run()
}

#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
mod probe {
    use seokdok::guids::{GUID_PROFILE, GUID_TEXT_SERVICE};
    use std::cell::RefCell;
    use std::fmt::Write as _;
    use windows::core::{w, Interface};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoRegisterClassObject, IClassFactory, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, REGCLS_MULTIPLEUSE,
    };
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
    use windows::core::{GUID, HSTRING, PCSTR};
    use windows::Win32::UI::Input::Ime::{
        ImmGetCompositionStringW, ImmGetContext, ImmGetConversionStatus, ImmGetOpenStatus, ImmReleaseContext,
        ImmSetConversionStatus, ImmSetOpenStatus, GCS_COMPSTR, GCS_RESULTSTR, IME_COMPOSITION_STRING, IME_CONVERSION_MODE,
        IME_CMODE_NATIVE, IME_SENTENCE_MODE,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, SetFocus, HKL, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    };
    use windows::Win32::UI::TextServices::{
        CLSID_TF_InputProcessorProfiles, ITfInputProcessorProfileMgr, TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE,
        TF_IPPMF_FORPROCESS, TF_PROFILETYPE_INPUTPROCESSOR,
    };
    use windows::Win32::UI::WindowsAndMessaging::*;

    thread_local! {
        static LOG: RefCell<String> = RefCell::new(String::new());
        static TEXT: RefCell<String> = RefCell::new(String::new());
    }

    fn log(s: String) {
        LOG.with(|l| writeln!(l.borrow_mut(), "{s}").unwrap());
    }

    fn comp_string(hwnd: HWND, kind: IME_COMPOSITION_STRING) -> String {
        unsafe {
            let himc = ImmGetContext(hwnd);
            let n = ImmGetCompositionStringW(himc, kind, None, 0);
            let mut buf = vec![0u16; (n.max(0) as usize) / 2];
            if n > 0 {
                ImmGetCompositionStringW(himc, kind, Some(buf.as_mut_ptr() as *mut _), n as u32);
            }
            let _ = ImmReleaseContext(hwnd, himc);
            String::from_utf16_lossy(&buf)
        }
    }

    extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        match msg {
            WM_IME_STARTCOMPOSITION => {
                log("WM_IME_STARTCOMPOSITION".into());
                LRESULT(0)
            }
            WM_IME_COMPOSITION => {
                let flags = lp.0 as u32;
                let mut line = format!("WM_IME_COMPOSITION flags={flags:#x}");
                if flags & GCS_RESULTSTR.0 != 0 {
                    let r = comp_string(hwnd, GCS_RESULTSTR);
                    TEXT.with(|t| t.borrow_mut().push_str(&r));
                    let _ = write!(line, " result={r:?}");
                }
                if flags & GCS_COMPSTR.0 != 0 {
                    let _ = write!(line, " comp={:?}", comp_string(hwnd, GCS_COMPSTR));
                }
                log(line);
                LRESULT(0)
            }
            WM_IME_ENDCOMPOSITION => {
                log("WM_IME_ENDCOMPOSITION".into());
                LRESULT(0)
            }
            WM_CHAR => {
                let c = char::from_u32(wp.0 as u32).unwrap_or('?');
                TEXT.with(|t| t.borrow_mut().push(c));
                log(format!("WM_CHAR {c:?}"));
                LRESULT(0)
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
        }
    }

    fn key(vk: u16, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk),
                    dwFlags: if up { KEYEVENTF_KEYUP } else { Default::default() },
                    ..Default::default()
                },
            },
        }
    }

    pub fn run() -> windows::core::Result<()> {
        let keys: String = std::env::args().nth(1).unwrap_or_else(|| "eocnd".into());
        // `--native`: put the context in Hangul mode first, as an IMM32 app may.
        let native = std::env::args().any(|a| a == "--native");
        // `--dll=PATH`: serve the text service from this build instead of the
        // installed one, for this process only (no registration, no admin).
        let dll = std::env::args().find_map(|a| a.strip_prefix("--dll=").map(String::from));
        // `--ms`: Microsoft's Korean IME instead, as a control.
        let (clsid, profile) = if std::env::args().any(|a| a == "--ms") {
            (GUID::from_u128(0xA028AE76_01B1_46C2_99C4_ACD9858AE02F), GUID::from_u128(0xB5FE1F02_D5F2_4445_9C03_C568F23C99A1))
        } else {
            (GUID_TEXT_SERVICE, GUID_PROFILE)
        };
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            if let Some(path) = &dll {
                let h = LoadLibraryW(&HSTRING::from(path.as_str()))?;
                type GetClassObject = unsafe extern "system" fn(*const GUID, *const GUID, *mut *mut core::ffi::c_void) -> windows::core::HRESULT;
                let f: GetClassObject = std::mem::transmute(GetProcAddress(h, PCSTR(b"DllGetClassObject\0".as_ptr())).expect("DllGetClassObject"));
                let mut out = std::ptr::null_mut();
                f(&GUID_TEXT_SERVICE, &IClassFactory::IID, &mut out).ok()?;
                let factory = IClassFactory::from_raw(out);
                let cookie = CoRegisterClassObject(&GUID_TEXT_SERVICE, &factory, CLSCTX_INPROC_SERVER, REGCLS_MULTIPLEUSE)?;
                log(format!("serving the text service from {path} (cookie {cookie})"));
                std::mem::forget(factory);
            }
            let mgr: ITfInputProcessorProfileMgr =
                CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
            let r = mgr.ActivateProfile(
                TF_PROFILETYPE_INPUTPROCESSOR,
                0x0412,
                &clsid,
                &profile,
                HKL::default(),
                TF_IPPMF_FORPROCESS | TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE,
            );
            log(format!("ActivateProfile: {r:?}"));
            let _ = mgr.cast::<windows::core::IUnknown>();

            let hinst = GetModuleHandleW(None)?;
            let class = w!("seokdok_imm_probe");
            RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst.into(), lpszClassName: class, ..Default::default() });
            let hwnd = CreateWindowExW(
                Default::default(), class, w!("seokdok IMM probe"), WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                100, 100, 400, 200, None, None, Some(hinst.into()), None,
            )?;
            let _ = SetForegroundWindow(hwnd);
            let _ = SetFocus(Some(hwnd));
            log(format!("foreground ok: {}", GetForegroundWindow() == hwnd));
            log(format!("keyboard layout: {:?}", windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout(0)));

            // One key per timer tick, so the message loop never blocks; `.`
            // skips a tick (to let the dictionaries load). Before every key the
            // window must still be in the foreground: SendInput goes to the
            // focused window, and a key must never reach another application.
            SetTimer(Some(hwnd), 1, 150, None);
            let mut msg = MSG::default();
            let mut queue: std::collections::VecDeque<char> = keys.chars().collect();
            let mut started = false;
            let mut idle = 0;
            while GetMessageW(&mut msg, None, 0, 0).into() {
                if msg.message == WM_TIMER {
                    if !started {
                        started = true;
                        let himc = ImmGetContext(hwnd);
                        let (mut conv, mut sent) = (IME_CONVERSION_MODE(0), IME_SENTENCE_MODE(0));
                        let _ = ImmGetConversionStatus(himc, Some(&mut conv), Some(&mut sent));
                        log(format!("before: open={} conversion={:#x}", ImmGetOpenStatus(himc).as_bool(), conv.0));
                        if native {
                            let _ = ImmSetOpenStatus(himc, true);
                            let _ = ImmSetConversionStatus(himc, IME_CMODE_NATIVE, sent);
                        }
                        let _ = ImmReleaseContext(hwnd, himc);
                        continue;
                    }
                    match queue.pop_front() {
                        Some(ch) => {
                            if GetForegroundWindow() != hwnd {
                                log("lost the foreground; stopped typing".into());
                                queue.clear();
                                let _ = DestroyWindow(hwnd);
                                continue;
                            }
                            if ch == '.' {
                                continue;
                            }
                            // `~` is Enter, `_` is Space. TSF reads real input
                            // only (posted WM_KEYDOWN bypasses it), hence SendInput
                            // and the foreground check before every key.
                            let vk = match ch { '~' => 0x0D, '_' => 0x20, c => c.to_ascii_uppercase() as u16 };
                            SendInput(&[key(vk, false), key(vk, true)], std::mem::size_of::<INPUT>() as i32);
                        }
                        None => {
                            idle += 1;
                            if idle > 6 {
                                let _ = DestroyWindow(hwnd);
                            }
                        }
                    }
                    continue;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        let text = TEXT.with(|t| t.borrow().clone());
        log(format!("TEXT {text:?}"));
        let out = LOG.with(|l| l.borrow().clone());
        std::fs::write("imm_probe.log", &out).ok();
        print!("{out}");
        Ok(())
    }
}
