//! Loads the conversion engine from the `dict` directory next to the DLL plus
//! the user's dictionaries, on a background thread. Until the load finishes,
//! the converter returns the Hangul unchanged.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use seokdok_core::custom::Custom;
use seokdok_core::engine::Builder;
use seokdok_core::{Candidate, Engine, Options, Segment};

use crate::session::Converter;

/// Read the dictionaries of `dict_dir` and the user files of `user_dir`
/// (`user.tsv`, `user-sino.tsv`), and build the engine. A user file comes
/// before the shipped file of the same name, so it takes priority.
pub fn load_engine(dict_dir: &Path, user_dir: Option<&Path>, options: Options) -> (Engine, Vec<String>) {
    load_engine_with(dict_dir, user_dir, None, options)
}

/// The customisation exported from seokdok.mkpo.li (`custom.json` in the
/// user directory), if present and readable.
pub fn load_custom(user_dir: Option<&Path>) -> Option<Custom> {
    let text = std::fs::read_to_string(user_dir?.join("custom.json")).ok()?;
    Custom::from_json(&text).ok()
}

/// As [`load_engine`], with the words and phrases of `custom` in front.
/// Files are read and parsed one at a time, so only one file's text is held
/// at once (the iOS keyboard extension has little memory).
pub fn load_engine_with(dict_dir: &Path, user_dir: Option<&Path>, custom: Option<&Custom>, options: Options) -> (Engine, Vec<String>) {
    let (custom_native, custom_sino) = custom.map(Custom::user_tsv).unwrap_or_default();
    let mut b = Builder::new();
    let mut feed = |names: &[&str], custom_text: &str, add: fn(&mut Builder, &str, &str)| {
        for n in names {
            let user_file = n.starts_with("user");
            if user_file && !custom_text.is_empty() {
                add(&mut b, "custom.json", custom_text);
            }
            let dirs = [user_dir.filter(|_| user_file), Some(dict_dir)];
            for d in dirs.into_iter().flatten() {
                if let Ok(text) = std::fs::read_to_string(d.join(n)) {
                    add(&mut b, n, &text);
                }
            }
        }
    };
    feed(Engine::NATIVE_FILES, &custom_native, Builder::native);
    feed(Engine::SINO_FILES, &custom_sino, Builder::sino);
    b.build(options)
}

#[derive(Default)]
pub struct Host {
    engine: RwLock<Option<Arc<Engine>>>,
    /// Rewrite rules from custom.json, applied around the engine.
    custom: RwLock<Option<Arc<Custom>>>,
    /// The options asked for last; a load in progress finishes with these.
    options: RwLock<Option<Options>>,
    generation: AtomicU64,
}

impl Host {
    pub const fn new() -> Self {
        Host { engine: RwLock::new(None), custom: RwLock::new(None), options: RwLock::new(None), generation: AtomicU64::new(0) }
    }

    pub fn is_ready(&self) -> bool {
        self.engine.read().map(|g| g.is_some()).unwrap_or(false)
    }

    /// Whether a load has been requested since process start.
    pub fn started(&self) -> bool {
        self.generation.load(Ordering::SeqCst) > 0
    }

    pub fn set(&self, engine: Engine) {
        if let Ok(mut g) = self.engine.write() {
            *g = Some(Arc::new(engine));
        }
    }

    /// Start (or restart) loading on a background thread. A load that is
    /// superseded by a later one discards its result.
    pub fn start_load(&'static self, dict_dir: std::path::PathBuf, user_dir: Option<std::path::PathBuf>, options: Options) {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        if let Ok(mut wanted) = self.options.write() {
            *wanted = Some(options.clone());
        }
        std::thread::spawn(move || {
            let custom = load_custom(user_dir.as_deref());
            let (mut engine, _errors) = load_engine_with(&dict_dir, user_dir.as_deref(), custom.as_ref(), options);
            if self.generation.load(Ordering::SeqCst) == generation {
                if let Ok(mut g) = self.custom.write() {
                    *g = custom.map(Arc::new);
                }
                // Held across `set`, so `apply_options` cannot fall between.
                let wanted = self.options.read();
                if let Ok(Some(o)) = wanted.as_deref() {
                    engine.options = o.clone();
                }
                self.set(engine);
            }
        });
    }

    /// Give the running engine `options` without reading the dictionaries
    /// again. False when a conversion on another thread holds the engine and
    /// it kept its old options: the caller then starts a load.
    pub fn apply_options(&self, options: &Options) -> bool {
        // Asked at every key, so the usual answer takes no write lock. A load
        // in progress does not count: the running engine serves until it ends.
        let settled = |host: &Self| {
            let wanted = host.options.read().is_ok_and(|w| w.as_ref() == Some(options));
            wanted && host.engine.read().is_ok_and(|e| e.as_ref().is_none_or(|e| e.options == *options))
        };
        if settled(self) {
            return true;
        }
        let Ok(mut wanted) = self.options.write() else { return false };
        *wanted = Some(options.clone());
        let Ok(mut engine) = self.engine.write() else { return false };
        match engine.as_mut() {
            // A load in progress picks the options up when it finishes.
            None => true,
            Some(arc) => match Arc::get_mut(arc) {
                Some(e) => {
                    e.options = options.clone();
                    true
                }
                None => false,
            },
        }
    }
}

