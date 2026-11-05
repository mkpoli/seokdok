//! The input-indicator button on the taskbar: its icon shows 釋, 混, 한 or A, a
//! left click toggles the input mode, and a right click opens the quick menu
//! ([`crate::quick_menu`]).
//!
//! Since Windows 8 only an item whose GUID is `GUID_LBI_INPUTMODE` is shown,
//! and only the first such item, so there is one button (see
//! `docs/mode-keys.md`). `InitMenu` is for `TF_LBI_STYLE_BTN_MENU` items; this
//! is a plain button that draws its own popup on a right click, as Mozc does.
//!
//! One COM object implements the langbar interfaces (`ITfLangBarItemButton`,
//! its base `ITfLangBarItem`, and `ITfSource` for the shell's update sink).
//! The mode is shared with the text service through [`ModeShared`]: a click
//! records a request that the key handler applies at the next key (it needs a
//! document context to commit the composition), and the text service shows
//! mode changes made from the keyboard.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows::core::{implement, w, IUnknown, Interface, Ref, BSTR, GUID, PCWSTR, PWSTR};
use windows::Win32::Foundation::{E_INVALIDARG, E_NOINTERFACE, HINSTANCE, POINT, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
use windows::Win32::UI::TextServices::{
    ITfLangBarItem, ITfLangBarItemButton, ITfLangBarItemButton_Impl, ITfLangBarItemSink,
    ITfLangBarItem_Impl, ITfMenu, ITfSource, ITfSource_Impl, TfLBIClick, GUID_LBI_INPUTMODE,
    TF_LANGBARITEMINFO, TF_LBI_CLK_RIGHT, TF_LBI_ICON, TF_LBI_STYLE_BTN_BUTTON,
    TF_LBI_STYLE_SHOWNINTRAY, TF_LBI_TEXT, TF_LBI_TOOLTIP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExW, DestroyMenu, DestroyWindow, GetSystemMetrics, InsertMenuItemW,
    LoadImageW, SetForegroundWindow, TrackPopupMenu, HICON, HMENU, IMAGE_ICON, LR_CREATEDIBSECTION, MENUITEMINFOW,
    MENU_ITEM_STATE, MENU_ITEM_TYPE, MFS_CHECKED, MFS_DISABLED, MFT_RADIOCHECK, MFT_SEPARATOR, MFT_STRING,
    MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, SM_CYSMICON, TPM_LEFTALIGN, TPM_NONOTIFY, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TPM_TOPALIGN, WS_POPUP,
};

use crate::config::Script;
use crate::dll_instance;
use crate::guids::{
    GUID_TEXT_SERVICE, IDI_MODE_HANGUL, IDI_MODE_LATN, IDI_MODE_SEOKDOK, IDI_MODE_SINO,
};
use crate::quick_menu::{self, Action, Check, Entry, Indicator};
use crate::session::Mode;

/// Mode state shared by the button and the text service.
pub struct ModeShared {
    shown: Cell<Mode>,
    requested: Cell<Option<Mode>>,
    script: Cell<Script>,
    sink: RefCell<Option<ITfLangBarItemSink>>,
}

impl ModeShared {
    pub fn new(mode: Mode) -> Self {
        ModeShared { shown: Cell::new(mode), requested: Cell::new(None), script: Cell::new(Script::default()), sink: RefCell::new(None) }
    }

    pub fn shown(&self) -> Mode {
        self.shown.get()
    }

    fn indicator(&self) -> Indicator {
        Indicator::of(self.shown.get(), self.script.get())
    }

    /// `conversion.script`; the button repaints at once.
    pub fn set_script(&self, script: Script) {
        if self.script.replace(script) != script {
            self.show(self.shown.get());
        }
    }

    /// Display `mode` (the text service changed it) and repaint the button.
    pub fn show(&self, mode: Mode) {
        // 韓㐎 專用 has no 釋讀; the session reports it the same way.
        let mode = if self.script.get() == Script::HangulOnly && mode == Mode::Seokdok { Mode::Hangul } else { mode };
        self.shown.set(mode);
        if let Some(sink) = self.sink.borrow().as_ref() {
            // SAFETY: a valid sink advised via AdviseSink.
            unsafe {
                let _ = sink.OnUpdate(TF_LBI_ICON | TF_LBI_TEXT | TF_LBI_TOOLTIP);
            }
        }
    }

    /// The user picked `mode` on the button.
    fn request(&self, mode: Mode) {
        self.requested.set(Some(mode));
        self.show(mode);
    }

    pub fn has_request(&self) -> bool {
        self.requested.get().is_some()
    }

    /// Take a pending request from the button.
    pub fn take_request(&self) -> Option<Mode> {
        self.requested.take()
    }

    /// Do what a menu entry says. A setting goes to the file at once, where
    /// other applications find it; the session here takes it at the next key.
    fn choose(&self, action: Action) {
        if action == Action::Settings {
            crate::settings_app::open();
            return;
        }
        match crate::config::update(|c| action.change(c)) {
            Ok(config) => {
                self.set_script(config.conversion.script);
                crate::dll::apply_engine_options();
                if let Some(mode) = action.mode() {
                    self.request(mode);
                }
            }
            Err(e) => crate::debug_log::log(|| format!("menu: config.toml left as it is: {e}")),
        }
    }
}

/// Put `entries` in a popup menu.
///
/// # Safety
/// `menu` must be a valid popup menu.
unsafe fn fill_menu(menu: HMENU, entries: &[Entry]) -> windows::core::Result<()> {
    for (at, entry) in entries.iter().enumerate() {
        let mut info = MENUITEMINFOW { cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32, fMask: MIIM_FTYPE, ..Default::default() };
        let mut text = Vec::new();
        // A tab right-aligns what follows, the way menus show shortcuts.
        let mut set_text = |info: &mut MENUITEMINFOW, label: &str, hint: &str, ftype: MENU_ITEM_TYPE, state: MENU_ITEM_STATE| {
            let shown = if hint.is_empty() { label.to_string() } else { format!("{label}\t{hint}") };
            text = shown.encode_utf16().chain([0]).collect();
            info.fMask |= MIIM_STRING | MIIM_STATE;
            info.fType = ftype;
            info.fState = state;
            info.dwTypeData = PWSTR(text.as_mut_ptr());
        };
        match entry {
            Entry::Separator => info.fType = MFT_SEPARATOR,
            Entry::Heading { label, hint } => set_text(&mut info, label, hint, MFT_STRING, MFS_DISABLED),
            Entry::Choice { action, label, check, hint } => {
                let (ftype, state) = match *check {
                    Check::Radio(on) => (MFT_STRING | MFT_RADIOCHECK, if on { MFS_CHECKED } else { MENU_ITEM_STATE(0) }),
                    Check::Box(on) => (MFT_STRING, if on { MFS_CHECKED } else { MENU_ITEM_STATE(0) }),
                    Check::Plain => (MFT_STRING, MENU_ITEM_STATE(0)),
                };
                set_text(&mut info, label, hint, ftype, state);
                info.fMask |= MIIM_ID;
                info.wID = action.id();
            }
        }
        InsertMenuItemW(menu, at as u32, true, &info)?;
    }
    Ok(())
}

/// Show the menu at `point` until the user chooses or dismisses it.
fn pick(entries: &[Entry], point: POINT) -> Option<Action> {
    // SAFETY: the menu and the window are created and destroyed here.
    unsafe {
        let menu = CreatePopupMenu().ok()?;
        let filled = fill_menu(menu, entries);
        // The owner must belong to this thread. With nothing focused here,
        // a hidden window stands in for the time of the menu. It has to be the
        // foreground window, or a click elsewhere does not close the menu.
        let focused = GetFocus();
        let stand_in = if focused.is_invalid() {
            CreateWindowExW(Default::default(), w!("STATIC"), PCWSTR::null(), WS_POPUP, 0, 0, 0, 0, None, None, Some(dll_instance().into()), None).ok()
        } else {
            None
        };
        if let Some(hwnd) = stand_in {
            let _ = SetForegroundWindow(hwnd);
        }
        // Keep the menu's left edge inside the work area.
        let mut x = point.x;
        let mut monitor = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST), &mut monitor).as_bool() {
            x = x.clamp(monitor.rcWork.left, monitor.rcWork.right);
        }
        // TPM_NONOTIFY keeps the owner, an application's window, out of it.
        let flags = TPM_NONOTIFY | TPM_RETURNCMD | TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RIGHTBUTTON;
        let id = if filled.is_ok() {
            TrackPopupMenu(menu, flags, x, point.y, None, stand_in.unwrap_or(focused), None).0 as u32
        } else {
            0
        };
        let _ = DestroyMenu(menu);
        if let Some(hwnd) = stand_in {
            let _ = DestroyWindow(hwnd);
        }
        Action::from_id(id)
    }
}

