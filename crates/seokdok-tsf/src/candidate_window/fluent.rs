//! The Windows 11 drawing: Direct2D and DirectWrite on a software render
//! target bound to the window's device context.
//!
//! Both libraries are loaded at run time, on the first popup, so they cost a
//! typing-only process nothing and a system without them still gets a working
//! input method. A software target needs no Direct3D device, which keeps the
//! footprint small in every application the input method is loaded into.
//!
//! Measurements are in device-independent pixels, modelled on the Windows 11
//! Korean IME candidate window and WinUI's ListView: rows of 32, a selection
//! backplate of radius 4 inset by 4, an accent indicator of 3 by 16.

use std::cell::Cell;
use std::ffi::c_void;

use windows::core::{s, w, Error, Interface, Result, GUID, HRESULT, HSTRING, PCSTR, PCWSTR};
use windows::Win32::Foundation::{E_FAIL, RECT};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_IGNORE, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_POINT_2F, D2D_RECT_F,
};
use windows::Win32::Graphics::Direct2D::{
    ID2D1DCRenderTarget, ID2D1Factory, ID2D1SolidColorBrush, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_OPTIONS,
    D2D1_FACTORY_TYPE, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_SOFTWARE, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_ROUNDED_RECT,
};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory, IDWriteTextFormat, IDWriteTextLayout, DWRITE_FACTORY_TYPE, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
    DWRITE_LINE_SPACING_METHOD_UNIFORM, DWRITE_WORD_WRAPPING_NO_WRAP,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32};

use super::theme::{Color, Palette};
use crate::session::{Attr, Popup, Run};

/// The family Windows' own Korean text uses. DirectWrite shapes conjoining old
/// Hangul in it and falls back through the system for what it lacks, such as
/// the rarer Hanja extensions.
const FAMILY: PCWSTR = w!("Malgun Gothic");
const LOCALE: PCWSTR = w!("ko-KR");

const TEXT_SIZE: f32 = 16.0;
const SMALL_SIZE: f32 = 14.0;
/// Every line of text shares this height and baseline, so a fallback font
/// cannot move a row, and text of both sizes sits on one baseline.
const LINE: f32 = 24.0;
const BASELINE: f32 = 18.0;

const ROW: f32 = 32.0;
const FOOTER: f32 = 28.0;
/// Panel edge to the selection backplate.
const INSET: f32 = 4.0;
/// Panel edge to the text.
const MARGIN: f32 = 16.0;
const NUMBER_GAP: f32 = 8.0;
const COLUMN_GAP: f32 = 16.0;
const CELL_GAP: f32 = 2.0;
const CORNER: f32 = 4.0;
const INDICATOR_WIDTH: f32 = 3.0;
const INDICATOR_HEIGHT: f32 = 16.0;
const INDICATOR_CORNER: f32 = 1.5;

type CreateD2D =
    unsafe extern "system" fn(D2D1_FACTORY_TYPE, *const GUID, *const D2D1_FACTORY_OPTIONS, *mut *mut c_void) -> HRESULT;
type CreateDWrite = unsafe extern "system" fn(DWRITE_FACTORY_TYPE, *const GUID, *mut *mut c_void) -> HRESULT;

/// The libraries to start from: the system's, or names that do not exist when
/// exercising the fallback.
pub struct Libraries {
    pub d2d1: &'static str,
    pub dwrite: &'static str,
}

pub const SYSTEM_LIBRARIES: Libraries = Libraries { d2d1: "d2d1.dll", dwrite: "dwrite.dll" };
pub const MISSING_LIBRARIES: Libraries = Libraries { d2d1: "seokdok-no-d2d1.dll", dwrite: "seokdok-no-dwrite.dll" };

/// Find `symbol` in a system library. The library stays loaded for the life of
/// the process, since the factories' code lives there.
fn load(library: &str, symbol: PCSTR) -> Result<unsafe extern "system" fn() -> isize> {
    let library = HSTRING::from(library);
    // SAFETY: both names are valid strings; only System32 is searched.
    unsafe {
        let module = LoadLibraryExW(&library, None, LOAD_LIBRARY_SEARCH_SYSTEM32)?;
        GetProcAddress(module, symbol).ok_or_else(Error::from_win32)
    }
}

/// One rectangle or line of text, in paint order.
enum Op {
    Fill { rect: D2D_RECT_F, radius: f32, color: Color },
    Text { x: f32, y: f32, layout: IDWriteTextLayout, color: Color },
}

