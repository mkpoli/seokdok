//! Reproduce how an application sees the input method. By default an IMM32
//! application (Qt, as in Telegram Desktop): a plain window that reads
//! compositions with ImmGetCompositionString the way Qt does, never touching
//! TSF itself. With `--tsf`, an application with a transitory context of its
//! own (Chromium, the console): a window that owns a TSF context, set up the
//! way Windows Terminal does it.
//!
//! Run on Windows with seokdok installed:
//!   imm_probe.exe [keys]        default keys "eocnd" (대충), log to imm_probe.log
//! It activates the seokdok profile for this process, types the keys with
//! SendInput while its own window is in the foreground (checked before every
//! key, and only after nobody has used the machine for a while) and logs every
//! IME message it receives. Options: `--native` (Hangul
//! mode first), `--ms` (Microsoft's Korean IME, as a control), `--dll=PATH`
//! (serve the text service from a build, for this process only), `--idle=SECS`.
//! With `--tsf`: `--refuse-start` (reject every composition) and
//! `--refuse-write=N` (end the composition right after its Nth write). In the
//! keys, `_` is Space, `~` is Enter and `.` waits one tick.

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    probe::run()
}

#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
mod probe {
    use seokdok::guids::{GUID_PROFILE, GUID_TEXT_SERVICE};
    use std::cell::{Cell, RefCell};
    use std::fmt::Write as _;
    use windows::core::{implement, w, Interface, Ref, BOOL};
    use windows::Win32::Foundation::{E_NOTIMPL, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoRegisterClassObject, IClassFactory, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, REGCLS_MULTIPLEUSE,
    };
    use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
    use windows::Win32::System::Variant::VARIANT;
    use windows::core::{GUID, HSTRING, PCSTR};
    use windows::Win32::UI::Input::Ime::{
        ImmGetCompositionStringW, ImmGetContext, ImmGetConversionStatus, ImmGetOpenStatus, ImmReleaseContext,
        ImmSetConversionStatus, ImmSetOpenStatus, GCS_COMPSTR, GCS_RESULTSTR, IME_COMPOSITION_STRING, IME_CONVERSION_MODE,
        IME_CMODE_NATIVE, IME_SENTENCE_MODE,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetLastInputInfo, SendInput, SetFocus, HKL, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, LASTINPUTINFO,
        VIRTUAL_KEY,
    };
    use windows::Win32::UI::TextServices::{
        CLSID_TF_InputProcessorProfiles, CLSID_TF_ThreadMgr, ITfCompositionView, ITfContext, ITfContextOwner,
        ITfContextOwnerCompositionServices, ITfContextOwnerCompositionSink, ITfContextOwnerCompositionSink_Impl,
        ITfContextOwner_Impl, ITfEditRecord, ITfEditSession, ITfEditSession_Impl, ITfInputProcessorProfileMgr, ITfRange,
        ITfSource, ITfTextEditSink, ITfTextEditSink_Impl, ITfThreadMgr, TF_ES_READ, TF_ES_SYNC,
        TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE, TF_IPPMF_FORPROCESS, TF_PROFILETYPE_INPUTPROCESSOR, TS_SS_NOHIDDENTEXT,
        TS_SS_TRANSITORY, TS_STATUS,
    };
    use windows::Win32::UI::WindowsAndMessaging::*;

    thread_local! {
        static LOG: RefCell<String> = RefCell::new(String::new());
        static TEXT: RefCell<String> = RefCell::new(String::new());
        /// The virtual keys this probe types; any other key is somebody's.
        static OURS: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
        /// `--tsf`: the context this window owns, and the client id to read it with.
        static CONTEXT: RefCell<Option<(ITfContext, u32)>> = const { RefCell::new(None) };
    }

    fn log(s: String) {
        LOG.with(|l| writeln!(l.borrow_mut(), "{s}").unwrap());
    }

    /// A document the way an application with its own TSF context holds one.
    /// TSF keeps the text; this object only answers for the window and watches.
    #[implement(ITfContextOwnerCompositionSink, ITfContextOwner, ITfTextEditSink)]
    struct Store {
        hwnd: HWND,
        refuse_start: bool,
        /// Writes left until the composition is ended; 0 never ends it.
        refuse_write: Cell<u32>,
        composing: Cell<bool>,
    }

