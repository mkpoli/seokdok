//! GUIDs, names and resource ids. The GUIDs are specific to seokdok.

use windows::core::{w, GUID, PCWSTR};

/// Text-service CLSID: {AAF18869-67ED-48A1-A3F6-42E961E994A1}
pub const GUID_TEXT_SERVICE: GUID = GUID::from_u128(0xAAF18869_67ED_48A1_A3F6_42E961E994A1);

/// Language profile: {5A2C201D-E01B-475F-A23D-3A6A64D996A4}
pub const GUID_PROFILE: GUID = GUID::from_u128(0x5A2C201D_E01B_475F_A23D_3A6A64D996A4);

/// Display attribute of Hangul that has not been converted.
pub const GUID_DA_INPUT: GUID = GUID::from_u128(0x120E3069_68A8_441A_A00A_C8DC2DA5D954);

/// Display attribute of a converted segment.
pub const GUID_DA_CONVERTED: GUID = GUID::from_u128(0xA824E0C2_E97A_449C_855B_2F9D484E07F4);

/// Display attribute of the focused converted segment.
pub const GUID_DA_FOCUS: GUID = GUID::from_u128(0xA63357BA_AF96_4DB0_82B6_86778F0E8A24);

/// Language-bar mode item: {D840F24E-DD08-4BD2-89F0-EEAA83ABDE79}
pub const GUID_LANGBAR_ITEM: GUID = GUID::from_u128(0xD840F24E_DD08_4BD2_89F0_EEAA83ABDE79);

/// Icon resource ids (see `resources.rc`); the profile icon is id 101.
pub const IDI_MODE_SEOKDOK: u16 = 102;
pub const IDI_MODE_HANGUL: u16 = 103;
pub const IDI_MODE_LATN: u16 = 104;

pub const SERVICE_NAME: PCWSTR = w!("seokdokTextService");

/// Name shown in the language switcher.
pub const DISPLAY_DESCRIPTION: PCWSTR = w!("釋讀 (seokdok)");

/// LANGID of ko-KR.
pub const LANGID_KO_KR: u16 = 0x0412;
