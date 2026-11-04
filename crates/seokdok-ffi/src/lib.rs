//! A C interface over the shared input-method session, declared in
//! `include/seokdok.h`: used by the fcitx5 addon (crates/seokdok-fcitx5), the
//! iOS keyboard (ios/) and the macOS input method (macos/).
//!
//! The addon feeds keys in and reads back a view: text to commit, the preedit
//! runs and the candidate page. Strings returned by the getters stay valid
//! until the next call that changes the view.

use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::sync::Mutex;

use seokdok_session::config::{self, Config, InputMode, Layout, Script};
use seokdok_session::engine_host::HOST;
use seokdok_session::keys::Key;
use seokdok_session::mode_keys::{KeyName, ModeKeyboard};
use seokdok_session::script_cycle;
use seokdok_session::session::{Attr, CandidateList, Effect, Mode, Popup, Run, Session};

/// Key kinds passed from C (see `seokdok.h`).
pub const SK_LETTER: i32 = 1;
pub const SK_PRINTABLE: i32 = 2;
pub const SK_SPACE: i32 = 3;
pub const SK_ENTER: i32 = 4;
pub const SK_ESC: i32 = 5;
pub const SK_BACKSPACE: i32 = 6;
pub const SK_TAB: i32 = 7;
pub const SK_LEFT: i32 = 8;
pub const SK_RIGHT: i32 = 9;
pub const SK_UP: i32 = 10;
pub const SK_DOWN: i32 = 11;
pub const SK_PAGEUP: i32 = 12;
pub const SK_PAGEDOWN: i32 = 13;
pub const SK_CONVERT: i32 = 14;
pub const SK_TOGGLE: i32 = 15;
pub const SK_OTHER: i32 = 16;

/// States of a key passed to [`seokdok_mode_key`].
pub const SK_KEY_UP: i32 = 0;
pub const SK_KEY_DOWN: i32 = 1;
pub const SK_KEY_REPEAT: i32 = 2;
pub const SK_MOD_CTRL: u32 = 1;

pub struct Ime {
    session: Session,
    keyboard: ModeKeyboard,
    commit: String,
    preedit: Vec<Run>,
    popup: Option<Popup>,
    list: Option<CandidateList>,
    // C strings handed out, kept alive until the view changes.
    cstrs: Vec<CString>,
    dict_dir: PathBuf,
}

/// The dictionary folder the shared engine was last loaded from. Sessions share
/// one engine, so a new session (a new text field, a new application) reloads
/// only when this changes.
static LOADED: Mutex<Option<PathBuf>> = Mutex::new(None);

fn mode_of(m: InputMode) -> Mode {
    match m {
        InputMode::Seokdok => Mode::Seokdok,
        InputMode::Hangul => Mode::Hangul,
        InputMode::Latin => Mode::Latin,
    }
}

/// Where the dictionaries are: $SEOKDOK_DICT, then the XDG data directories.
fn find_dict(explicit: Option<PathBuf>) -> PathBuf {
    let mut dirs: Vec<PathBuf> = Vec::new();
    dirs.extend(explicit);
    dirs.extend(std::env::var_os("SEOKDOK_DICT").map(PathBuf::from));
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data_home = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| home.map(|h| h.join(".local/share")));
    dirs.extend(data_home.map(|d| d.join("seokdok/dict")));
    let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    dirs.extend(data_dirs.split(':').filter(|s| !s.is_empty()).map(|d| PathBuf::from(d).join("seokdok/dict")));
    dirs.iter().find(|d| d.join("sino.tsv").exists()).cloned().unwrap_or_else(|| PathBuf::from("/usr/share/seokdok/dict"))
}

impl Ime {
    fn new(dict_dir: PathBuf) -> Ime {
        let cfg = config::reload();
        let mut ime = Ime {
            session: Session::new(mode_of(cfg.input.default_mode), cfg.input.space_converts),
            keyboard: ModeKeyboard::new(&cfg.input.keys),
            commit: String::new(),
            preedit: Vec::new(),
            popup: None,
            list: None,
            cstrs: Vec::new(),
            dict_dir,
        };
        ime.sync();
        ime
    }