/// A laid-out popup, ready to paint any number of times.
pub struct Scene {
    ops: Vec<Op>,
    panel: Color,
    /// Size in device-independent pixels, and the dpi it was laid out for.
    pub width: f32,
    pub height: f32,
    pub dpi: f32,
}

impl Scene {
    pub fn pixel_size(&self) -> (i32, i32) {
        let scale = self.dpi / 96.0;
        ((self.width * scale).ceil() as i32, (self.height * scale).ceil() as i32)
    }
}

fn d2d_color(c: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r: c.r as f32 / 255.0, g: c.g as f32 / 255.0, b: c.b as f32 / 255.0, a: c.a as f32 / 255.0 }
}

fn rect(left: f32, top: f32, right: f32, bottom: f32) -> D2D_RECT_F {
    D2D_RECT_F { left, top, right, bottom }
}

/// A line of text and its width.
struct Label {
    layout: IDWriteTextLayout,
    width: f32,
}

struct Item {
    number: Label,
    text: Label,
    note: Option<Label>,
}

pub struct Renderer {
    d2d: ID2D1Factory,
    dwrite: IDWriteFactory,
    text_format: IDWriteTextFormat,
    small_format: IDWriteTextFormat,
    target: Option<(ID2D1DCRenderTarget, ID2D1SolidColorBrush)>,
    /// Makes the next paint fail, to exercise the way back to GDI.
    fail_next_paint: Cell<bool>,
}

impl Renderer {
    pub fn start(libraries: &Libraries) -> Result<Self> {
        let create_d2d = load(libraries.d2d1, s!("D2D1CreateFactory"))?;
        let create_dwrite = load(libraries.dwrite, s!("DWriteCreateFactory"))?;
        // SAFETY: the exports have these documented signatures, and each
        // returned pointer is an owned reference to the requested interface.
        let (d2d, dwrite) = unsafe {
            let create_d2d: CreateD2D = std::mem::transmute(create_d2d);
            let create_dwrite: CreateDWrite = std::mem::transmute(create_dwrite);
            let mut raw = std::ptr::null_mut();
            create_d2d(D2D1_FACTORY_TYPE_SINGLE_THREADED, &ID2D1Factory::IID, std::ptr::null(), &mut raw).ok()?;
            let d2d = ID2D1Factory::from_raw(raw);
            let mut raw = std::ptr::null_mut();
            create_dwrite(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory::IID, &mut raw).ok()?;
            (d2d, IDWriteFactory::from_raw(raw))
        };
        let text_format = make_format(&dwrite, TEXT_SIZE)?;
        let small_format = make_format(&dwrite, SMALL_SIZE)?;
        Ok(Self { d2d, dwrite, text_format, small_format, target: None, fail_next_paint: Cell::new(false) })
    }

    pub fn fail_next_paint(&self) {
        self.fail_next_paint.set(true);
    }

    fn label(&self, s: &str, format: &IDWriteTextFormat) -> Result<Label> {
        let text: Vec<u16> = s.encode_utf16().collect();
        // SAFETY: format is valid; the layout copies the text.
        unsafe {
            let layout = self.dwrite.CreateTextLayout(&text, format, 4096.0, LINE)?;
            let mut metrics = Default::default();
            layout.GetMetrics(&mut metrics)?;
            Ok(Label { layout, width: metrics.widthIncludingTrailingWhitespace.ceil() })
        }
    }

