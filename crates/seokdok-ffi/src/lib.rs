//! A C interface over the shared input-method session, declared in
//! `include/seokdok.h`: used by the fcitx5 addon (crates/seokdok-fcitx5), the
//! iOS keyboard (ios/), the macOS input method (macos/) and the Windows settings
//! app (windows/SeokdokSettings), which loads it as seokdok_ffi.dll.
//!
//! The addon feeds keys in and reads back a view: text to commit, the preedit
//! runs and the candidate page. Strings returned by the getters stay valid
//! until the next call that changes the view.

use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;
use std::sync::Mutex;

use seokdok_session::config::{self, Config, InputMode, Layout, Script};
use seokdok_core::custom::Custom;
use seokdok_session::engine_host::HOST;
use seokdok_session::keys::Key;
use seokdok_session::learned;
use seokdok_session::mode_keys::{Chord, KeyName, ModeKeyboard};
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
        self.session.set_learned(learned::shared(cfg.input.learn));
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

/// Say whether the field in focus is a password, e-mail, URL or number field,
/// or asks not to be learned from: in one, the picks in `learned.tsv` are
/// neither used nor added to. A front end that keeps such fields from the
/// session need not call this.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_set_private_field(ime: *mut Ime, private_field: bool) {
    (*ime).session.set_private_field(private_field);
}

/// Forget what was picked: delete `learned.tsv` beside config.toml and empty
/// the table in memory. Other processes notice at their next conversion. False
/// when the file could not be deleted.
#[no_mangle]
pub extern "C" fn seokdok_learned_forget() -> bool {
    Config::user_dir().is_none_or(|dir| learned::forget(&dir).is_ok())
}

/// Whether `input.learn` is on.
#[no_mangle]
pub extern "C" fn seokdok_learn() -> bool {
    config::current().input.learn
}

