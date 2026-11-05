//! The candidate popup: a window listing one page of candidates, each as a
//! number, the text and a dimmer note, with the position in the list below.
//! For IMM32 applications it also carries the text being composed on a line
//! of its own above the candidates (see `composition.rs`).
//!
//! `fluent.rs` draws it the way Windows 11 draws its own IME's list. If that
//! cannot start or fails while painting, `gdi.rs` takes over for the rest of
//! the window's life. A file named `gdi-candidates` in
//! %LOCALAPPDATA%\seokdok asks for `gdi.rs` from the start.
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
    DWMWCP_ROUND, DWM_WINDOW_CORNER_PREFERENCE,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, EndPaint, GetDC, GetDeviceCaps, GetMonitorInfoW, InvalidateRect, MonitorFromPoint,
    ReleaseDC, UpdateWindow, HMONITOR, LOGPIXELSY, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClassLongPtrW, GetClientRect, GetForegroundWindow, GetGUIThreadInfo,
    GetWindowLongPtrW, LoadCursorW, RegisterClassExW, SetClassLongPtrW, SetWindowLongPtrW,
    SetWindowPos, ShowWindow, CS_DROPSHADOW, GCL_STYLE, GUITHREADINFO, GWLP_USERDATA, GWL_STYLE, HWND_TOPMOST,
    IDC_ARROW, MA_NOACTIVATE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SW_HIDE, SW_SHOWNOACTIVATE, WM_MOUSEACTIVATE, WM_PAINT, WNDCLASSEXW, WS_BORDER, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};

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

