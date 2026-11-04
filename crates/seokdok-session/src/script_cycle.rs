//! The rotation of `conversion.script` by `input.keys.script_cycle`. The file
//! changes first, so every application follows it and one that does not parse
//! stops the rotation with nothing changed. The session then commits what is
//! being composed and goes to 釋讀 mode, where the script shows.

use std::io;

use crate::config::{self, Config, Script};
use crate::keys::Key;
use crate::session::{Converter, Effect, Mode, Session};

/// A rotation that reached the file.
#[derive(Debug)]
pub struct Rotated {
    /// The settings now. The front end gives their options to the engine.
    pub config: Config,
    /// What the session did, to apply to the document.
    pub effects: Vec<Effect>,
}

fn advance(config: &mut Config) {
    config.conversion.script = config.conversion.script.next();
}

fn enter(session: &mut Session, conv: &dyn Converter, written: io::Result<Config>) -> io::Result<Rotated> {
    let config = written?;
    let mut effects = session.handle(Key::CycleScript, conv).effects;
    effects.extend(session.set_hangul_only(config.conversion.script == Script::HangulOnly));
    Ok(Rotated { config, effects })
}

/// Go to the next script. An error leaves the file and the session as they were.
pub fn rotate(session: &mut Session, conv: &dyn Converter) -> io::Result<Rotated> {
    let written = config::update(advance);
    enter(session, conv, written)
}

/// What a tap on an on-screen mode key did.
#[derive(Debug)]
pub struct Stepped {
    /// The settings now, when the script changed.
    pub config: Option<Config>,
    pub effects: Vec<Effect>,
}

fn step_after(session: &mut Session, script: Script, rotated: impl FnOnce(&mut Session) -> io::Result<Rotated>) -> Stepped {
    let latin = session.mode() == Mode::Latin;
    if script == Script::HangulOnly && !latin {
        return Stepped { config: None, effects: session.set_mode(Mode::Latin) };
    }
    // Out of Latin or plain 한글, the script on show is this one.
    if script != Script::HangulOnly && session.mode() != Mode::Seokdok {
        return Stepped { config: None, effects: session.set_mode(Mode::Seokdok) };
    }
    match rotated(session) {
        Ok(r) => Stepped { config: Some(r.config), effects: r.effects },
        // A keyboard that cannot write its settings still reaches Latin and back.
        Err(_) => Stepped { config: None, effects: session.set_mode(if latin { Mode::Seokdok } else { Mode::Latin }) },
    }
}

