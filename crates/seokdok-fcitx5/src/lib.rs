//! A C interface over the shared input-method session, for the fcitx5 addon
//! in `addon/seokdok.cpp`.
//!
//! The addon feeds keys in and reads back a view: text to commit, the preedit
//! runs and the candidate page. Strings returned by the getters stay valid
//! until the next call that changes the view.

use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;

use seokdok_session::config::{self, Config, InputMode, Layout};
use seokdok_session::engine_host::HOST;
use seokdok_session::keys::Key;
use seokdok_session::session::{Attr, Effect, Mode, Popup, Run, Session};

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

pub struct Ime {
    session: Session,
    commit: String,
    preedit: Vec<Run>,
    popup: Option<Popup>,
    // C strings handed out, kept alive until the view changes.
    cstrs: Vec<CString>,
    dict_dir: PathBuf,
    options_key: String,
}

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
            commit: String::new(),
            preedit: Vec::new(),
            popup: None,
            cstrs: Vec::new(),
            dict_dir,
            options_key: String::new(),
        };
        ime.sync();
        ime
    }

    /// Re-read the settings; reload the engine when its options changed.
    fn sync(&mut self) {
        let cfg: Config = config::reload();
        self.session.set_space_converts(cfg.input.space_converts);
        self.session.set_horizontal(cfg.input.candidate_layout == Layout::Horizontal);
        self.session.set_old_clusters(cfg.input.old_clusters);
        let key = format!("{:?}", cfg.conversion);
        if key != self.options_key || !HOST.started() {
            self.options_key = key;
            HOST.start_load(self.dict_dir.clone(), Config::user_dir(), cfg.engine_options());
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
    if out.eaten {
        ime.apply(out.effects);
    }
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

/// Whether Alt tapped alone should switch modes (from the settings).
#[no_mangle]
pub extern "C" fn seokdok_alt_toggles() -> bool {
    config::current().input.alt_toggles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_and_converting_through_the_c_interface() {
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
            assert!(seokdok_take_commit(ime).is_null());
            assert!(seokdok_key(ime, SK_TOGGLE, 0, false));
            assert_eq!(seokdok_mode(ime), 2);
            assert!(!seokdok_key(ime, SK_LETTER, 'g' as u32, false));
            seokdok_free(ime);
        }
    }
}
