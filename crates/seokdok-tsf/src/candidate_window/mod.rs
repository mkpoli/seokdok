//! The candidate popup: a window listing one page of candidates, each as a
//! number, the text and a dimmer note, with the position in the list below.
//! For IMM32 applications it also carries the text being composed on a line
//! of its own above the candidates (see `composition.rs`).
//!
//! `windows.candidate_style` picks the drawing at every show: `fluent.rs` the
//! way Windows 11 draws its own IME's list, `gdi.rs` the classic one. If Direct2D
//! cannot start or fails while painting, `gdi.rs` takes over for the rest of the
//! window's life.
//!
//! Display only: `WS_EX_NOACTIVATE`, `SW_SHOWNOACTIVATE` and `MA_NOACTIVATE`
//! keep focus in the document. The key handler drives selection through the session; this
//! module renders what the session reports.

mod fluent;
mod gdi;
mod theme;

use std::cell::RefCell;
use std::sync::Once;

use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
    DWMWCP_ROUND,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, EndPaint, GetDC, GetMonitorInfoW, InvalidateRect, MonitorFromPoint, ReleaseDC,
    UpdateWindow, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetForegroundWindow, GetGUIThreadInfo,
    GetWindowLongPtrW, LoadCursorW, RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CS_DROPSHADOW,
    GUITHREADINFO, GWLP_USERDATA, HWND_TOPMOST, IDC_ARROW, MA_NOACTIVATE, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE,
    WM_MOUSEACTIVATE, WM_PAINT, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::config::{CandidateStyle, CandidateTheme};
use crate::dll_instance;
use crate::placement::{self, Area, Caret};
use crate::session::{Popup, Run};

pub use theme::{Color, Palette};

const CLASS_NAME: PCWSTR = w!("SeokdokCandidateWindow");

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
            style: CS_DROPSHADOW,
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

/// Direct2D, started on the first popup that asks for it.
enum Fluent {
    Unstarted(&'static fluent::Libraries),
    Running(Box<fluent::Renderer>),
    Failed,
}

impl Fluent {
    fn renderer(&mut self) -> Option<&mut fluent::Renderer> {
        if let Fluent::Unstarted(libraries) = *self {
            *self = match fluent::Renderer::start(libraries) {
                Ok(renderer) => Fluent::Running(Box::new(renderer)),
                Err(error) => {
                    crate::debug_log::log(|| format!("Direct2D did not start, drawing with GDI: {error}"));
                    Fluent::Failed
                }
            };
        }
        match self {
            Fluent::Running(renderer) => Some(renderer),
            _ => None,
        }
    }
}

/// How a popup is drawn: the settings as they stood when it was shown.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Look {
    pub style: CandidateStyle,
    pub theme: CandidateTheme,
}

impl Look {
    fn configured() -> Self {
        let windows = crate::config::current().windows;
        Look { style: windows.candidate_style, theme: windows.candidate_theme }
    }
}

/// What the compositor was last told about the window.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Frame {
    rounded: bool,
    dark: bool,
}

/// What the window procedure paints.
enum Paint {
    Fluent(fluent::Scene),
    Gdi(gdi::Paint),
}

struct Content {
    preedit: Option<Vec<Run>>,
    popup: Option<Popup>,
    anchor: Option<Caret>,
    look: Look,
}

struct Inner {
    gdi: gdi::Gdi,
    fluent: Fluent,
    frame: Option<Frame>,
    paint: Option<Paint>,
    /// The latest `show`, for drawing it again after a fallback.
    content: Option<Content>,
}

pub struct CandidateWindow {
    hwnd: HWND,
    /// Boxed for a stable address in the window's user data.
    inner: Box<RefCell<Inner>>,
}

impl CandidateWindow {
    /// Create the hidden popup. `None` if creation fails.
    pub fn new() -> Option<Self> {
        Self::with_libraries(&fluent::SYSTEM_LIBRARIES)
    }

    /// The same on a system without the Direct2D libraries.
    #[doc(hidden)]
    pub fn without_direct2d() -> Option<Self> {
        Self::with_libraries(&fluent::MISSING_LIBRARIES)
    }