    /// Lay out the preedit line and the candidates for `dpi`.
    pub fn layout(&self, preedit: Option<&[Run]>, popup: Option<&Popup>, palette: &Palette, dpi: f32) -> Result<Scene> {
        let scale = dpi / 96.0;
        let snap = |v: f32| (v * scale).round() / scale;
        // A stroke of 1, at least one device pixel.
        let stroke = snap(1.0).max(1.0 / scale);
        let mut ops = Vec::new();

        // Measure everything first: the panel's width depends on every part.
        let preedit_labels = preedit
            .map(|runs| {
                runs.iter().map(|r| Ok((self.label(&r.text, &self.text_format)?, r.attr == Attr::Focus))).collect::<Result<Vec<_>>>()
            })
            .transpose()?;
        let horizontal = popup.is_some_and(|p| p.horizontal);
        let selected = popup.map_or(usize::MAX, |p| p.selected);
        let mut items = Vec::new();
        for (i, (text, note)) in popup.map_or(&[][..], |p| &p.items[..]).iter().enumerate() {
            items.push(Item {
                number: self.label(&format!("{}", i + 1), &self.small_format)?,
                text: self.label(text, &self.text_format)?,
                note: (!note.is_empty() && !horizontal).then(|| self.label(note, &self.small_format)).transpose()?,
            });
        }
        let footer = match popup {
            Some(p) if horizontal || p.total > p.items.len() => {
                let note = p.items.get(p.selected).map_or("", |i| i.1.as_str());
                let note = (horizontal && !note.is_empty()).then(|| self.label(note, &self.small_format)).transpose()?;
                Some((note, self.label(&format!("{} / {}", p.position, p.total), &self.small_format)?))
            }
            _ => None,
        };
        let number_width = self.label("9", &self.small_format)?.width;
        let text_width = items.iter().map(|i| i.text.width).fold(0.0, f32::max);
        let note_width = items.iter().filter_map(|i| i.note.as_ref().map(|n| n.width)).fold(0.0, f32::max);
        // A cell in a horizontal list: the indicator and half a margin on each side.
        let cell_width = |item: &Item| MARGIN + INDICATOR_WIDTH + item.number.width + NUMBER_GAP + item.text.width;

        let mut width: f32 = 0.0;
        if let Some(labels) = &preedit_labels {
            width = width.max(MARGIN * 2.0 + labels.iter().map(|(l, _)| l.width).sum::<f32>());
        }
        if let Some((note, position)) = &footer {
            let note = note.as_ref().map_or(0.0, |n| n.width + COLUMN_GAP);
            width = width.max(MARGIN * 2.0 + note + position.width);
        }
        if horizontal {
            let cells: f32 = items.iter().map(|i| cell_width(i) + CELL_GAP).sum();
            width = width.max(INSET * 2.0 + cells - CELL_GAP);
        } else if popup.is_some() {
            let note = if note_width > 0.0 { COLUMN_GAP + note_width } else { 0.0 };
            width = width.max(MARGIN * 2.0 + number_width + NUMBER_GAP + text_width + note);
        }
        let width = width.ceil();

        let centered = |top: f32, row: f32| top + (row - LINE) / 2.0;
        let divider = |ops: &mut Vec<Op>, y: f32| {
            let y = snap(y);
            ops.push(Op::Fill { rect: rect(0.0, y, width, y + stroke), radius: 0.0, color: palette.divider });
        };
        let selection = |ops: &mut Vec<Op>, left: f32, right: f32, top: f32| {
            ops.push(Op::Fill { rect: rect(left, top, right, top + ROW), radius: CORNER, color: palette.selection });
            let t = snap(top + (ROW - INDICATOR_HEIGHT) / 2.0);
            ops.push(Op::Fill {
                rect: rect(left, t, left + INDICATOR_WIDTH, t + INDICATOR_HEIGHT),
                radius: INDICATOR_CORNER,
                color: palette.accent,
            });
        };
        let ink = |is_selected: bool| if is_selected { palette.selected_text } else { palette.text };
        let dim = |is_selected: bool| if is_selected { palette.selected_secondary } else { palette.secondary };

        let mut y = INSET;
        if let Some(labels) = preedit_labels {
            let mut x = MARGIN;
            let top = centered(y, ROW);
            for (label, focus) in labels {
                let right = x + label.width;
                if focus {
                    ops.push(Op::Fill { rect: rect(x, y + 2.0, right, y + ROW - 2.0), radius: 2.0, color: palette.selection });
                }
                let under = snap(top + LINE - 1.0);
                let (thickness, color) = if focus { (stroke * 2.0, palette.accent) } else { (stroke, palette.secondary) };
                ops.push(Op::Fill { rect: rect(x, under, right, under + thickness), radius: 0.0, color });
                ops.push(Op::Text { x, y: top, layout: label.layout, color: ink(focus) });
                x = right;
            }
            y += ROW;
            if popup.is_some() {
                y += INSET;
                divider(&mut ops, y);
                y += INSET;
            }
        }

        if horizontal {
            let mut x = INSET;
            let top = centered(y, ROW);
            for (i, item) in items.into_iter().enumerate() {
                let cell = cell_width(&item);
                let is_selected = i == selected;
                if is_selected {
                    selection(&mut ops, x, x + cell, y);
                }
                let number_x = x + MARGIN / 2.0 + INDICATOR_WIDTH;
                let text_x = number_x + item.number.width + NUMBER_GAP;
                ops.push(Op::Text { x: number_x, y: top, layout: item.number.layout, color: dim(is_selected) });
                ops.push(Op::Text { x: text_x, y: top, layout: item.text.layout, color: ink(is_selected) });
                x += cell + CELL_GAP;
            }
            y += ROW;
        } else if popup.is_some() {
            let text_x = MARGIN + number_width + NUMBER_GAP;
            for (i, item) in items.into_iter().enumerate() {
                let is_selected = i == selected;
                if is_selected {
                    selection(&mut ops, INSET, width - INSET, y);
                }
                let top = centered(y, ROW);
                ops.push(Op::Text { x: MARGIN, y: top, layout: item.number.layout, color: dim(is_selected) });
                ops.push(Op::Text { x: text_x, y: top, layout: item.text.layout, color: ink(is_selected) });
                if let Some(note) = item.note {
                    let x = width - MARGIN - note.width;
                    ops.push(Op::Text { x, y: top, layout: note.layout, color: dim(is_selected) });
                }
                y += ROW;
            }
        }

        if let Some((note, position)) = footer {
            y += INSET;
            divider(&mut ops, y);
            let top = centered(y, FOOTER);
            if let Some(note) = note {
                ops.push(Op::Text { x: MARGIN, y: top, layout: note.layout, color: palette.secondary });
            }
            let x = if horizontal { width - MARGIN - position.width } else { MARGIN };
            ops.push(Op::Text { x, y: top, layout: position.layout, color: palette.secondary });
            y += FOOTER - INSET;
        }
        let height = y + INSET;

        // The flyout's own stroke, which WinUI draws too. It is all the outline
        // the popup has where the compositor adds none: Windows 10, a virtual
        // machine, a remote session.
        let (w, h) = (width, height);
        for r in [rect(0.0, 0.0, w, stroke), rect(0.0, h - stroke, w, h), rect(0.0, 0.0, stroke, h), rect(w - stroke, 0.0, w, h)] {
            ops.push(Op::Fill { rect: r, radius: 0.0, color: palette.outline });
        }

        Ok(Scene { ops, panel: palette.panel, width, height, dpi })
    }