/// Turn `input.learn` on or off in config.toml. Sessions pick the change up on
/// their next [`seokdok_sync`]. False when the file could not be written, or
/// does not parse and was left alone.
#[no_mangle]
pub extern "C" fn seokdok_set_learn(on: bool) -> bool {
    config::update(|c| c.input.learn = on).is_ok()
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

/// Whether the focused segment can shorten or lengthen by a syllable.
///
/// # Safety
/// `ime` is a live session; out pointers are NULL or writable.
#[no_mangle]
pub unsafe extern "C" fn seokdok_list_can_resize(ime: *mut Ime, shorten: *mut bool, lengthen: *mut bool) {
    let Some(l) = (*ime).list.as_ref() else { return };
    if !shorten.is_null() {
        *shorten = l.can_shorten;
    }
    if !lengthen.is_null() {
        *lengthen = l.can_lengthen;
    }
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

/// The settings a new file would hold, as JSON. Free the result with
/// [`seokdok_string_free`].
#[no_mangle]
pub extern "C" fn seokdok_config_defaults_json() -> *mut c_char {
    owned(serde_json::to_string(&Config::default()).unwrap_or_default())
}

/// Why config.toml cannot be changed, or NULL when it can. A file that does
/// not parse reads as the defaults and is left alone, so it has to be fixed by
/// hand first. Free the result with [`seokdok_string_free`].
#[no_mangle]
pub extern "C" fn seokdok_config_error() -> *mut c_char {
    let Some(path) = Config::path() else { return owned("no settings folder".into()) };
    match Config::update_file(&path, |_| {}) {
        Ok(_) => std::ptr::null_mut(),
        Err(e) => owned(e.to_string()),
    }
}

/// `patch` applied to `base` as a JSON Merge Patch (RFC 7386): objects merge,
/// null removes a key, anything else replaces.
fn merge(base: &mut serde_json::Value, patch: &serde_json::Value) {
    use serde_json::Value;
    let Value::Object(fields) = patch else {
        *base = patch.clone();
        return;
    };
    if !base.is_object() {
        *base = Value::Object(Default::default());
    }
    let Value::Object(target) = base else { return };
    for (key, value) in fields {
        if value.is_null() {
            target.remove(key);
        } else {
            merge(target.entry(key.as_str()).or_insert(Value::Null), value);
        }
    }
}

/// Whether `config` holds what `patch` asked for. The reader ignores keys it
/// does not know, so a misnamed key would otherwise vanish without a word.
fn holds(config: &serde_json::Value, patch: &serde_json::Value) -> bool {
    use serde_json::Value;
    let Value::Object(fields) = patch else { return config == patch };
    fields.iter().all(|(key, value)| match config.get(key) {
        Some(found) => holds(found, value),
        None => value.is_null(),
    })
}

/// `config` with `patch` applied.
fn patched(config: &Config, patch: &serde_json::Value) -> Result<Config, String> {
    let mut value = serde_json::to_value(config).map_err(|e| e.to_string())?;
    merge(&mut value, patch);
    let next: Config = serde_json::from_value(value).map_err(|e| e.to_string())?;
    let back = serde_json::to_value(&next).map_err(|e| e.to_string())?;
    if !holds(&back, patch) {
        return Err(format!("not a setting: {patch}"));
    }
    Ok(next)
}

/// Change only what the JSON Merge Patch `patch` names, through
/// [`config::update`], which reads the file at that moment: what another
/// application wrote before then survives, and running applications see the new
/// file at their next key. NULL when done,
/// else the reason, and the file is as it was. Free the result with
/// [`seokdok_string_free`].
///
/// # Safety
/// `patch` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn seokdok_config_update_json(patch: *const c_char) -> *mut c_char {
    if patch.is_null() {
        return owned("no settings given".into());
    }
    let patch: serde_json::Value = match serde_json::from_str(&CStr::from_ptr(patch).to_string_lossy()) {
        Ok(p) => p,
        Err(e) => return owned(e.to_string()),
    };
    let mut refused = None;
    let done = config::update(|c| match patched(c, &patch) {
        Ok(next) => *c = next,
        Err(e) => refused = Some(e),
    });
    match (refused, done) {
        (Some(e), _) => owned(e),
        (None, Err(e)) => owned(e.to_string()),
        (None, Ok(_)) => std::ptr::null_mut(),
    }
}

/// The definition of the product this library was built as: the JSON of
/// `products/<id>.json`. Free the result with [`seokdok_string_free`].
#[no_mangle]
pub extern "C" fn seokdok_product_json() -> *mut c_char {
    owned(seokdok_product::CURRENT.json.to_string())
}

/// Whether `text` is an entry the `[input.keys]` lists understand.
///
/// # Safety
/// `text` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn seokdok_chord_valid(text: *const c_char) -> bool {
    !text.is_null() && CStr::from_ptr(text).to_str().ok().and_then(Chord::parse).is_some()
}

/// The text of custom.json, or NULL when there is none. Free the result with
/// [`seokdok_string_free`].
#[no_mangle]
pub extern "C" fn seokdok_custom_json() -> *mut c_char {
    let text = Config::user_dir().and_then(|d| std::fs::read_to_string(d.join("custom.json")).ok());
    text.map_or(std::ptr::null_mut(), owned)
}

/// Replace custom.json with `json`, a `seokdok-custom` document. NULL when
/// done, else the reason, and the file is as it was. Free the result with
/// [`seokdok_string_free`].
///
/// # Safety
/// `json` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn seokdok_custom_save_json(json: *const c_char) -> *mut c_char {
    if json.is_null() {
        return owned("no document given".into());
    }
    let text = CStr::from_ptr(json).to_string_lossy();
    let custom = match Custom::from_json(&text) {
        Ok(c) => c,
        Err(e) => return owned(e),
    };
    if let Some((i, e)) = custom.errors.first() {
        return owned(format!("rule {}: {e}", i + 1));
    }
    let Some(dir) = Config::user_dir() else { return owned("no settings folder".into()) };
    let path = dir.join("custom.json");
    let tmp = dir.join(format!("custom.json.{}.tmp", std::process::id()));
    let written = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&tmp, text.as_bytes())).and_then(|()| std::fs::rename(&tmp, &path));
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written.err().map_or(std::ptr::null_mut(), |e| owned(e.to_string()))
}

/// Read the dictionaries and custom.json again, in the background.
///
/// # Safety
/// `ime` is a live session.
#[no_mangle]
pub unsafe extern "C" fn seokdok_reload(ime: *mut Ime) {
    let ime = &*ime;
    HOST.start_load(ime.dict_dir.clone(), Config::user_dir(), config::current().engine_options());
}