    fn with_libraries(libraries: &'static fluent::Libraries) -> Option<Self> {
        register_class();
        // SAFETY: the class is registered; a popup has no parent.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                CLASS_NAME,
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                100,
                26,
                None,
                None,
                Some(hinstance()),
                None,
            )
            .ok()?
        };
        let inner = Box::new(RefCell::new(Inner {
            gdi: gdi::Gdi::default(),
            fluent: Fluent::Unstarted(libraries),
            frame: None,
            paint: None,
            content: None,
        }));
        // SAFETY: the box outlives the window, which Drop destroys first.
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, &*inner as *const RefCell<Inner> as isize);
        }
        Some(Self { hwnd, inner })
    }

    /// Show the preedit line (if any) and `popup` (if any) by `anchor`, or by
    /// the focus caret when the application did not report a position.
    pub fn show(&self, preedit: Option<&[Run]>, popup: Option<&Popup>, anchor: Option<Caret>) {
        self.show_as(Look::configured(), preedit, popup, anchor);
    }

    /// The same in a look of the caller's choosing.
    #[doc(hidden)]
    pub fn show_as(&self, look: Look, preedit: Option<&[Run]>, popup: Option<&Popup>, anchor: Option<Caret>) {
        self.inner.borrow_mut().content =
            Some(Content { preedit: preedit.map(<[Run]>::to_vec), popup: popup.cloned(), anchor, look });
        present(self.hwnd, &self.inner);
    }

    pub fn hide(&self) {
        // SAFETY: hwnd valid.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    #[doc(hidden)]
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Whether the latest popup was laid out with the Fluent drawing.
    #[doc(hidden)]
    pub fn is_fluent(&self) -> bool {
        matches!(self.inner.borrow().paint, Some(Paint::Fluent(_)))
    }

    /// Make the next paint fail, as a broken driver would.
    #[doc(hidden)]
    pub fn fail_next_paint(&self) {
        if let Fluent::Running(renderer) = &self.inner.borrow().fluent {
            renderer.fail_next_paint();
        }
    }
}

impl Drop for CandidateWindow {
    fn drop(&mut self) {
        // SAFETY: hwnd was created in `with_libraries`.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Lay out the latest content in its look and show it.
fn present(hwnd: HWND, cell: &RefCell<Inner>) {
    let (x, y, width, height) = {
        let mut inner = cell.borrow_mut();
        let inner = &mut *inner;
        let Some(content) = &inner.content else { return };
        let preedit = content.preedit.as_deref();
        let popup = content.popup.as_ref();
        let anchor = content.anchor.unwrap_or_else(caret_screen_pos);
        let look = content.look;
        let dpi = dpi_at(anchor);

        let mut drawn = None;
        if look.style == CandidateStyle::Windows11 {
            if let Some(renderer) = inner.fluent.renderer() {
                let palette = Palette::current(look.theme);
                match renderer.layout(preedit, popup, &palette, dpi as f32) {
                    Ok(scene) => {
                        let (width, height) = scene.pixel_size();
                        drawn = Some((Paint::Fluent(scene), width, height, Frame { rounded: true, dark: palette.dark }));
                    }
                    Err(error) => {
                        crate::debug_log::log(|| format!("Fluent candidate layout failed, drawing with GDI: {error}"));
                        inner.fluent = Fluent::Failed;
                    }
                }
            }
        }
        let (paint, width, height, frame) = drawn.unwrap_or_else(|| {
            let palette = Palette::classic(look.theme);
            // SAFETY: hwnd valid; the DC is released below.
            let (paint, width, height) = unsafe {
                let hdc = GetDC(Some(hwnd));
                let result = inner.gdi.layout(hdc, preedit, popup, &palette, dpi);
                ReleaseDC(Some(hwnd), hdc);
                result
            };
            (Paint::Gdi(paint), width, height, Frame { rounded: false, dark: palette.dark })
        });
        inner.paint = Some(paint);
        if inner.frame != Some(frame) {
            inner.frame = Some(frame);
            set_frame(hwnd, frame);
        }
        let (x, y, width) = placement::place(anchor, width, height, work_area_at(anchor));
        (x, y, width, height)
    };
    crate::debug_log::log(|| format!("popup at {x},{y} size {width}x{height}"));
    // SAFETY: hwnd is valid for this object's lifetime. No borrow is held
    // here: the window procedure paints during UpdateWindow.
    unsafe {
        let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), x, y, width, height, SWP_NOACTIVATE);
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = InvalidateRect(Some(hwnd), None, true);
        let _ = UpdateWindow(hwnd);
    }
}

