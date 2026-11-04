//! A small native Win32 settings dialog, opened from the Start-menu shortcut
//! (`ShowSettings`) and `ITfFnConfigure::Show`.
//!
//! Controls are bound one-to-one to [`crate::config::Config`]: the dialog
//! loads the file, and on **OK** reads every control back, saves, reloads the
//! configuration and restarts the engine load so the options take effect.
//! It is a popup window with `BUTTON`, `COMBOBOX`, `EDIT` and `STATIC`
//! controls under a nested modal message loop, as `DialogBox` does.

use std::ffi::c_void;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT};
use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetDlgItem, GetMessageW,
    GetSystemMetrics, GetWindowTextW, IsDialogMessageW, IsWindow, LoadCursorW, RegisterClassExW,
    SendMessageW, SetForegroundWindow, SetWindowPos, ShowWindow, TranslateMessage, BM_GETCHECK,
    BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, BS_GROUPBOX, BS_PUSHBUTTON, CBS_DROPDOWNLIST,
    CB_ADDSTRING, CB_GETCURSEL, CB_SETCURSEL, ES_NUMBER, HMENU, IDCANCEL, IDC_ARROW, IDOK, MSG,
    SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE, SW_SHOW, WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_CREATE,
    WM_SETFONT, WNDCLASSEXW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_POPUP, WS_SYSMENU, WS_TABSTOP,
    WS_VISIBLE, WS_VSCROLL,
};

use crate::config::{Config, FragmentOpt, InputMode, Layout, Script, StyleOpt};

const CLASS_NAME: PCWSTR = w!("SeokdokSettingsDialog");

const ID_MODE: i32 = 1001; // input.default_mode (3-way)
const ID_SPACE: i32 = 1002; // input.space_converts
const ID_ALT: i32 = 1003; // input.alt_toggles
const ID_LAYOUT: i32 = 1004; // input.candidate_layout (2-way)
const ID_OLD: i32 = 1005; // input.old_clusters
const ID_STYLE: i32 = 1010; // conversion.style (2-way)
const ID_FRAGMENT: i32 = 1011; // conversion.fragment (2-way)
const ID_CODA: i32 = 1012; // conversion.coda_always
const ID_RESYL: i32 = 1013; // conversion.resyllabify
const ID_HADA: i32 = 1014; // conversion.hada_kun
const ID_MAXCAND: i32 = 1015; // conversion.max_candidates (edit)
const ID_SCRIPT: i32 = 1016; // conversion.script (3-way)
const ID_OLDHA: i32 = 1017; // conversion.old_ha

const MODE_ITEMS: [(PCWSTR, InputMode); 3] = [
    (w!("釋讀 / Seokdok"), InputMode::Seokdok),
    (w!("한글 / Hangul"), InputMode::Hangul),
    (w!("A / Latin"), InputMode::Latin),
];
const SCRIPT_ITEMS: [(PCWSTR, Script); 3] = [
    (w!("釋讀 / Seokdok"), Script::Seokdok),
    (w!("國漢文混用 / Hanja for Sino-Korean words only"), Script::SinoOnly),
    (w!("韓㐎 專用 / Hangul only, no conversion"), Script::HangulOnly),
];
const LAYOUT_ITEMS: [(PCWSTR, Layout); 2] = [
    (w!("縱 / Vertical"), Layout::Vertical),
    (w!("橫 / Horizontal  (← → move)"), Layout::Horizontal),
];
const STYLE_ITEMS: [(PCWSTR, StyleOpt); 2] = [
    (w!("表音式 / Surface  (見ㅆ다)"), StyleOpt::Surface),
    (w!("形態式 / Morpheme  (見았다)"), StyleOpt::Morpheme),
];
const FRAGMENT_ITEMS: [(PCWSTR, FragmentOpt); 2] = [
    (w!("漢字에 吸收 / Absorb  (見라)"), FragmentOpt::Absorb),
    (w!("字母로 / Letter  (見ㅏ라)"), FragmentOpt::Letter),
];

