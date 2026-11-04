//! TOML configuration: `%APPDATA%\seokdok\config.toml` on Windows,
//! `~/Library/Application Support/seokdok/config.toml` on macOS,
//! `$XDG_CONFIG_HOME/seokdok/config.toml` (default `~/.config/seokdok`) elsewhere.
//!
//! The file and the settings screens edit the same [`Config`]. Every field has
//! a default, so a partial or malformed file still loads.

use seokdok_core::{Fragment, Itda, Options, Style};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::SystemTime;

use crate::mode_keys::ModeKeys;

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

impl Script {
    /// The script `input.keys.script_cycle` goes to from this one.
    pub fn next(self) -> Script {
        match self {
            Script::Seokdok => Script::SinoOnly,
            Script::SinoOnly => Script::HangulOnly,
            Script::HangulOnly => Script::Seokdok,
        }
    }
}

/// The spelling of 있다 listed first. The values are the file's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ItdaOpt {
    #[default]
    #[serde(rename = "有")]
    Yu,
    #[serde(rename = "在")]
    Jae,
    #[serde(rename = "在ㅆ")]
    JaeCoda,
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
    /// only the convert keys and Tab convert.
    pub space_converts: bool,
    /// Candidate window direction; horizontal lets ← → walk the candidates.
    pub candidate_layout: Layout,
    /// Modern letters typed in a row form old-Hangul clusters: ㅅ+ㄱ → ᄭ,
    /// ㅂ+ㅇ → ㅸ. The old letters on the Shift layer work either way.
    pub old_clusters: bool,
    /// The keys that switch modes and convert, by what they do. Last, so
    /// that it serializes after the plain values of this section.
    pub keys: ModeKeys,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            default_mode: InputMode::Seokdok,
            space_converts: true,
            candidate_layout: Layout::Vertical,
            old_clusters: false,
            keys: ModeKeys::default(),
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
    pub itda: ItdaOpt,
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
            itda: ItdaOpt::Yu,
            old_ha: o.old_ha,
            script: if o.sino_only { Script::SinoOnly } else { Script::Seokdok },
            max_candidates: o.max_candidates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Windows {
    /// What the input indicator's menu shows: the keys of
    /// [`crate::quick_menu::MenuItem`]. Other entries are ignored.
    pub menu_items: Vec<String>,
}

