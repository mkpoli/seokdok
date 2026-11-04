//! The plain GDI drawing, used when Direct2D cannot start or stops working.

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, GetTextExtentPoint32W, SelectObject,
    SetBkMode, SetTextColor, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DT_LEFT, DT_NOPREFIX,
    DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL, HDC, HFONT, OUT_DEFAULT_PRECIS, TRANSPARENT,
};

use crate::session::{Attr, Popup, Run};

const FACE: PCWSTR = w!("Malgun Gothic");
const PAD: i32 = 6;
const GAP: i32 = 14;

/// What the window procedure paints.
pub struct Paint {
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

fn colorref(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

/// The font and line height, scaled to the screen once.
pub struct Gdi {
    font: HFONT,
    /// Pixel height of a line (96 dpi scaled to the screen).
    pub line_height: i32,
}

impl Gdi {
    pub fn new(dpi: i32) -> Self {
        let height = -(18 * dpi / 96);
        // SAFETY: plain font creation; deleted in Drop.
        let font = unsafe {
            CreateFontW(
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
            )
        };
        Self { font, line_height: 26 * dpi / 96 }
    }

    /// Measure on `hdc`. Returns what to paint and the client size.
    pub fn layout(&self, hdc: HDC, preedit: Option<&[Run]>, popup: Option<&Popup>) -> (Paint, i32, i32) {
        let empty = Popup { items: Vec::new(), selected: 0, position: 0, total: 0, horizontal: false };
        let list = popup.unwrap_or(&empty);
        // SAFETY: hdc is valid; the old font is restored below.
        let old = unsafe { SelectObject(hdc, self.font.into()) };
        let number_width = text_extent(hdc, "9.");
        let text_width = list.items.iter().map(|i| text_extent(hdc, &i.0)).max().unwrap_or(0);
        let note_width = list.items.iter().map(|i| text_extent(hdc, &i.1)).max().unwrap_or(0);
        let preedit_width = preedit.map_or(0, |runs| runs.iter().map(|r| text_extent(hdc, &r.text)).sum());
        let top = if preedit.is_some() { self.line_height + if popup.is_some() { PAD } else { 0 } } else { 0 };
        // Horizontal: the candidates in one row, the note and position below.
        let mut cells = Vec::new();
        let (width, height) = if popup.is_none() {
            (0, PAD * 2)
        } else if list.horizontal {
            let widths: Vec<i32> =
                list.items.iter().map(|i| number_width + GAP / 2 + text_extent(hdc, &i.0) + GAP).collect();
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
        // SAFETY: restores what was selected above.
        unsafe { SelectObject(hdc, old) };
        let width = width.max(PAD * 2 + preedit_width);
        let height = height + top;
        let paint = Paint {
            preedit: preedit.map(<[Run]>::to_vec),
            top,
            popup: popup.cloned(),
            font: self.font,
            row_height: self.line_height,
            cells,
            number_width,
            text_width,
        };
        (paint, width, height)
    }
}

impl Drop for Gdi {
    fn drop(&mut self) {
        // SAFETY: font was created in `new`.
        unsafe {
            let _ = DeleteObject(self.font.into());
        }
    }
}

/// Paint `paint` into the client rectangle `rc` of `hdc`.
pub fn paint(hdc: HDC, rc: RECT, paint: &Paint) {
    // SAFETY: hdc is valid for the duration of the paint; every object created
    // here is deleted and the old font restored before returning.
    unsafe {
        let bg = CreateSolidBrush(colorref(0xFF, 0xFF, 0xFF));
        FillRect(hdc, &rc, bg);
        let _ = DeleteObject(bg.into());

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
    }
}