    /// Re-read the settings; the engine takes changed options as it stands.
    fn sync(&mut self) {
        let cfg: Config = config::reload();
        self.session.set_space_converts(cfg.input.space_converts);
        self.session.set_horizontal(cfg.input.candidate_layout == Layout::Horizontal);
        self.session.set_old_clusters(cfg.input.old_clusters);
        self.keyboard.set_keys(&cfg.input.keys);
        self.keyboard.reset();
        let fx = self.session.set_hangul_only(cfg.conversion.script == Script::HangulOnly);
        self.apply(fx);
        let mut loaded = LOADED.lock().unwrap_or_else(|e| e.into_inner());
        if loaded.as_ref() != Some(&self.dict_dir) || !HOST.started() {
            *loaded = Some(self.dict_dir.clone());
            HOST.start_load(self.dict_dir.clone(), Config::user_dir(), cfg.engine_options());
        } else {
            self.give_engine_options(&cfg);
        }
    }

    /// Give the engine these options, reading the dictionaries again only if
    /// another thread holds it.
    fn give_engine_options(&self, cfg: &Config) {
        let options = cfg.engine_options();
        if !HOST.apply_options(&options) {
            HOST.start_load(self.dict_dir.clone(), Config::user_dir(), options);
        }
    }

    /// `input.keys.script_cycle`; a config.toml that does not parse changes nothing.
    fn rotate_script(&mut self) {
        if let Ok(r) = script_cycle::rotate(&mut self.session, &HOST) {
            self.give_engine_options(&r.config);
            self.apply(r.effects);
        }
    }

    fn apply(&mut self, effects: Vec<Effect>) {
        self.cstrs.clear();
        for e in effects {
            match e {
                Effect::Commit(t) => {
                    self.commit.push_str(&t);
                    self.preedit.clear();
                }
                Effect::Preedit(runs) => self.preedit = runs,
                Effect::Cancel => self.preedit.clear(),
                Effect::Popup(p) => self.popup = p,
            }
        }
    }

    fn cstr(&mut self, s: &str) -> *const c_char {
        let c = CString::new(s.replace('\0', "")).unwrap();
        let p = c.as_ptr();
        self.cstrs.push(c);
        p
    }
}

fn key_of(kind: i32, ch: u32, shift: bool) -> Option<Key> {
    let c = char::from_u32(ch);
    Some(match kind {
        SK_LETTER => Key::Letter { ch: c?.to_ascii_lowercase(), shift },
        SK_PRINTABLE => Key::Printable(c?),
        SK_SPACE => Key::Space { shift },
        SK_ENTER => Key::Enter,
        SK_ESC => Key::Esc,
        SK_BACKSPACE => Key::Backspace,
        SK_TAB => Key::Tab { shift },
        SK_LEFT => Key::Left { shift },
        SK_RIGHT => Key::Right { shift },
        SK_UP => Key::Up,
        SK_DOWN => Key::Down,
        SK_PAGEUP => Key::PageUp,
        SK_PAGEDOWN => Key::PageDown,
        SK_CONVERT => Key::Convert,
        SK_TOGGLE => Key::HangulToggle,
        SK_OTHER => Key::Other,
        _ => return None,
    })
}

/// Create a session. `dict_dir` may be NULL to search the usual places.
///
/// # Safety
/// `dict_dir` is NULL or a NUL-terminated UTF-8 path.
#[no_mangle]
pub unsafe extern "C" fn seokdok_new(dict_dir: *const c_char) -> *mut Ime {
    let explicit = (!dict_dir.is_null()).then(|| PathBuf::from(CStr::from_ptr(dict_dir).to_string_lossy().into_owned()));
    Box::into_raw(Box::new(Ime::new(find_dict(explicit))))
}

/// # Safety
/// `ime` came from [`seokdok_new`] and is not used afterwards.
#[no_mangle]
pub unsafe extern "C" fn seokdok_free(ime: *mut Ime) {
    if !ime.is_null() {
        drop(Box::from_raw(ime));
    }
}

/// Re-read settings (call when the input method is activated).
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_sync(ime: *mut Ime) {
    (*ime).sync();
}

/// Feed one key press. Returns true when the key was used.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_key(ime: *mut Ime, kind: i32, ch: u32, shift: bool) -> bool {
    let ime = &mut *ime;
    let Some(key) = key_of(kind, ch, shift) else { return false };
    let out = ime.session.handle(key, &HOST);
    // A key left to the application can still commit first.
    ime.apply(out.effects);
    out.eaten
}

