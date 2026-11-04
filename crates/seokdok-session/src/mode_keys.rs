//! The keys that switch modes and convert: the vocabulary of the `[input.keys]`
//! lists in config.toml, each platform's defaults, and the tracker that turns
//! key presses and modifier taps into session [`Key`]s.
//!
//! A front end names the physical key ("Hangul", "RAlt", "F9") and reports its
//! modifiers; everything else is decided here, so every platform reads a list
//! the same way.

use serde::{Deserialize, Serialize};

use crate::keys::Key;
use crate::session::{Mode, Session};

pub const MOD_CTRL: u8 = 1;
pub const MOD_SHIFT: u8 = 2;
pub const MOD_ALT: u8 = 4;
pub const MOD_SUPER: u8 = 8;

/// A modifier key on one side of the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    LCtrl,
    RCtrl,
    LShift,
    RShift,
    LAlt,
    RAlt,
    LSuper,
    RSuper,
}

impl Modifier {
    fn bit(self) -> u8 {
        1 << self as u8
    }
}

const CTRL_BITS: u8 = 0b0000_0011;
const SHIFT_BITS: u8 = 0b0000_1100;
const ALT_BITS: u8 = 0b0011_0000;
const SUPER_BITS: u8 = 0b1100_0000;

/// A physical key a list can name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyName {
    Modifier(Modifier),
    /// 한/영.
    Hangul,
    /// 한자.
    Hanja,
    /// 半角/全角.
    Zenkaku,
    /// 変換.
    Henkan,
    /// 無変換.
    Muhenkan,
    /// カタカナ/ひらがな, or かな on a Mac.
    Kana,
    /// 英数 on a Mac.
    Eisu,
    /// The Windows ImeOn key (あ), right of Space.
    ImeOn,
    /// The Windows ImeOff key (A), left of Space.
    ImeOff,
    Space,
    Enter,
    Tab,
    Esc,
    /// F1 to F24.
    Function(u8),
    /// A letter or digit, upper case.
    Char(char),
}

impl KeyName {
    /// The key a config entry or a front end names. Modifier groups such as
    /// "Alt" name no single key and are handled by [`Chord::parse`].
    pub fn from_name(name: &str) -> Option<KeyName> {
        use Modifier::*;
        let lower = name.trim().to_ascii_lowercase();
        Some(match lower.as_str() {
            "lctrl" | "lcontrol" => KeyName::Modifier(LCtrl),
            "rctrl" | "rcontrol" => KeyName::Modifier(RCtrl),
            "lshift" => KeyName::Modifier(LShift),
            "rshift" => KeyName::Modifier(RShift),
            "lalt" | "loption" => KeyName::Modifier(LAlt),
            "ralt" | "roption" => KeyName::Modifier(RAlt),
            "lsuper" | "lwin" | "lcmd" => KeyName::Modifier(LSuper),
            "rsuper" | "rwin" | "rcmd" => KeyName::Modifier(RSuper),
            "hangul" => KeyName::Hangul,
            "hanja" => KeyName::Hanja,
            "zenkaku" => KeyName::Zenkaku,
            "henkan" => KeyName::Henkan,
            "muhenkan" => KeyName::Muhenkan,
            "kana" => KeyName::Kana,
            "eisu" => KeyName::Eisu,
            "imeon" => KeyName::ImeOn,
            "imeoff" => KeyName::ImeOff,
            "space" => KeyName::Space,
            "enter" | "return" => KeyName::Enter,
            "tab" => KeyName::Tab,
            "esc" | "escape" => KeyName::Esc,
            s => {
                if let Some(n) = s.strip_prefix('f').and_then(|n| n.parse::<u8>().ok()) {
                    if (1..=24).contains(&n) {
                        return Some(KeyName::Function(n));
                    }
                    return None;
                }
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if c.is_ascii_alphanumeric() => KeyName::Char(c.to_ascii_uppercase()),
                    _ => return None,
                }
            }
        })
    }

    /// A key the session reads while composing (Shift+Space is 띄어쓰기).
    fn composes(self) -> bool {
        matches!(self, KeyName::Space | KeyName::Enter | KeyName::Tab | KeyName::Esc)
    }
}

/// One entry of a key list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chord {
    /// Modifiers pressed and released with nothing else in between. Each
    /// element is the set of keys that satisfy one modifier ("Alt" is either
    /// side, "RAlt" only the right one).
    Tap(Vec<u8>),
    /// A key pressed with exactly these `MOD_*` modifiers.
    Press { mods: u8, key: KeyName },
}

