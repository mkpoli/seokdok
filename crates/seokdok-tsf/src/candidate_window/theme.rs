//! The colours of the Fluent candidate window, read from the system on every
//! show so a change of theme or accent reaches the next popup.
//!
//! The values are WinUI's theme resources: `Common_themeresources_any.xaml`
//! for text, fills and strokes, `AcrylicBrush_themeresources.xaml` for the
//! panel (the fallback colour of the flyout acrylic), and
//! `ListViewItem_themeresources.xaml` for the selection indicator.

use windows::core::{w, PCWSTR};
use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;
use windows::Win32::Graphics::Gdi::{
    GetSysColor, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWTEXT, SYS_COLOR_INDEX,
};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_BINARY, RRF_RT_REG_DWORD};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

/// Straight (not premultiplied) RGBA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

const fn argb(v: u32) -> Color {
    Color { a: (v >> 24) as u8, r: (v >> 16) as u8, g: (v >> 8) as u8, b: v as u8 }
}

/// Windows' default accent shades, for when the system reports none.
const DEFAULT_DARK1: Color = argb(0xFF00_67C0);
const DEFAULT_LIGHT2: Color = argb(0xFF4C_C2FF);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub panel: Color,
    pub selection: Color,
    pub text: Color,
    pub selected_text: Color,
    pub secondary: Color,
    pub divider: Color,
    pub outline: Color,
    pub accent: Color,
}

impl Palette {
    /// `AcrylicBackgroundFillColorDefaultBrush` (fallback), `SubtleFillColorSecondary`,
    /// `TextFillColorPrimary`, `TextFillColorSecondary`, `DividerStrokeColorDefault`,
    /// `SurfaceStrokeColorFlyout`; `AccentFillColorDefault` is `SystemAccentColorDark1`.
    pub const fn light(accent_dark1: Color) -> Self {
        let text = argb(0xE400_0000);
        Self {
            dark: false,
            panel: argb(0xFFF9_F9F9),
            selection: argb(0x0900_0000),
            text,
            selected_text: text,
            secondary: argb(0x9E00_0000),
            divider: argb(0x0F00_0000),
            outline: argb(0x0F00_0000),
            accent: accent_dark1,
        }
    }

    /// The same resources in the dark dictionary, where `AccentFillColorDefault`
    /// is `SystemAccentColorLight2`.
    pub const fn dark(accent_light2: Color) -> Self {
        let text = argb(0xFFFF_FFFF);
        Self {
            dark: true,
            panel: argb(0xFF2C_2C2C),
            selection: argb(0x0FFF_FFFF),
            text,
            selected_text: text,
            secondary: argb(0xC5FF_FFFF),
            divider: argb(0x15FF_FFFF),
            outline: argb(0x3300_0000),
            accent: accent_light2,
        }
    }

    /// With Windows' default accent, for previews.
    pub const fn default_light() -> Self {
        Self::light(DEFAULT_DARK1)
    }

    pub const fn default_dark() -> Self {
        Self::dark(DEFAULT_LIGHT2)
    }

    /// The user's contrast theme, as WinUI maps it: window colours, with the
    /// selection in the highlight colours.
    fn high_contrast() -> Self {
        let text = sys_color(COLOR_WINDOWTEXT);
        let highlight_text = sys_color(COLOR_HIGHLIGHTTEXT);
        Self {
            dark: false,
            panel: sys_color(COLOR_WINDOW),
            selection: sys_color(COLOR_HIGHLIGHT),
            text,
            selected_text: highlight_text,
            secondary: text,
            divider: text,
            outline: text,
            accent: highlight_text,
        }
    }

    /// What the system asks for now.
    pub fn current() -> Self {
        if high_contrast_on() {
            return Self::high_contrast();
        }
        let shades = accent_palette().or_else(accent_from_dwm);
        if apps_use_light_theme() {
            Self::light(shades.map_or(DEFAULT_DARK1, |s| s.dark1))
        } else {
            Self::dark(shades.map_or(DEFAULT_LIGHT2, |s| s.light2))
        }
    }
}