fn monitor_at(anchor: Caret) -> HMONITOR {
    // SAFETY: MonitorFromPoint takes plain values.
    unsafe { MonitorFromPoint(POINT { x: anchor.x, y: anchor.top }, MONITOR_DEFAULTTONEAREST) }
}

/// The dpi of the monitor at a point, as this thread's awareness sees it.
fn dpi_at(anchor: Caret) -> u32 {
    let (mut dpi_x, mut dpi_y) = (96, 96);
    // SAFETY: the outputs are valid for the call.
    if unsafe { GetDpiForMonitor(monitor_at(anchor), MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }.is_err() {
        return 96;
    }
    dpi_x.max(96)
}

/// The monitor's area outside the taskbar.
fn work_area_at(anchor: Caret) -> Area {
    let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    // SAFETY: info is sized and valid for the call. The nearest monitor always exists.
    let _ = unsafe { GetMonitorInfoW(monitor_at(anchor), &mut info) };
    let work = info.rcWork;
    Area { left: work.left, top: work.top, right: work.right, bottom: work.bottom }
}

/// Tell the compositor about the look. On Windows 11 it rounds the window and
/// adds its edge, in the theme's colour, and the class shadow follows the shape.
/// Windows 10 refuses the corner preference and keeps the corners square.
fn set_frame(hwnd: HWND, frame: Frame) {
    let corners = if frame.rounded { DWMWCP_ROUND } else { DWMWCP_DONOTROUND };
    let dark = BOOL::from(frame.dark);
    // SAFETY: hwnd is valid and both values live through the calls. Failing
    // only costs the corners or the edge colour.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corners as *const _ as *const _,
            std::mem::size_of_val(&corners) as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const _ as *const _,
            std::mem::size_of_val(&dark) as u32,
        );
    }
}