impl Converter for Host {
    fn segments(&self, input: &str) -> Vec<Segment> {
        let engine = self.engine.read().ok().and_then(|g| g.clone());
        let custom = self.custom.read().ok().and_then(|g| g.clone());
        match engine {
            Some(e) => {
                let text = custom.as_ref().map(|c| c.rewrite_input(input)).unwrap_or_else(|| input.to_string());
                let mut segs = e.segments(&text);
                if let Some(c) = &custom {
                    for s in &mut segs {
                        s.candidates = c.rewrite_output(std::mem::take(&mut s.candidates));
                    }
                }
                segs
            }
            None => vec![Segment {
                reading: input.to_string(),
                candidates: vec![Candidate { text: input.to_string(), note: String::new() }],
            }],
        }
    }

    fn whole(&self, input: &str) -> Vec<Candidate> {
        let engine = self.engine.read().ok().and_then(|g| g.clone());
        let custom = self.custom.read().ok().and_then(|g| g.clone());
        match engine {
            Some(e) => {
                let text = custom.as_ref().map(|c| c.rewrite_input(input)).unwrap_or_else(|| input.to_string());
                let cands = e.convert(&text);
                match &custom {
                    Some(c) => c.rewrite_output(cands),
                    None => cands,
                }
            }
            None => vec![Candidate { text: input.to_string(), note: String::new() }],
        }
    }
}

/// The engine shared by every text service instance in the process.
pub static HOST: Host = Host::new();

#[cfg(test)]
mod tests {
    use super::*;

    fn dict_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict")
    }

    #[test]
    fn loads_the_shipped_dictionaries() {
        let (engine, _errors) = load_engine(&dict_dir(), None, Options::default());
        assert!(!engine.natives.is_empty());
        assert!(engine.sino_count() > 0);
        let segs = engine.segments("안녕하세요");
        assert!(!segs.is_empty());
    }

    #[test]
    fn user_dictionary_is_loaded() {
        let dir = std::env::temp_dir().join(format!("seokdok-user-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("user.tsv"), "가게\t鋪\t명사\t\tuser\t99\t\n").unwrap();
        let (engine, _errors) = load_engine(&dict_dir(), Some(&dir), Options::default());
        let segs = engine.segments("가게");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(segs.iter().flat_map(|s| &s.candidates).any(|c| c.text == "鋪"));
    }

    #[test]
    fn custom_json_words_and_rules_apply() {
        let dir = std::env::temp_dir().join(format!("seokdok-custom-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("custom.json"),
            r#"{"format":"seokdok-custom","phrases":[{"reading":"풀어읽기모임","text":"釋讀會X"}],"words":[],
              "rules":[{"on":true,"stage":"output","mode":"xform","regex":false,"pattern":"天","replace":"乾"}]}"#,
        )
        .unwrap();
        let custom = load_custom(Some(&dir)).unwrap();
        let (engine, _) = load_engine_with(&dict_dir(), Some(&dir), Some(&custom), Options::default());
        let _ = std::fs::remove_dir_all(&dir);
        let host = Host::new();
        host.set(engine);
        *host.custom.write().unwrap() = Some(Arc::new(custom));
        assert_eq!(host.segments("풀어읽기모임")[0].candidates[0].text, "釋讀會X");
        assert!(host.segments("하늘").iter().flat_map(|s| &s.candidates).all(|c| c.text != "天"));
    }

    #[test]
    fn options_apply_to_a_running_engine() {
        let build = |o: &Options| load_engine(&dict_dir(), None, o.clone()).0;
        let host = Host::new();
        host.set(build(&Options::default()));
        for (o, word) in [
            (Options { sino_only: true, ..Options::default() }, "하늘을 보다"),
            (Options { old_ha: true, ..Options::default() }, "하고"),
            (Options { hada_kun: true, ..Options::default() }, "안녕해요"),
        ] {
            let expected: Vec<String> = build(&o).convert(word).into_iter().map(|c| c.text).collect();
            let whole = |host: &Host| -> Vec<String> { host.whole(word).into_iter().map(|c| c.text).collect() };
            assert_ne!(expected, whole(&host), "{o:?} changes nothing for {word}");
            assert!(host.apply_options(&o));
            assert_eq!(whole(&host), expected);
            assert!(host.apply_options(&Options::default()));
        }
    }

    #[test]
    fn options_reach_the_running_engine_during_a_load() {
        let host = Host::new();
        host.set(load_engine(&dict_dir(), None, Options::default()).0);
        let sino = Options { sino_only: true, ..Options::default() };
        // What `start_load` records while the old engine keeps serving.
        *host.options.write().unwrap() = Some(sino.clone());
        assert!(host.apply_options(&sino));
        assert_eq!(host.engine.read().unwrap().as_ref().unwrap().options, sino);
    }

    #[test]
    fn options_wait_for_a_busy_engine_or_a_load() {
        let host = Host::new();
        let sino = Options { sino_only: true, ..Options::default() };
        // Nothing loaded yet: the options wait for the load.
        assert!(host.apply_options(&sino));
        assert!(host.apply_options(&Options::default()));
        host.set(load_engine(&dict_dir(), None, Options::default()).0);
        // Another thread is converting: the caller is told to start a load.
        let held = host.engine.read().unwrap().clone().unwrap();
        assert!(!host.apply_options(&sino));
        drop(held);
    }

    #[test]
    fn converter_passes_through_before_load() {
        let host = Host::new();
        let segs = host.segments("한글");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].candidates[0].text, "한글");
    }
}
