//! The Windows quick menu, without the Windows part: which entries it has for
//! the settings and mode at hand, what choosing one does, and what the icon of
//! the input indicator shows. `seokdok-tsf` draws these.

use crate::config::{Config, FragmentOpt, Layout, Script, StyleOpt};
use crate::session::Mode;

/// What `windows.menu_items` can name, in the menu's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Script,
    InputMode,
    CandidateLayout,
    OldHa,
    OldClusters,
    SpaceConverts,
    Style,
    Fragment,
    CodaAlways,
    Resyllabify,
    HadaKun,
}

impl MenuItem {
    pub const ALL: [MenuItem; 11] = [
        MenuItem::Script,
        MenuItem::InputMode,
        MenuItem::CandidateLayout,
        MenuItem::OldHa,
        MenuItem::OldClusters,
        MenuItem::SpaceConverts,
        MenuItem::Style,
        MenuItem::Fragment,
        MenuItem::CodaAlways,
        MenuItem::Resyllabify,
        MenuItem::HadaKun,
    ];

    /// The name in `windows.menu_items`.
    pub fn key(self) -> &'static str {
        match self {
            MenuItem::Script => "script",
            MenuItem::InputMode => "input_mode",
            MenuItem::CandidateLayout => "candidate_layout",
            MenuItem::OldHa => "old_ha",
            MenuItem::OldClusters => "old_clusters",
            MenuItem::SpaceConverts => "space_converts",
            MenuItem::Style => "style",
            MenuItem::Fragment => "fragment",
            MenuItem::CodaAlways => "coda_always",
            MenuItem::Resyllabify => "resyllabify",
            MenuItem::HadaKun => "hada_kun",
        }
    }

    /// The text of the menu entry, or of the group for the two that have
    /// several entries.
    pub fn label(self) -> &'static str {
        match self {
            MenuItem::Script => "文字 / Script",
            MenuItem::InputMode => "入力 / Input",
            MenuItem::CandidateLayout => "候補 橫으로 / Horizontal candidates",
            MenuItem::OldHa => "하 → ᄒᆞ 優先 / Old spelling of 하 first",
            MenuItem::OldClusters => "옛한글 겹자모 / Old Hangul clusters",
            MenuItem::SpaceConverts => "Space로 變換 / Space converts",
            MenuItem::Style => "形態式 / Morpheme style  (見았다)",
            MenuItem::Fragment => "남는 母音 字母로 / Leftover vowel as a letter",
            MenuItem::CodaAlways => "末音添記 늘 / Always write the final consonant",
            MenuItem::Resyllabify => "받침 넘기기 / Resyllabify",
            MenuItem::HadaKun => "하다를 爲로 / Prefer 爲 for 하다",
        }
    }

    /// Whether this setting is on. Only for the entries that are one check
    /// box; the two groups have one check per entry.
    fn is_on(self, c: &Config) -> bool {
        match self {
            MenuItem::Script | MenuItem::InputMode => false,
            MenuItem::CandidateLayout => c.input.candidate_layout == Layout::Horizontal,
            MenuItem::OldHa => c.conversion.old_ha,
            MenuItem::OldClusters => c.input.old_clusters,
            MenuItem::SpaceConverts => c.input.space_converts,
            MenuItem::Style => c.conversion.style == StyleOpt::Morpheme,
            MenuItem::Fragment => c.conversion.fragment == FragmentOpt::Letter,
            MenuItem::CodaAlways => c.conversion.coda_always,
            MenuItem::Resyllabify => c.conversion.resyllabify,
            MenuItem::HadaKun => c.conversion.hada_kun,
        }
    }

    fn flip(self, c: &mut Config) {
        match self {
            MenuItem::Script | MenuItem::InputMode => {}
            MenuItem::CandidateLayout => {
                c.input.candidate_layout = match c.input.candidate_layout {
                    Layout::Vertical => Layout::Horizontal,
                    Layout::Horizontal => Layout::Vertical,
                }
            }
            MenuItem::OldHa => c.conversion.old_ha ^= true,
            MenuItem::OldClusters => c.input.old_clusters ^= true,
            MenuItem::SpaceConverts => c.input.space_converts ^= true,
            MenuItem::Style => {
                c.conversion.style = match c.conversion.style {
                    StyleOpt::Surface => StyleOpt::Morpheme,
                    StyleOpt::Morpheme => StyleOpt::Surface,
                }
            }
            MenuItem::Fragment => {
                c.conversion.fragment = match c.conversion.fragment {
                    FragmentOpt::Absorb => FragmentOpt::Letter,
                    FragmentOpt::Letter => FragmentOpt::Absorb,
                }
            }
            MenuItem::CodaAlways => c.conversion.coda_always ^= true,
            MenuItem::Resyllabify => c.conversion.resyllabify ^= true,
            MenuItem::HadaKun => c.conversion.hada_kun ^= true,
        }
    }
}