fn sys_color(index: SYS_COLOR_INDEX) -> Color {
    // SAFETY: plain query of a system colour.
    let v = unsafe { GetSysColor(index) };
    Color { r: v as u8, g: (v >> 8) as u8, b: (v >> 16) as u8, a: 0xFF }
}

fn high_contrast_on() -> bool {
    let mut hc = HIGHCONTRASTW { cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32, ..Default::default() };
    // SAFETY: hc is sized and writable for the call.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            hc.cbSize,
            Some(&mut hc as *mut HIGHCONTRASTW as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    ok.is_ok() && hc.dwFlags.contains(HCF_HIGHCONTRASTON)
}

const PERSONALIZE: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
const ACCENT: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent");

/// The "default app mode" in Settings > Personalization > Colors. Light when
/// the value is missing, which is also the system's own default.
fn apps_use_light_theme() -> bool {
    let mut value = 1u32;
    let mut size = 4u32;
    // SAFETY: value and size describe a writable DWORD.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PERSONALIZE,
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    status.is_err() || value != 0
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Shades {
    light2: Color,
    dark1: Color,
}

/// `AccentPalette` holds eight RGBA colours, lightest first: Light3, Light2,
/// Light1, the accent, Dark1, Dark2, Dark3, and one more.
fn parse_accent_palette(bytes: &[u8]) -> Option<Shades> {
    if bytes.len() != 32 {
        return None;
    }
    let at = |i: usize| Color { r: bytes[i * 4], g: bytes[i * 4 + 1], b: bytes[i * 4 + 2], a: 0xFF };
    Some(Shades { light2: at(1), dark1: at(4) })
}

fn accent_palette() -> Option<Shades> {
    let mut bytes = [0u8; 32];
    let mut size = bytes.len() as u32;
    // SAFETY: bytes and size describe a writable buffer.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            ACCENT,
            w!("AccentPalette"),
            RRF_RT_REG_BINARY,
            None,
            Some(bytes.as_mut_ptr() as *mut _),
            Some(&mut size),
        )
    };
    if status.is_err() {
        return None;
    }
    parse_accent_palette(&bytes[..size as usize])
}

/// The documented colourization colour stands in for both shades.
fn accent_from_dwm() -> Option<Shades> {
    let mut value = 0u32;
    let mut opaque = false.into();
    // SAFETY: both pointers are valid for the call.
    unsafe { DwmGetColorizationColor(&mut value, &mut opaque) }.ok()?;
    let color = Color { a: 0xFF, ..argb(value) };
    Some(Shades { light2: color, dark1: color })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_palette_picks_the_shades_winui_uses() {
        // Windows' default blue.
        let bytes: [u8; 32] = [
            0x99, 0xEB, 0xFF, 0, 0x4C, 0xC2, 0xFF, 0, 0x00, 0x91, 0xF8, 0, 0x00, 0x78, 0xD4, 0, 0x00, 0x67, 0xC0, 0,
            0x00, 0x3E, 0x92, 0, 0x00, 0x1A, 0x68, 0, 0xF7, 0x63, 0x0C, 0,
        ];
        let shades = parse_accent_palette(&bytes).unwrap();
        assert_eq!(shades.light2, Color { r: 0x4C, g: 0xC2, b: 0xFF, a: 0xFF });
        assert_eq!(shades.dark1, Color { r: 0x00, g: 0x67, b: 0xC0, a: 0xFF });
    }

    #[test]
    fn a_palette_of_another_size_is_rejected() {
        assert_eq!(parse_accent_palette(&[0; 28]), None);
    }

    #[test]
    fn the_current_palette_is_opaque_where_it_must_be() {
        let p = Palette::current();
        assert_eq!(p.panel.a, 0xFF);
        assert_eq!(p.accent.a, 0xFF);
    }
}