    impl ITfContextOwnerCompositionSink_Impl for Store_Impl {
        fn OnStartComposition(&self, _view: Ref<'_, ITfCompositionView>) -> windows::core::Result<BOOL> {
            log(format!("store: OnStartComposition, {}", if self.refuse_start { "refused" } else { "accepted" }));
            self.composing.set(!self.refuse_start);
            Ok((!self.refuse_start).into())
        }
        fn OnUpdateComposition(&self, _view: Ref<'_, ITfCompositionView>, _range: Ref<'_, ITfRange>) -> windows::core::Result<()> {
            Ok(())
        }
        fn OnEndComposition(&self, _view: Ref<'_, ITfCompositionView>) -> windows::core::Result<()> {
            log("store: OnEndComposition".into());
            self.composing.set(false);
            Ok(())
        }
    }

    impl ITfTextEditSink_Impl for Store_Impl {
        fn OnEndEdit(&self, _pic: Ref<'_, ITfContext>, _ec: u32, _record: Ref<'_, ITfEditRecord>) -> windows::core::Result<()> {
            let left = self.refuse_write.get();
            if self.composing.get() && left > 0 {
                self.refuse_write.set(left - 1);
                if left == 1 {
                    // SAFETY: hwnd is this probe's window.
                    unsafe { PostMessageW(Some(self.hwnd), WM_APP, WPARAM(0), LPARAM(0))? };
                }
            }
            Ok(())
        }
    }

    impl ITfContextOwner_Impl for Store_Impl {
        fn GetACPFromPoint(&self, _pt: *const POINT, _flags: u32) -> windows::core::Result<i32> {
            Err(E_NOTIMPL.into())
        }
        fn GetTextExt(&self, _start: i32, _end: i32, rect: *mut RECT, clipped: *mut BOOL) -> windows::core::Result<()> {
            // SAFETY: TSF passes valid out pointers.
            unsafe {
                *rect = self.GetScreenExt()?;
                *clipped = false.into();
            }
            Ok(())
        }
        fn GetScreenExt(&self) -> windows::core::Result<RECT> {
            let mut rect = RECT::default();
            // SAFETY: hwnd is this probe's window; rect is a valid out pointer.
            unsafe { GetWindowRect(self.hwnd, &mut rect)? };
            Ok(rect)
        }
        fn GetStatus(&self) -> windows::core::Result<TS_STATUS> {
            Ok(TS_STATUS { dwDynamicFlags: 0, dwStaticFlags: TS_SS_TRANSITORY | TS_SS_NOHIDDENTEXT })
        }
        fn GetWnd(&self) -> windows::core::Result<HWND> {
            Ok(self.hwnd)
        }
        fn GetAttribute(&self, _guid: *const GUID) -> windows::core::Result<VARIANT> {
            Ok(VARIANT::default())
        }
    }

    /// Reads the whole document.
    #[implement(ITfEditSession)]
    struct Reader {
        context: ITfContext,
    }

    impl ITfEditSession_Impl for Reader_Impl {
        fn DoEditSession(&self, ec: u32) -> windows::core::Result<()> {
            // SAFETY: ec is valid for this session; the buffer outlives GetText.
            unsafe {
                let range = self.context.GetStart(ec)?;
                let mut shifted = 0;
                range.ShiftEnd(ec, 4096, &mut shifted, std::ptr::null())?;
                let mut buf = [0u16; 4096];
                let mut len = 0;
                range.GetText(ec, 0, &mut buf, &mut len)?;
                log(format!("document {:?}", String::from_utf16_lossy(&buf[..len as usize])));
            }
            Ok(())
        }
    }

    fn log_document() {
        CONTEXT.with(|c| {
            if let Some((context, client)) = c.borrow().as_ref() {
                let session: ITfEditSession = Reader { context: context.clone() }.into();
                // SAFETY: context and session are valid.
                let _ = unsafe { context.RequestEditSession(*client, &session, TF_ES_SYNC | TF_ES_READ) };
            }
        });
    }

