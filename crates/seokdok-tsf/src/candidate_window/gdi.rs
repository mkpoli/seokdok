//! The classic drawing, after the Windows 10 and earlier candidate lists: a
//! square panel with a thin outline and the selected row filled. Plain GDI, so
//! it also stands in when Direct2D cannot start or stops working.
//!
//! Measurements are in pixels at 96 dpi, scaled for the monitor at each show.

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, GetLayout, SelectObject, SetBkMode, SetLayout,
    SetTextColor, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DC_LAYOUT, DEFAULT_CHARSET, DRAW_TEXT_FORMAT, DT_CALCRECT,
    DT_LEFT, DT_NOCLIP, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL, HDC, HFONT,
    OUT_DEFAULT_PRECIS, TRANSPARENT,
};

use super::theme::{Color, Palette};
use crate::session::{Attr, Popup, Run};

const FACE: PCWSTR = w!("Malgun Gothic");
const TEXT_SIZE: i32 = 16;
const ROW: i32 = 24;
/// Panel edge to the text.
const MARGIN: i32 = 8;
const NUMBER_GAP: i32 = 6;
const COLUMN_GAP: i32 = 16;

const SINGLE_LINE: DRAW_TEXT_FORMAT = DRAW_TEXT_FORMAT(DT_LEFT.0 | DT_SINGLELINE.0 | DT_NOPREFIX.0);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// The width `DrawTextW` gives the text, with its shaping of old Hangul.
fn text_extent(hdc: HDC, s: &str) -> i32 {
    let mut r = RECT::default();
    // SAFETY: hdc is a valid device context with a font selected.
    unsafe { DrawTextW(hdc, &mut wide(s), &mut r, SINGLE_LINE | DT_CALCRECT) };
    r.right - r.left
}

fn colorref(c: Color) -> COLORREF {
    COLORREF(c.r as u32 | (c.g as u32) << 8 | (c.b as u32) << 16)
}

/// One rectangle or line of text, in paint order.
enum Op {
    Fill(RECT, Color),
    Text { x: i32, top: i32, bottom: i32, text: String, color: Color },
}

/// What the window procedure paints.
pub struct Paint {
    ops: Vec<Op>,
    panel: Color,
    font: HFONT,
}

/// The font for the dpi last asked for.
#[derive(Default)]
pub struct Gdi {
    font: HFONT,
    dpi: u32,
}

