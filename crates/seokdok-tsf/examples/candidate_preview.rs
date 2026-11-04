//! Render and exercise the candidate window without an input method around it.
//! Meant for a throwaway machine such as a CI runner: `real` moves the mouse,
//! clicks and takes the foreground.
//!
//!   candidate_preview offscreen OUT   every sample at 100% and 200%, light and
//!                                     dark, plus the GDI drawing
//!   candidate_preview real LABEL OUT  the live window over a document window,
//!                                     captured from the screen, checking that
//!                                     focus never moves
//!   candidate_preview fallback OUT    the live window with Direct2D missing,
//!                                     and after a paint failure
//!   candidate_preview memory fluent|gdi
//!                                     first-show time and memory growth
//!
//! `offscreen` also writes `composed-*`: the pixels with the rounded corners,
//! edge and shadow of the Windows 11 compositor imitated in this file, because
//! Windows does not round windows inside a virtual machine.

#[cfg(not(windows))]
fn main() {
    eprintln!("candidate_preview runs on Windows");
}

#[cfg(windows)]
fn main() {
    app::main();
}

#[cfg(windows)]
mod app {
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    use seokdok::candidate_window::preview::{self, Image};
    use seokdok::candidate_window::{CandidateWindow, Palette};
    use seokdok::session::{Attr, Popup, Run};
    use windows::core::w;
    use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC, DeleteObject, GetDC, GetDIBits,
        ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HBRUSH, SRCCOPY,
    };
    use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX};
    use windows::Win32::System::Threading::GetCurrentProcess;
    use windows::Win32::UI::HiDpi::{
        SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetFocus, SendInput, SetFocus, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect,
        IsWindowVisible, PeekMessageW, RegisterClassExW, SetCursorPos, SetForegroundWindow, ShowWindow, TranslateMessage,
        GWL_EXSTYLE, MSG, PM_REMOVE, SW_SHOW, WINDOW_EX_STYLE, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_OVERLAPPEDWINDOW,
    };

    struct Sample {
        name: &'static str,
        preedit: Option<Vec<Run>>,
        popup: Option<Popup>,
    }

    fn run(text: &str, attr: Attr) -> Run {
        Run { text: text.to_string(), attr }
    }

    fn items(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter().map(|(t, n)| (t.to_string(), n.to_string())).collect()
    }

    /// The first list is the one in Microsoft's Korean IME screenshot, for a
    /// side-by-side comparison.
    fn samples() -> Vec<Sample> {
        let hanja = items(&[
            ("家", "집 가"),
            ("價", "값 가"),
            ("可", "옳을 가"),
            ("加", "더할 가"),
            ("假", "거짓 가"),
            ("歌", "노래 가"),
            ("街", "거리 가"),
            ("伽", "절 가"),
            ("暇", "겨를 가"),
        ]);
        let old = items(&[
            ("ᄉᆞ랑", "옛한글"),
            ("追加ᄒᆞᆯ까요", "末音添記"),
            ("ᄒᆞ", ""),
            ("ᄂᆞᆫ", ""),
            ("ᄅᆞᆯ", ""),
            ("韓㐎 專用", "확장 A"),
            ("𠮷", "확장 B"),
        ]);
        let short = items(&[("工夫", "공부"), ("功夫", "공부"), ("空夫", ""), ("孔夫子", "공부자")]);
        vec![
            Sample {
                name: "list",
                preedit: None,
                popup: Some(Popup { items: hanja, selected: 0, position: 1, total: 24, horizontal: false }),
            },
            Sample {
                name: "old-hangul",
                preedit: None,
                popup: Some(Popup { items: old, selected: 1, position: 2, total: 7, horizontal: false }),
            },
            Sample {
                name: "floating",
                preedit: Some(vec![
                    run("先生님께서 ", Attr::Converted),
                    run("工夫", Attr::Focus),
                    run("ᄒᆞ셨다", Attr::Input),
                ]),
                popup: Some(Popup { items: short.clone(), selected: 0, position: 1, total: 4, horizontal: false }),
            },
            Sample {
                name: "horizontal",
                preedit: None,
                popup: Some(Popup { items: short, selected: 1, position: 2, total: 12, horizontal: true }),
            },
            Sample {
                name: "preedit-only",
                preedit: Some(vec![run("ᄉᆞ랑ᄒᆞ다", Attr::Input)]),
                popup: None,
            },
        ]
    }

    fn save(image: &Image, path: &Path) {
        let file = std::fs::File::create(path).expect("create PNG");
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), image.width as u32, image.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let rgb: Vec<u8> = image.bgra.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0]]).collect();
        encoder.write_header().expect("PNG header").write_image_data(&rgb).expect("PNG data");
    }

    fn mix(under: [f32; 3], over: [f32; 3], alpha: f32) -> [f32; 3] {
        [
            under[0] + (over[0] - under[0]) * alpha,
            under[1] + (over[1] - under[1]) * alpha,
            under[2] + (over[2] - under[2]) * alpha,
        ]
    }

    /// Signed distance from a pixel centre to a rounded rectangle, negative inside.
    fn rounded_distance(x: f32, y: f32, width: f32, height: f32, radius: f32) -> f32 {
        let qx = (x - width / 2.0).abs() - (width / 2.0 - radius);
        let qy = (y - height / 2.0).abs() - (height / 2.0 - radius);
        qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius
    }

    /// Imitate what the Windows 11 compositor adds around the client area: an
    /// 8-radius clip, a one-unit edge and a soft shadow, over the system's base
    /// fill (`SolidBackgroundFillColorBase`: #F3F3F3, #202020).
    fn compose(image: &Image, scale: i32, dark: bool) -> Image {
        let margin = 24 * scale;
        let (w, h) = (image.width + margin * 2, image.height + margin * 2);
        let backdrop = if dark { [0x20 as f32, 0x20 as f32, 0x20 as f32] } else { [0xF3 as f32, 0xF3 as f32, 0xF3 as f32] };
        let edge = if dark { [0x45 as f32, 0x45 as f32, 0x45 as f32] } else { [0xD9 as f32, 0xD9 as f32, 0xD9 as f32] };
        let shadow_strength = if dark { 0.45 } else { 0.18 };
        let radius = 8.0 * scale as f32;
        let (iw, ih) = (image.width as f32, image.height as f32);
        let blur = 12.0 * scale as f32;
        let mut bgra = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (px, py) = ((x - margin) as f32 + 0.5, (y - margin) as f32 + 0.5);
                let d = rounded_distance(px, py, iw, ih, radius);
                let shadow_d = rounded_distance(px, py - 2.0 * scale as f32, iw, ih, radius);
                let falloff = (1.0 - (shadow_d.max(0.0) / blur)).clamp(0.0, 1.0);
                let mut color = mix(backdrop, [0.0, 0.0, 0.0], falloff * falloff * shadow_strength);
                let inside = (0.5 - d).clamp(0.0, 1.0);
                if inside > 0.0 && (x - margin) >= 0 && (y - margin) >= 0 && (x - margin) < image.width && (y - margin) < image.height {
                    let i = (((y - margin) * image.width + (x - margin)) * 4) as usize;
                    let p = [image.bgra[i + 2] as f32, image.bgra[i + 1] as f32, image.bgra[i] as f32];
                    let stroke = (d + scale as f32 + 0.5).clamp(0.0, 1.0);
                    color = mix(color, mix(p, edge, stroke), inside);
                }
                let o = ((y * w + x) * 4) as usize;
                bgra[o] = color[2].round() as u8;
                bgra[o + 1] = color[1].round() as u8;
                bgra[o + 2] = color[0].round() as u8;
                bgra[o + 3] = 0xFF;
            }
        }
        Image { width: w, height: h, bgra }
    }

    fn offscreen(out: &Path) {
        for line in preview::report().expect("Direct2D") {
            println!("{line}");
        }
        for sample in samples() {
            let preedit = sample.preedit.as_deref();
            let popup = sample.popup.as_ref();
            for (dpi, percent) in [(96u32, 100), (192, 200)] {
                for (palette, theme) in [(Palette::default_light(), "light"), (Palette::default_dark(), "dark")] {
                    let image = preview::fluent(preedit, popup, &palette, dpi).expect("Direct2D paint");
                    let name = format!("{}-{theme}-{percent}", sample.name);
                    save(&image, &out.join(format!("fluent-{name}.png")));
                    save(&compose(&image, (dpi / 96) as i32, palette.dark), &out.join(format!("composed-{name}.png")));
                }
                let image = preview::gdi(preedit, popup, dpi).expect("GDI paint");
                save(&image, &out.join(format!("gdi-{}-{percent}.png", sample.name)));
            }
        }
    }

    unsafe extern "system" fn document_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    fn pump(duration: Duration) {
        let end = Instant::now() + duration;
        while Instant::now() < end {
            let mut msg = MSG::default();
            // SAFETY: msg is valid; this drains the thread's own queue.
            unsafe {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// A plain window in the theme's colour that stands in for the document.
    fn document(dark: bool) -> HWND {
        let color = if dark { 0x0020_2020 } else { 0x00F3_F3F3 };
        // SAFETY: the class and window are valid for the process's life.
        unsafe {
            let brush: HBRUSH = CreateSolidBrush(COLORREF(color));
            let class = if dark { w!("SeokdokPreviewDocDark") } else { w!("SeokdokPreviewDocLight") };
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(document_proc),
                hbrBackground: brush,
                lpszClassName: class,
                ..Default::default()
            };
            RegisterClassExW(&wc);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class,
                w!("document"),
                WS_OVERLAPPEDWINDOW,
                20,
                20,
                900,
                680,
                None,
                None,
                None,
                None,
            )
            .expect("document window");
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
            let _ = SetFocus(Some(hwnd));
            hwnd
        }
    }

    /// The screen around a window, as the compositor shows it.
    fn capture(hwnd: HWND, out: &Path) {
        let margin = 24;
        let mut rc = RECT::default();
        // SAFETY: every GDI object is released below; the buffer matches the
        // header passed to GetDIBits.
        unsafe {
            GetWindowRect(hwnd, &mut rc).expect("window rectangle");
            let (w, h) = (rc.right - rc.left + margin * 2, rc.bottom - rc.top + margin * 2);
            let screen = GetDC(None);
            let dc = CreateCompatibleDC(Some(screen));
            let bitmap = CreateCompatibleBitmap(screen, w, h);
            let old = SelectObject(dc, bitmap.into());
            BitBlt(dc, 0, 0, w, h, Some(screen), rc.left - margin, rc.top - margin, SRCCOPY | CAPTUREBLT).expect("BitBlt");
            let mut info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bgra = vec![0u8; (w * h * 4) as usize];
            GetDIBits(dc, bitmap, 0, h as u32, Some(bgra.as_mut_ptr() as *mut _), &mut info, DIB_RGB_COLORS);
            SelectObject(dc, old);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            ReleaseDC(None, screen);
            save(&Image { width: w, height: h, bgra }, out);
        }
    }

    fn click(x: i32, y: i32) {
        let input = |flags| INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi: MOUSEINPUT { dwFlags: flags, ..Default::default() } },
        };
        // SAFETY: the inputs are fully initialized.
        unsafe {
            let _ = SetCursorPos(x, y);
            SendInput(&[input(MOUSEEVENTF_LEFTDOWN), input(MOUSEEVENTF_LEFTUP)], std::mem::size_of::<INPUT>() as i32);
        }
    }

    /// The foreground and focus windows, which must never change.
    fn focus_state() -> (HWND, HWND) {
        // SAFETY: plain queries.
        unsafe { (GetForegroundWindow(), GetFocus()) }
    }

    fn real(label: &str, out: &Path) {
        // SAFETY: called before any window exists.
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let palette = Palette::current();
        println!("{label}: system palette {palette:?}");
        let doc = document(palette.dark);
        pump(Duration::from_millis(600));
        let before = focus_state();
        println!("{label}: document {doc:?}, foreground and focus {before:?}");
        let window = CandidateWindow::new().expect("candidate window");
        let mut failures = 0;
        for sample in samples() {
            window.show(sample.preedit.as_deref(), sample.popup.as_ref(), Some((120, 140)));
            pump(Duration::from_millis(400));
            capture(window.hwnd(), &out.join(format!("real-{label}-{}.png", sample.name)));
            let mut rc = RECT::default();
            // SAFETY: valid outputs.
            let (ex, visible) = unsafe {
                let _ = GetWindowRect(window.hwnd(), &mut rc);
                (GetWindowLongPtrW(window.hwnd(), GWL_EXSTYLE), IsWindowVisible(window.hwnd()).as_bool())
            };
            // A click on the popup must not activate it either.
            click(rc.left + 20, rc.top + 20);
            pump(Duration::from_millis(200));
            let after = focus_state();
            let ok = after == before && visible && ex & WS_EX_NOACTIVATE.0 as isize != 0 && window.is_fluent();
            println!("{label}/{}: {:?} visible={visible} focus kept={} fluent={}", sample.name, rc, after == before, window.is_fluent());
            if !ok {
                failures += 1;
            }
            window.hide();
            pump(Duration::from_millis(100));
        }
        if failures > 0 {
            println!("{label}: {failures} sample(s) failed");
            std::process::exit(1);
        }
    }

    fn fallback(out: &Path) {
        // SAFETY: called before any window exists.
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let doc = document(false);
        pump(Duration::from_millis(600));
        println!("document {doc:?}");
        let before = focus_state();
        let list = samples().remove(0);
        let check = |name: &str, window: &CandidateWindow| {
            pump(Duration::from_millis(400));
            capture(window.hwnd(), &out.join(format!("real-fallback-{name}.png")));
            let ok = !window.is_fluent() && focus_state() == before;
            println!("fallback/{name}: gdi={} focus kept={}", !window.is_fluent(), focus_state() == before);
            if !ok {
                std::process::exit(1);
            }
            window.hide();
        };

        let missing = CandidateWindow::without_direct2d().expect("window");
        missing.show(None, list.popup.as_ref(), Some((120, 140)));
        check("missing-libraries", &missing);
        drop(missing);

        let failing = CandidateWindow::new().expect("window");
        failing.show(None, list.popup.as_ref(), Some((120, 140)));
        pump(Duration::from_millis(300));
        assert!(failing.is_fluent(), "Direct2D should start here");
        failing.fail_next_paint();
        failing.show(None, list.popup.as_ref(), Some((120, 140)));
        check("paint-failure", &failing);
    }

    fn memory_counters() -> (usize, usize) {
        let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
        counters.cb = std::mem::size_of_val(&counters) as u32;
        // SAFETY: the structure is sized and writable.
        unsafe {
            K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters as *mut _ as *mut _, counters.cb)
                .expect("memory counters");
        }
        (counters.WorkingSetSize, counters.PrivateUsage)
    }

    fn memory(kind: &str) {
        let list = samples().remove(0);
        let window = if kind == "gdi" { CandidateWindow::without_direct2d() } else { CandidateWindow::new() }.expect("window");
        pump(Duration::from_millis(200));
        let (ws0, private0) = memory_counters();
        let start = Instant::now();
        window.show(None, list.popup.as_ref(), Some((120, 140)));
        let first = start.elapsed();
        pump(Duration::from_millis(200));
        let (ws1, private1) = memory_counters();
        let start = Instant::now();
        for _ in 0..1000 {
            window.show(None, list.popup.as_ref(), Some((120, 140)));
        }
        let per = start.elapsed() / 1000;
        pump(Duration::from_millis(200));
        let (ws2, private2) = memory_counters();
        let mb = |v: usize| v as f64 / 1_048_576.0;
        println!(
            "{kind}: first show {first:?}, later shows {per:?}; base ws {:.1} MB private {:.1} MB; \
             after first show +{:.1} MB ws +{:.1} MB private; after 1000 more +{:.1} MB ws +{:.1} MB private",
            mb(ws0),
            mb(private0),
            mb(ws1.saturating_sub(ws0)),
            mb(private1.saturating_sub(private0)),
            mb(ws2.saturating_sub(ws1)),
            mb(private2.saturating_sub(private1)),
        );
    }

    pub fn main() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let dir = |arg: &String| {
            let path = PathBuf::from(arg);
            std::fs::create_dir_all(&path).expect("output directory");
            path
        };
        match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
            ["offscreen", _] => offscreen(&dir(&args[1])),
            ["real", label, _] => real(label, &dir(&args[2])),
            ["fallback", _] => fallback(&dir(&args[1])),
            ["memory", kind] => memory(kind),
            _ => {
                eprintln!("usage: candidate_preview offscreen OUT | real LABEL OUT | fallback OUT | memory fluent|gdi");
                std::process::exit(2);
            }
        }
    }
}