/// Free a string returned by a `seokdok_config_*` or `seokdok_custom_*` function.
///
/// # Safety
/// `s` is NULL or came from one of those functions and is not used again.
#[no_mangle]
pub unsafe extern "C" fn seokdok_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// 音讀 through the C interface: `cargo test -p seokdok-ffi --features eumdok`.
#[cfg(all(test, feature = "eumdok"))]
mod eumdok_tests {
    use super::*;

    unsafe fn convert(ime: *mut Ime, keys: &str) -> String {
        for k in keys.chars() {
            assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
        }
        assert!(seokdok_key(ime, SK_SPACE, 0, false));
        assert!(seokdok_key(ime, SK_ENTER, 0, false));
        CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string()
    }

    #[test]
    fn sound_readings_only_and_two_scripts() {
        let dir = std::env::temp_dir().join(format!("eumdok-ffi-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        config::reload();
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        unsafe {
            let ime = seokdok_new(c.as_ptr());
            while !seokdok_ready() {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            assert_eq!(convert(ime, "rhdqn"), "工夫");
            assert_eq!(convert(ime, "gksmf").trim(), "하늘");
            assert_eq!(convert(ime, "gkrh").trim(), "하고");
            let mut seen = Vec::new();
            for _ in 0..4 {
                seokdok_mode_control(ime);
                seen.push((seokdok_script(), seokdok_mode(ime)));
            }
            assert_eq!(seen, [(2, 1), (2, 2), (1, 0), (2, 1)]);
            seokdok_free(ime);
            let json = CStr::from_ptr(seokdok_product_json()).to_str().unwrap();
            assert!(json.contains("\"id\": \"eumdok\""));
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(all(test, not(feature = "eumdok")))]
mod tests {
    use super::*;

    /// The tests share the process environment and the cached settings.
    static SERIAL: Mutex<()> = Mutex::new(());

    /// Point the settings and `learned.tsv` at a new temporary folder, for as
    /// long as this lives.
    struct TempConfig(PathBuf);

    impl TempConfig {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("seokdok-ffi-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
            config::reload();
            TempConfig(dir)
        }
    }

    impl Drop for TempConfig {
        fn drop(&mut self) {
            std::env::remove_var("SEOKDOK_CONFIG_DIR");
            config::reload();
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Type `keys`, convert, step `steps` candidates on and confirm; the text.
    unsafe fn pick_nth(ime: *mut Ime, keys: &str, steps: usize) -> String {
        for k in keys.chars() {
            assert!(seokdok_key(ime, SK_LETTER, k as u32, false));
        }
        assert!(seokdok_key(ime, SK_SPACE, 0, false));
        for _ in 0..steps {
            assert!(seokdok_key(ime, SK_DOWN, 0, false));
        }
        assert!(seokdok_key(ime, SK_ENTER, 0, false));
        let text = CStr::from_ptr(seokdok_take_commit(ime)).to_str().unwrap().to_string();
        seokdok_reset(ime);
        text.trim_start().to_string()
    }

    fn wait_for_engine() {
        for _ in 0..400 {
            if HOST.is_ready() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        panic!("the engine did not load");
    }

    #[test]
    fn picks_are_learned_forgotten_and_kept_out_of_private_fields() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let temp = TempConfig::new("learn");
        let dict = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let c = CString::new(dict.to_str().unwrap()).unwrap();
        unsafe {
            let ime = seokdok_new(c.as_ptr());
            wait_for_engine();
            // 나: the first candidate, then two picks of the third form.
            let first = pick_nth(ime, "sk", 0);
            let other = pick_nth(ime, "sk", 2);
            assert_ne!(first, other);

            // In a private field nothing counts, and nothing shows.
            seokdok_set_private_field(ime, true);
            assert_eq!(pick_nth(ime, "sk", 2), other);
            assert_eq!(pick_nth(ime, "sk", 0), first);
            seokdok_set_private_field(ime, false);

            // Taking the usual form outside one clears the earlier pick.
            assert_eq!(pick_nth(ime, "sk", 0), first);
            pick_nth(ime, "sk", 2);
            assert_eq!(pick_nth(ime, "sk", 0), first, "one pick put it first");
            pick_nth(ime, "sk", 2);
            assert_eq!(pick_nth(ime, "sk", 0), first, "the usual form cleared that pick too");
            pick_nth(ime, "sk", 2);
            pick_nth(ime, "sk", 2);
            assert_eq!(pick_nth(ime, "sk", 0), other, "two picks put it first");

            // A session in another field shares the table; the file is on disk.
            let second = seokdok_new(c.as_ptr());
            assert_eq!(pick_nth(second, "sk", 0), other);
            let _ = seokdok_session::learned::shared(true).unwrap().save_now();
            let path = temp.0.join(learned::FILE_NAME);
            assert!(std::fs::read_to_string(&path).unwrap().contains(&other));

            assert!(seokdok_learned_forget());
            assert!(!path.exists());
            assert_eq!(pick_nth(ime, "sk", 0), first);

            // Off: nothing is used or added.
            let json = CString::new(r#"{"input":{"learn":false}}"#).unwrap();
            assert!(seokdok_config_save_json(json.as_ptr()));
            seokdok_sync(ime);
            pick_nth(ime, "sk", 2);
            pick_nth(ime, "sk", 2);
            assert_eq!(pick_nth(ime, "sk", 0), first);
            assert!(!path.exists());
            seokdok_free(second);
            seokdok_free(ime);
        }
    }

    #[test]
    fn typing_and_converting_through_the_c_interface() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let _temp = TempConfig::new("typing");
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
            let (mut shorter, mut longer) = (false, true);
            seokdok_list_can_resize(ime, &mut shorter, &mut longer);
            assert!(shorter, "two syllables");
            assert!(!longer, "nothing follows");
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
        let _temp = TempConfig::new("old");
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

    #[test]
    fn a_patch_changes_only_what_it_names() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-patch-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        let update = |patch: &str| unsafe {
            let e = seokdok_config_update_json(CString::new(patch).unwrap().as_ptr());
            let message = (!e.is_null()).then(|| CStr::from_ptr(e).to_str().unwrap().to_string());
            seokdok_string_free(e);
            message
        };
        let path = dir.join("config.toml");
        assert_eq!(update(r#"{"conversion":{"old_ha":true}}"#), None);
        // The rotation key writes the script while the window is open, and the
        // window then changes something else: the script stays as rotated.
        Config::update_file(&path, |c| c.conversion.script = Script::SinoOnly).unwrap();
        assert_eq!(update(r#"{"input":{"keys":{"convert":["F9"]},"old_clusters":true}}"#), None);
        assert_eq!(update(r#"{"composition":{"a.exe":"floating","b.exe":"inline"}}"#), None);
        assert_eq!(update(r#"{"composition":{"a.exe":null}}"#), None);
        let c = Config::load_from(&path);
        assert!(c.conversion.old_ha && c.input.old_clusters);
        assert_eq!(c.conversion.script, Script::SinoOnly);
        assert_eq!(c.input.keys.convert, ["F9"]);
        assert_eq!(c.input.keys.toggle, Config::default().input.keys.toggle);
        assert_eq!(c.composition.keys().collect::<Vec<_>>(), ["b.exe"]);
        // Nothing is written for a key the settings do not have, a bad value or bad JSON.
        let before = std::fs::read_to_string(&path).unwrap();
        assert!(update(r#"{"input":{"keys":{"no_such_list":["F9"]}}}"#).unwrap().contains("not a setting"));
        assert!(update(r#"{"conversion":{"script":"klingon"}}"#).is_some());
        assert!(update("{").is_some());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
        unsafe {
            assert!(seokdok_config_error().is_null());
            // A broken file is reported and kept.
            let broken = "[conversion]\nmax_candidates = \"x\"\n";
            std::fs::write(&path, broken).unwrap();
            let e = seokdok_config_error();
            assert!(!e.is_null());
            seokdok_string_free(e);
            assert!(update(r#"{"conversion":{"old_ha":false}}"#).is_some());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), broken);
            let d = seokdok_config_defaults_json();
            let defaults: Config = serde_json::from_str(CStr::from_ptr(d).to_str().unwrap()).unwrap();
            seokdok_string_free(d);
            assert_eq!(defaults, Config::default());
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn chords_are_checked_by_the_session() {
        let valid = |s: &str| unsafe { seokdok_chord_valid(CString::new(s).unwrap().as_ptr()) };
        for good in ["Hangul", "Shift+Hangul", "LCtrl+LAlt", "RAlt", "F24", "Ctrl+1"] {
            assert!(valid(good), "{good}");
        }
        for bad in ["", "한/영", "F25", "Hangul+Hanja"] {
            assert!(!valid(bad), "{bad}");
        }
    }

    #[test]
    fn custom_json_is_checked_before_it_is_written() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-custom-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        let save = |text: &str| unsafe {
            let e = seokdok_custom_save_json(CString::new(text).unwrap().as_ptr());
            let message = (!e.is_null()).then(|| CStr::from_ptr(e).to_str().unwrap().to_string());
            seokdok_string_free(e);
            message
        };
        let read = || unsafe {
            let p = seokdok_custom_json();
            let text = (!p.is_null()).then(|| CStr::from_ptr(p).to_str().unwrap().to_string());
            seokdok_string_free(p);
            text
        };
        assert_eq!(read(), None);
        let good = r#"{"format":"seokdok-custom","version":1,"phrases":[{"reading":"풀어읽기모임","text":"釋讀會"}],"words":[],"rules":[]}"#;
        assert_eq!(save(good), None);
        assert_eq!(read().as_deref(), Some(good));
        assert!(save(r#"{"format":"other"}"#).is_some());
        assert!(save(r#"{"format":"seokdok-custom","rules":[{"regex":true,"pattern":"("}]}"#).unwrap().starts_with("rule 1"));
        assert!(save("{").is_some());
        assert_eq!(read().as_deref(), Some(good));
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// What the settings app does to preview a setting: `trigger` converts,
    /// Enter commits, and in Latin the keys come out as themselves.
    unsafe fn type_sample(ime: *mut Ime, keys: &str, mode: i32, trigger: i32) -> String {
        seokdok_reset(ime);
        seokdok_set_mode(ime, mode);
        for k in keys.chars() {
            if k == ' ' {
                seokdok_key(ime, SK_SPACE, ' ' as u32, true);
            } else {
                seokdok_key(ime, SK_LETTER, k as u32, k.is_ascii_uppercase());
            }
        }
        seokdok_key(ime, trigger, if trigger == SK_SPACE { ' ' as u32 } else { 0 }, false);
        seokdok_key(ime, SK_ENTER, 0, false);
        let commit = seokdok_take_commit(ime);
        if commit.is_null() {
            return if mode == 2 { keys.to_string() } else { String::new() };
        }
        CStr::from_ptr(commit).to_str().unwrap().to_string()
    }

    /// The examples the settings app shows are what the engine writes.
    #[test]
    fn the_settings_examples_come_from_the_engine() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("seokdok-ffi-examples-{}", std::process::id()));
        std::env::set_var("SEOKDOK_CONFIG_DIR", &dir);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let table = std::fs::read_to_string(root.join("windows/SeokdokSettings/examples.tsv")).unwrap();
        let c = CString::new(root.join("dict").to_str().unwrap()).unwrap();
        unsafe {
            let ime = seokdok_new(c.as_ptr());
            for _ in 0..400 {
                if HOST.is_ready() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            let mut wrong = Vec::new();
            for row in table.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
                let [id, patch, keys, expected] = row.split('\t').collect::<Vec<_>>()[..] else { panic!("{row}") };
                let _ = std::fs::remove_file(dir.join("config.toml"));
                let e = seokdok_config_update_json(CString::new(patch).unwrap().as_ptr());
                assert!(e.is_null(), "{id}");
                seokdok_sync(ime);
                let mode = mode_of(config::current().input.default_mode);
                let mode = match mode {
                    Mode::Seokdok => 0,
                    Mode::Hangul => 1,
                    Mode::Latin => 2,
                };
                let trigger = if id.starts_with("space.") { SK_SPACE } else { SK_CONVERT };
                let got = type_sample(ime, keys, mode, trigger);
                if got != expected.replace('␣', " ") {
                    wrong.push(format!("{id}: {keys} gave {got}, table says {expected}"));
                }
            }
            seokdok_free(ime);
            assert!(wrong.is_empty(), "{}", wrong.join("\n"));
        }
        std::env::remove_var("SEOKDOK_CONFIG_DIR");
        config::reload();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
