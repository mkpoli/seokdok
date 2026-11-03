//! The candidate popup: a GDI window listing one page of candidates, each as
//! a number, the text and a dimmer note, with the position in the list below.
//! For IMM32 applications it also carries the text being composed on a line
//! of its own above the candidates (see `composition.rs`).
//!
//! Display only: `WS_EX_NOACTIVATE` + `SW_SHOWNOACTIVATE` keep focus in the
//! document. The key handler drives selection through the session; this
//! module renders what the session reports.

use std::cell::RefCell;
use std::sync::Once;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetDC, GetDeviceCaps, GetTextExtentPoint32W, InvalidateRect, ReleaseDC, SelectObject,
    SetBkMode, SetTextColor, UpdateWindow, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL,
    HDC, HFONT, LOGPIXELSY, OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetGUIThreadInfo,
    GetSystemMetrics, LoadCursorW, RegisterClassExW, SetWindowPos, ShowWindow, GUITHREADINFO,
    HWND_TOPMOST, IDC_ARROW, SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE,
    WM_PAINT, WNDCLASSEXW, WS_BORDER, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::dll_instance;
use crate::session::{Attr, Popup, Run};

const CLASS_NAME: PCWSTR = w!("SeokdokCandidateWindow");
const FACE: PCWSTR = w!("Malgun Gothic");
const PAD: i32 = 6;
const GAP: i32 = 14;

/// What the window procedure paints.
struct Paint {
    /// Text being composed, drawn on the first line, and its height.
    preedit: Option<Vec<Run>>,
    top: i32,
    popup: Option<Popup>,
    font: HFONT,
    row_height: i32,
    /// Horizontal layout: left edge and width of each item.
    cells: Vec<(i32, i32)>,
    number_width: i32,
    text_width: i32,
}

thread_local! {
    static PAINT: RefCell<Option<Paint>> = const { RefCell::new(None) };
}

static REGISTER: Once = Once::new();

fn hinstance() -> HINSTANCE {
    HINSTANCE(dll_instance().0)
}

fn register_class() {
    REGISTER.call_once(|| {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance(),
            lpszClassName: CLASS_NAME,
            // SAFETY: IDC_ARROW is a built-in cursor.
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
            ..Default::default()
        };
        // SAFETY: wc is fully initialized.
        unsafe {
            RegisterClassExW(&wc);
        }
    });
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn text_extent(hdc: HDC, s: &str) -> i32 {
    let w = wide(s);
    let mut size = SIZE::default();
    // SAFETY: hdc is a valid device context with a font selected.
    unsafe {
        let _ = GetTextExtentPoint32W(hdc, &w, &mut size);
    }
    size.cx
}

pub struct CandidateWindow {
    hwnd: HWND,
    font: HFONT,
    /// Pixel height of the font (96 dpi scaled to the screen).
    line_height: i32,
}