/// An on-screen mode key, where Latin has no key of its own: 釋 → 混 → 한 → A → 釋.
pub fn step_on_screen(session: &mut Session, conv: &dyn Converter) -> Stepped {
    step_after(session, config::current().conversion.script, |s| rotate(s, conv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quick_menu::Indicator;
    use seokdok_core::{Candidate, Segment};
    use std::path::{Path, PathBuf};

    struct Stub;

    impl Converter for Stub {
        fn segments(&self, input: &str) -> Vec<Segment> {
            vec![Segment { reading: input.into(), candidates: vec![Candidate { text: format!("{input}1"), note: String::new() }] }]
        }
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("seokdok-cycle-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("config.toml")
    }

    fn rotate_at(path: &Path, s: &mut Session) -> io::Result<Rotated> {
        enter(s, &Stub, Config::update_file(path, advance))
    }

    fn type_gks(s: &mut Session) {
        for c in "gks".chars() {
            s.handle(Key::Letter { ch: c, shift: false }, &Stub);
        }
    }

    #[test]
    fn the_key_goes_round_three_scripts_in_the_file_and_on_the_icon() {
        let path = temp("round");
        let mut s = Session::new(Mode::Seokdok, true);
        for (script, glyph) in [(Script::SinoOnly, "混"), (Script::HangulOnly, "한"), (Script::Seokdok, "釋")] {
            let r = rotate_at(&path, &mut s).unwrap();
            assert_eq!(r.config.conversion.script, script);
            assert_eq!(Config::load_from(&path).conversion.script, script);
            assert_eq!(Indicator::of(s.mode(), script).glyph(), glyph);
        }
    }

    #[test]
    fn the_rotation_starts_from_the_file_and_keeps_the_other_settings() {
        let path = temp("file");
        std::fs::write(&path, "[conversion]\nscript = \"sino_only\"\nold_ha = true\n").unwrap();
        let mut s = Session::new(Mode::Seokdok, true);
        let r = rotate_at(&path, &mut s).unwrap();
        assert_eq!(r.config.conversion.script, Script::HangulOnly);
        assert!(Config::load_from(&path).conversion.old_ha);
        assert_eq!(s.mode(), Mode::Hangul);
    }

    #[test]
    fn latin_and_plain_hangul_go_to_seokdok_mode_at_the_next_script() {
        for start in [Mode::Latin, Mode::Hangul] {
            let path = temp("modes");
            let mut s = Session::new(start, true);
            let r = rotate_at(&path, &mut s).unwrap();
            assert_eq!(r.config.conversion.script, Script::SinoOnly);
            assert_eq!(Indicator::of(s.mode(), Script::SinoOnly), Indicator::Mixed);
        }
        // Out of 韓㐎 專用, Latin lands in conversion too.
        let path = temp("latin");
        std::fs::write(&path, "[conversion]\nscript = \"hangul_only\"\n").unwrap();
        let mut s = Session::new(Mode::Latin, true);
        s.set_hangul_only(true);
        rotate_at(&path, &mut s).unwrap();
        assert_eq!(s.mode(), Mode::Seokdok);
    }

    #[test]
    fn a_composition_is_committed_first() {
        let path = temp("commit");
        let mut s = Session::new(Mode::Seokdok, true);
        type_gks(&mut s);
        let r = rotate_at(&path, &mut s).unwrap();
        assert_eq!(r.effects[0], Effect::Commit("한".into()));
        assert!(!s.is_composing());

        // A conversion commits as shown, and the session stays clean into 韓㐎 專用.
        type_gks(&mut s);
        s.handle(Key::Convert, &Stub);
        let r = rotate_at(&path, &mut s).unwrap();
        assert_eq!(r.effects[0], Effect::Commit("한1".into()));
        assert_eq!(s.mode(), Mode::Hangul);
    }

    #[test]
    fn a_file_that_does_not_parse_changes_nothing() {
        let path = temp("broken");
        let broken = "[conversion\nscript = \"sino_only\"\n";
        std::fs::write(&path, broken).unwrap();
        for start in [Mode::Seokdok, Mode::Hangul, Mode::Latin] {
            let mut s = Session::new(start, true);
            if start != Mode::Latin {
                type_gks(&mut s);
            }
            assert!(rotate_at(&path, &mut s).is_err());
            assert_eq!(s.mode(), start);
            assert_eq!(s.is_composing(), start != Mode::Latin);
            assert_eq!(std::fs::read_to_string(&path).unwrap(), broken);
        }
    }

    #[test]
    fn plain_hangul_stays_reachable_after_a_rotation() {
        let path = temp("plain");
        let mut s = Session::new(Mode::Seokdok, true);
        rotate_at(&path, &mut s).unwrap();
        s.handle(Key::ToHangul, &Stub);
        assert_eq!(s.mode(), Mode::Hangul);
        assert_eq!(Indicator::of(s.mode(), Script::SinoOnly), Indicator::Hangul);
    }

    #[test]
    fn the_on_screen_key_adds_latin_after_hangul_only() {
        let path = temp("screen");
        let mut s = Session::new(Mode::Seokdok, true);
        let mut script = Script::Seokdok;
        let mut shown = Vec::new();
        for _ in 0..8 {
            let stepped = step_after(&mut s, script, |s| rotate_at(&path, s));
            if let Some(c) = stepped.config {
                script = c.conversion.script;
            }
            shown.push(Indicator::of(s.mode(), script).glyph());
        }
        assert_eq!(shown, ["混", "한", "A", "釋", "混", "한", "A", "釋"]);
    }

    #[test]
    fn the_on_screen_key_enters_the_current_script_from_a_start_outside_it() {
        for start in [Mode::Latin, Mode::Hangul] {
            let path = temp("screen-start");
            let mut s = Session::new(start, true);
            let stepped = step_after(&mut s, Script::SinoOnly, |s| rotate_at(&path, s));
            assert!(stepped.config.is_none());
            assert_eq!(Indicator::of(s.mode(), Script::SinoOnly), Indicator::Mixed);
            assert!(!path.exists());
        }
    }

    #[test]
    fn the_on_screen_key_reaches_latin_when_the_file_cannot_be_written() {
        let path = temp("screen-broken");
        std::fs::write(&path, "[conversion\n").unwrap();
        let mut s = Session::new(Mode::Seokdok, true);
        type_gks(&mut s);
        let stepped = step_after(&mut s, Script::Seokdok, |s| rotate_at(&path, s));
        assert_eq!(stepped.effects[0], Effect::Commit("한".into()));
        assert_eq!(s.mode(), Mode::Latin);
        step_after(&mut s, Script::Seokdok, |s| rotate_at(&path, s));
        assert_eq!(s.mode(), Mode::Seokdok);
    }
}