    fn make_target(&self) -> Result<(ID2D1DCRenderTarget, ID2D1SolidColorBrush)> {
        let properties = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
            pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_IGNORE },
            dpiX: 96.0,
            dpiY: 96.0,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        // SAFETY: properties is fully initialized.
        unsafe {
            let target = self.d2d.CreateDCRenderTarget(&properties)?;
            let brush = target.CreateSolidColorBrush(&d2d_color(Color { r: 0, g: 0, b: 0, a: 0xFF }), None)?;
            Ok((target, brush))
        }
    }

    /// Paint `scene` into `rc` of `hdc`. A lost target is rebuilt once; any
    /// other failure is for the caller to act on.
    pub fn paint(&mut self, hdc: HDC, rc: &RECT, scene: &Scene) -> Result<()> {
        if self.fail_next_paint.take() {
            return Err(E_FAIL.into());
        }
        match self.paint_once(hdc, rc, scene) {
            Err(e) if e.code() == windows::Win32::Foundation::D2DERR_RECREATE_TARGET => {
                self.target = None;
                self.paint_once(hdc, rc, scene)
            }
            other => other,
        }
    }

    fn paint_once(&mut self, hdc: HDC, rc: &RECT, scene: &Scene) -> Result<()> {
        if self.target.is_none() {
            self.target = Some(self.make_target()?);
        }
        let Some((target, brush)) = &self.target else { return Err(E_FAIL.into()) };
        // SAFETY: hdc is valid for this paint and every draw call happens
        // between BeginDraw and EndDraw with objects from this target.
        unsafe {
            target.BindDC(hdc, rc)?;
            target.SetDpi(scene.dpi, scene.dpi);
            target.BeginDraw();
            target.Clear(Some(&d2d_color(scene.panel)));
            for op in &scene.ops {
                match op {
                    Op::Fill { rect, radius, color } => {
                        brush.SetColor(&d2d_color(*color));
                        if *radius > 0.0 {
                            target.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect: *rect, radiusX: *radius, radiusY: *radius }, brush);
                        } else {
                            target.FillRectangle(rect, brush);
                        }
                    }
                    Op::Text { x, y, layout, color } => {
                        brush.SetColor(&d2d_color(*color));
                        target.DrawTextLayout(D2D_POINT_2F { x: *x, y: *y }, layout, brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
                    }
                }
            }
            target.EndDraw(None, None)
        }
    }

    /// Whether a family is installed, for the preview's report.
    pub fn family_installed(&self, family: &str) -> bool {
        let name = HSTRING::from(family);
        // SAFETY: the outputs are valid for the calls.
        unsafe {
            let mut collection = None;
            if self.dwrite.GetSystemFontCollection(&mut collection, false).is_err() {
                return false;
            }
            let Some(collection) = collection else { return false };
            let mut index = 0;
            let mut exists = false.into();
            collection.FindFamilyName(&name, &mut index, &mut exists).is_ok() && exists.as_bool()
        }
    }

    /// How many clusters DirectWrite makes of `text`: conjoining old Hangul
    /// has to shape into one per syllable.
    pub fn cluster_count(&self, text: &str) -> Result<u32> {
        let label = self.label(text, &self.text_format)?;
        let mut count = 0;
        // SAFETY: no buffer asks for the count, which comes back alongside
        // E_NOT_SUFFICIENT_BUFFER.
        let _ = unsafe { label.layout.GetClusterMetrics(None, &mut count) };
        Ok(count)
    }
}