/// The focus caret, or failing that the bottom left of the foreground window,
/// nearer most message fields than its top.
fn caret_screen_pos() -> Caret {
    let mut gti = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
    // SAFETY: gti is sized; thread id 0 = the foreground thread. The points are
    // valid outputs and the windows were just named by the system.
    unsafe {
        if GetGUIThreadInfo(0, &mut gti).is_ok() && !gti.hwndCaret.is_invalid() {
            let mut pt = POINT { x: gti.rcCaret.left, y: gti.rcCaret.top };
            if ClientToScreen(gti.hwndCaret, &mut pt).as_bool() {
                return Caret { x: pt.x, top: pt.y, bottom: pt.y + gti.rcCaret.bottom - gti.rcCaret.top };
            }
        }
        let foreground = GetForegroundWindow();
        let mut client = RECT::default();
        let mut pt = POINT::default();
        if GetClientRect(foreground, &mut client).is_ok() {
            pt.y = client.bottom;
            let _ = ClientToScreen(foreground, &mut pt);
        }
        Caret { x: pt.x, top: pt.y, bottom: pt.y }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // WS_EX_NOACTIVATE alone lets a click take the foreground from a document
    // of the same thread.
    if msg == WM_MOUSEACTIVATE {
        return LRESULT(MA_NOACTIVATE as isize);
    }
    if msg != WM_PAINT {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let cell = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<Inner>;
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut rc = RECT::default();
    let _ = GetClientRect(hwnd, &mut rc);
    let mut failed = false;
    // SAFETY: set in `with_libraries`, before the window can be shown, and the
    // box outlives the window. A paint that arrives mid-update has nothing
    // settled to draw and is followed by the update's own; a panic here would
    // take the host application down.
    if let Some(Ok(mut inner)) = cell.as_ref().map(RefCell::try_borrow_mut) {
        let inner = &mut *inner;
        match (&mut inner.fluent, &inner.paint) {
            (Fluent::Running(renderer), Some(Paint::Fluent(scene))) => {
                if let Err(error) = renderer.paint(hdc, &rc, scene) {
                    crate::debug_log::log(|| format!("Direct2D paint failed, drawing with GDI: {error}"));
                    failed = true;
                }
            }
            (_, Some(Paint::Gdi(paint))) => gdi::paint(hdc, rc, paint),
            _ => {}
        }
        if failed {
            inner.fluent = Fluent::Failed;
            inner.paint = None;
        }
    }
    let _ = EndPaint(hwnd, &ps);
    if failed {
        present(hwnd, &*cell);
    }
    LRESULT(0)
}

/// Offscreen rendering with the same drawing code, for the preview example.
#[doc(hidden)]
pub mod preview {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC,
    };

    use super::{fluent, gdi, Palette};
    use crate::session::{Popup, Run};

    /// Top-down BGRA pixels.
    pub struct Image {
        pub width: i32,
        pub height: i32,
        pub bgra: Vec<u8>,
    }

    fn render(
        width: i32,
        height: i32,
        draw: impl FnOnce(HDC, &RECT) -> windows::core::Result<()>,
    ) -> windows::core::Result<Image> {
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        // SAFETY: the bitmap and DC are created and released here; the pixel
        // pointer is valid while the bitmap lives and is read after GdiFlush.
        unsafe {
            let dc = CreateCompatibleDC(None);
            let mut bits = std::ptr::null_mut();
            let bitmap = match CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
                Ok(bitmap) => bitmap,
                Err(error) => {
                    let _ = DeleteDC(dc);
                    return Err(error);
                }
            };
            let old = SelectObject(dc, bitmap.into());
            let rc = RECT { left: 0, top: 0, right: width, bottom: height };
            let result = draw(dc, &rc);
            let _ = GdiFlush();
            let size = width as usize * height as usize * 4;
            let mut bgra = std::slice::from_raw_parts(bits as *const u8, size).to_vec();
            for px in bgra.chunks_exact_mut(4) {
                px[3] = 0xFF;
            }
            SelectObject(dc, old);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            result.map(|()| Image { width, height, bgra })
        }
    }

    pub fn fluent(
        preedit: Option<&[Run]>,
        popup: Option<&Popup>,
        palette: &Palette,
        dpi: u32,
    ) -> windows::core::Result<Image> {
        let mut renderer = fluent::Renderer::start(&fluent::SYSTEM_LIBRARIES)?;
        let scene = renderer.layout(preedit, popup, palette, dpi as f32)?;
        let (width, height) = scene.pixel_size();
        render(width, height, |dc, rc| renderer.paint(dc, rc, &scene))
    }

    pub fn gdi(
        preedit: Option<&[Run]>,
        popup: Option<&Popup>,
        palette: &Palette,
        dpi: u32,
    ) -> windows::core::Result<Image> {
        let mut gdi = gdi::Gdi::default();
        // SAFETY: a throwaway DC for measuring, deleted below.
        let (paint, width, height) = unsafe {
            let dc = CreateCompatibleDC(None);
            let result = gdi.layout(dc, preedit, popup, palette, dpi);
            let _ = DeleteDC(dc);
            result
        };
        render(width, height, |dc, rc| {
            gdi::paint(dc, *rc, &paint);
            Ok(())
        })
    }

    /// Installed-font and shaping facts for the preview's report.
    pub fn report() -> windows::core::Result<Vec<String>> {
        let renderer = fluent::Renderer::start(&fluent::SYSTEM_LIBRARIES)?;
        let mut lines = Vec::new();
        for family in ["Malgun Gothic", "Segoe UI", "Segoe UI Variable", "Batang", "SimSun-ExtB"] {
            lines.push(format!("font {family}: {}", renderer.family_installed(family)));
        }
        for text in ["ᄉᆞ랑", "ᄒᆞᆯ", "追加ᄒᆞᆯ까요"] {
            lines.push(format!("clusters {text}: {}", renderer.cluster_count(text)?));
        }
        Ok(lines)
    }

    #[cfg(test)]
    mod tests {
        use windows::Win32::Graphics::Gdi::{SetLayout, LAYOUT_RTL};

        use super::*;

        #[test]
        fn a_mirrored_device_context_does_not_mirror_the_popup() {
            let popup = Popup {
                items: vec![("家".into(), "집 가".into())],
                selected: 0,
                position: 1,
                total: 9,
                horizontal: false,
            };
            let palette = Palette::default_light();
            let mut renderer = fluent::Renderer::start(&fluent::SYSTEM_LIBRARIES).unwrap();
            let scene = renderer.layout(None, Some(&popup), &palette, 96.0).unwrap();
            let (width, height) = scene.pixel_size();
            let plain = render(width, height, |dc, rc| renderer.paint(dc, rc, &scene)).unwrap();
            let mirrored = render(width, height, |dc, rc| {
                // SAFETY: dc is the render's own.
                unsafe { SetLayout(dc, LAYOUT_RTL) };
                renderer.paint(dc, rc, &scene)
            })
            .unwrap();
            assert!(plain.bgra == mirrored.bgra);
        }

        #[test]
        fn a_mirrored_device_context_does_not_mirror_the_classic_popup() {
            let popup = Popup {
                items: vec![("家".into(), "집 가".into())],
                selected: 0,
                position: 1,
                total: 9,
                horizontal: false,
            };
            let palette = Palette::default_classic_light();
            let mut gdi = gdi::Gdi::default();
            // SAFETY: a throwaway DC for measuring, deleted below.
            let (paint, width, height) = unsafe {
                let dc = CreateCompatibleDC(None);
                let result = gdi.layout(dc, None, Some(&popup), &palette, 96);
                let _ = DeleteDC(dc);
                result
            };
            let draw = |dc, rc: &RECT| {
                gdi::paint(dc, *rc, &paint);
                Ok(())
            };
            let plain = render(width, height, draw).unwrap();
            let mirrored = render(width, height, |dc, rc| {
                // SAFETY: dc is the render's own.
                unsafe { SetLayout(dc, LAYOUT_RTL) };
                draw(dc, rc)
            })
            .unwrap();
            assert!(plain.bgra == mirrored.bgra);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Popup {
        Popup { items: vec![("家".into(), "집 가".into())], selected: 0, position: 1, total: 1, horizontal: false }
    }

    const CARET: Caret = Caret { x: 10, top: 10, bottom: 30 };
    const MODERN: Look = Look { style: CandidateStyle::Windows11, theme: CandidateTheme::Light };
    const CLASSIC: Look = Look { style: CandidateStyle::Classic, theme: CandidateTheme::Dark };

    #[test]
    fn the_classic_style_never_starts_direct2d() {
        let window = CandidateWindow::new().unwrap();
        window.show_as(CLASSIC, None, Some(&sample()), Some(CARET));
        assert!(!window.is_fluent());
        assert!(matches!(window.inner.borrow().fluent, Fluent::Unstarted(_)));
    }

    #[test]
    fn the_style_can_change_between_shows() {
        let window = CandidateWindow::new().unwrap();
        for (look, fluent) in [(MODERN, true), (CLASSIC, false), (MODERN, true)] {
            window.show_as(look, None, Some(&sample()), Some(CARET));
            assert_eq!(window.is_fluent(), fluent);
        }
    }

    #[test]
    fn a_failed_paint_stays_classic_whatever_the_setting() {
        let window = CandidateWindow::new().unwrap();
        window.show_as(MODERN, None, Some(&sample()), Some(CARET));
        window.fail_next_paint();
        window.show_as(MODERN, None, Some(&sample()), Some(CARET));
        window.show_as(MODERN, None, Some(&sample()), Some(CARET));
        assert!(!window.is_fluent());
    }

    #[test]
    fn missing_libraries_fall_back_to_gdi() {
        let window = CandidateWindow::without_direct2d().unwrap();
        window.show(None, Some(&sample()), Some(Caret { x: 10, top: 10, bottom: 30 }));
        assert!(!window.is_fluent());
        assert!(matches!(window.inner.borrow().paint, Some(Paint::Gdi(_))));
    }

    #[test]
    fn a_failed_paint_falls_back_to_gdi() {
        let window = CandidateWindow::new().unwrap();
        window.show(None, Some(&sample()), Some(Caret { x: 10, top: 10, bottom: 30 }));
        assert!(window.is_fluent());
        window.fail_next_paint();
        window.show(None, Some(&sample()), Some(Caret { x: 10, top: 10, bottom: 30 }));
        assert!(!window.is_fluent());
        assert!(matches!(window.inner.borrow().paint, Some(Paint::Gdi(_))));
    }
}