fn gui_font() -> WPARAM {
    // SAFETY: DEFAULT_GUI_FONT is a built-in stock object.
    WPARAM(unsafe { GetStockObject(DEFAULT_GUI_FONT).0 } as usize)
}

/// Open the modal settings dialog over `parent`.
pub fn show(parent: HWND) {
    register_class();
    let hinst = crate::dll_instance();

    let title: Vec<u16> = format!("釋讀 seokdok v{} — 設定 / Settings\0", env!("CARGO_PKG_VERSION"))
        .encode_utf16()
        .collect();

    let (w, h) = (470, 578);
    // SAFETY: GetSystemMetrics is always safe.
    let (sx, sy) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let (x, y) = ((sx - w).max(0) / 2, (sy - h).max(0) / 2);

    // SAFETY: class is registered; parent may be null (no owner).
    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            CLASS_NAME,
            PCWSTR(title.as_ptr()),
            WS_POPUP | WS_CAPTION | WS_SYSMENU,
            x,
            y,
            w,
            h,
            Some(parent),
            None,
            Some(hinst.into()),
            None,
        )
    };
    let Ok(hwnd) = hwnd else { return };

    // SAFETY: hwnd valid; standard modal-popup loop.
    unsafe {
        let parent_was_enabled = !parent.is_invalid();
        if parent_was_enabled {
            let _ = EnableWindow(parent, false);
        }
        let _ = SetWindowPos(hwnd, None, x, y, 0, 0, SWP_NOSIZE);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if !IsWindow(Some(hwnd)).as_bool() {
                break;
            }
        }

        if parent_was_enabled {
            let _ = EnableWindow(parent, true);
            let _ = SetForegroundWindow(parent);
        }
    }
}

fn register_class() {
    use std::sync::Once;
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: crate::dll_instance().into(),
            lpszClassName: CLASS_NAME,
            // SAFETY: IDC_ARROW is a built-in cursor.
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(
                (windows::Win32::Graphics::Gdi::COLOR_BTNFACE.0 + 1) as isize as *mut c_void,
            ),
            ..Default::default()
        };
        // SAFETY: wc is fully initialized.
        unsafe {
            RegisterClassExW(&wc);
        }
    });
}