impl Default for Windows {
    fn default() -> Self {
        Windows { menu_items: ["script", "input_mode", "candidate_layout", "old_ha"].map(String::from).to_vec() }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub input: Input,
    pub conversion: Conversion,
    pub windows: Windows,
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
            itda: match c.itda {
                ItdaOpt::Yu => Itda::Yu,
                ItdaOpt::Jae => Itda::Jae,
                ItdaOpt::JaeCoda => Itda::JaeCoda,
            },
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

    /// Change the file through `change`, writing only if the settings differ.
    /// A file that does not parse is left alone and is an error: loading it
    /// gives the defaults, and saving those would lose what it holds.
    pub fn update_file(path: &Path, change: impl FnOnce(&mut Config)) -> std::io::Result<Config> {
        let mut config = match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(std::io::Error::other)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
            Err(e) => return Err(e),
        };
        let before = config.clone();
        change(&mut config);
        if config != before {
            config.save_to(path)?;
        }
        Ok(config)
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
        let itda = match c.itda {
            ItdaOpt::Yu => "有",
            ItdaOpt::Jae => "在",
            ItdaOpt::JaeCoda => "在ㅆ",
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
# On Windows, changes apply at the next key or when an application gets the
# focus.  Elsewhere, re-activate the input method (switch away and back).
# Delete this file to regenerate it with the defaults.

[input]
# Mode a new session starts in.
#   seokdok = type Hangul, convert to 釋讀
#   hangul  = type Hangul without conversion
#   latin   = pass keys through
default_mode = \"{mode}\"
# Space converts the composition.  (true | false)
# When false, Space inserts a space; the convert keys below and Tab convert.
space_converts = {space}
# Candidate window direction.
#   vertical   = a column
#   horizontal = a row; ← → move through the candidates
candidate_layout = \"{layout}\"
# Old Hangul: Shift+A D G K give ㅿ ㆁ ㆆ ㆍ, Shift+Z X C V B N the sibilants
# ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ.  With this on, modern letters typed in a row also join into
# old clusters: ㅅ ㄱ → ᄭ, ㅂ ㅇ → ㅸ, ㅎ ㅎ → ᅘ.  (true | false)
old_clusters = {old}

# The keys that switch modes and convert.  Each list holds keys, written
#   Hangul Hanja Zenkaku Henkan Muhenkan Kana Eisu ImeOn ImeOff
#   Space Enter Tab Esc F1..F24 and letters or digits,
# after any of Ctrl+ Shift+ Alt+ (Option+) Super+ (Win+ Cmd+): Shift+Hangul.
# Modifiers alone mean tapping them with no other key: RAlt, or LCtrl+LAlt.
# They can name a side (LAlt RAlt LCtrl RCtrl LShift RShift) or not (Alt).
# Entries that are not keys are ignored.  Lists left out keep their defaults
# for this platform; an empty list [] turns that action off.
# Keys the system takes first never arrive here: Ctrl+Space on macOS and
# Android, Globe and Caps Lock on macOS when the system settings give them to
# input sources, and the keys fcitx5 uses to switch when it has more than one
# input method.
[input.keys]
# Korean ↔ Latin, back to whichever of 釋讀 and 한글 was last in use.
toggle = {k_toggle}
# To Latin.
latin = {k_latin}
# From Latin back to Korean.
korean = {k_korean}
# To 釋讀 mode at the next conversion.script, in every application:
# 釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀.
script_cycle = {k_cycle}
# To 釋讀.
seokdok = {k_seokdok}
# To 한글.
hangul = {k_hangul}
# Convert the composition.
convert = {k_convert}

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
# The spelling of 있다 listed first; the others stay offered.
#   有   = 有다
#   在   = 在다, past 在었다
#   在ㅆ = 在ㅆ다
itda = \"{itda}\"
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

[windows]
# What the menu of the input indicator on the taskbar shows (right-click its
# icon).  Settings… is always there.
#   script           = 釋 混 한: conversion.script
#   input_mode       = 釋讀, 한글, A
#   candidate_layout = horizontal candidates: input.candidate_layout
#   old_ha           = conversion.old_ha
#   old_clusters     = input.old_clusters
#   space_converts   = input.space_converts
#   style            = 形態式: conversion.style
#   fragment         = leftover vowel as a letter: conversion.fragment
#   coda_always      = conversion.coda_always
#   resyllabify      = conversion.resyllabify
#   hada_kun         = conversion.hada_kun
menu_items = {menu_items}

# Windows: where the text being typed is drawn, by executable name.
#   inline   = in the document
#   floating = in a popup at the caret
# An application not listed is decided automatically.
#   \"example.exe\" = \"floating\"
[composition]
{composition}",
            mode = mode,
            space = self.input.space_converts,
            old = self.input.old_clusters,
            k_toggle = toml_list(&self.input.keys.toggle),
            k_latin = toml_list(&self.input.keys.latin),
            k_korean = toml_list(&self.input.keys.korean),
            k_cycle = toml_list(&self.input.keys.script_cycle),
            k_seokdok = toml_list(&self.input.keys.seokdok),
            k_hangul = toml_list(&self.input.keys.hangul),
            k_convert = toml_list(&self.input.keys.convert),
            layout = match self.input.candidate_layout {
                Layout::Vertical => "vertical",
                Layout::Horizontal => "horizontal",
            },
            style = style,
            fragment = fragment,
            coda = c.coda_always,
            resyl = c.resyllabify,
            hada = c.hada_kun,
            itda = itda,
            old_ha = c.old_ha,
            script = script,
            max = c.max_candidates,
            menu_items = toml_list(&self.windows.menu_items),
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
        // Each process has its own, so two writers cannot mix their text.
        let tmp = path.with_extension(format!("toml.{}.tmp", std::process::id()));
        std::fs::write(&tmp, self.to_documented_toml())?;
        std::fs::rename(&tmp, path)
    }
}

/// A TOML array of strings on one line.
fn toml_list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| toml::Value::String(s.clone()).to_string()).collect();
    format!("[{}]", quoted.join(", "))
}

/// What tells one version of the file from another without reading it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
}

fn stamp_of(path: &Path) -> Option<FileStamp> {
    let m = std::fs::metadata(path).ok()?;
    Some(FileStamp { modified: m.modified().ok(), len: m.len() })
}

/// The settings in use and the version of the file they came from.
#[derive(Default)]
struct Cache {
    config: Option<Config>,
    stamp: Option<FileStamp>,
}

impl Cache {
    fn reload(&mut self, path: Option<&Path>) -> Config {
        // The stamp comes first: a write between the two is seen as a change.
        self.stamp = path.and_then(stamp_of);
        let config = path.map(Config::load_from).unwrap_or_default();
        self.config = Some(config.clone());
        config
    }