impl Chord {
    pub fn parse(text: &str) -> Option<Chord> {
        let mut parts = Vec::new();
        let mut mods = 0;
        let mut key = None;
        for part in text.split('+') {
            let lower = part.trim().to_ascii_lowercase();
            let bits = match lower.as_str() {
                "ctrl" | "control" => CTRL_BITS,
                "shift" => SHIFT_BITS,
                "alt" | "option" => ALT_BITS,
                "super" | "win" | "cmd" | "command" | "meta" => SUPER_BITS,
                _ => match KeyName::from_name(&lower)? {
                    KeyName::Modifier(m) => m.bit(),
                    other => {
                        if key.replace(other).is_some() {
                            return None;
                        }
                        continue;
                    }
                },
            };
            parts.push(bits);
            mods |= flag_of(bits);
        }
        Some(match key {
            Some(key) => Chord::Press { mods, key },
            None if parts.is_empty() => return None,
            None => Chord::Tap(parts),
        })
    }

    /// Does this tap chord match the modifiers held at the peak of a tap?
    fn tapped_by(&self, peak: u8) -> bool {
        let Chord::Tap(parts) = self else { return false };
        let all = parts.iter().fold(0, |a, b| a | b);
        peak & !all == 0 && parts.iter().all(|p| peak & p != 0)
    }
}

fn flag_of(bits: u8) -> u8 {
    let mut f = 0;
    if bits & CTRL_BITS != 0 {
        f |= MOD_CTRL;
    }
    if bits & SHIFT_BITS != 0 {
        f |= MOD_SHIFT;
    }
    if bits & ALT_BITS != 0 {
        f |= MOD_ALT;
    }
    if bits & SUPER_BITS != 0 {
        f |= MOD_SUPER;
    }
    f
}

/// The platform a default key set is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Mac,
    Linux,
    Android,
    Ios,
}

impl Platform {
    pub const fn current() -> Platform {
        if cfg!(windows) {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::Mac
        } else if cfg!(target_os = "android") {
            Platform::Android
        } else if cfg!(target_os = "ios") {
            Platform::Ios
        } else {
            Platform::Linux
        }
    }
}

/// `[input.keys]`: what each list of keys does. An entry is a key ("Hangul",
/// "Shift+Hangul", "F9") or modifiers tapped alone ("RAlt", "LCtrl+LAlt").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModeKeys {
    /// Korean ↔ Latin; back to whichever of 釋讀 and 한글 was last in use.
    pub toggle: Vec<String>,
    /// To Latin.
    pub latin: Vec<String>,
    /// From Latin back to Korean.
    pub korean: Vec<String>,
    /// To 釋讀 mode at the next script: 釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀.
    pub script_cycle: Vec<String>,
    /// To 釋讀.
    pub seokdok: Vec<String>,
    /// To 한글.
    pub hangul: Vec<String>,
    /// Convert the composition.
    pub convert: Vec<String>,
}