/// Create a child control with `id` at the given rect, using the GUI font.
unsafe fn child(
    parent: HWND,
    class: PCWSTR,
    text: PCWSTR,
    style: WINDOW_STYLE,
    (x, y, w, h): (i32, i32, i32, i32),
    id: i32,
) -> HWND {
    let hwnd = CreateWindowExW(
        Default::default(),
        class,
        text,
        WS_CHILD | WS_VISIBLE | style,
        x,
        y,
        w,
        h,
        Some(parent),
        Some(HMENU(id as isize as *mut c_void)),
        Some(crate::dll_instance().into()),
        None,
    )
    .unwrap_or_default();
    if !hwnd.is_invalid() {
        SendMessageW(hwnd, WM_SETFONT, Some(gui_font()), Some(LPARAM(1)));
    }
    hwnd
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: standard window-procedure callbacks; hwnd is our window.
    unsafe {
        match msg {
            WM_CREATE => {
                build_controls(hwnd);
                LRESULT(0)
            }
            WM_COMMAND => {
                match (wparam.0 & 0xFFFF) as i32 {
                    id if id == IDOK.0 => {
                        apply(hwnd);
                        let _ = DestroyWindow(hwnd);
                    }
                    id if id == IDCANCEL.0 => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Add a drop-down list with `items` and select the one at `selected`.
unsafe fn combo<T: PartialEq + Copy>(
    hwnd: HWND,
    id: i32,
    rect: (i32, i32, i32, i32),
    items: &[(PCWSTR, T)],
    current: T,
) {
    let c = child(
        hwnd,
        w!("COMBOBOX"),
        w!(""),
        WINDOW_STYLE(CBS_DROPDOWNLIST as u32) | WS_TABSTOP | WS_VSCROLL,
        rect,
        id,
    );
    let mut sel = 0usize;
    for (i, (text, value)) in items.iter().enumerate() {
        SendMessageW(c, CB_ADDSTRING, Some(WPARAM(0)), Some(LPARAM(text.0 as isize)));
        if *value == current {
            sel = i;
        }
    }
    SendMessageW(c, CB_SETCURSEL, Some(WPARAM(sel)), None);
}

/// Build the grouped controls, loaded from the current file.
unsafe fn build_controls(hwnd: HWND) {
    let cfg = Config::load();
    let pad = 14;
    let cw = 424;
    let gx = pad + 14;
    let cbw = cw - 28;
    let checkbox = WINDOW_STYLE(BS_AUTOCHECKBOX as u32) | WS_TABSTOP;

    let group = |label: PCWSTR, y: i32, h: i32| unsafe {
        child(hwnd, w!("BUTTON"), label, WINDOW_STYLE(BS_GROUPBOX as u32), (pad, y, cw, h), 0);
    };
    let check = |id: i32, label: PCWSTR, y: i32, on: bool| unsafe {
        let c = child(hwnd, w!("BUTTON"), label, checkbox, (gx, y, cbw, 22), id);
        SendMessageW(c, BM_SETCHECK, Some(WPARAM(on as usize)), None);
    };
    let label = |text: PCWSTR, x: i32, y: i32, w: i32| unsafe {
        child(hwnd, w!("STATIC"), text, WINDOW_STYLE(0), (x, y, w, 20), 0);
    };

    let mut y = 8;

    // --- Input ---
    group(w!("入力 / Input"), y, 172);
    label(w!("始作 模式 / Start in:"), gx, y + 28, 150);
    combo(hwnd, ID_MODE, (gx + 160, y + 24, 220, 200), &MODE_ITEMS, cfg.input.default_mode);
    label(w!("候補 窓 / Candidates:"), gx, y + 62, 150);
    combo(hwnd, ID_LAYOUT, (gx + 160, y + 58, 220, 200), &LAYOUT_ITEMS, cfg.input.candidate_layout);
    check(
        ID_SPACE,
        w!("Space로 變換 / Space converts (off: Hanja key, F7, Tab)"),
        y + 90,
        cfg.input.space_converts,
    );
    check(ID_ALT, w!("Alt 한 番으로 韓/A 轉換 / Tap Alt to switch"), y + 116, cfg.input.alt_toggles);
    check(ID_OLD, w!("옛한글 겹자모 (ㅅㄱ→ᄭ, ㅂㅇ→ㅸ) / Old Hangul clusters"), y + 142, cfg.input.old_clusters);
    y += 184;

    // --- Conversion ---
    group(w!("表記 / Spelling"), y, 270);
    let mut cy = y + 24;
    label(w!("文字 / Script:"), gx, cy + 4, 150);
    combo(hwnd, ID_SCRIPT, (gx + 160, cy, 220, 200), &SCRIPT_ITEMS, cfg.conversion.script);
    cy += 34;
    label(w!("末音添記 / Style:"), gx, cy + 4, 150);
    combo(hwnd, ID_STYLE, (gx + 160, cy, 220, 200), &STYLE_ITEMS, cfg.conversion.style);
    cy += 34;
    label(w!("남는 母音 / Leftover vowel:"), gx, cy + 4, 150);
    combo(hwnd, ID_FRAGMENT, (gx + 160, cy, 220, 200), &FRAGMENT_ITEMS, cfg.conversion.fragment);
    cy += 34;
    for (id, text, on) in [
        (ID_CODA, w!("末音添記 늘 / Always write the final consonant  (有ㅆ다)"), cfg.conversion.coda_always),
        (ID_RESYL, w!("받침 넘기기 / Resyllabify  (在쓰면)"), cfg.conversion.resyllabify),
        (ID_HADA, w!("하다를 爲로 / Prefer 爲 for 하다"), cfg.conversion.hada_kun),
        (ID_OLDHA, w!("하 → ᄒᆞ 優先 / Old spelling of 하 first (하고 → ᄒᆞ고)"), cfg.conversion.old_ha),
    ] {
        check(id, text, cy, on);
        cy += 26;
    }
    label(w!("最大 候補 數 / Max candidates:"), gx, cy + 4, 190);
    let maxtext: Vec<u16> = format!("{}\0", cfg.conversion.max_candidates).encode_utf16().collect();
    child(
        hwnd,
        w!("EDIT"),
        PCWSTR(maxtext.as_ptr()),
        WINDOW_STYLE(ES_NUMBER as u32) | WS_TABSTOP | WS_BORDER,
        (gx + 200, cy + 2, 56, 22),
        ID_MAXCAND,
    );
    y += 282;

    // --- Buttons ---
    child(
        hwnd,
        w!("BUTTON"),
        w!("OK"),
        WINDOW_STYLE(BS_DEFPUSHBUTTON as u32) | WS_TABSTOP,
        (pad + cw - 210, y, 95, 28),
        IDOK.0,
    );
    child(
        hwnd,
        w!("BUTTON"),
        w!("取消 / Cancel"),
        WINDOW_STYLE(BS_PUSHBUTTON as u32) | WS_TABSTOP,
        (pad + cw - 105, y, 105, 28),
        IDCANCEL.0,
    );
}

unsafe fn selected(hwnd: HWND, id: i32) -> Option<usize> {
    let c = GetDlgItem(Some(hwnd), id).ok()?;
    usize::try_from(SendMessageW(c, CB_GETCURSEL, None, None).0).ok()
}

/// Read every control, save over the file, and refresh the running state.
unsafe fn apply(hwnd: HWND) {
    let checked = |id: i32| -> bool {
        match GetDlgItem(Some(hwnd), id) {
            Ok(c) => SendMessageW(c, BM_GETCHECK, None, None).0 == 1,
            Err(_) => false,
        }
    };

    let mut cfg = Config::load();
    if let Some((_, v)) = selected(hwnd, ID_MODE).and_then(|i| MODE_ITEMS.get(i)) {
        cfg.input.default_mode = *v;
    }
    cfg.input.space_converts = checked(ID_SPACE);
    cfg.input.alt_toggles = checked(ID_ALT);
    cfg.input.old_clusters = checked(ID_OLD);
    if let Some((_, v)) = selected(hwnd, ID_LAYOUT).and_then(|i| LAYOUT_ITEMS.get(i)) {
        cfg.input.candidate_layout = *v;
    }
    if let Some((_, v)) = selected(hwnd, ID_STYLE).and_then(|i| STYLE_ITEMS.get(i)) {
        cfg.conversion.style = *v;
    }
    if let Some((_, v)) = selected(hwnd, ID_FRAGMENT).and_then(|i| FRAGMENT_ITEMS.get(i)) {
        cfg.conversion.fragment = *v;
    }
    cfg.conversion.coda_always = checked(ID_CODA);
    cfg.conversion.resyllabify = checked(ID_RESYL);
    cfg.conversion.hada_kun = checked(ID_HADA);
    cfg.conversion.old_ha = checked(ID_OLDHA);
    if let Some((_, v)) = selected(hwnd, ID_SCRIPT).and_then(|i| SCRIPT_ITEMS.get(i)) {
        cfg.conversion.script = *v;
    }
    // Keep the existing value if the field is blank, zero or garbage.
    if let Some(n) = read_number(hwnd, ID_MAXCAND) {
        if n >= 1 {
            cfg.conversion.max_candidates = n;
        }
    }

    let _ = cfg.save();
    crate::config::reload();
    crate::dll::load_engine();
}

/// Read an EDIT control's text as a `usize`, if it parses.
unsafe fn read_number(hwnd: HWND, id: i32) -> Option<usize> {
    let c = GetDlgItem(Some(hwnd), id).ok()?;
    let mut buf = [0u16; 16];
    let n = GetWindowTextW(c, &mut buf) as usize;
    String::from_utf16_lossy(&buf[..n]).trim().parse().ok()
}
