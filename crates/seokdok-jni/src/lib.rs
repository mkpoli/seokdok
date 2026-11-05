//! JNI bindings for the Android keyboard (android/): the C interface of
//! `seokdok-ffi` wrapped as the static native methods of
//! `li.mkpo.seokdok.Native` (android/app/src/main/java/li/mkpo/seokdok/Native.kt).
//!
//! A session handle is the `Ime` pointer as a `long`. Strings are copied into
//! Java objects before returning, so nothing from the C side outlives a call.

use std::ffi::{c_char, CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr;

use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jint, jintArray, jlong, jobjectArray, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use seokdok_ffi as ffi;
use seokdok_ffi::Ime;
use seokdok_session::config::{self, Config};
use seokdok_session::engine_host::HOST;
use seokdok_session::learned;

fn ime(h: jlong) -> *mut Ime {
    h as *mut Ime
}

fn text(p: *const c_char) -> Option<String> {
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
}

fn jbool(b: bool) -> jboolean {
    if b {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

fn get(env: &mut JNIEnv, s: &JString) -> Option<String> {
    if s.is_null() {
        return None;
    }
    env.get_string(s).ok().map(Into::into)
}

fn new_string(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s).map_or(ptr::null_mut(), |j| j.into_raw())
}

fn string_array(env: &mut JNIEnv, items: &[String]) -> jobjectArray {
    let Ok(arr) = env.new_object_array(items.len() as i32, "java/lang/String", JObject::null()) else {
        return ptr::null_mut();
    };
    for (i, s) in items.iter().enumerate() {
        let Ok(j) = env.new_string(s) else { return ptr::null_mut() };
        if env.set_object_array_element(&arr, i as i32, &j).is_err() {
            return ptr::null_mut();
        }
        let _ = env.delete_local_ref(j);
    }
    arr.into_raw()
}

fn int_array(env: &mut JNIEnv, items: &[i32]) -> jintArray {
    let Ok(arr) = env.new_int_array(items.len() as i32) else { return ptr::null_mut() };
    if env.set_int_array_region(&arr, 0, items).is_err() {
        return ptr::null_mut();
    }
    arr.into_raw()
}

/// The settings in `dir/config.toml` with every default filled in, as TOML.
pub fn config_text(dir: &Path) -> String {
    toml::to_string(&Config::load_from(&dir.join("config.toml"))).unwrap_or_default()
}

/// Parse `text` (missing keys take their defaults) and write it to
/// `dir/config.toml` in the documented form the other front ends write.
pub fn save_config(dir: &Path, text: &str) -> std::io::Result<()> {
    Config::parse(text).save_to(&dir.join("config.toml"))
}

/// Create a session over the dictionaries in `dictDir`, reading config.toml
/// and custom.json from `configDir`.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_create<'l>(
    mut env: JNIEnv<'l>,
    _: JClass<'l>,
    dict_dir: JString<'l>,
    config_dir: JString<'l>,
) -> jlong {
    if let Some(dir) = get(&mut env, &config_dir) {
        // Read by Config::user_dir(); there is no HOME or XDG directory here.
        std::env::set_var("SEOKDOK_CONFIG_DIR", dir);
    }
    let dict = get(&mut env, &dict_dir).and_then(|d| CString::new(d).ok());
    let p = dict.as_ref().map_or(ptr::null(), |c| c.as_ptr());
    unsafe { ffi::seokdok_new(p) as jlong }
}

#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_free(_: JNIEnv, _: JClass, h: jlong) {
    unsafe { ffi::seokdok_free(ime(h)) }
}

/// Re-read the settings (on every new input field).
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_sync(_: JNIEnv, _: JClass, h: jlong) {
    unsafe { ffi::seokdok_sync(ime(h)) }
}

/// The field asks not to be learned from (see `seokdok_set_private_field`).
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_setPrivateField(_: JNIEnv, _: JClass, h: jlong, private_field: jboolean) {
    unsafe { ffi::seokdok_set_private_field(ime(h), private_field != JNI_FALSE) }
}

/// Forget what was picked: `dir/learned.tsv` and this process's table. The
/// settings screen can run before the keyboard has set the config folder, so
/// it names the folder. False when the file could not be deleted.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_forgetLearned<'l>(mut env: JNIEnv<'l>, _: JClass<'l>, dir: JString<'l>) -> jboolean {
    let Some(dir) = get(&mut env, &dir) else { return JNI_FALSE };
    jbool(learned::forget(Path::new(&dir)).is_ok())
}

/// Feed a key (`kind` is one of the SK_* values); true when it was used.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_key(_: JNIEnv, _: JClass, h: jlong, kind: jint, ch: jint, shift: jboolean) -> jboolean {
    jbool(unsafe { ffi::seokdok_key(ime(h), kind, ch as u32, shift != JNI_FALSE) })
}

/// Offer a key to the `[input.keys]` lists (see `seokdok_mode_key`); `name` is
/// null for a key they cannot name. True when it was used.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_modeKey<'l>(mut env: JNIEnv<'l>, _: JClass<'l>, h: jlong, name: JString<'l>, mods: jint, state: jint) -> jboolean {
    let name = get(&mut env, &name).and_then(|n| CString::new(n).ok());
    let p = name.as_ref().map_or(ptr::null(), |c| c.as_ptr());
    jbool(unsafe { ffi::seokdok_mode_key(ime(h), p, mods as u32, state) })
}

#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_flush(_: JNIEnv, _: JClass, h: jlong) {
    unsafe { ffi::seokdok_flush(ime(h)) }
}

#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_reset(_: JNIEnv, _: JClass, h: jlong) {
    unsafe { ffi::seokdok_reset(ime(h)) }
}