impl Gdi {
    fn font_for(&mut self, dpi: u32) -> HFONT {
        if self.dpi != dpi {
            // SAFETY: plain font creation. The old font is only in the Paint
            // that the caller's layout is about to replace.
            unsafe {
                if !self.font.is_invalid() {
                    let _ = DeleteObject(self.font.into());
                }
                self.font = CreateFontW(
                    -(TEXT_SIZE * dpi as i32 / 96),
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
            }
            self.dpi = dpi;
        }
        self.font
    }

    /// Measure on `hdc`. Returns what to paint and the window's size.
    pub fn layout(
        &mut self,
        hdc: HDC,
        preedit: Option<&[Run]>,
        popup: Option<&Popup>,
        palette: &Palette,
        dpi: u32,
    ) -> (Paint, i32, i32) {
        let font = self.font_for(dpi);
        // SAFETY: hdc is valid; the old font is restored below.
        let old = unsafe { SelectObject(hdc, font.into()) };
        let (ops, width, height) = lay_out(hdc, dpi as i32, preedit, popup, palette);
        // SAFETY: restores what was selected above.
        unsafe { SelectObject(hdc, old) };
        (Paint { ops, panel: palette.panel, font }, width, height)
    }
}

impl Drop for Gdi {
    fn drop(&mut self) {
        if !self.font.is_invalid() {
            // SAFETY: the font was created in `font_for`.
            unsafe {
                let _ = DeleteObject(self.font.into());
            }
        }
    }
}

/// The operations and the window's size, with the font selected in `hdc`.
fn lay_out(hdc: HDC, dpi: i32, preedit: Option<&[Run]>, popup: Option<&Popup>, palette: &Palette) -> (Vec<Op>, i32, i32) {
    let px = |v: i32| v * dpi / 96;
    let extent = |s: &str| text_extent(hdc, s);
    let stroke = px(1).max(1);
    let (row, margin, number_gap, column_gap) = (px(ROW), px(MARGIN), px(NUMBER_GAP), px(COLUMN_GAP));
    let horizontal = popup.is_some_and(|p| p.horizontal);
    let items = popup.map_or(&[][..], |p| &p.items[..]);
    let number_width = extent("9.");
    let text_width = items.iter().map(|i| extent(&i.0)).max().unwrap_or(0);
    let note_width = items.iter().map(|i| extent(&i.1)).max().unwrap_or(0);
    let cell_width = |text: &str| margin + number_width + number_gap + extent(text);

    let footer = popup.filter(|p| horizontal || p.total > p.items.len()).map(|p| {
        let note = if horizontal { p.items.get(p.selected).map_or("", |i| i.1.as_str()) } else { "" };
        (note.to_string(), format!("{} / {}", p.position, p.total))
    });

    // Everything that sets the panel's width.
    let mut content = 0;
    if let Some(runs) = preedit {
        content = content.max(margin * 2 + runs.iter().map(|r| extent(&r.text)).sum::<i32>());
    }
    if let Some((note, position)) = &footer {
        let note = if note.is_empty() { 0 } else { extent(note) + column_gap };
        content = content.max(margin * 2 + note + extent(position));
    }
    if horizontal {
        content = content.max(items.iter().map(|i| cell_width(&i.0)).sum());
    } else if popup.is_some() {
        let note = if note_width > 0 { column_gap + note_width } else { 0 };
        content = content.max(margin * 2 + number_width + number_gap + text_width + note);
    }
    let width = content + stroke * 2;
    let inner = |top: i32, bottom: i32| RECT { left: stroke, top, right: width - stroke, bottom };

    let mut ops = Vec::new();
    let mut y = stroke;
    if let Some(runs) = preedit {
        let mut x = stroke + margin;
        for run in runs {
            let w = extent(&run.text);
            let focus = run.attr == Attr::Focus;
            if focus {
                ops.push(Op::Fill(RECT { left: x, top: y, right: x + w, bottom: y + row }, palette.selection));
            }
            let color = if focus { palette.selected_text } else { palette.text };
            ops.push(Op::Text { x, top: y, bottom: y + row, text: run.text.clone(), color });
            if run.attr == Attr::Input {
                let under = y + row - px(3);
                ops.push(Op::Fill(RECT { left: x, top: under, right: x + w, bottom: under + stroke }, palette.secondary));
            }
            x += w;
        }
        y += row;
        if popup.is_some() {
            ops.push(Op::Fill(inner(y, y + stroke), palette.divider));
            y += stroke;
        }
    }

    let selected = popup.map_or(usize::MAX, |p| p.selected);
    let ink = |i: usize| if i == selected { palette.selected_text } else { palette.text };
    let dim = |i: usize| if i == selected { palette.selected_secondary } else { palette.secondary };
    if horizontal {
        let mut x = stroke;
        for (i, (text, _)) in items.iter().enumerate() {
            let w = cell_width(text);
            if i == selected {
                ops.push(Op::Fill(RECT { left: x, top: y, right: x + w, bottom: y + row }, palette.selection));
            }
            let number_x = x + margin / 2;
            ops.push(Op::Text { x: number_x, top: y, bottom: y + row, text: format!("{}.", i + 1), color: dim(i) });
            let text_x = number_x + number_width + number_gap;
            ops.push(Op::Text { x: text_x, top: y, bottom: y + row, text: text.clone(), color: ink(i) });
            x += w;
        }
        y += row;
    } else if popup.is_some() {
        let text_x = stroke + margin + number_width + number_gap;
        let note_x = text_x + text_width + column_gap;
        for (i, (text, note)) in items.iter().enumerate() {
            if i == selected {
                ops.push(Op::Fill(inner(y, y + row), palette.selection));
            }
            ops.push(Op::Text { x: stroke + margin, top: y, bottom: y + row, text: format!("{}.", i + 1), color: dim(i) });
            ops.push(Op::Text { x: text_x, top: y, bottom: y + row, text: text.clone(), color: ink(i) });
            if !note.is_empty() {
                ops.push(Op::Text { x: note_x, top: y, bottom: y + row, text: note.clone(), color: dim(i) });
            }
            y += row;
        }
    }

    if let Some((note, position)) = footer {
        ops.push(Op::Fill(inner(y, y + stroke), palette.divider));
        y += stroke;
        let color = palette.secondary;
        if !note.is_empty() {
            ops.push(Op::Text { x: stroke + margin, top: y, bottom: y + row, text: note, color });
        }
        let x = if horizontal { width - stroke - margin - extent(&position) } else { stroke + margin };
        ops.push(Op::Text { x, top: y, bottom: y + row, text: position, color });
        y += row;
    }
    let height = y + stroke;

    for r in [
        RECT { left: 0, top: 0, right: width, bottom: stroke },
        RECT { left: 0, top: height - stroke, right: width, bottom: height },
        RECT { left: 0, top: 0, right: stroke, bottom: height },
        RECT { left: width - stroke, top: 0, right: width, bottom: height },
    ] {
        ops.push(Op::Fill(r, palette.outline));
    }
    (ops, width, height)
}

fn fill(hdc: HDC, rect: &RECT, color: Color) {
    // SAFETY: hdc is valid for this paint; the brush is deleted here.
    unsafe {
        let brush = CreateSolidBrush(colorref(color));
        FillRect(hdc, rect, brush);
        let _ = DeleteObject(brush.into());
    }
}

/// Paint `paint` into the client rectangle `rc` of `hdc`.
pub fn paint(hdc: HDC, rc: RECT, paint: &Paint) {
    // SAFETY: hdc is valid for the duration of the paint. The layout and the
    // font are restored before returning. A popup inherits a right-to-left
    // host's mirrored layout, which would reverse the text.
    unsafe {
        let layout = GetLayout(hdc);
        if layout != 0 {
            SetLayout(hdc, DC_LAYOUT(0));
        }
        fill(hdc, &rc, paint.panel);
        let old_font = SelectObject(hdc, paint.font.into());
        SetBkMode(hdc, TRANSPARENT);
        for op in &paint.ops {
            match op {
                Op::Fill(rect, color) => fill(hdc, rect, *color),
                Op::Text { x, top, bottom, text, color } => {
                    SetTextColor(hdc, colorref(*color));
                    let mut r = RECT { left: *x, top: *top, right: rc.right, bottom: *bottom };
                    DrawTextW(hdc, &mut wide(text), &mut r, SINGLE_LINE | DT_VCENTER | DT_NOCLIP);
                }
            }
        }
        SelectObject(hdc, old_font);
        if layout != 0 {
            SetLayout(hdc, DC_LAYOUT(layout));
        }
    }
}
