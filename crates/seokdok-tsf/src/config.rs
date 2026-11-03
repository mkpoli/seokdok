//! TOML configuration at `%APPDATA%\seokdok\config.toml`.
//!
//! The file and the settings dialog edit the same [`Config`]. Every field has
//! a default, so a partial or malformed file still loads.

use seokdok_core::{Fragment, Options, Style};
use serde::{Deserialize, Serialize};
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
}

impl Default for Input {
    fn default() -> Self {
        Input { default_mode: InputMode::Seokdok, space_converts: true, alt_toggles: true, candidate_layout: Layout::Vertical }
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
            max_candidates: o.max_candidates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub input: Input,
    pub conversion: Conversion,
}

impl Config {
    pub fn user_dir() -> Option<PathBuf> {
        Some(PathBuf::from(std::env::var_os("APPDATA")?).join("seokdok"))
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
        let style = match c.style {
            StyleOpt::Surface => "surface",
            StyleOpt::Morpheme => "morpheme",
        };
        let fragment = match c.fragment {
            FragmentOpt::Absorb => "absorb",
            FragmentOpt::Letter => "letter",
        };
        format!(
            "\
# seokdok configuration: %APPDATA%\\seokdok\\config.toml
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

# Own words, phrases and rewrite rules: export them from
# https://seokdok.mkpo.li/#custom and save the file as
# %APPDATA%\\seokdok\\custom.json beside this one.

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
# Maximum number of whole-phrase candidates.  (integer, 1 or more)
max_candidates = {max}
",
            mode = mode,
            space = self.input.space_converts,
            alt = self.input.alt_toggles,
            layout = match self.input.candidate_layout {
                Layout::Vertical => "vertical",
                Layout::Horizontal => "horizontal",
            },
            style = style,
            fragment = fragment,
            coda = c.coda_always,
            resyl = c.resyllabify,
            hada = c.hada_kun,
            max = c.max_candidates,
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

    fn non_default() -> Config {
        Config {
            input: Input { default_mode: InputMode::Hangul, space_converts: false, alt_toggles: false, candidate_layout: Layout::Horizontal },
            conversion: Conversion {
                style: StyleOpt::Morpheme,
                fragment: FragmentOpt::Letter,
                coda_always: true,
                resyllabify: true,
                hada_kun: true,
                max_candidates: 5,
            },
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
        assert!(o.coda_always && o.resyllabify && o.hada_kun);
        assert_eq!(o.max_candidates, 5);
        let d = Config::default().engine_options();
        assert_eq!(d.style, Style::Surface);
        assert_eq!(d.max_candidates, Options::default().max_candidates);
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