/// Text to commit since the last call, or null.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_takeCommit(mut env: JNIEnv, _: JClass, h: jlong) -> jstring {
    match text(unsafe { ffi::seokdok_take_commit(ime(h)) }) {
        Some(t) => new_string(&mut env, &t),
        None => ptr::null_mut(),
    }
}

/// The preedit runs' texts.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_preeditTexts(mut env: JNIEnv, _: JClass, h: jlong) -> jobjectArray {
    let n = unsafe { ffi::seokdok_preedit_len(ime(h)) };
    let runs: Vec<String> = (0..n).map(|i| text(unsafe { ffi::seokdok_preedit_run(ime(h), i, ptr::null_mut()) }).unwrap_or_default()).collect();
    string_array(&mut env, &runs)
}

/// The preedit runs' attributes: 0 input, 1 converted, 2 focus.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_preeditAttrs(mut env: JNIEnv, _: JClass, h: jlong) -> jintArray {
    let n = unsafe { ffi::seokdok_preedit_len(ime(h)) };
    let attrs: Vec<i32> = (0..n)
        .map(|i| {
            let mut a = 0;
            unsafe { ffi::seokdok_preedit_run(ime(h), i, &mut a) };
            a
        })
        .collect();
    int_array(&mut env, &attrs)
}

/// The candidates on the current page as text, note, text, note, …; empty
/// when the list is closed.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_candidates(mut env: JNIEnv, _: JClass, h: jlong) -> jobjectArray {
    let n = unsafe { ffi::seokdok_popup_len(ime(h)) };
    let mut out = Vec::with_capacity(n * 2);
    for i in 0..n {
        let mut note: *const c_char = ptr::null();
        let t = text(unsafe { ffi::seokdok_popup_item(ime(h), i, &mut note) }).unwrap_or_default();
        out.push(t);
        out.push(text(note).unwrap_or_default());
    }
    string_array(&mut env, &out)
}

/// Selected index on the page, 1-based position, total, horizontal (0/1).
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_candidateInfo(mut env: JNIEnv, _: JClass, h: jlong) -> jintArray {
    let (mut selected, mut position, mut total, mut horizontal) = (0usize, 0usize, 0usize, false);
    unsafe { ffi::seokdok_popup_info(ime(h), &mut selected, &mut position, &mut total, &mut horizontal) };
    int_array(&mut env, &[selected as i32, position as i32, total as i32, horizontal as i32])
}

/// 0 釋讀, 1 Hangul, 2 Latin.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_mode(_: JNIEnv, _: JClass, h: jlong) -> jint {
    unsafe { ffi::seokdok_mode(ime(h)) }
}

/// The jamo a 두벌식 key types (upper case for the Shift layer), or 0.
/// The on-screen mode key (see `seokdok_mode_control`).
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_modeControl(_: JNIEnv, _: JClass, h: jlong) {
    unsafe { ffi::seokdok_mode_control(ime(h)) }
}

/// `conversion.script`: 0 釋讀, 1 國漢文混用, 2 韓㐎 專用.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_script(_: JNIEnv, _: JClass) -> jint {
    ffi::seokdok_script()
}

#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_keyJamo(_: JNIEnv, _: JClass, key: jint) -> jint {
    ffi::seokdok_key_jamo(key as u32) as jint
}

/// Whether the dictionaries have finished loading.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_ready(_: JNIEnv, _: JClass) -> jboolean {
    jbool(ffi::seokdok_ready())
}

/// The settings in `dir/config.toml`, every key present, as TOML.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_loadConfig<'l>(mut env: JNIEnv<'l>, _: JClass<'l>, dir: JString<'l>) -> jstring {
    let dir = get(&mut env, &dir).map(PathBuf::from).unwrap_or_default();
    let t = config_text(&dir);
    new_string(&mut env, &t)
}

/// Write `toml` to `dir/config.toml`, documented; false when it failed.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_saveConfig<'l>(
    mut env: JNIEnv<'l>,
    _: JClass<'l>,
    dir: JString<'l>,
    toml: JString<'l>,
) -> jboolean {
    let (Some(dir), Some(t)) = (get(&mut env, &dir), get(&mut env, &toml)) else { return JNI_FALSE };
    jbool(save_config(Path::new(&dir), &t).is_ok())
}

/// Reload the dictionaries and custom.json if the keyboard has loaded them
/// in this process (after custom.json changed); settings changes reload on
/// their own.
#[no_mangle]
pub extern "system" fn Java_li_mkpo_seokdok_Native_reload<'l>(mut env: JNIEnv<'l>, _: JClass<'l>, dict_dir: JString<'l>) {
    let Some(dict) = get(&mut env, &dict_dir) else { return };
    if HOST.started() {
        HOST.start_load(PathBuf::from(dict), Config::user_dir(), config::reload().engine_options());
    }
}

#[cfg(all(test, not(feature = "eumdok")))]
mod tests {
    use super::*;
    use seokdok_session::config::{InputMode, StyleOpt};

    #[test]
    fn config_round_trips_through_the_settings_text() {
        let dir = std::env::temp_dir().join(format!("seokdok-jni-{}", std::process::id()));
        let first = config_text(&dir);
        assert_eq!(Config::parse(&first), Config::default());
        // What the settings screen sends back after two changes.
        let edited = first.replace("default_mode = \"seokdok\"", "default_mode = \"latin\"").replace("style = \"surface\"", "style = \"morpheme\"");
        save_config(&dir, &edited).unwrap();
        let c = Config::load_from(&dir.join("config.toml"));
        assert_eq!(c.input.default_mode, InputMode::Latin);
        assert_eq!(c.conversion.style, StyleOpt::Morpheme);
        assert!(std::fs::read_to_string(dir.join("config.toml")).unwrap().starts_with("# seokdok configuration"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