/// Which drawing the window uses. The first popup decides.
enum Backend {
    Unstarted(&'static fluent::Libraries),
    Fluent {
        renderer: Box<fluent::Renderer>,
        /// The theme the frame was last told about.
        dark: Option<bool>,
    },
    Gdi,
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
}

struct Inner {
    gdi: gdi::Gdi,
    backend: Backend,
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
        // SAFETY: GetDC(None) is the screen; released below.
        let dpi = unsafe {
            let hdc = GetDC(None);
            let dpi = GetDeviceCaps(Some(hdc), LOGPIXELSY).max(96);
            ReleaseDC(None, hdc);
            dpi
        };
        let gdi = gdi::Gdi::new(dpi);
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
                gdi.line_height,
                None,
                None,
                Some(hinstance()),
                None,
            )
            .ok()?
        };
        let inner =
            Box::new(RefCell::new(Inner { gdi, backend: Backend::Unstarted(libraries), paint: None, content: None }));
        // SAFETY: the box outlives the window, which Drop destroys first.
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, &*inner as *const RefCell<Inner> as isize);
        }
        Some(Self { hwnd, inner })
    }

    /// Show the preedit line (if any) and `popup` (if any) by `anchor`, or by
    /// the focus caret when the application did not report a position.
    pub fn show(&self, preedit: Option<&[Run]>, popup: Option<&Popup>, anchor: Option<Caret>) {
        self.inner.borrow_mut().content =
            Some(Content { preedit: preedit.map(<[Run]>::to_vec), popup: popup.cloned(), anchor });
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

    /// Whether the Fluent drawing is in use.
    #[doc(hidden)]
    pub fn is_fluent(&self) -> bool {
        matches!(self.inner.borrow().backend, Backend::Fluent { .. })
    }

    /// Make the next paint fail, as a broken driver would.
    #[doc(hidden)]
    pub fn fail_next_paint(&self) {
        if let Backend::Fluent { renderer, .. } = &self.inner.borrow().backend {
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

/// Lay out the latest content with the current backend and show it.
fn present(hwnd: HWND, cell: &RefCell<Inner>) {
    start_backend(hwnd, cell);
    let placed = {
        let mut inner = cell.borrow_mut();
        let inner = &mut *inner;
        let Some(content) = &inner.content else { return };
        let preedit = content.preedit.as_deref();
        let popup = content.popup.as_ref();
        let anchor = content.anchor.unwrap_or_else(caret_screen_pos);
        if let Backend::Fluent { renderer, dark } = &mut inner.backend {
            let palette = Palette::current();
            if *dark != Some(palette.dark) {
                *dark = Some(palette.dark);
                tell_frame_theme(hwnd, palette.dark);
            }
            match layout_fluent(renderer, preedit, popup, &palette, anchor) {
                Ok((scene, placed)) => {
                    inner.paint = Some(Paint::Fluent(scene));
                    Some(placed)
                }
                Err(error) => {
                    crate::debug_log::log(|| format!("Fluent candidate layout failed, drawing with GDI: {error}"));
                    inner.backend = Backend::Gdi;
                    inner.paint = None;
                    None
                }
            }
        } else {
            // SAFETY: hwnd valid; the DC is released below.
            let (paint, width, height) = unsafe {
                let hdc = GetDC(Some(hwnd));
                let result = inner.gdi.layout(hdc, preedit, popup);
                ReleaseDC(Some(hwnd), hdc);
                result
            };
            inner.paint = Some(Paint::Gdi(paint));
            let (x, y, width) = placement::place(anchor, width, height, work_area_at(anchor));
            // The border is outside the client area.
            Some((x, y, width + 2, height + 2))
        }
    };
    let Some((x, y, width, height)) = placed else {
        set_frame(hwnd, true, DWMWCP_DONOTROUND);
        return present(hwnd, cell);
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

/// Lay out for the monitor the text is on, which the popup stays on.
fn layout_fluent(
    renderer: &fluent::Renderer,
    preedit: Option<&[Run]>,
    popup: Option<&Popup>,
    palette: &Palette,
    anchor: Caret,
) -> windows::core::Result<(fluent::Scene, (i32, i32, i32, i32))> {
    let scene = renderer.layout(preedit, popup, palette, dpi_at(anchor) as f32)?;
    let (width, height) = scene.pixel_size();
    let (x, y, width) = placement::place(anchor, width, height, work_area_at(anchor));
    Ok((scene, (x, y, width, height)))
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

/// Decide the backend on the first popup.
fn start_backend(hwnd: HWND, cell: &RefCell<Inner>) {
    let mut inner = cell.borrow_mut();
    let Backend::Unstarted(libraries) = inner.backend else { return };
    if gdi_requested() {
        inner.backend = Backend::Gdi;
        return;
    }
    inner.backend = match fluent::Renderer::start(libraries) {
        Ok(renderer) => {
            set_frame(hwnd, false, DWMWCP_ROUND);
            Backend::Fluent { renderer: Box::new(renderer), dark: None }
        }
        Err(error) => {
            crate::debug_log::log(|| format!("Direct2D did not start, drawing with GDI: {error}"));
            Backend::Gdi
        }
    };
}

/// `gdi-candidates` beside the debug trace's `debug` file.
fn gdi_requested() -> bool {
    std::env::var_os("LOCALAPPDATA")
        .is_some_and(|d| std::path::Path::new(&d).join("seokdok").join("gdi-candidates").exists())
}

/// Switch between the two looks. The Fluent frame has no border of its own: on
/// Windows 11 the compositor rounds the window and adds its edge, and the class
/// shadow follows the rounded shape. Windows 10 refuses the corner preference
/// and keeps the corners square.
fn set_frame(hwnd: HWND, border: bool, corners: DWM_WINDOW_CORNER_PREFERENCE) {
    // SAFETY: hwnd is valid. Only frame bits change, and the flags keep
    // activation, position and z-order as they are.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let bit = WS_BORDER.0 as isize;
        SetWindowLongPtrW(hwnd, GWL_STYLE, if border { style | bit } else { style & !bit });
        let class = GetClassLongPtrW(hwnd, GCL_STYLE);
        let shadow = CS_DROPSHADOW.0 as usize;
        SetClassLongPtrW(hwnd, GCL_STYLE, (if border { class & !shadow } else { class | shadow }) as isize);
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corners as *const _ as *const _,
            std::mem::size_of_val(&corners) as u32,
        );
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

/// So the compositor's edge and shadow match the panel.
fn tell_frame_theme(hwnd: HWND, dark: bool) {
    let value = BOOL::from(dark);
    // SAFETY: value lives through the call. Failing only costs the edge colour.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &value as *const _ as *const _,
            std::mem::size_of_val(&value) as u32,
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
        match (&mut inner.backend, &inner.paint) {
            (Backend::Fluent { renderer, .. }, Some(Paint::Fluent(scene))) => {
                if let Err(error) = renderer.paint(hdc, &rc, scene) {
                    crate::debug_log::log(|| format!("Direct2D paint failed, drawing with GDI: {error}"));
                    failed = true;
                }
            }
            (_, Some(Paint::Gdi(paint))) => gdi::paint(hdc, rc, paint),
            _ => {}
        }
        if failed {
            inner.backend = Backend::Gdi;
            inner.paint = None;
        }
    }
    let _ = EndPaint(hwnd, &ps);
    if failed {
        set_frame(hwnd, true, DWMWCP_DONOTROUND);
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

    pub fn gdi(preedit: Option<&[Run]>, popup: Option<&Popup>, dpi: u32) -> windows::core::Result<Image> {
        let gdi = gdi::Gdi::new(dpi as i32);
        // SAFETY: a throwaway DC for measuring, deleted below.
        let (paint, width, height) = unsafe {
            let dc = CreateCompatibleDC(None);
            let result = gdi.layout(dc, preedit, popup);
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Popup {
        Popup { items: vec![("家".into(), "집 가".into())], selected: 0, position: 1, total: 1, horizontal: false }
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