fn make_format(dwrite: &IDWriteFactory, size: f32) -> Result<IDWriteTextFormat> {
    // SAFETY: the family and locale are valid strings.
    unsafe {
        let format = dwrite.CreateTextFormat(
            FAMILY,
            None,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            size,
            LOCALE,
        )?;
        format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
        format.SetLineSpacing(DWRITE_LINE_SPACING_METHOD_UNIFORM, LINE, BASELINE)?;
        Ok(format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn popup(n: usize, horizontal: bool, total: usize) -> Popup {
        Popup {
            items: (0..n).map(|i| (format!("候補{i}"), "집 가".to_string())).collect(),
            selected: 1,
            position: 2,
            total,
            horizontal,
        }
    }

    fn renderer() -> Renderer {
        Renderer::start(&SYSTEM_LIBRARIES).expect("Direct2D on Windows")
    }

    #[test]
    fn a_missing_library_is_an_error() {
        assert!(Renderer::start(&MISSING_LIBRARIES).is_err());
        assert!(Renderer::start(&Libraries { d2d1: SYSTEM_LIBRARIES.d2d1, dwrite: MISSING_LIBRARIES.dwrite }).is_err());
    }

    #[test]
    fn rows_are_32_tall_inside_a_4_inset() {
        let s = renderer().layout(None, Some(&popup(3, false, 3)), &Palette::default_light(), 96.0).unwrap();
        assert_eq!(s.height, INSET + 3.0 * ROW + INSET);
    }

    #[test]
    fn a_footer_appears_only_when_there_are_more_pages() {
        let r = renderer();
        let palette = Palette::default_light();
        let one = r.layout(None, Some(&popup(3, false, 3)), &palette, 96.0).unwrap();
        let more = r.layout(None, Some(&popup(3, false, 12)), &palette, 96.0).unwrap();
        assert_eq!(more.height - one.height, FOOTER);
    }

    #[test]
    fn pixels_scale_with_the_dpi() {
        let r = renderer();
        let palette = Palette::default_dark();
        let p = popup(5, false, 20);
        let a = r.layout(None, Some(&p), &palette, 96.0).unwrap().pixel_size();
        let b = r.layout(None, Some(&p), &palette, 192.0).unwrap().pixel_size();
        assert_eq!((a.0 * 2, a.1 * 2), b);
    }

    #[test]
    fn old_hangul_shapes_into_one_cluster_per_syllable() {
        let r = renderer();
        assert_eq!(r.cluster_count("ᄉᆞ랑").unwrap(), 2);
        assert_eq!(r.cluster_count("ᄒᆞᆯ").unwrap(), 1);
    }

    #[test]
    fn nothing_is_drawn_outside_the_panel() {
        let r = renderer();
        let palette = Palette::default_light();
        for p in [popup(9, false, 40), popup(9, true, 40)] {
            let s = r.layout(None, Some(&p), &palette, 96.0).unwrap();
            for op in &s.ops {
                if let Op::Fill { rect, .. } = op {
                    assert!(
                        rect.left >= 0.0 && rect.right <= s.width && rect.top >= 0.0 && rect.bottom <= s.height,
                        "{rect:?} in {} x {}",
                        s.width,
                        s.height
                    );
                }
            }
        }
    }
}