    /// Give `hwnd` a transitory context of its own, as Windows Terminal does
    /// in src/tsf/Implementation.cpp.
    unsafe fn own_a_context(hwnd: HWND, refuse_start: bool, refuse_write: u32) -> windows::core::Result<()> {
        let manager: ITfThreadMgr = CoCreateInstance(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER)?;
        let client = manager.Activate()?;
        let documents = manager.CreateDocumentMgr()?;
        let store: ITfContextOwnerCompositionSink =
            Store { hwnd, refuse_start, refuse_write: Cell::new(refuse_write), composing: Cell::new(false) }.into();
        let (mut context, mut cookie) = (None, 0);
        documents.CreateContext(client, 0, &store, &mut context, &mut cookie)?;
        let context = context.ok_or(windows::core::Error::from(E_NOTIMPL))?;
        let source: ITfSource = context.cast()?;
        source.AdviseSink(&ITfContextOwner::IID, &store)?;
        source.AdviseSink(&ITfTextEditSink::IID, &store)?;
        documents.Push(&context)?;
        manager.AssociateFocus(hwnd, &documents).ok();
        manager.SetFocus(&documents)?;
        CONTEXT.with(|c| *c.borrow_mut() = Some((context, client)));
        // Stays until the process exits.
        std::mem::forget((manager, documents, store));
        log("window owns a transitory context".into());
        Ok(())
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
            WM_KEYDOWN | WM_KEYUP | WM_IME_KEYDOWN | WM_IME_KEYUP | WM_IME_CHAR => {
                let name = match msg { WM_KEYDOWN => "WM_KEYDOWN", WM_KEYUP => "WM_KEYUP", WM_IME_KEYDOWN => "WM_IME_KEYDOWN", WM_IME_KEYUP => "WM_IME_KEYUP", _ => "WM_IME_CHAR" };
                log(format!("{name} {:#x}", wp.0));
                if msg == WM_KEYUP && !OURS.with(|o| o.borrow().contains(&(wp.0 as u16))) {
                    log("a key this probe did not type: somebody is using the machine; stopped".into());
                    unsafe { DestroyWindow(hwnd).ok() };
                    return LRESULT(0);
                }
                unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
            }
            WM_APP => {
                CONTEXT.with(|c| {
                    if let Some((context, _)) = c.borrow().as_ref() {
                        let services: windows::core::Result<ITfContextOwnerCompositionServices> = context.cast();
                        // SAFETY: services is a valid interface of the context.
                        let r = services.and_then(|s| unsafe { s.TerminateComposition(None) });
                        log(format!("store: ended the composition: {r:?}"));
                    }
                });
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

    /// The window takes the foreground, so whatever somebody is typing would
    /// land in it. Wait until nobody has touched the keyboard or mouse for
    /// `secs`; give up after five minutes.
    fn wait_for_idle(secs: u32) -> bool {
        let last = || {
            let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
            // SAFETY: info is a valid LASTINPUTINFO with cbSize set.
            let _ = unsafe { GetLastInputInfo(&mut info) };
            info.dwTime
        };
        let (mut seen, mut quiet) = (last(), 0);
        for _ in 0..300 {
            if quiet >= secs {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            let now = last();
            quiet = if now == seen { quiet + 1 } else { 0 };
            seen = now;
        }
        false
    }

    fn vk_of(ch: char) -> u16 {
        match ch {
            '~' => 0x0D,
            '_' => 0x20,
            c => c.to_ascii_uppercase() as u16,
        }
    }

    pub fn run() -> windows::core::Result<()> {
        let keys: String = std::env::args().nth(1).unwrap_or_else(|| "eocnd".into());
        OURS.with(|o| *o.borrow_mut() = keys.chars().filter(|c| *c != '.').map(vk_of).collect());
        // `--idle=SECS`: how long nobody may have touched the machine before
        // the window takes the foreground (default 20).
        let idle = std::env::args().find_map(|a| a.strip_prefix("--idle=").and_then(|v| v.parse().ok())).unwrap_or(20);
        if !wait_for_idle(idle) {
            println!("the machine stayed busy; nothing was typed");
            return Ok(());
        }
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
            if std::env::args().any(|a| a == "--tsf") {
                let refuse_start = std::env::args().any(|a| a == "--refuse-start");
                let refuse_write = std::env::args().find_map(|a| a.strip_prefix("--refuse-write=").and_then(|v| v.parse().ok())).unwrap_or(0);
                own_a_context(hwnd, refuse_start, refuse_write)?;
            }
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
                    log_document();
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
                            let vk = vk_of(ch);
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