/// Offer a key to the `[input.keys]` lists before [`seokdok_key`]. Call it for
/// every key that goes down, and for modifiers that come up. `name` is the
/// key's name in those lists, or NULL for a key they cannot name. `mods` holds
/// the SK_MOD_* flags held with it, and `state` is SK_KEY_DOWN, SK_KEY_UP or
/// SK_KEY_REPEAT for auto-repeat. True when the key was used: its effects are
/// in the view, and it must not reach [`seokdok_key`] or the application.
///
/// # Safety
/// `ime` is a live session; `name` is NULL or a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn seokdok_mode_key(ime: *mut Ime, name: *const c_char, mods: u32, state: i32) -> bool {
    let ime = &mut *ime;
    let key = if name.is_null() { None } else { CStr::from_ptr(name).to_str().ok().and_then(KeyName::from_name) };
    let mods = mods as u8;
    let bound = match state {
        SK_KEY_DOWN => ime.keyboard.press(key, mods, &ime.session),
        SK_KEY_UP => key.and_then(|k| ime.keyboard.release(k, &ime.session)),
        // A held key acts once; its repeats go no further. They still count as
        // a key pressed while a modifier is being tapped.
        _ => return ime.keyboard.press(key, mods, &ime.session).is_some_and(|k| ime.session.eats(k)),
    };
    let Some(bound) = bound else { return false };
    if bound == Key::CycleScript {
        ime.rotate_script();
        return true;
    }
    let out = ime.session.handle(bound, &HOST);
    ime.apply(out.effects);
    out.eaten
}

/// Commit whatever is being composed (focus loss, deactivation).
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_flush(ime: *mut Ime) {
    let ime = &mut *ime;
    let fx = ime.session.flush();
    ime.apply(fx);
}

/// Forget the composition without committing (the client reset it).
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_reset(ime: *mut Ime) {
    let ime = &mut *ime;
    ime.session.reset();
    ime.preedit.clear();
    ime.popup = None;
}

/// Text to commit since the last call, or NULL. Valid until the next call.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_take_commit(ime: *mut Ime) -> *const c_char {
    let ime = &mut *ime;
    if ime.commit.is_empty() {
        return std::ptr::null();
    }
    let t = std::mem::take(&mut ime.commit);
    ime.cstr(&t)
}

/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_preedit_len(ime: *mut Ime) -> usize {
    (*ime).preedit.len()
}

/// Run `i` of the preedit; `attr` receives 0 input, 1 converted, 2 focus.
///
/// # Safety
/// `ime` is a live session, `i` < preedit_len, `attr` is NULL or writable.
#[no_mangle]
pub unsafe extern "C" fn seokdok_preedit_run(ime: *mut Ime, i: usize, attr: *mut i32) -> *const c_char {
    let ime = &mut *ime;
    let Some(run) = ime.preedit.get(i).cloned() else { return std::ptr::null() };
    if !attr.is_null() {
        *attr = match run.attr {
            Attr::Input => 0,
            Attr::Converted => 1,
            Attr::Focus => 2,
        };
    }
    ime.cstr(&run.text)
}

/// Candidates on the current page; 0 when the window is closed.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_popup_len(ime: *mut Ime) -> usize {
    (*ime).popup.as_ref().map_or(0, |p| p.items.len())
}

/// Candidate `i` on the page; `note` receives its explanation.
///
/// # Safety
/// `ime` is a live session; `note` is NULL or writable.
#[no_mangle]
pub unsafe extern "C" fn seokdok_popup_item(ime: *mut Ime, i: usize, note: *mut *const c_char) -> *const c_char {
    let ime = &mut *ime;
    let Some((text, n)) = ime.popup.as_ref().and_then(|p| p.items.get(i).cloned()) else { return std::ptr::null() };
    if !note.is_null() {
        *note = ime.cstr(&n);
    }
    ime.cstr(&text)
}

/// Selected index on the page, 1-based position, total, and layout.
///
/// # Safety
/// `ime` is a live session; out pointers are NULL or writable.
#[no_mangle]
pub unsafe extern "C" fn seokdok_popup_info(
    ime: *mut Ime,
    selected: *mut usize,
    position: *mut usize,
    total: *mut usize,
    horizontal: *mut bool,
) {
    let Some(p) = (*ime).popup.as_ref() else { return };
    for (ptr, v) in [(selected, p.selected), (position, p.position), (total, p.total)] {
        if !ptr.is_null() {
            *ptr = v;
        }
    }
    if !horizontal.is_null() {
        *horizontal = p.horizontal;
    }
}