    /// Whether another writer changed the file; the cache follows it if so.
    fn refresh(&mut self, path: Option<&Path>) -> bool {
        if self.config.is_some() && self.stamp == path.and_then(stamp_of) {
            return false;
        }
        self.reload(path);
        true
    }
}

static GLOBAL: RwLock<Cache> = RwLock::new(Cache { config: None, stamp: None });

/// The cached configuration, loaded on first use.
pub fn current() -> Config {
    if let Ok(g) = GLOBAL.read() {
        if let Some(c) = g.config.as_ref() {
            return c.clone();
        }
    }
    reload()
}

/// Re-read the file and refresh the cache.
pub fn reload() -> Config {
    let path = Config::path();
    match GLOBAL.write() {
        Ok(mut g) => g.reload(path.as_deref()),
        Err(_) => Config::load(),
    }
}

/// Follow the file if something changed it, such as the same input method in
/// another application. True when it changed. This only reads the file's
/// timestamp and length, so it can run at every key.
pub fn refresh() -> bool {
    let path = Config::path();
    GLOBAL.write().map(|mut g| g.refresh(path.as_deref())).unwrap_or(false)
}

static CUSTOM_STAMP: RwLock<Option<Option<FileStamp>>> = RwLock::new(None);

/// Whether custom.json changed since the last look, as [`refresh`] does for
/// config.toml. The first look only takes note: the engine reads the file as it
/// loads. Its timestamp and length only, so this can run at every key.
pub fn custom_changed() -> bool {
    let stamp = Config::user_dir().and_then(|d| stamp_of(&d.join("custom.json")));
    if let Ok(g) = CUSTOM_STAMP.read() {
        if *g == Some(stamp) {
            return false;
        }
    }
    let Ok(mut g) = CUSTOM_STAMP.write() else { return false };
    g.replace(stamp).is_some_and(|before| before != stamp)
}

