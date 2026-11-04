//! Language-bar mode button: 釋讀 conversion, plain Hangul, and Latin
//! pass-through.
//!
//! One COM object implements the langbar interfaces (`ITfLangBarItemButton`,
//! its base `ITfLangBarItem`, and `ITfSource` for the shell's update sink).
//! The mode is shared with the text service through [`ModeShared`]: a click
//! records a request that the key handler applies at the next key (it needs a
//! document context to commit the composition), and the text service shows
//! mode changes made from the keyboard.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows::core::{implement, IUnknown, Interface, Ref, BSTR, GUID, PCWSTR};
use windows::Win32::Foundation::{E_INVALIDARG, E_NOINTERFACE, HINSTANCE, POINT, RECT};
use windows::Win32::Graphics::Gdi::HBITMAP;
use windows::Win32::UI::TextServices::{
    ITfLangBarItem, ITfLangBarItemButton, ITfLangBarItemButton_Impl, ITfLangBarItemSink,
    ITfLangBarItem_Impl, ITfMenu, ITfSource, ITfSource_Impl, TfLBIClick, TF_LANGBARITEMINFO,
    TF_LBI_ICON, TF_LBI_STYLE_BTN_MENU, TF_LBI_TEXT, TF_LBI_TOOLTIP,
};
use windows::Win32::UI::WindowsAndMessaging::{LoadIconW, HICON};

use crate::dll_instance;
use crate::guids::{
    GUID_LANGBAR_ITEM, GUID_TEXT_SERVICE, IDI_MODE_HANGUL, IDI_MODE_LATN, IDI_MODE_SEOKDOK,
};
use crate::session::Mode;

const MENU_SEOKDOK: u32 = 1;
const MENU_HANGUL: u32 = 2;
const MENU_LATIN: u32 = 3;
const MENU_SETTINGS: u32 = 100;
const TF_LBMENUF_SEPARATOR: u32 = 0x0000_0004;
const TF_LBMENUF_RADIOCHECKED: u32 = 0x0000_0008;

/// Mode state shared by the button and the text service.
pub struct ModeShared {
    shown: Cell<Mode>,
    requested: Cell<Option<Mode>>,
    /// 韓㐎 專用: the button never shows 釋讀.
    hangul_only: Cell<bool>,
    sink: RefCell<Option<ITfLangBarItemSink>>,
}

impl ModeShared {
    pub fn new(mode: Mode) -> Self {
        ModeShared { shown: Cell::new(mode), requested: Cell::new(None), hangul_only: Cell::new(false), sink: RefCell::new(None) }
    }

    pub fn shown(&self) -> Mode {
        self.shown.get()
    }

    /// 韓㐎 專用 on or off; the button repaints at once.
    pub fn set_hangul_only(&self, v: bool) {
        if self.hangul_only.replace(v) != v {
            self.show(self.shown.get());
        }
    }

    /// Display `mode` (the text service changed it) and repaint the button.
    pub fn show(&self, mode: Mode) {
        let mode = if self.hangul_only.get() && mode == Mode::Seokdok { Mode::Hangul } else { mode };
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

    /// Take a pending request from the button.
    pub fn take_request(&self) -> Option<Mode> {
        self.requested.take()
    }
}

/// # Safety
/// `menu` must be a valid `ITfMenu` (supplied by the framework in `InitMenu`).
unsafe fn add_menu_item(menu: &ITfMenu, id: u32, flags: u32, text: &str) -> windows::core::Result<()> {
    let wide: Vec<u16> = text.encode_utf16().collect();
    menu.AddMenuItem(id, flags, HBITMAP::default(), HBITMAP::default(), &wide, std::ptr::null_mut())
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
        for (slot, c) in desc.iter_mut().zip("seokdok".encode_utf16()) {
            *slot = c;
        }
        // SAFETY: pinfo checked non-null; the struct is fully overwritten.
        unsafe {
            *pinfo = TF_LANGBARITEMINFO {
                clsidService: GUID_TEXT_SERVICE,
                guidItem: GUID_LANGBAR_ITEM,
                dwStyle: TF_LBI_STYLE_BTN_MENU,
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
        Ok(BSTR::from(match self.shared.shown() {
            Mode::Seokdok => "釋讀 (seokdok): 釋讀 conversion",
            Mode::Hangul => "釋讀 (seokdok): 한글 Hangul",
            Mode::Latin => "釋讀 (seokdok): A Latin",
        }))
    }
}

impl ITfLangBarItemButton_Impl for ModeButton_Impl {
    fn OnClick(&self, _click: TfLBIClick, _pt: &POINT, _prcarea: *const RECT) -> windows::core::Result<()> {
        // A click toggles between conversion and plain Hangul; Latin is
        // reached from the menu or the Han/Eng key.
        // With 韓㐎 專用 on there is no conversion to toggle: it switches
        // Hangul and Latin.
        let next = match (self.shared.hangul_only.get(), self.shared.shown()) {
            (true, Mode::Latin) => Mode::Hangul,
            (true, _) => Mode::Latin,
            (false, Mode::Seokdok) => Mode::Hangul,
            (false, _) => Mode::Seokdok,
        };
        self.shared.request(next);
        Ok(())
    }

    fn InitMenu(&self, pmenu: Ref<'_, ITfMenu>) -> windows::core::Result<()> {
        let menu = pmenu.ok()?;
        let flag = |m: Mode| if self.shared.shown() == m { TF_LBMENUF_RADIOCHECKED } else { 0 };
        // SAFETY: `menu` is a valid ITfMenu provided by the framework.
        unsafe {
            if !self.shared.hangul_only.get() {
                add_menu_item(menu, MENU_SEOKDOK, flag(Mode::Seokdok), "釋讀 Seokdok")?;
            }
            add_menu_item(menu, MENU_HANGUL, flag(Mode::Hangul), "한글 Hangul")?;
            add_menu_item(menu, MENU_LATIN, flag(Mode::Latin), "A Latin")?;
            add_menu_item(menu, 0, TF_LBMENUF_SEPARATOR, "")?;
            add_menu_item(menu, MENU_SETTINGS, 0, "設定 / 설정 / Settings…")?;
        }
        Ok(())
    }

    fn OnMenuSelect(&self, wid: u32) -> windows::core::Result<()> {
        match wid {
            MENU_SEOKDOK => self.shared.request(Mode::Seokdok),
            MENU_HANGUL => self.shared.request(Mode::Hangul),
            MENU_LATIN => self.shared.request(Mode::Latin),
            MENU_SETTINGS => crate::settings_dialog::show(windows::Win32::Foundation::HWND::default()),
            _ => {}
        }
        Ok(())
    }

    fn GetIcon(&self) -> windows::core::Result<HICON> {
        let id = match self.shared.shown() {
            Mode::Seokdok => IDI_MODE_SEOKDOK,
            Mode::Hangul => IDI_MODE_HANGUL,
            Mode::Latin => IDI_MODE_LATN,
        };
        let hinst = HINSTANCE(dll_instance().0);
        // SAFETY: hinst is this DLL's module; the id is a valid icon resource.
        unsafe { LoadIconW(Some(hinst), PCWSTR(id as usize as *const u16)) }
    }

    fn GetText(&self) -> windows::core::Result<BSTR> {
        Ok(BSTR::from(match self.shared.shown() {
            Mode::Seokdok => "釋",
            Mode::Hangul => "한",
            Mode::Latin => "A",
        }))
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