/// Read the focused segment's whole list and return its length; 0 while
/// nothing is converted. [`seokdok_list_item`] reads what this call read.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_list_len(ime: *mut Ime) -> usize {
    let ime = &mut *ime;
    ime.list = ime.session.candidate_list();
    ime.list.as_ref().map_or(0, |l| l.items.len())
}

/// Candidate `i` of the list; `reading` receives the short reading to show
/// beside it, which may be empty.
///
/// # Safety
/// `ime` is a live session; `reading` is NULL or writable.
#[no_mangle]
pub unsafe extern "C" fn seokdok_list_item(ime: *mut Ime, i: usize, reading: *mut *const c_char) -> *const c_char {
    let ime = &mut *ime;
    let Some((text, r)) = ime.list.as_ref().and_then(|l| l.items.get(i).cloned()) else { return std::ptr::null() };
    if !reading.is_null() {
        *reading = ime.cstr(&r);
    }
    ime.cstr(&text)
}

/// Index of the selected candidate in the list.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_list_selected(ime: *mut Ime) -> usize {
    (*ime).list.as_ref().map_or(0, |l| l.selected)
}

/// Choose candidate `i` of the list and confirm through the focused segment.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_list_pick(ime: *mut Ime, i: usize) {
    let ime = &mut *ime;
    let fx = ime.session.pick_candidate(i);
    ime.apply(fx);
}

/// 0 釋讀, 1 Hangul, 2 Latin.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_mode(ime: *mut Ime) -> i32 {
    match (*ime).session.mode() {
        Mode::Seokdok => 0,
        Mode::Hangul => 1,
        Mode::Latin => 2,
    }
}

/// Switch to mode 0 釋讀, 1 Hangul or 2 Latin, committing any composition
/// (read it with [`seokdok_take_commit`]). Other values are ignored.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_set_mode(ime: *mut Ime, mode: i32) {
    let ime = &mut *ime;
    let m = match mode {
        0 => Mode::Seokdok,
        1 => Mode::Hangul,
        2 => Mode::Latin,
        _ => return,
    };
    let fx = ime.session.set_mode(m);
    ime.apply(fx);
}

/// The on-screen mode key: 釋 → 混 → 한 → A → 釋. The script is written to
/// config.toml; with no file to write, the key still moves between Korean and
/// Latin. What was being composed is committed (see [`seokdok_take_commit`]).
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_mode_control(ime: *mut Ime) {
    let ime = &mut *ime;
    let stepped = script_cycle::step_on_screen(&mut ime.session, &HOST);
    if let Some(cfg) = &stepped.config {
        ime.give_engine_options(cfg);
    }
    ime.apply(stepped.effects);
}

/// The jamo a 두벌식 key types (`key` is a Latin letter, upper case for the
/// Shift layer), or 0. For labelling on-screen keys.
#[no_mangle]
pub extern "C" fn seokdok_key_jamo(key: u32) -> u32 {
    char::from_u32(key).and_then(seokdok_core::dubeolsik::key_to_jamo).map_or(0, |j| j as u32)
}

/// Whether the dictionaries have finished loading (conversion works before
/// that, but offers only Hangul).
#[no_mangle]
pub extern "C" fn seokdok_ready() -> bool {
    HOST.is_ready()
}

/// Whether the script setting is 韓㐎 專用. The session then reports Hangul
/// for the 釋讀 mode; front ends use this to leave the user's input source
/// as it is.
#[no_mangle]
pub extern "C" fn seokdok_hangul_only() -> bool {
    config::current().conversion.script == Script::HangulOnly
}

/// `conversion.script`: 0 釋讀, 1 國漢文混用, 2 韓㐎 專用.
#[no_mangle]
pub extern "C" fn seokdok_script() -> i32 {
    match config::current().conversion.script {
        Script::Seokdok => 0,
        Script::SinoOnly => 1,
        Script::HangulOnly => 2,
    }
}

/// Whether Space converts (from the settings); when false, only the convert
/// key does.
#[no_mangle]
pub extern "C" fn seokdok_space_converts() -> bool {
    config::current().input.space_converts
}

fn owned(s: String) -> *mut c_char {
    CString::new(s.replace('\0', "")).unwrap().into_raw()
}