/// Change the file (see [`Config::update_file`]) and the cache with it.
pub fn update(change: impl FnOnce(&mut Config)) -> std::io::Result<Config> {
    let Some(path) = Config::path() else {
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    };
    Config::update_file(&path, change)?;
    Ok(reload())
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
             [conversion]\nstyle = \"morpheme\"\nfragment = \"letter\"\ncoda_always = true\nresyllabify = false\nhada_kun = true\nitda = \"在\"\nold_ha = true\nscript = \"sino_only\"\nmax_candidates = 20\n",
        );
        assert_eq!(c.input.default_mode, InputMode::Latin);
        assert!(!c.input.space_converts);
        assert!(c.input.old_clusters);
        assert_eq!(c.input.keys, ModeKeys::default());
        assert_eq!(c.conversion.style, StyleOpt::Morpheme);
        assert_eq!(c.conversion.fragment, FragmentOpt::Letter);
        assert!(c.conversion.coda_always && c.conversion.hada_kun && !c.conversion.resyllabify);
        assert!(c.conversion.old_ha);
        assert_eq!(c.conversion.itda, ItdaOpt::Jae);
        assert_eq!(c.engine_options().itda, Itda::Jae);
        assert_eq!(Config::default().engine_options().itda, Itda::Yu);
        assert_eq!(c.conversion.script, Script::SinoOnly);
        assert_eq!(c.conversion.max_candidates, 20);
    }

    fn non_default() -> Config {
        Config {
            input: Input {
                default_mode: InputMode::Hangul,
                space_converts: false,
                candidate_layout: Layout::Horizontal,
                old_clusters: true,
                keys: ModeKeys {
                    toggle: vec!["F12".into(), "Shift+Hangul".into()],
                    latin: vec![],
                    korean: vec!["Henkan".into()],
                    script_cycle: vec!["LCtrl+LAlt".into()],
                    seokdok: vec!["Ctrl+1".into()],
                    hangul: vec!["Ctrl+2".into()],
                    convert: vec!["Hanja".into()],
                },
            },
            conversion: Conversion {
                style: StyleOpt::Morpheme,
                fragment: FragmentOpt::Letter,
                coda_always: true,
                resyllabify: true,
                hada_kun: true,
                itda: ItdaOpt::JaeCoda,
                old_ha: true,
                script: Script::HangulOnly,
                max_candidates: 5,
            },
            windows: Windows { menu_items: vec!["script".into(), "no such item".into()] },
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
    fn key_lists_override_one_at_a_time() {
        let c = Config::parse("[input.keys]\ntoggle = [\"F12\"]\nconvert = []\n");
        let d = ModeKeys::default();
        assert_eq!(c.input.keys.toggle, ["F12"]);
        assert!(c.input.keys.convert.is_empty());
        assert_eq!(c.input.keys.latin, d.latin);
        // The dotted form the Android settings screen writes.
        let c = Config::parse("[input]\nkeys.toggle = [\"F12\", \"RAlt\"]\nspace_converts = false\n");
        assert_eq!(c.input.keys.toggle, ["F12", "RAlt"]);
        assert_eq!(c.input.keys.latin, d.latin);
        assert!(!c.input.space_converts);
        // The plain serializer the Android settings screen reads.
        let text = toml::to_string(&non_default()).unwrap();
        assert_eq!(Config::parse(&text), non_default());
    }

    #[test]
    fn any_string_in_a_list_survives_the_file() {
        let mut c = Config::default();
        c.input.keys.toggle = vec!["bad\"entry\\".into(), "F12\n".into(), "\t한/영".into()];
        assert_eq!(Config::parse(&c.to_documented_toml()), c);
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
    fn menu_items_are_read_and_their_default_is_listed() {
        let c = Config::parse("[windows]\nmenu_items = [\"old_ha\", \"unknown\"]\n");
        assert_eq!(c.windows.menu_items, ["old_ha", "unknown"]);
        assert_eq!(c.input, Input::default());
        assert_eq!(Config::parse("").windows, Windows::default());
        assert!(Config::parse("[windows]\nmenu_items = []\n").windows.menu_items.is_empty());
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("seokdok-cfg-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("config.toml")
    }

    #[test]
    fn a_cache_follows_a_write_from_another_process() {
        let path = scratch("refresh");
        let p = Some(path.as_path());
        let mut cache = Cache::default();
        // No file yet: the first look loads the defaults.
        assert!(cache.refresh(p));
        assert!(!cache.refresh(p));
        assert_eq!(cache.config.as_ref().unwrap(), &Config::default());

        let mut other = Config::default();
        other.conversion.script = Script::SinoOnly;
        other.save_to(&path).unwrap();
        assert!(cache.refresh(p));
        assert_eq!(cache.config.as_ref().unwrap().conversion.script, Script::SinoOnly);
        assert!(!cache.refresh(p));

        // A change the first process made itself is not news to it.
        other.input.old_clusters = true;
        other.save_to(&path).unwrap();
        cache.reload(p);
        assert!(!cache.refresh(p));
        assert!(cache.config.as_ref().unwrap().input.old_clusters);

        std::fs::remove_file(&path).unwrap();
        assert!(cache.refresh(p));
        assert_eq!(cache.config.as_ref().unwrap(), &Config::default());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn custom_changed_follows_the_file() {
        let path = scratch("custom");
        let dir = path.parent().unwrap().to_path_buf();
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        assert!(!custom_changed());
        assert!(!custom_changed());
        std::fs::write(dir.join("custom.json"), "{}").unwrap();
        assert!(custom_changed());
        assert!(!custom_changed());
        std::fs::remove_file(dir.join("custom.json")).unwrap();
        assert!(custom_changed());
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn script_cycle_is_the_only_name_of_the_chord_list() {
        let c = Config::parse("[input.keys]\nseokdok_hangul = [\"F9\"]\n");
        assert_eq!(c.input.keys.script_cycle, Config::default().input.keys.script_cycle);
        let c = Config::parse("[input.keys]\nscript_cycle = [\"F9\"]\n");
        assert_eq!(c.input.keys.script_cycle, ["F9"]);
    }

    #[test]
    fn update_file_writes_changes_only_and_spares_a_broken_file() {
        let path = scratch("update");
        // A missing file starts from the defaults.
        let c = Config::update_file(&path, |c| c.conversion.old_ha = true).unwrap();
        assert!(c.conversion.old_ha);
        assert_eq!(Config::load_from(&path), c);

        // Other settings survive.
        let c = Config::update_file(&path, |c| c.conversion.script = Script::HangulOnly).unwrap();
        assert!(c.conversion.old_ha);
        assert_eq!(c.conversion.script, Script::HangulOnly);

        // Nothing changed: the file, comments added by hand included, stays.
        let text = std::fs::read_to_string(&path).unwrap() + "# a note\n";
        std::fs::write(&path, &text).unwrap();
        Config::update_file(&path, |_| {}).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);

        let broken = "[conversion]\nmax_candidates = \"x\"\n";
        std::fs::write(&path, broken).unwrap();
        assert!(Config::update_file(&path, |c| c.conversion.old_ha = false).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), broken);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
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