/// What choosing an entry does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Script(Script),
    Mode(Mode),
    Flip(MenuItem),
    Settings,
}

impl Action {
    /// The menu command id, above 0: `TrackPopupMenu` returns 0 for no choice.
    pub fn id(self) -> u32 {
        match self {
            Action::Script(Script::Seokdok) => 1,
            Action::Script(Script::SinoOnly) => 2,
            Action::Script(Script::HangulOnly) => 3,
            Action::Mode(Mode::Seokdok) => 11,
            Action::Mode(Mode::Hangul) => 12,
            Action::Mode(Mode::Latin) => 13,
            Action::Flip(item) => 21 + MenuItem::ALL.iter().position(|i| *i == item).unwrap_or(0) as u32,
            Action::Settings => 100,
        }
    }

    pub fn from_id(id: u32) -> Option<Action> {
        Some(match id {
            1 => Action::Script(Script::Seokdok),
            2 => Action::Script(Script::SinoOnly),
            3 => Action::Script(Script::HangulOnly),
            11 => Action::Mode(Mode::Seokdok),
            12 => Action::Mode(Mode::Hangul),
            13 => Action::Mode(Mode::Latin),
            100 => Action::Settings,
            _ => Action::Flip(*MenuItem::ALL.get(id.checked_sub(21)? as usize)?),
        })
    }

    /// Make the choice in `config`. Modes and Settings… are not in the file.
    pub fn change(self, config: &mut Config) {
        match self {
            Action::Script(s) => config.conversion.script = s,
            Action::Flip(item) => item.flip(config),
            Action::Mode(_) | Action::Settings => {}
        }
    }