impl CandidateWindow {
    /// Create the hidden popup. `None` if creation fails.
    pub fn new() -> Option<Self> {
        register_class();
        // SAFETY: GetDC(None) is the screen; released below.
        let (font, line_height) = unsafe {
            let hdc = GetDC(None);
            let dpi = GetDeviceCaps(Some(hdc), LOGPIXELSY).max(96);
            ReleaseDC(None, hdc);
            let height = -(18 * dpi / 96);
            let font = CreateFontW(
                height,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                FF_DONTCARE.0 as u32,
                FACE,
            );
            (font, 26 * dpi / 96)
        };
        // SAFETY: the class is registered; a popup has no parent.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                CLASS_NAME,
                PCWSTR::null(),
                WS_POPUP | WS_BORDER,
                0,
                0,
                100,
                line_height,
                None,
                None,
                Some(hinstance()),
                None,
            )
            .ok()?
        };
        Some(Self { hwnd, font, line_height })
    }

    /// Show the preedit line (if any) and `popup` (if any) below `anchor`
    /// (screen coordinates), or near the focus caret when the application did
    /// not report a position.
    pub fn show(&self, preedit: Option<&[Run]>, popup: Option<&Popup>, anchor: Option<(i32, i32)>) {
        let empty = Popup { items: Vec::new(), selected: 0, position: 0, total: 0, horizontal: false };
        let list = popup.unwrap_or(&empty);
        // Measure with the popup's font.
        // SAFETY: hwnd valid; the DC is released below.
        let (number_width, text_width, note_width, preedit_width) = unsafe {
            let hdc = GetDC(Some(self.hwnd));
            let old = SelectObject(hdc, self.font.into());
            let n = text_extent(hdc, "9.");
            let t = list.items.iter().map(|i| text_extent(hdc, &i.0)).max().unwrap_or(0);
            let o = list.items.iter().map(|i| text_extent(hdc, &i.1)).max().unwrap_or(0);
            let p = preedit.map_or(0, |runs| runs.iter().map(|r| text_extent(hdc, &r.text)).sum());
            SelectObject(hdc, old);
            ReleaseDC(Some(self.hwnd), hdc);
            (n, t, o, p)
        };
        let top = if preedit.is_some() { self.line_height + if popup.is_some() { PAD } else { 0 } } else { 0 };
        // Horizontal: the candidates in one row, the note and position below.
        let mut cells = Vec::new();
        let (width, height) = if popup.is_none() {
            (0, PAD * 2)
        } else if list.horizontal {
            // SAFETY: hwnd valid; the DC is released below.
            let widths: Vec<i32> = unsafe {
                let hdc = GetDC(Some(self.hwnd));
                let old = SelectObject(hdc, self.font.into());
                let w = list.items.iter().map(|i| number_width + GAP / 2 + text_extent(hdc, &i.0) + GAP).collect();
                SelectObject(hdc, old);
                ReleaseDC(Some(self.hwnd), hdc);
                w
            };
            let mut x = PAD;
            for w in &widths {
                cells.push((x, *w));
                x += w;
            }
            let row = x + PAD;
            let foot = PAD * 2 + note_width + GAP + number_width * 4;
            (row.max(foot), PAD * 2 + 2 * self.line_height)
        } else {
            let footer = list.total > list.items.len();
            let rows = list.items.len() as i32 + footer as i32;
            (
                PAD * 2 + number_width + GAP + text_width + if note_width > 0 { GAP + note_width } else { 0 },
                PAD * 2 + rows * self.line_height,
            )
        };
        let width = width.max(PAD * 2 + preedit_width);
        let height = height + top;

        PAINT.with(|p| {
            *p.borrow_mut() = Some(Paint {
                preedit: preedit.map(<[Run]>::to_vec),
                top,
                popup: popup.cloned(),
                font: self.font,
                row_height: self.line_height,
                cells,
                number_width,
                text_width,
            })
        });

        let (mut x, mut y) = anchor.unwrap_or_else(caret_screen_pos);
        y += 2;
        // SAFETY: GetSystemMetrics is always safe.
        let (sw, sh) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
        let width = width.min(sw);
        if x + width > sw {
            x = (sw - width).max(0);
        }
        if y + height > sh {
            // Flip above the line when there is no room below.
            y = (y - height - self.line_height - 4).max(0);
        }
        // SAFETY: hwnd is valid for this object's lifetime.
        unsafe {
            let _ = SetWindowPos(self.hwnd, Some(HWND_TOPMOST), x, y, width + 2, height + 2, SWP_NOACTIVATE);
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            let _ = InvalidateRect(Some(self.hwnd), None, true);
            let _ = UpdateWindow(self.hwnd);
        }
    }

    pub fn hide(&self) {
        // SAFETY: hwnd valid.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

impl Drop for CandidateWindow {
    fn drop(&mut self) {
        // SAFETY: hwnd and font were created in `new`.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
            let _ = DeleteObject(self.font.into());
        }
    }
}

/// Focus caret in screen coordinates, with a corner as the last resort.
fn caret_screen_pos() -> (i32, i32) {
    let mut gti = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
    // SAFETY: gti is sized; thread id 0 = the foreground thread.
    unsafe {
        if GetGUIThreadInfo(0, &mut gti).is_ok() && !gti.hwndCaret.is_invalid() {
            let mut pt = POINT { x: gti.rcCaret.left, y: gti.rcCaret.bottom };
            if ClientToScreen(gti.hwndCaret, &mut pt).as_bool() {
                return (pt.x, pt.y);
            }
        }
    }
    (120, 120)
}

