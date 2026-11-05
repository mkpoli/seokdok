//! GUIDs, names and resource ids. The GUIDs and names are the product's own
//! (products/<id>.json), so the input methods install side by side.

use seokdok_product::CURRENT;
use windows::core::GUID;

/// Text-service CLSID.
pub const GUID_TEXT_SERVICE: GUID = GUID::from_u128(CURRENT.windows.clsid);

/// Language profile.
pub const GUID_PROFILE: GUID = GUID::from_u128(CURRENT.windows.profile);

/// Display attribute of Hangul that has not been converted.
pub const GUID_DA_INPUT: GUID = GUID::from_u128(CURRENT.windows.da_input);

/// Display attribute of a converted segment.
pub const GUID_DA_CONVERTED: GUID = GUID::from_u128(CURRENT.windows.da_converted);

/// Display attribute of the focused converted segment.
pub const GUID_DA_FOCUS: GUID = GUID::from_u128(CURRENT.windows.da_focus);

/// Icon resource ids (see `resources.rc`); the profile icon is id 101.
pub const IDI_MODE_SEOKDOK: u16 = 102;
pub const IDI_MODE_HANGUL: u16 = 103;
pub const IDI_MODE_LATN: u16 = 104;
pub const IDI_MODE_SINO: u16 = 105;

pub const SERVICE_NAME: &str = CURRENT.windows.service_name;

/// Name shown in the language switcher.
pub const DISPLAY_DESCRIPTION: &str = CURRENT.windows.display_name;

/// LANGID of ko-KR.
pub const LANGID_KO_KR: u16 = 0x0412;
