//! TOML configuration: `%APPDATA%\seokdok\config.toml` on Windows,
//! `~/Library/Application Support/seokdok/config.toml` on macOS,
//! `$XDG_CONFIG_HOME/seokdok/config.toml` (default `~/.config/seokdok`) elsewhere.
//!
//! The file and the settings dialog edit the same [`Config`]. Every field has
//! a default, so a partial or malformed file still loads.

use seokdok_core::{Fragment, Options, Style};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    /// Type Hangul and convert it to 釋讀.
    #[default]
    Seokdok,
    /// Type Hangul without conversion.
    Hangul,
    /// Pass keys through.
    Latin,
}

/// Where the text being composed is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    /// In the document, as a composition.
    Inline,
    /// In a popup at the caret; the document gets the text when it is confirmed.
    Floating,
}

/// What the input method writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Script {
    /// 釋讀: Hanja for 音讀 and 訓讀 words.
    #[default]
    Seokdok,
    /// 國漢文混用: Hanja for 音讀 words only, native words stay in Hangul.
    SinoOnly,
    /// 韓㐎 專用: Hangul only, nothing is converted.
    HangulOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleOpt {
    /// 表音式
    #[default]
    Surface,
    /// 形態式
    Morpheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentOpt {
    #[default]
    Absorb,
    Letter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Input {
    pub default_mode: InputMode,
    /// Space converts the composition. When false, Space inserts a space and
    /// only the Hanja key, F7 and Tab convert.
    pub space_converts: bool,
    /// Alt pressed and released on its own switches 釋讀 ↔ Latin, like the
    /// Han/Eng key.
    pub alt_toggles: bool,
    /// Candidate window direction; horizontal lets ← → walk the candidates.
    pub candidate_layout: Layout,
    /// Modern letters typed in a row form old-Hangul clusters: ㅅ+ㄱ → ᄭ,
    /// ㅂ+ㅇ → ㅸ. The old letters on the Shift layer work either way.
    pub old_clusters: bool,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            default_mode: InputMode::Seokdok,
            space_converts: true,
            alt_toggles: true,
            candidate_layout: Layout::Vertical,
            old_clusters: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Conversion {
    pub style: StyleOpt,
    pub fragment: FragmentOpt,
    pub coda_always: bool,
    pub resyllabify: bool,
    pub hada_kun: bool,
    pub old_ha: bool,
    pub script: Script,
    pub max_candidates: usize,
}

impl Default for Conversion {
    fn default() -> Self {
        let o = Options::default();
        Conversion {
            style: StyleOpt::Surface,
            fragment: FragmentOpt::Absorb,
            coda_always: o.coda_always,
            resyllabify: o.resyllabify,
            hada_kun: o.hada_kun,
            old_ha: o.old_ha,
            script: if o.sino_only { Script::SinoOnly } else { Script::Seokdok },
            max_candidates: o.max_candidates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub input: Input,
    pub conversion: Conversion,
    /// Where to draw the composition, by executable name. An application not
    /// listed is decided automatically.
    pub composition: BTreeMap<String, Surface>,
}

impl Config {
    /// The folder holding config.toml and custom.json: $SEOKDOK_CONFIG_DIR
    /// when set (the iOS app and keyboard share an App Group container),
    /// else %APPDATA%\seokdok on Windows, ~/Library/Application
    /// Support/seokdok on macOS, and $XDG_CONFIG_HOME/seokdok or
    /// ~/.config/seokdok elsewhere.
    pub fn user_dir() -> Option<PathBuf> {
        if let Some(d) = std::env::var_os("SEOKDOK_CONFIG_DIR").map(PathBuf::from).filter(|p| p.is_absolute()) {
            return Some(d);
        }
        if cfg!(windows) {
            return Some(PathBuf::from(std::env::var_os("APPDATA")?).join("seokdok"));
        }
        if cfg!(target_os = "macos") {
            return Some(PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support/seokdok"));
        }
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".config")))?;
        Some(base.join("seokdok"))
    }

    pub fn path() -> Option<PathBuf> {
        Some(Self::user_dir()?.join("config.toml"))
    }

    pub fn engine_options(&self) -> Options {
        let c = &self.conversion;
        Options {
            style: match c.style {
                StyleOpt::Surface => Style::Surface,
                StyleOpt::Morpheme => Style::Morpheme,
            },
            fragment: match c.fragment {
                FragmentOpt::Absorb => Fragment::Absorb,
                FragmentOpt::Letter => Fragment::Letter,
            },
            coda_always: c.coda_always,
            resyllabify: c.resyllabify,
            hada_kun: c.hada_kun,
            old_ha: c.old_ha,
            sino_only: c.script == Script::SinoOnly,
            max_candidates: c.max_candidates.max(1),
        }
    }

    pub fn load() -> Self {
        match Self::path() {
            Some(p) => Self::load_from(&p),
            None => Self::default(),
        }
    }

    pub fn load_from(path: &Path) -> Self {
        std::fs::read_to_string(path).map(|t| Self::parse(&t)).unwrap_or_default()
    }

    pub fn parse(text: &str) -> Self {
        toml::from_str(text).unwrap_or_default()
    }

    /// The file as written to disk: every option with a comment and its
    /// allowed values.
    pub fn to_documented_toml(&self) -> String {
        let mode = match self.input.default_mode {
            InputMode::Seokdok => "seokdok",
            InputMode::Hangul => "hangul",
            InputMode::Latin => "latin",
        };
        let c = &self.conversion;
        let script = match c.script {
            Script::Seokdok => "seokdok",
            Script::SinoOnly => "sino_only",
            Script::HangulOnly => "hangul_only",
        };
        let style = match c.style {
            StyleOpt::Surface => "surface",
            StyleOpt::Morpheme => "morpheme",
        };
        let fragment = match c.fragment {
            FragmentOpt::Absorb => "absorb",
            FragmentOpt::Letter => "letter",
        };
        let composition: String = self
            .composition
            .iter()
            .map(|(app, surface)| {
                let surface = match surface {
                    Surface::Inline => "inline",
                    Surface::Floating => "floating",
                };
                format!("\"{app}\" = \"{surface}\"\n")
            })
            .collect();
        format!(
            "\
# seokdok configuration
#   Windows: %APPDATA%\\seokdok\\config.toml
#   macOS:   ~/Library/Application Support/seokdok/config.toml
#   Linux:   ~/.config/seokdok/config.toml
#
# Re-activate the input method (switch away and back) to apply changes.
# Delete this file to regenerate it with the defaults.

[input]
# Mode a new session starts in.
#   seokdok = type Hangul, convert to 釋讀
#   hangul  = type Hangul without conversion
#   latin   = pass keys through
default_mode = \"{mode}\"
# Space converts the composition.  (true | false)
# When false, Space inserts a space; the Hanja key, F7 and Tab convert.
space_converts = {space}
# Alt pressed and released alone switches 釋讀 ↔ Latin, like Han/Eng.  (true | false)
alt_toggles = {alt}
# Candidate window direction.
#   vertical   = a column
#   horizontal = a row; ← → move through the candidates
candidate_layout = \"{layout}\"
# Old Hangul: Shift+A D G K give ㅿ ㆁ ㆆ ㆍ, Shift+Z X C V B N the sibilants
# ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ.  With this on, modern letters typed in a row also join into
# old clusters: ㅅ ㄱ → ᄭ, ㅂ ㅇ → ㅸ, ㅎ ㅎ → ᅘ.  (true | false)
old_clusters = {old}

# Own words, phrases and rewrite rules: export them from
# https://seokdok.mkpo.li/#custom and save the file as
# custom.json beside this file.

[conversion]
# How the Hangul after a 訓讀 stem is written.
#   surface  = 表音式: 見ㅆ다
#   morpheme = 形態式: 見았다
style = \"{style}\"
# In the surface style, a vowel left over from a contraction.
#   absorb = the Hanja absorbs it: 見라
#   letter = written as a letter: 見ㅏ라
fragment = \"{fragment}\"
# Write the final consonant of every consonant-final 訓讀 stem.  (true | false)
coda_always = {coda}
# Write a final consonant as the onset of a following ㅇ syllable.  (true | false)
resyllabify = {resyl}
# Prefer 爲 for the light verb 하다.  (true | false)
hada_kun = {hada}
# Put the old spelling of 하 first and the modern one behind it: 하고 gives
# ᄒᆞ고 then 하고, 追加할까요 gives 追加ᄒᆞᆯ까요 first.  Only 하 한 할 함 합 해
# change (ᄒᆞ ᄒᆞᆫ ᄒᆞᆯ ᄒᆞᆷ ᄒᆞᆸ ᄒᆡ).  (true | false)
# Without it the old spelling is the third candidate.
old_ha = {old_ha}
# What is written.
#   seokdok     = 釋讀: Hanja for 音讀 and 訓讀 words
#   sino_only   = 國漢文混用: Hanja for 音讀 words only, native words stay in Hangul
#   hangul_only = 韓㐎 專用: Hangul only; nothing is converted, Space types a
#                 space and Enter commits
script = \"{script}\"
# Maximum number of whole-phrase candidates.  (integer, 1 or more)
max_candidates = {max}

# Windows: where the text being typed is drawn, by executable name.
#   inline   = in the document
#   floating = in a popup at the caret
# An application not listed is decided automatically.
#   \"example.exe\" = \"floating\"
[composition]
{composition}",
            mode = mode,
            space = self.input.space_converts,
            alt = self.input.alt_toggles,
            old = self.input.old_clusters,
            layout = match self.input.candidate_layout {
                Layout::Vertical => "vertical",
                Layout::Horizontal => "horizontal",
            },
            style = style,
            fragment = fragment,
            coda = c.coda_always,
            resyl = c.resyllabify,
            hada = c.hada_kun,
            old_ha = c.old_ha,
            script = script,
            max = c.max_candidates,
            composition = composition,
        )
    }

    pub fn save(&self) -> std::io::Result<()> {
        match Self::path() {
            Some(p) => self.save_to(&p),
            None => Ok(()),
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, self.to_documented_toml())?;
        std::fs::rename(&tmp, path)
    }
}

static GLOBAL: RwLock<Option<Config>> = RwLock::new(None);

/// The cached configuration, loaded on first use.
pub fn current() -> Config {
    if let Ok(g) = GLOBAL.read() {
        if let Some(c) = g.as_ref() {
            return c.clone();
        }
    }
    reload()
}

/// Re-read the file and refresh the cache.
pub fn reload() -> Config {
    let c = Config::load();
    if let Ok(mut g) = GLOBAL.write() {
        *g = Some(c.clone());
    }
    c
}

/// Write the default file when none exists, so it can be found and edited.
pub fn ensure_file() {
    if let Some(p) = Config::path() {
        if !p.exists() {
            let _ = Config::default().save_to(&p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ios_app_partial_file_parses() {
        // What ios/App/SeokdokApp.swift writes; missing keys keep defaults.
        let c = Config::parse(
            "[input]\ndefault_mode = \"latin\"\nspace_converts = false\ncandidate_layout = \"vertical\"\nold_clusters = true\n\n\
             [conversion]\nstyle = \"morpheme\"\nfragment = \"letter\"\ncoda_always = true\nresyllabify = false\nhada_kun = true\nold_ha = true\nscript = \"sino_only\"\nmax_candidates = 20\n",
        );
        assert_eq!(c.input.default_mode, InputMode::Latin);
        assert!(!c.input.space_converts);
        assert!(c.input.old_clusters);
        assert!(c.input.alt_toggles);
        assert_eq!(c.conversion.style, StyleOpt::Morpheme);
        assert_eq!(c.conversion.fragment, FragmentOpt::Letter);
        assert!(c.conversion.coda_always && c.conversion.hada_kun && !c.conversion.resyllabify);
        assert!(c.conversion.old_ha);
        assert_eq!(c.conversion.script, Script::SinoOnly);
        assert_eq!(c.conversion.max_candidates, 20);
    }

    fn non_default() -> Config {
        Config {
            input: Input {
                default_mode: InputMode::Hangul,
                space_converts: false,
                alt_toggles: false,
                candidate_layout: Layout::Horizontal,
                old_clusters: true,
            },
            conversion: Conversion {
                style: StyleOpt::Morpheme,
                fragment: FragmentOpt::Letter,
                coda_always: true,
                resyllabify: true,
                hada_kun: true,
                old_ha: true,
                script: Script::HangulOnly,
                max_candidates: 5,
            },
            composition: BTreeMap::from([
                ("Telegram.exe".to_string(), Surface::Floating),
                ("vivaldi.exe".to_string(), Surface::Inline),
            ]),
        }
    }

    #[test]
    fn documented_toml_round_trips_every_field() {
        let c = non_default();
        assert_eq!(Config::parse(&c.to_documented_toml()), c);
        let d = Config::default();
        assert_eq!(Config::parse(&d.to_documented_toml()), d);
    }

    #[test]
    fn partial_and_malformed_files() {
        let c = Config::parse("[input]\nspace_converts = false\n");
        assert!(!c.input.space_converts);
        assert_eq!(c.conversion, Conversion::default());
        assert_eq!(Config::parse("= = ["), Config::default());
        assert_eq!(Config::parse("[conversion]\nmax_candidates = \"x\"\n"), Config::default());
    }

    #[test]
    fn maps_to_engine_options() {
        let o = non_default().engine_options();
        assert_eq!(o.style, Style::Morpheme);
        assert_eq!(o.fragment, Fragment::Letter);
        assert!(o.coda_always && o.resyllabify && o.hada_kun && o.old_ha && !o.sino_only);
        assert_eq!(o.max_candidates, 5);
        let mut sino = non_default();
        sino.conversion.script = Script::SinoOnly;
        assert!(sino.engine_options().sino_only);
        let d = Config::default().engine_options();
        assert_eq!(d.style, Style::Surface);
        assert_eq!(d.max_candidates, Options::default().max_candidates);
        assert!(!d.old_ha);
    }

    #[test]
    fn save_and_load() {
        let dir = std::env::temp_dir().join(format!("seokdok-cfg-{}", std::process::id()));
        let path = dir.join("config.toml");
        let c = non_default();
        c.save_to(&path).unwrap();
        assert_eq!(Config::load_from(&path), c);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(Config::load_from(&path), Config::default());
    }
}
