//! The input methods this repository builds. They share the engine, the
//! dictionaries and the session; `products/<id>.json` holds what differs, and
//! each front end builds one of them: `--features eumdok` here, and the same
//! file read by the build scripts of the installer, the apps and the site.

/// Identifiers only the Windows input method and its settings app use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Windows {
    pub clsid: u128,
    pub profile: u128,
    pub da_input: u128,
    pub da_converted: u128,
    pub da_focus: u128,
    pub settings_exe: &'static str,
    pub service_name: &'static str,
    pub candidate_class: &'static str,
    pub display_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Product {
    pub id: &'static str,
    pub hanja: &'static str,
    pub hangul: &'static str,
    pub latin: &'static str,
    /// The character of the app icon.
    pub glyph: &'static str,
    /// The folder of config.toml, custom.json and learned.tsv.
    pub config_dir: &'static str,
    /// What the conversion mode is called where a person chooses it.
    pub conversion_label: &'static str,
    /// The conversion mode's entry in the Windows quick menu.
    pub menu_conversion: &'static str,
    /// The values of `conversion.script` it offers, in the rotation's order.
    pub scripts: &'static [&'static str],
    pub default_script: &'static str,
    /// Settings it neither shows nor reads: the keys of `[conversion]` and of
    /// `windows.menu_items`, and `kun_words` for the list of inflecting words.
    pub hidden_settings: &'static [&'static str],
    pub windows: Windows,
    /// The whole definition, for front ends that read it at run time.
    pub json: &'static str,
}

impl Product {
    pub fn offers_script(&self, key: &str) -> bool {
        self.scripts.contains(&key)
    }

    pub fn hides(&self, setting: &str) -> bool {
        self.hidden_settings.contains(&setting)
    }
}

include!(concat!(env!("OUT_DIR"), "/products.rs"));

/// The product being built.
#[cfg(not(feature = "eumdok"))]
pub const CURRENT: Product = SEOKDOK;
/// The product being built.
#[cfg(feature = "eumdok")]
pub const CURRENT: Product = EUMDOK;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_products_share_no_identifier() {
        let (a, b) = (SEOKDOK.windows, EUMDOK.windows);
        for (x, y) in [(a.clsid, b.clsid), (a.profile, b.profile), (a.da_input, b.da_input), (a.da_converted, b.da_converted), (a.da_focus, b.da_focus)] {
            assert_ne!(x, y);
        }
        assert_ne!(SEOKDOK.config_dir, EUMDOK.config_dir);
        assert_ne!(a.settings_exe, b.settings_exe);
        assert_ne!(a.service_name, b.service_name);
        assert_ne!(a.candidate_class, b.candidate_class);
    }

    #[test]
    fn eumdok_offers_no_seokdok_script() {
        assert!(!EUMDOK.offers_script("seokdok"));
        assert!(EUMDOK.offers_script(EUMDOK.default_script));
        assert_eq!(EUMDOK.scripts, ["sino_only", "hangul_only"]);
    }

    #[test]
    fn seokdok_stays_as_it_was_released() {
        assert_eq!(SEOKDOK.windows.clsid, 0xAAF18869_67ED_48A1_A3F6_42E961E994A1);
        assert_eq!(SEOKDOK.windows.profile, 0x5A2C201D_E01B_475F_A23D_3A6A64D996A4);
        assert_eq!(SEOKDOK.config_dir, "seokdok");
        assert_eq!(SEOKDOK.default_script, "seokdok");
        assert!(SEOKDOK.hidden_settings.is_empty());
    }
}