#[implement(ITfLangBarItemButton, ITfLangBarItem, ITfSource)]
pub struct ModeButton {
    shared: Rc<ModeShared>,
}

impl ModeButton {
    pub fn new(shared: Rc<ModeShared>) -> Self {
        Self { shared }
    }
}

impl ITfLangBarItem_Impl for ModeButton_Impl {
    fn GetInfo(&self, pinfo: *mut TF_LANGBARITEMINFO) -> windows::core::Result<()> {
        if pinfo.is_null() {
            return Err(E_INVALIDARG.into());
        }
        let mut desc = [0u16; 32];
        for (slot, c) in desc.iter_mut().zip(seokdok_product::CURRENT.id.encode_utf16()) {
            *slot = c;
        }
        // SAFETY: pinfo checked non-null; the struct is fully overwritten.
        unsafe {
            *pinfo = TF_LANGBARITEMINFO {
                clsidService: GUID_TEXT_SERVICE,
                guidItem: GUID_LBI_INPUTMODE,
                dwStyle: TF_LBI_STYLE_BTN_BUTTON | TF_LBI_STYLE_SHOWNINTRAY,
                ulSort: 0,
                szDescription: desc,
            };
        }
        Ok(())
    }

    fn GetStatus(&self) -> windows::core::Result<u32> {
        Ok(0)
    }