    /// The input mode this choice leads to. A script is chosen to write in it,
    /// so it leaves plain Hangul and Latin. With 韓㐎 專用 the session reports
    /// 釋讀 as 한글 and goes back to 釋讀 when the setting is turned off.
    pub fn mode(self) -> Option<Mode> {
        match self {
            Action::Script(_) => Some(Mode::Seokdok),
            Action::Mode(m) => Some(m),
            Action::Flip(_) | Action::Settings => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// One choice of a group.
    Radio(bool),
    /// A setting that is on or off.
    Box(bool),
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Separator,
    /// A line that cannot be chosen: a group's title, or with `hint` a key that
    /// has no entry of its own (변환).
    Heading { label: &'static str, hint: String },
    /// `hint` names the keys that do the same, empty when none is bound.
    Choice { action: Action, label: &'static str, check: Check, hint: String },
}

/// Keys found only on Japanese keyboards.
const JIS_KEYS: [&str; 5] = ["Zenkaku", "Henkan", "Muhenkan", "ImeOn", "ImeOff"];
/// How many keys a hint names: enough to recognise one's own, and short.
const HINT_KEYS: usize = 2;

/// `keys` as a hint: what this keyboard can press, with the Korean keys under
/// their printed names.
fn hint(keys: &[&[String]], jis: bool) -> String {
    let usable = |k: &&String| jis || !k.split('+').any(|part| JIS_KEYS.contains(&part));
    let list = keys.iter().map(|l| l.iter().filter(usable).collect::<Vec<_>>()).find(|l| !l.is_empty()).unwrap_or_default();
    let name = |k: &String| k.replace("Hangul", "한/영").replace("Hanja", "한자");
    list.into_iter().take(HINT_KEYS).map(name).collect::<Vec<_>>().join(", ")
}

const SCRIPTS: [(Script, &str); 3] = [
    (Script::Seokdok, "釋 釋讀 / Seokdok"),
    (Script::SinoOnly, "混 國漢文混用 / Hanja for Sino-Korean words only"),
    (Script::HangulOnly, "한 韓㐎 專用 / Hangul only"),
];

const MODES: [(Mode, &str); 3] =
    [(Mode::Seokdok, "釋讀 / Conversion"), (Mode::Hangul, "한글 / Hangul"), (Mode::Latin, "A / Latin")];

/// The menu for these settings, with `shown` the input mode in use. Settings…
/// is always there, so what `windows.menu_items` removes can be brought back.
/// `jis` says the keyboard is Japanese, for the keys the hints name.
pub fn entries(config: &Config, shown: Mode, jis: bool) -> Vec<Entry> {
    let keys = &config.input.keys;
    let mode_hint = |m: Mode| match m {
        Mode::Seokdok => hint(&[&keys.seokdok, &keys.seokdok_hangul], jis),
        Mode::Hangul => hint(&[&keys.hangul, &keys.seokdok_hangul], jis),
        Mode::Latin => hint(&[&keys.latin, &keys.toggle], jis),
    };
    let heading = |label| Entry::Heading { label, hint: String::new() };
    let chosen = |item: MenuItem| config.windows.menu_items.iter().any(|k| k == item.key());
    let script = config.conversion.script;
    let mut out = Vec::new();
    let mut group = |group: &[Entry]| {
        if !group.is_empty() {
            if !out.is_empty() {
                out.push(Entry::Separator);
            }
            out.extend_from_slice(group);
        }
    };

    if chosen(MenuItem::Script) {
        let mut g = vec![heading(MenuItem::Script.label())];
        g.extend(SCRIPTS.map(|(s, label)| Entry::Choice {
            action: Action::Script(s),
            label,
            check: Check::Radio(s == script),
            hint: String::new(),
        }));
        group(&g);
    }
    if chosen(MenuItem::InputMode) {
        let mut g = vec![heading(MenuItem::InputMode.label())];
        // 韓㐎 專用 has no 釋讀 to tell from 한글.
        g.extend(MODES.iter().filter(|(m, _)| script != Script::HangulOnly || *m != Mode::Seokdok).map(
            |(m, label)| Entry::Choice { action: Action::Mode(*m), label, check: Check::Radio(*m == shown), hint: mode_hint(*m) },
        ));
        let convert = hint(&[&keys.convert], jis);
        if !convert.is_empty() {
            g.push(Entry::Heading { label: "變換 / Convert", hint: convert });
        }
        group(&g);
    }
    let flips: Vec<Entry> = MenuItem::ALL
        .into_iter()
        .filter(|i| !matches!(i, MenuItem::Script | MenuItem::InputMode) && chosen(*i))
        .map(|i| Entry::Choice { action: Action::Flip(i), label: i.label(), check: Check::Box(i.is_on(config)), hint: String::new() })
        .collect();
    group(&flips);
    group(&[Entry::Choice { action: Action::Settings, label: "設定 / 설정 / Settings…", check: Check::Plain, hint: String::new() }]);
    out
}

/// What the icon of the input indicator shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indicator {
    Seokdok,
    Mixed,
    Hangul,
    Latin,
}

impl Indicator {
    /// What will be typed in `mode` under `script`: conversion is where the
    /// script shows, and 한 means no conversion from either side.
    pub fn of(mode: Mode, script: Script) -> Indicator {
        match (mode, script) {
            (Mode::Latin, _) => Indicator::Latin,
            (Mode::Hangul, _) | (Mode::Seokdok, Script::HangulOnly) => Indicator::Hangul,
            (Mode::Seokdok, Script::Seokdok) => Indicator::Seokdok,
            (Mode::Seokdok, Script::SinoOnly) => Indicator::Mixed,
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Indicator::Seokdok => "釋",
            Indicator::Mixed => "混",
            Indicator::Hangul => "한",
            Indicator::Latin => "A",
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Indicator::Seokdok => "釋讀 (seokdok): 釋讀",
            Indicator::Mixed => "釋讀 (seokdok): 國漢文混用",
            Indicator::Hangul => "釋讀 (seokdok): 한글",
            Indicator::Latin => "釋讀 (seokdok): A Latin",
        }
    }
}

/// Where a left click on the icon goes: between conversion and plain Hangul,
/// or with 韓㐎 專用, where there is no conversion, between Hangul and Latin.
pub fn click_target(shown: Mode, script: Script) -> Mode {
    match (script == Script::HangulOnly, shown) {
        (true, Mode::Latin) => Mode::Hangul,
        (true, _) => Mode::Latin,
        (false, Mode::Seokdok) => Mode::Hangul,
        (false, _) => Mode::Seokdok,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Session;

    fn with_items(items: &[&str]) -> Config {
        let mut c = Config::default();
        c.windows.menu_items = items.iter().map(|s| s.to_string()).collect();
        c
    }

    fn actions(entries: &[Entry]) -> Vec<Action> {
        entries.iter().filter_map(|e| if let Entry::Choice { action, .. } = e { Some(*action) } else { None }).collect()
    }

    fn check_of(entries: &[Entry], wanted: Action) -> Check {
        entries
            .iter()
            .find_map(|e| match e {
                Entry::Choice { action, check, .. } if *action == wanted => Some(*check),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn every_action_has_its_own_id() {
        let mut all: Vec<Action> = vec![Action::Settings];
        all.extend(SCRIPTS.map(|(s, _)| Action::Script(s)));
        all.extend(MODES.map(|(m, _)| Action::Mode(m)));
        all.extend(MenuItem::ALL.map(Action::Flip));
        let mut ids: Vec<u32> = all.iter().map(|a| a.id()).collect();
        for a in &all {
            assert_ne!(a.id(), 0);
            assert_eq!(Action::from_id(a.id()), Some(*a));
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
        for stray in [0, 4, 14, 20, 21 + MenuItem::ALL.len() as u32, 99, 101] {
            assert_eq!(Action::from_id(stray), None, "{stray}");
        }
    }

    #[test]
    fn the_default_menu_has_script_mode_layout_old_ha_and_settings() {
        let c = Config::default();
        let e = entries(&c, Mode::Seokdok, false);
        assert_eq!(
            actions(&e),
            [
                Action::Script(Script::Seokdok),
                Action::Script(Script::SinoOnly),
                Action::Script(Script::HangulOnly),
                Action::Mode(Mode::Seokdok),
                Action::Mode(Mode::Hangul),
                Action::Mode(Mode::Latin),
                Action::Flip(MenuItem::CandidateLayout),
                Action::Flip(MenuItem::OldHa),
                Action::Settings,
            ]
        );
        assert_eq!(e[0], Entry::Heading { label: "文字 / Script", hint: String::new() });
        assert_eq!(e.iter().filter(|x| **x == Entry::Separator).count(), 3);
    }

    #[test]
    fn menu_items_choose_what_appears() {
        // Unknown names are ignored, and the file's order does not matter.
        let c = with_items(&["hada_kun", "nonsense", "script"]);
        let a = actions(&entries(&c, Mode::Seokdok, false));
        assert_eq!(
            a,
            [
                Action::Script(Script::Seokdok),
                Action::Script(Script::SinoOnly),
                Action::Script(Script::HangulOnly),
                Action::Flip(MenuItem::HadaKun),
                Action::Settings,
            ]
        );
        // Every item can be shown, each by its key.
        let keys: Vec<&str> = MenuItem::ALL.map(MenuItem::key).to_vec();
        let c = with_items(&keys);
        let a = actions(&entries(&c, Mode::Seokdok, false));
        for item in MenuItem::ALL {
            if !matches!(item, MenuItem::Script | MenuItem::InputMode) {
                assert!(a.contains(&Action::Flip(item)), "{item:?}");
            }
        }
        // With nothing chosen, Settings… alone remains, and no separator.
        let e = entries(&with_items(&[]), Mode::Seokdok, false);
        assert_eq!(e.len(), 1);
        assert_eq!(actions(&e), [Action::Settings]);
    }

    #[test]
    fn checks_follow_the_settings_and_the_mode() {
        let mut c = Config::default();
        c.conversion.script = Script::SinoOnly;
        c.conversion.old_ha = true;
        let e = entries(&c, Mode::Hangul, false);
        assert_eq!(check_of(&e, Action::Script(Script::SinoOnly)), Check::Radio(true));
        assert_eq!(check_of(&e, Action::Script(Script::Seokdok)), Check::Radio(false));
        assert_eq!(check_of(&e, Action::Mode(Mode::Hangul)), Check::Radio(true));
        assert_eq!(check_of(&e, Action::Mode(Mode::Seokdok)), Check::Radio(false));
        assert_eq!(check_of(&e, Action::Flip(MenuItem::OldHa)), Check::Box(true));
        assert_eq!(check_of(&e, Action::Flip(MenuItem::CandidateLayout)), Check::Box(false));
        assert_eq!(check_of(&e, Action::Settings), Check::Plain);
    }

    #[test]
    fn hangul_only_has_no_seokdok_input_mode() {
        let mut c = Config::default();
        c.conversion.script = Script::HangulOnly;
        let a = actions(&entries(&c, Mode::Hangul, false));
        assert!(!a.contains(&Action::Mode(Mode::Seokdok)));
        assert!(a.contains(&Action::Mode(Mode::Hangul)));
        assert!(a.contains(&Action::Script(Script::Seokdok)));
    }

    #[test]
    fn every_flip_turns_its_setting_on_and_back() {
        for item in MenuItem::ALL {
            if matches!(item, MenuItem::Script | MenuItem::InputMode) {
                continue;
            }
            let start = Config::default();
            let mut c = start.clone();
            let action = Action::Flip(item);
            action.change(&mut c);
            assert_ne!(c, start, "{item:?}");
            assert_ne!(item.is_on(&c), item.is_on(&start), "{item:?}");
            action.change(&mut c);
            assert_eq!(c, start, "{item:?}");
            assert_eq!(action.mode(), None);
        }
    }

    #[test]
    fn choices_reach_the_written_file() {
        let path = std::env::temp_dir().join(format!("seokdok-menu-{}", std::process::id())).join("config.toml");
        let checks: [(Action, fn(&Config) -> bool); 3] = [
            (Action::Script(Script::SinoOnly), |c| c.conversion.script == Script::SinoOnly),
            (Action::Flip(MenuItem::CandidateLayout), |c| c.input.candidate_layout == Layout::Horizontal),
            (Action::Flip(MenuItem::OldHa), |c| c.conversion.old_ha),
        ];
        for (action, read) in checks {
            Config::update_file(&path, |c| action.change(c)).unwrap();
            assert!(read(&Config::load_from(&path)), "{action:?}");
        }
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn modes_and_settings_do_not_touch_the_file() {
        for action in [Action::Mode(Mode::Latin), Action::Settings] {
            let mut c = Config::default();
            action.change(&mut c);
            assert_eq!(c, Config::default());
        }
    }

    /// What the text service does at the next key after a choice.
    fn choose(session: &mut Session, config: &mut Config, action: Action) -> Indicator {
        action.change(config);
        session.set_hangul_only(config.conversion.script == Script::HangulOnly);
        if let Some(m) = action.mode() {
            session.set_mode(m);
        }
        Indicator::of(session.mode(), config.conversion.script)
    }

    #[test]
    fn choosing_a_script_shows_on_the_icon_from_any_mode() {
        for start in [Mode::Seokdok, Mode::Hangul, Mode::Latin] {
            let mut s = Session::new(start, true);
            let mut c = Config::default();
            assert_eq!(choose(&mut s, &mut c, Action::Script(Script::SinoOnly)), Indicator::Mixed);
            assert_eq!(s.mode(), Mode::Seokdok);
            assert_eq!(choose(&mut s, &mut c, Action::Script(Script::HangulOnly)), Indicator::Hangul);
            assert_eq!(s.mode(), Mode::Hangul);
            assert_eq!(choose(&mut s, &mut c, Action::Script(Script::Seokdok)), Indicator::Seokdok);
            assert_eq!(s.mode(), Mode::Seokdok);
        }
    }

    #[test]
    fn the_icon_tells_the_script_only_in_conversion() {
        use Indicator as I;
        let table = [
            (Mode::Seokdok, Script::Seokdok, I::Seokdok, "釋"),
            (Mode::Seokdok, Script::SinoOnly, I::Mixed, "混"),
            (Mode::Seokdok, Script::HangulOnly, I::Hangul, "한"),
            (Mode::Hangul, Script::Seokdok, I::Hangul, "한"),
            (Mode::Hangul, Script::SinoOnly, I::Hangul, "한"),
            (Mode::Latin, Script::SinoOnly, I::Latin, "A"),
            (Mode::Latin, Script::HangulOnly, I::Latin, "A"),
        ];
        for (mode, script, indicator, glyph) in table {
            assert_eq!(I::of(mode, script), indicator);
            assert_eq!(indicator.glyph(), glyph);
        }
    }

    #[test]
    fn a_click_toggles_conversion_or_with_hangul_only_latin() {
        assert_eq!(click_target(Mode::Seokdok, Script::SinoOnly), Mode::Hangul);
        assert_eq!(click_target(Mode::Hangul, Script::Seokdok), Mode::Seokdok);
        assert_eq!(click_target(Mode::Latin, Script::Seokdok), Mode::Seokdok);
        assert_eq!(click_target(Mode::Hangul, Script::HangulOnly), Mode::Latin);
        assert_eq!(click_target(Mode::Latin, Script::HangulOnly), Mode::Hangul);
    }

    #[test]
    fn entries_name_their_keys() {
        use crate::mode_keys::{ModeKeys, Platform};
        let mut c = Config::default();
        c.input.keys = ModeKeys::for_platform(Platform::Windows);
        let hint_of = |e: &[Entry], wanted: Action| {
            e.iter().find_map(|x| match x {
                Entry::Choice { action, hint, .. } if *action == wanted => Some(hint.clone()),
                _ => None,
            })
        };
        let e = entries(&c, Mode::Seokdok, false);
        assert_eq!(hint_of(&e, Action::Mode(Mode::Latin)).unwrap(), "한/영, RAlt");
        assert_eq!(hint_of(&e, Action::Mode(Mode::Hangul)).unwrap(), "LCtrl+LAlt, Shift+한/영");
        assert_eq!(hint_of(&e, Action::Settings).unwrap(), "");
        assert!(e.contains(&Entry::Heading { label: "變換 / Convert", hint: "한자, RCtrl".into() }));
        // A Japanese keyboard has its own keys, and a key the user bound shows.
        let jis = entries(&c, Mode::Seokdok, true);
        assert_eq!(hint_of(&jis, Action::Mode(Mode::Latin)).unwrap(), "Muhenkan, ImeOff");
        c.input.keys.latin = vec!["F9".into()];
        assert_eq!(hint_of(&entries(&c, Mode::Seokdok, false), Action::Mode(Mode::Latin)).unwrap(), "F9");
    }
}