fn colorref(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg != WM_PAINT {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut rc = RECT::default();
    let _ = GetClientRect(hwnd, &mut rc);

    let bg = CreateSolidBrush(colorref(0xFF, 0xFF, 0xFF));
    FillRect(hdc, &rc, bg);
    let _ = DeleteObject(bg.into());

    PAINT.with(|p| {
        let guard = p.borrow();
        let Some(paint) = guard.as_ref() else { return };
        let old_font = SelectObject(hdc, paint.font.into());
        SetBkMode(hdc, TRANSPARENT);
        let highlight = CreateSolidBrush(colorref(0xCC, 0xE4, 0xF7));

        // The text being composed: input underlined, the focused segment
        // highlighted, converted text plain.
        if let Some(runs) = &paint.preedit {
            let mut x = PAD;
            let line = CreateSolidBrush(colorref(0x40, 0x40, 0x40));
            for run in runs {
                let w = text_extent(hdc, &run.text);
                let cell = RECT { left: x, top: PAD, right: x + w, bottom: PAD + paint.row_height };
                if run.attr == Attr::Focus {
                    FillRect(hdc, &cell, highlight);
                }
                SetTextColor(hdc, colorref(0x10, 0x10, 0x10));
                let mut t = wide(&run.text);
                let mut r = cell;
                DrawTextW(hdc, &mut t, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
                let under = RECT { left: x, top: cell.bottom - 2, right: x + w, bottom: cell.bottom - 1 };
                FillRect(hdc, &under, line);
                x += w;
            }
            let _ = DeleteObject(line.into());
        }
        let Some(popup) = &paint.popup else {
            let _ = DeleteObject(highlight.into());
            SelectObject(hdc, old_font);
            return;
        };
        let base = paint.top;

        if popup.horizontal {
            let draw = |s: &str, r: RECT, color: COLORREF| {
                SetTextColor(hdc, color);
                let mut line = wide(s);
                let mut r = r;
                DrawTextW(hdc, &mut line, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
            };
            for (i, ((text, _), (x, w))) in popup.items.iter().zip(&paint.cells).enumerate() {
                let cell = RECT { left: *x, top: base + PAD, right: x + w - GAP / 2, bottom: base + PAD + paint.row_height };
                if i == popup.selected {
                    FillRect(hdc, &cell, highlight);
                }
                draw(&format!("{}", i + 1), RECT { left: x + 4, ..cell }, colorref(0x70, 0x70, 0x70));
                draw(text, RECT { left: x + 4 + paint.number_width, ..cell }, colorref(0x10, 0x10, 0x10));
            }
            let top = base + PAD + paint.row_height;
            let foot = RECT { left: PAD, top, right: rc.right - PAD, bottom: top + paint.row_height };
            let note = popup.items.get(popup.selected).map(|i| i.1.as_str()).unwrap_or("");
            draw(&format!("{}  {} / {}", note, popup.position, popup.total), foot, colorref(0x80, 0x80, 0x80));
            let _ = DeleteObject(highlight.into());
            SelectObject(hdc, old_font);
            return;
        }
        let text_x = PAD + paint.number_width + GAP;
        let note_x = text_x + paint.text_width + GAP;
        for (i, (text, note)) in popup.items.iter().enumerate() {
            let top = base + PAD + i as i32 * paint.row_height;
            let row = RECT { left: 1, top, right: rc.right - 1, bottom: top + paint.row_height };
            if i == popup.selected {
                FillRect(hdc, &row, highlight);
            }
            let draw = |s: &str, x: i32, color: COLORREF| {
                SetTextColor(hdc, color);
                let mut line = wide(s);
                let mut r = RECT { left: x, ..row };
                DrawTextW(hdc, &mut line, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
            };
            draw(&format!("{}.", i + 1), PAD, colorref(0x70, 0x70, 0x70));
            draw(text, text_x, colorref(0x10, 0x10, 0x10));
            if !note.is_empty() {
                draw(note, note_x, colorref(0x80, 0x80, 0x80));
            }
        }
        if popup.total > popup.items.len() {
            let top = base + PAD + popup.items.len() as i32 * paint.row_height;
            let mut r = RECT { left: PAD, top, right: rc.right - PAD, bottom: top + paint.row_height };
            SetTextColor(hdc, colorref(0x80, 0x80, 0x80));
            let mut line = wide(&format!("{} / {}", popup.position, popup.total));
            DrawTextW(hdc, &mut line, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
        }
        let _ = DeleteObject(highlight.into());
        SelectObject(hdc, old_font);
    });

    let _ = EndPaint(hwnd, &ps);
    LRESULT(0)
}