    fn Show(&self, _fshow: windows::core::BOOL) -> windows::core::Result<()> {
        Ok(())
    }

    fn GetTooltipString(&self) -> windows::core::Result<BSTR> {
        Ok(BSTR::from(self.shared.indicator().tooltip().as_str()))
    }
}

impl ITfLangBarItemButton_Impl for ModeButton_Impl {
    fn OnClick(&self, click: TfLBIClick, pt: &POINT, _prcarea: *const RECT) -> windows::core::Result<()> {
        if click == TF_LBI_CLK_RIGHT {
            // A check mark must show what the file says now: choosing an entry
            // flips the value in the file.
            crate::config::refresh();
            let entries = quick_menu::entries(&crate::config::current(), self.shared.shown(), japanese_keyboard());
            if let Some(action) = pick(&entries, *pt) {
                self.shared.choose(action);
            }
        } else {
            self.shared.request(quick_menu::click_target(self.shared.shown(), self.shared.script.get()));
        }
        Ok(())
    }

    fn InitMenu(&self, _pmenu: Ref<'_, ITfMenu>) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnMenuSelect(&self, _wid: u32) -> windows::core::Result<()> {
        Ok(())
    }

    fn GetIcon(&self) -> windows::core::Result<HICON> {
        let id = match self.shared.indicator() {
            Indicator::Seokdok => IDI_MODE_SEOKDOK,
            Indicator::Mixed => IDI_MODE_SINO,
            Indicator::Hangul => IDI_MODE_HANGUL,
            Indicator::Latin => IDI_MODE_LATN,
        };
        let hinst = HINSTANCE(dll_instance().0);
        // SAFETY: hinst is this DLL's module; the id is a valid icon resource.
        // Not LR_SHARED: the caller destroys the icon.
        unsafe {
            let size = GetSystemMetrics(SM_CYSMICON);
            let icon = LoadImageW(Some(hinst), PCWSTR(id as usize as *const u16), IMAGE_ICON, size, size, LR_CREATEDIBSECTION)?;
            Ok(HICON(icon.0))
        }
    }

    fn GetText(&self) -> windows::core::Result<BSTR> {
        Ok(BSTR::from(self.shared.indicator().glyph()))
    }
}

impl ITfSource_Impl for ModeButton_Impl {
    fn AdviseSink(&self, riid: *const GUID, punk: Ref<'_, IUnknown>) -> windows::core::Result<u32> {
        // SAFETY: riid is a valid GUID pointer from the caller.
        if unsafe { riid.as_ref() } != Some(&ITfLangBarItemSink::IID) {
            return Err(E_NOINTERFACE.into());
        }
        let sink: ITfLangBarItemSink = punk.ok()?.cast()?;
        *self.shared.sink.borrow_mut() = Some(sink);
        Ok(1)
    }

    fn UnadviseSink(&self, _dwcookie: u32) -> windows::core::Result<()> {
        *self.shared.sink.borrow_mut() = None;
        Ok(())
    }
}

/// Whether the keyboard is a Japanese one (type 7), whose 半角/全角 and 変換 keys
/// the menu's hints may name.
fn japanese_keyboard() -> bool {
    // SAFETY: no pointers; 0 asks for the keyboard type.
    unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardType(0) == 7 }
}