/// The settings as JSON (the fields of config.toml, by section). Free the
/// result with [`seokdok_string_free`].
#[no_mangle]
pub extern "C" fn seokdok_config_json() -> *mut c_char {
    owned(serde_json::to_string(&config::reload()).unwrap_or_default())
}

/// Write settings given as JSON to config.toml (missing fields take their
/// defaults) and refresh the cached copy. Sessions pick the change up on
/// their next [`seokdok_sync`]. False when the JSON is invalid or the file
/// could not be written.
///
/// # Safety
/// `json` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn seokdok_config_save_json(json: *const c_char) -> bool {
    if json.is_null() {
        return false;
    }
    let Ok(cfg) = serde_json::from_str::<Config>(&CStr::from_ptr(json).to_string_lossy()) else { return false };
    let saved = cfg.save().is_ok();
    config::reload();
    saved
}

/// The folder holding config.toml and custom.json, or NULL when there is
/// none. Free the result with [`seokdok_string_free`].
#[no_mangle]
pub extern "C" fn seokdok_config_dir() -> *mut c_char {
    Config::user_dir().map_or(std::ptr::null_mut(), |d| owned(d.to_string_lossy().into_owned()))
}

/// Free a string returned by `seokdok_config_json` or `seokdok_config_dir`.
///
/// # Safety
/// `s` is NULL or came from one of those functions and is not used again.
#[no_mangle]
pub unsafe extern "C" fn seokdok_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tests share the process environment and the cached settings.
    static SERIAL: Mutex<()> = Mutex::new(());

    #[test]
    fn typing_and_converting_through_the_c_interface() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        unsafe {
            let ime = seokdok_new(c.as_ptr());
            // Wait for the background load.
            for _ in 0..200 {
                if HOST.is_ready() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            for k in "gksmfdmf".chars() {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
            }
            assert_eq!(seokdok_preedit_len(ime), 1);
            let t = CStr::from_ptr(seokdok_preedit_run(ime, 0, std::ptr::null_mut())).to_str().unwrap().to_string();
            assert_eq!(t, "하늘을");
            assert!(seokdok_key(ime, SK_SPACE, 0, false));
            assert!(seokdok_key(ime, SK_ENTER, 0, false));
            let c = CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string();
            assert_eq!(c, "天을");
            assert_eq!(seokdok_list_len(ime), 0, "nothing is converted any more");
            for k in "dkssud".chars() {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
            }
            assert!(seokdok_key(ime, SK_SPACE, 0, false));
            let n = seokdok_list_len(ime);
            assert!(n > 2);
            let at = |i: usize| {
                let mut r: *const c_char = std::ptr::null();
                let t = seokdok_list_item(ime, i, &mut r);
                (CStr::from_ptr(t).to_str().unwrap().to_string(), CStr::from_ptr(r).to_str().unwrap().to_string())
            };
            let i = (0..n).find(|&i| at(i).0 == "安寧").unwrap();
            assert_eq!(at(i).1, "音", "the mark alone: the reading is what was typed");
            assert_eq!(seokdok_list_selected(ime), 0);
            seokdok_list_pick(ime, i);
            let c = CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string();
            assert_eq!(c, " 安寧", "after the word break the earlier Space conversion owes");
            assert!(seokdok_take_commit(ime).is_null());
            assert!(seokdok_key(ime, SK_TOGGLE, 0, false));
            assert_eq!(seokdok_mode(ime), 2);
            assert!(!seokdok_key(ime, SK_LETTER, 'g' as u32, false));
            seokdok_set_mode(ime, 1);
            assert_eq!(seokdok_mode(ime), 1);
            for k in "gksmf".chars() {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
            }
            seokdok_set_mode(ime, 0);
            let c = CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string();
            assert_eq!(c, "하늘");
            assert_eq!(seokdok_mode(ime), 0);
            assert_eq!(seokdok_preedit_len(ime), 0);
            seokdok_free(ime);
        }
    }

    #[test]
    fn mode_keys_through_the_c_interface() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        let name = |s: &str| CString::new(s).unwrap();
        // The chord writes the script to config.toml.
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-keys-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        config::reload();
        unsafe {
            let ime = seokdok_new(c.as_ptr());
            let down = |n: &str, mods: u32| seokdok_mode_key(ime, name(n).as_ptr(), mods, SK_KEY_DOWN);
            let up = |n: &str| seokdok_mode_key(ime, name(n).as_ptr(), 0, SK_KEY_UP);
            seokdok_set_mode(ime, 0);
            // Keys no list names go on to seokdok_key.
            assert!(!seokdok_mode_key(ime, std::ptr::null(), 0, SK_KEY_DOWN));
            assert!(!down("G", 0));
            assert!(seokdok_key(ime, SK_LETTER, 'g' as u32, false));
            // The toggle commits what is composed.
            assert!(down("Hangul", 0));
            assert_eq!(CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap(), "ㅎ");
            assert_eq!(seokdok_mode(ime), 2);
            // Its repeats are eaten and do nothing.
            assert!(seokdok_mode_key(ime, name("Hangul").as_ptr(), 0, SK_KEY_REPEAT));
            assert_eq!(seokdok_mode(ime), 2);
            assert!(down("Hangul", 0));
            assert_eq!(seokdok_mode(ime), 0);
            // A modifier tap fires on release; a chord with a key does not.
            assert!(!down("LCtrl", 0));
            assert!(!down("LAlt", 0));
            assert!(up("LCtrl"));
            assert_eq!(seokdok_script(), 1);
            assert_eq!(seokdok_mode(ime), 0);
            assert!(!up("LAlt"));
            assert!(!down("LCtrl", 0));
            assert!(!down("C", SK_MOD_CTRL));
            assert!(!up("LCtrl"));
            assert_eq!(seokdok_script(), 1);
            // Nor does one when another key held since before repeats.
            assert!(!down("LCtrl", 0));
            assert!(!down("LAlt", 0));
            assert!(!seokdok_mode_key(ime, name("G").as_ptr(), SK_MOD_CTRL, SK_KEY_REPEAT));
            assert!(!up("LCtrl"));
            assert!(!up("LAlt"));
            assert_eq!(seokdok_script(), 1);
            seokdok_free(ime);
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_hangul_stays_whole_through_the_c_interface() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        let run = |ime: *mut Ime, i: usize| unsafe { CStr::from_ptr(seokdok_preedit_run(ime, i, std::ptr::null_mut())).to_str().unwrap().to_string() };
        let holl = "\u{1112}\u{119E}\u{11AF}";
        unsafe {
            let ime = seokdok_new(c.as_ptr());
            for _ in 0..200 {
                if HOST.is_ready() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            // ᄒᆞᆯ까요 on 두벌식, ㆍ on Shift+K.
            for (k, shift) in [('g', false), ('k', true), ('f', false), ('R', true), ('k', false), ('d', false), ('y', false)] {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, shift));
            }
            assert_eq!(run(ime, 0), format!("{holl}까요"));
            assert!(seokdok_key(ime, SK_CONVERT, 0, false));
            // The whole run is one segment; shrink it down to the old syllable.
            assert!(seokdok_key(ime, SK_LEFT, 0, true));
            assert!(seokdok_key(ime, SK_LEFT, 0, true));
            assert_eq!(seokdok_preedit_len(ime), 2);
            assert!(seokdok_key(ime, SK_BACKSPACE, 0, false));
            assert_eq!(run(ime, 0), format!("{holl}까요"));
            assert!(seokdok_key(ime, SK_ENTER, 0, false));
            let c = CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string();
            assert_eq!(c, format!("{holl}까요"));
            seokdok_free(ime);
        }
    }

    #[test]
    fn hangul_only_setting_never_converts() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-hangul-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        unsafe {
            let json = CString::new(r#"{"conversion":{"script":"hangul_only"}}"#).unwrap();
            assert!(seokdok_config_save_json(json.as_ptr()));
            let ime = seokdok_new(c.as_ptr());
            assert_eq!(seokdok_mode(ime), 1);
            seokdok_set_mode(ime, 0);
            assert_eq!(seokdok_mode(ime), 1);
            for k in "dkssud".chars() {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
            }
            assert!(seokdok_key(ime, SK_SPACE, 0, false));
            let c = CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string();
            assert_eq!(c, "안녕 ");
            assert_eq!(seokdok_popup_len(ime), 0);
            // An arrow key commits the composition and is left to the application.
            for k in "gks".chars() {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
            }
            assert!(!seokdok_key(ime, SK_LEFT, 0, false));
            let c = CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string();
            assert_eq!(c, "한");
            seokdok_free(ime);
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Type `keys`, convert with Space and confirm with Enter.
    unsafe fn converted(ime: *mut Ime, keys: &str) -> String {
        for k in keys.chars() {
            assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
        }
        assert!(seokdok_key(ime, SK_SPACE, 0, false));
        assert!(seokdok_key(ime, SK_ENTER, 0, false));
        CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().trim_start().to_string()
    }

    #[test]
    fn the_chord_rotates_the_script_once_per_press() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-cycle-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        let chord = CString::new("Hangul").unwrap();
        let shift = seokdok_session::mode_keys::MOD_SHIFT as u32;
        let script_in_file = || Config::load_from(&dir.join("config.toml")).conversion.script;
        unsafe {
            let json = CString::new(r#"{"input":{"keys":{"script_cycle":["Shift+Hangul"]}}}"#).unwrap();
            assert!(seokdok_config_save_json(json.as_ptr()));
            let ime = seokdok_new(c.as_ptr());
            while !seokdok_ready() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let plain = converted(ime, "gksmf");
            assert_eq!(seokdok_script(), 0);

            // Into 國漢文混用; a held key goes no further, and the composition commits.
            for k in "gks".chars() {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
            }
            assert!(seokdok_mode_key(ime, chord.as_ptr(), shift, SK_KEY_DOWN));
            // The word break owed by the Space conversion comes first.
            assert_eq!(CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap(), " 한");
            assert!(seokdok_mode_key(ime, chord.as_ptr(), shift, SK_KEY_REPEAT));
            assert!(seokdok_mode_key(ime, chord.as_ptr(), shift, SK_KEY_REPEAT));
            assert_eq!(script_in_file(), Script::SinoOnly);
            assert_eq!(seokdok_script(), 1);
            assert_eq!(seokdok_mode(ime), 0);
            assert_eq!(converted(ime, "gksmf"), "하늘");

            // 韓㐎 專用, and from Latin back to 釋讀 with the first spelling again.
            assert!(seokdok_mode_key(ime, chord.as_ptr(), shift, SK_KEY_DOWN));
            assert_eq!(seokdok_mode(ime), 1);
            seokdok_set_mode(ime, 2);
            assert!(seokdok_mode_key(ime, chord.as_ptr(), shift, SK_KEY_DOWN));
            assert_eq!(script_in_file(), Script::Seokdok);
            assert_eq!(seokdok_mode(ime), 0);
            assert_eq!(converted(ime, "gksmf"), plain);

            // A file that does not parse stops it, and the composition stays.
            std::fs::write(dir.join("config.toml"), "[conversion\n").unwrap();
            for k in "gks".chars() {
                assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
            }
            seokdok_take_commit(ime);
            assert!(seokdok_mode_key(ime, chord.as_ptr(), shift, SK_KEY_DOWN));
            assert_eq!(seokdok_preedit_len(ime), 1);
            assert!(seokdok_take_commit(ime).is_null());
            seokdok_free(ime);
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_mode_control_steps_through_the_scripts_and_latin() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-control-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        unsafe {
            assert!(seokdok_config_save_json(CString::new("{}").unwrap().as_ptr()));
            let ime = seokdok_new(c.as_ptr());
            let mut seen = Vec::new();
            for _ in 0..4 {
                seokdok_mode_control(ime);
                seen.push((seokdok_script(), seokdok_mode(ime)));
            }
            assert_eq!(seen, [(1, 0), (2, 1), (2, 2), (0, 0)]);
            seokdok_free(ime);
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_round_trip_as_json() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-cfg-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        unsafe {
            let d = seokdok_config_dir();
            assert_eq!(CStr::from_ptr(d).to_str().unwrap(), dir.to_str().unwrap());
            seokdok_string_free(d);
            let json = CString::new(r#"{"input":{"space_converts":false},"conversion":{"style":"morpheme"}}"#).unwrap();
            assert!(seokdok_config_save_json(json.as_ptr()));
            assert!(!seokdok_config_save_json(CString::new("{").unwrap().as_ptr()));
            let j = seokdok_config_json();
            let back: Config = serde_json::from_str(CStr::from_ptr(j).to_str().unwrap()).unwrap();
            seokdok_string_free(j);
            assert!(!back.input.space_converts);
            assert_eq!(back.conversion.style, config::StyleOpt::Morpheme);
            assert_eq!(back.conversion.max_candidates, Config::default().conversion.max_candidates);
            assert!(std::fs::read_to_string(dir.join("config.toml")).unwrap().contains("style = \"morpheme\""));
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