fn list(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

impl ModeKeys {
    /// What is bound out of the box. Keys the system takes before an input
    /// method sees them are left out: Ctrl+Space and Globe on macOS,
    /// Ctrl+Space and the language key on Android. iOS gives a keyboard
    /// extension no hardware keys, so its lists are empty.
    pub fn for_platform(platform: Platform) -> ModeKeys {
        let empty = list(&[]);
        match platform {
            Platform::Windows => ModeKeys {
                toggle: list(&["Hangul", "Zenkaku", "RAlt"]),
                latin: list(&["Muhenkan", "ImeOff"]),
                korean: list(&["Henkan", "ImeOn"]),
                script_cycle: list(&["LCtrl+LAlt", "Shift+Hangul"]),
                seokdok: empty.clone(),
                hangul: empty,
                convert: list(&["Hanja", "RCtrl", "F7", "Henkan"]),
            },
            Platform::Mac => ModeKeys {
                toggle: list(&["Alt"]),
                latin: list(&["Eisu"]),
                korean: list(&["Kana"]),
                script_cycle: list(&["LCtrl+LAlt"]),
                seokdok: empty.clone(),
                hangul: empty,
                convert: list(&["Alt+Enter", "F7"]),
            },
            Platform::Linux => ModeKeys {
                toggle: list(&["Hangul", "Zenkaku", "Shift+Space", "RAlt"]),
                latin: list(&["Muhenkan"]),
                korean: list(&["Henkan"]),
                script_cycle: list(&["LCtrl+LAlt", "Shift+Hangul"]),
                seokdok: empty.clone(),
                hangul: empty,
                convert: list(&["Hanja", "F9", "F7", "Henkan"]),
            },
            Platform::Android => ModeKeys {
                toggle: list(&["Hangul", "Zenkaku", "Shift+Space"]),
                latin: list(&["Muhenkan"]),
                korean: list(&["Henkan"]),
                script_cycle: list(&["LCtrl+LAlt", "Shift+Hangul"]),
                seokdok: empty.clone(),
                hangul: empty,
                convert: list(&["Hanja", "F7", "Henkan"]),
            },
            Platform::Ios => ModeKeys {
                toggle: empty.clone(),
                latin: empty.clone(),
                korean: empty.clone(),
                script_cycle: empty.clone(),
                seokdok: empty.clone(),
                hangul: empty.clone(),
                convert: empty,
            },
        }
    }
}

impl Default for ModeKeys {
    fn default() -> Self {
        ModeKeys::for_platform(Platform::current())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Toggle,
    Latin,
    ScriptCycle,
    Seokdok,
    Hangul,
    Korean,
    Convert,
}

/// Reads key events against the lists: a key press becomes a session [`Key`]
/// at once, modifiers tapped alone on release.
#[derive(Clone, Debug)]
pub struct ModeKeyboard {
    source: ModeKeys,
    binds: Vec<(Action, Chord)>,
    /// Modifiers down now.
    held: u8,
    /// Every modifier down since none were.
    peak: u8,
    /// Another key went down, or a tap already fired, since none were.
    spent: bool,
}

impl ModeKeyboard {
    pub fn new(keys: &ModeKeys) -> Self {
        let mut k = ModeKeyboard { source: ModeKeys::for_platform(Platform::Ios), binds: Vec::new(), held: 0, peak: 0, spent: false };
        k.set_keys(keys);
        k
    }

    /// Use these lists. Entries that do not parse are ignored.
    pub fn set_keys(&mut self, keys: &ModeKeys) {
        if *keys == self.source {
            return;
        }
        let groups: [(Action, &[String]); 7] = [
            (Action::Toggle, &keys.toggle),
            (Action::Latin, &keys.latin),
            (Action::ScriptCycle, &keys.script_cycle),
            (Action::Seokdok, &keys.seokdok),
            (Action::Hangul, &keys.hangul),
            (Action::Korean, &keys.korean),
            (Action::Convert, &keys.convert),
        ];
        self.binds = groups
            .iter()
            .flat_map(|(action, entries)| entries.iter().filter_map(|e| Chord::parse(e).map(|c| (*action, c))))
            .collect();
        self.source = keys.clone();
    }

    /// Forget what is held (focus moved, and a release may never arrive).
    pub fn reset(&mut self) {
        self.held = 0;
        self.peak = 0;
        self.spent = false;
    }

    /// This modifier has just gone down from up. If it is already marked as
    /// held, its release never reached us (a host can swallow it), and what is
    /// remembered is stale: forget it, or no tap would fire until a typed key.
    pub fn modifier_pressed_afresh(&mut self, m: Modifier) {
        if self.held & m.bit() != 0 {
            self.reset();
        }
    }

    /// A modifier is down. Telling this twice changes nothing.
    pub fn modifier_down(&mut self, m: Modifier) {
        self.held |= m.bit();
        self.peak |= m.bit();
    }

    /// A key went down; `None` is a key no list can name. `mods` are the
    /// `MOD_*` flags held with it. Returns the session key when it is bound.
    /// Safe to call twice for one key event.
    pub fn press(&mut self, key: Option<KeyName>, mods: u8, session: &Session) -> Option<Key> {
        if let Some(KeyName::Modifier(m)) = key {
            self.modifier_down(m);
            return None;
        }
        // A typed key with nothing held: anything remembered as held is stale.
        if mods == 0 || self.held == 0 {
            self.reset();
        } else {
            self.spent = true;
        }
        self.peek_press(key, mods, session)
    }

    /// What this press would do, leaving the tracker as it is.
    pub fn peek_press(&self, key: Option<KeyName>, mods: u8, session: &Session) -> Option<Key> {
        let key = key?;
        // A key the composition itself uses keeps that meaning in 釋讀:
        // Shift+Space stays 띄어쓰기 and previous candidate.
        if key.composes() && mods & !MOD_SHIFT == 0 && session.is_composing() && session.mode() == Mode::Seokdok {
            return None;
        }
        self.resolve(session, |c| matches!(c, Chord::Press { mods: m, key: k } if *m == mods && *k == key))
    }

    /// A key came up. Returns the session key when this completes a tap.
    pub fn release(&mut self, key: KeyName, session: &Session) -> Option<Key> {
        let KeyName::Modifier(m) = key else { return None };
        let fired = self.peek_release(key, session);
        self.spent = true;
        self.held &= !m.bit();
        if self.held == 0 {
            self.reset();
        }
        fired
    }

    /// What releasing this key would do, leaving the tracker as it is.
    pub fn peek_release(&self, key: KeyName, session: &Session) -> Option<Key> {
        let KeyName::Modifier(m) = key else { return None };
        if self.spent || self.held & m.bit() == 0 {
            return None;
        }
        self.resolve(session, |c| c.tapped_by(self.peak))
    }

    fn resolve(&self, session: &Session, matches: impl Fn(&Chord) -> bool) -> Option<Key> {
        let bound = |a: Action| self.binds.iter().any(|(action, c)| *action == a && matches(c));
        let latin = session.mode() == Mode::Latin;
        if bound(Action::Toggle) {
            return Some(Key::HangulToggle);
        }
        if bound(Action::Latin) {
            return Some(Key::ToLatin);
        }
        if bound(Action::ScriptCycle) {
            return Some(Key::CycleScript);
        }
        if bound(Action::Seokdok) {
            return Some(Key::ToSeokdok);
        }
        if bound(Action::Hangul) {
            return Some(Key::ToHangul);
        }
        // 変換 is two things: out of Latin, and convert once in Korean.
        if bound(Action::Korean) && (latin || !bound(Action::Convert)) {
            return Some(Key::ToKorean);
        }
        bound(Action::Convert).then_some(Key::Convert)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Converter, Effect, Output};
    use seokdok_core::{Candidate, Segment};
    use KeyName::Modifier as M;
    use Modifier::*;

    /// One segment with two candidates.
    struct Stub;

    impl Converter for Stub {
        fn segments(&self, input: &str) -> Vec<Segment> {
            vec![Segment {
                reading: input.into(),
                candidates: vec![
                    Candidate { text: format!("{input}1"), note: String::new() },
                    Candidate { text: format!("{input}2"), note: String::new() },
                ],
            }]
        }
    }

    fn keyboard(p: Platform) -> ModeKeyboard {
        ModeKeyboard::new(&ModeKeys::for_platform(p))
    }

    fn fresh() -> Session {
        Session::new(Mode::Seokdok, true)
    }

    fn composing() -> Session {
        let mut s = fresh();
        for c in "gks".chars() {
            s.handle(Key::Letter { ch: c, shift: false }, &Stub);
        }
        s
    }

    fn press(k: &mut ModeKeyboard, s: &Session, chord: &str) -> Option<Key> {
        let Chord::Press { mods, key } = Chord::parse(chord).unwrap() else { panic!("{chord}") };
        k.press(Some(key), mods, s)
    }

    /// Press these modifiers in order and release them in order.
    fn tap(k: &mut ModeKeyboard, s: &Session, mods: &[Modifier]) -> Vec<Option<Key>> {
        for m in mods {
            assert_eq!(k.press(Some(M(*m)), 0, s), None);
        }
        mods.iter().map(|m| k.release(M(*m), s)).collect()
    }

    fn commit_of(o: &Output) -> Option<&str> {
        o.effects.iter().find_map(|e| match e {
            Effect::Commit(t) => Some(t.as_str()),
            _ => None,
        })
    }

    #[test]
    fn entries_parse() {
        assert_eq!(Chord::parse("Hangul"), Some(Chord::Press { mods: 0, key: KeyName::Hangul }));
        assert_eq!(Chord::parse(" shift + SPACE "), Some(Chord::Press { mods: MOD_SHIFT, key: KeyName::Space }));
        assert_eq!(Chord::parse("Alt+Enter"), Some(Chord::Press { mods: MOD_ALT, key: KeyName::Enter }));
        assert_eq!(Chord::parse("Ctrl+Alt+F12"), Some(Chord::Press { mods: MOD_CTRL | MOD_ALT, key: KeyName::Function(12) }));
        assert_eq!(Chord::parse("Ctrl+h"), Some(Chord::Press { mods: MOD_CTRL, key: KeyName::Char('H') }));
        assert_eq!(Chord::parse("RAlt"), Some(Chord::Tap(vec![RAlt.bit()])));
        assert_eq!(Chord::parse("Option"), Some(Chord::Tap(vec![ALT_BITS])));
        assert_eq!(Chord::parse("LCtrl+LAlt"), Some(Chord::Tap(vec![LCtrl.bit(), LAlt.bit()])));
        for bad in ["", "F25", "F0", "Nope", "Hangul+Hanja", "Ctrl+"] {
            assert_eq!(Chord::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn every_default_entry_parses() {
        for p in [Platform::Windows, Platform::Mac, Platform::Linux, Platform::Android, Platform::Ios] {
            let d = ModeKeys::for_platform(p);
            for e in [&d.toggle, &d.latin, &d.korean, &d.script_cycle, &d.seokdok, &d.hangul, &d.convert].into_iter().flatten() {
                assert!(Chord::parse(e).is_some(), "{p:?} {e}");
            }
        }
    }

    #[test]
    fn ios_binds_nothing() {
        let mut k = keyboard(Platform::Ios);
        let s = fresh();
        assert_eq!(press(&mut k, &s, "Hangul"), None);
        assert_eq!(tap(&mut k, &s, &[RAlt]), [None]);
    }

    #[test]
    fn familiar_keys_per_platform() {
        let s = fresh();
        let mut win = keyboard(Platform::Windows);
        assert_eq!(press(&mut win, &s, "Hangul"), Some(Key::HangulToggle));
        assert_eq!(press(&mut win, &s, "Zenkaku"), Some(Key::HangulToggle));
        assert_eq!(press(&mut win, &s, "Hanja"), Some(Key::Convert));
        assert_eq!(press(&mut win, &s, "F7"), Some(Key::Convert));
        assert_eq!(press(&mut win, &s, "Muhenkan"), Some(Key::ToLatin));
        assert_eq!(press(&mut win, &s, "ImeOff"), Some(Key::ToLatin));
        assert_eq!(press(&mut win, &s, "Shift+Hangul"), Some(Key::CycleScript));
        // Windows leaves Shift+Space to the session and the type 3 keyboard driver.
        assert_eq!(press(&mut win, &s, "Shift+Space"), None);
        assert_eq!(tap(&mut win, &s, &[RCtrl]), [Some(Key::Convert)]);
        assert_eq!(tap(&mut win, &s, &[LAlt]), [None]);

        let mut mac = keyboard(Platform::Mac);
        assert_eq!(press(&mut mac, &s, "Eisu"), Some(Key::ToLatin));
        assert_eq!(press(&mut mac, &s, "Alt+Enter"), Some(Key::Convert));
        assert_eq!(press(&mut mac, &s, "Enter"), None);
        assert_eq!(tap(&mut mac, &s, &[LAlt]), [Some(Key::HangulToggle)]);
        assert_eq!(tap(&mut mac, &s, &[RAlt]), [Some(Key::HangulToggle)]);

        let mut linux = keyboard(Platform::Linux);
        assert_eq!(press(&mut linux, &s, "Hangul"), Some(Key::HangulToggle));
        assert_eq!(press(&mut linux, &s, "Shift+Space"), Some(Key::HangulToggle));
        assert_eq!(press(&mut linux, &s, "Hanja"), Some(Key::Convert));
        assert_eq!(press(&mut linux, &s, "F9"), Some(Key::Convert));

        let mut android = keyboard(Platform::Android);
        assert_eq!(press(&mut android, &s, "Hangul"), Some(Key::HangulToggle));
        assert_eq!(press(&mut android, &s, "Shift+Space"), Some(Key::HangulToggle));
        assert_eq!(press(&mut android, &s, "Hanja"), Some(Key::Convert));
    }

    #[test]
    fn altgr_never_toggles() {
        // Windows reports AltGr as left Ctrl plus right Alt.
        let s = fresh();
        let mut win = keyboard(Platform::Windows);
        assert_eq!(tap(&mut win, &s, &[LCtrl, RAlt]), [None, None]);
        assert_eq!(tap(&mut win, &s, &[RAlt, LCtrl]), [None, None]);
        // AltGr+key: the key press spends the tap too.
        win.press(Some(M(LCtrl)), MOD_CTRL | MOD_ALT, &s);
        win.press(Some(M(RAlt)), MOD_CTRL | MOD_ALT, &s);
        win.press(Some(KeyName::Char('E')), MOD_CTRL | MOD_ALT, &s);
        assert_eq!(win.release(M(RAlt), &s), None);
        assert_eq!(win.release(M(LCtrl), &s), None);
        // Right Alt on its own still works afterwards.
        assert_eq!(tap(&mut win, &s, &[RAlt]), [Some(Key::HangulToggle)]);
    }

    #[test]
    fn a_tap_is_modifiers_alone() {
        let s = fresh();
        let mut k = keyboard(Platform::Windows);
        // RAlt with a letter is a shortcut.
        k.press(Some(M(RAlt)), MOD_ALT, &s);
        k.press(Some(KeyName::Char('F')), MOD_ALT, &s);
        assert_eq!(k.release(M(RAlt), &s), None);
        // An unnamed key counts as well.
        k.press(Some(M(RAlt)), MOD_ALT, &s);
        k.press(None, MOD_ALT, &s);
        assert_eq!(k.release(M(RAlt), &s), None);
        // Shift held too is another chord.
        assert_eq!(tap(&mut k, &s, &[RShift, RAlt]), [None, None]);
        // A chord fires once, on the first release.
        assert_eq!(tap(&mut k, &s, &[LCtrl, LAlt]), [Some(Key::CycleScript), None]);
        // Key repeat of the modifier changes nothing.
        k.press(Some(M(RAlt)), MOD_ALT, &s);
        k.press(Some(M(RAlt)), MOD_ALT, &s);
        assert_eq!(k.release(M(RAlt), &s), Some(Key::HangulToggle));
        // Peeking does not consume it.
        k.press(Some(M(RAlt)), MOD_ALT, &s);
        assert_eq!(k.peek_release(M(RAlt), &s), Some(Key::HangulToggle));
        assert_eq!(k.peek_release(M(RAlt), &s), Some(Key::HangulToggle));
        assert_eq!(k.release(M(RAlt), &s), Some(Key::HangulToggle));
        assert_eq!(k.peek_release(M(RAlt), &s), None);
    }

    #[test]
    fn a_typed_key_forgets_a_lost_release() {
        let s = fresh();
        let mut k = keyboard(Platform::Windows);
        // Alt+Tab away: the release never comes.
        k.press(Some(M(LAlt)), MOD_ALT, &s);
        k.press(Some(KeyName::Tab), MOD_ALT, &s);
        k.press(None, 0, &s);
        assert_eq!(tap(&mut k, &s, &[RAlt]), [Some(Key::HangulToggle)]);
        k.press(Some(M(LAlt)), MOD_ALT, &s);
        k.reset();
        assert_eq!(tap(&mut k, &s, &[RAlt]), [Some(Key::HangulToggle)]);
    }

    #[test]
    fn shift_space_toggles_except_while_composing_seokdok() {
        let mut k = keyboard(Platform::Linux);
        let mut s = fresh();
        // Nothing composed: the toggle.
        assert_eq!(press(&mut k, &s, "Shift+Space"), Some(Key::HangulToggle));
        // Composing 釋讀: 띄어쓰기, the session's own key.
        s = composing();
        assert_eq!(press(&mut k, &s, "Shift+Space"), None);
        let o = s.handle(Key::Space { shift: true }, &Stub);
        assert!(o.eaten && commit_of(&o).is_none());
        assert_eq!(s.buffer(), "한 ");
        // Converted: previous candidate, also the session's.
        s.handle(Key::Space { shift: false }, &Stub);
        assert_eq!(press(&mut k, &s, "Shift+Space"), None);
        // Plain 한글 composing: Space alone types the space, so the toggle wins.
        let mut plain = Session::new(Mode::Hangul, true);
        plain.handle(Key::Letter { ch: 'g', shift: false }, &Stub);
        assert_eq!(press(&mut k, &plain, "Shift+Space"), Some(Key::HangulToggle));
        let o = plain.handle(Key::HangulToggle, &Stub);
        assert_eq!(commit_of(&o), Some("ㅎ"));
        assert_eq!(plain.mode(), Mode::Latin);
        // Latin: back to Korean.
        assert_eq!(press(&mut k, &plain, "Shift+Space"), Some(Key::HangulToggle));
    }

    #[test]
    fn a_shortcut_on_a_composing_key_still_fires() {
        // Option-Return converts while composing; Return alone is the session's.
        let mut mac = keyboard(Platform::Mac);
        let s = composing();
        assert_eq!(press(&mut mac, &s, "Alt+Enter"), Some(Key::Convert));
        assert_eq!(press(&mut mac, &s, "Enter"), None);
    }

    #[test]
    fn henkan_leaves_latin_then_converts() {
        let mut k = keyboard(Platform::Windows);
        let mut s = composing();
        assert_eq!(press(&mut k, &s, "Henkan"), Some(Key::Convert));
        assert!(s.handle(Key::Convert, &Stub).eaten);
        s.set_mode(Mode::Latin);
        assert_eq!(press(&mut k, &s, "Henkan"), Some(Key::ToKorean));
        s.handle(Key::ToKorean, &Stub);
        assert_eq!(s.mode(), Mode::Seokdok);
        // Idle in Korean it converts nothing, so the key stays the application's.
        assert_eq!(press(&mut k, &s, "Henkan"), Some(Key::Convert));
        assert!(!s.eats(Key::Convert));
    }

    #[test]
    fn a_mac_kana_key_is_quiet_in_korean() {
        let mut mac = keyboard(Platform::Mac);
        let mut s = composing();
        assert_eq!(press(&mut mac, &s, "Kana"), Some(Key::ToKorean));
        let o = s.handle(Key::ToKorean, &Stub);
        assert!(o.eaten && o.effects.is_empty());
        assert_eq!(s.buffer(), "한");
    }

    #[test]
    fn lists_are_the_users_to_edit() {
        let keys = ModeKeys {
            toggle: list(&["F12"]),
            seokdok: list(&["Ctrl+1"]),
            hangul: list(&["Ctrl+2", "not a key"]),
            ..ModeKeys::for_platform(Platform::Ios)
        };
        let mut k = ModeKeyboard::new(&keys);
        let s = fresh();
        assert_eq!(press(&mut k, &s, "F12"), Some(Key::HangulToggle));
        assert_eq!(press(&mut k, &s, "Ctrl+1"), Some(Key::ToSeokdok));
        assert_eq!(press(&mut k, &s, "Ctrl+2"), Some(Key::ToHangul));
        assert_eq!(press(&mut k, &s, "Hangul"), None);
        k.set_keys(&ModeKeys::for_platform(Platform::Windows));
        assert_eq!(press(&mut k, &s, "Hangul"), Some(Key::HangulToggle));
        assert_eq!(press(&mut k, &s, "F12"), None);
    }

    #[test]
    fn a_press_peeks_without_a_trace() {
        let s = fresh();
        let mut k = keyboard(Platform::Windows);
        k.press(Some(M(RAlt)), MOD_ALT, &s);
        assert_eq!(k.peek_press(Some(KeyName::Hanja), 0, &s), Some(Key::Convert));
        assert_eq!(k.release(M(RAlt), &s), Some(Key::HangulToggle));
    }

    #[test]
    fn a_lost_release_does_not_stop_the_next_tap() {
        let s = Session::new(Mode::Seokdok, true);
        let keys = ModeKeys::for_platform(Platform::Windows);
        let mut k = ModeKeyboard::new(&keys);
        let tap = |k: &mut ModeKeyboard, lose_ctrl_up: bool| {
            k.modifier_pressed_afresh(LCtrl);
            k.modifier_down(LCtrl);
            k.modifier_pressed_afresh(LAlt);
            k.modifier_down(LAlt);
            let fired = k.release(M(LAlt), &s);
            if !lose_ctrl_up {
                k.release(M(LCtrl), &s);
            }
            fired
        };
        for _ in 0..3 {
            assert_eq!(tap(&mut k, true), Some(Key::CycleScript));
        }
        assert_eq!(tap(&mut k, false), Some(Key::CycleScript));
        assert_eq!(tap(&mut k, false), Some(Key::CycleScript));
        // The same for one key alone.
        k.reset();
        for _ in 0..3 {
            k.modifier_pressed_afresh(RCtrl);
            k.modifier_down(RCtrl);
            assert_eq!(k.release(M(RCtrl), &s), Some(Key::Convert));
            k.modifier_down(RCtrl);
        }
    }
}
