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
/// Files are read a line at a time, so no file's text is held whole (the iOS
/// keyboard extension has little memory). A file that cannot be read adds the
/// lines before the failure, if any.
pub fn load_engine_with(dict_dir: &Path, user_dir: Option<&Path>, custom: Option<&Custom>, options: Options) -> (Engine, Vec<String>) {
    let (custom_native, custom_sino) = custom.map(Custom::user_tsv).unwrap_or_default();
    let mut b = Builder::new();
    type AddFile = fn(&mut Builder, &str, &Path) -> std::io::Result<()>;
    let mut feed = |names: &[&str], custom_text: &str, add: fn(&mut Builder, &str, &str), add_file: AddFile| {
        for n in names {
            let user_file = n.starts_with("user");
            if user_file && !custom_text.is_empty() {
                add(&mut b, "custom.json", custom_text);
            }
            let dirs = [user_dir.filter(|_| user_file), Some(dict_dir)];
            for d in dirs.into_iter().flatten() {
                // A missing file is the usual case for the user's dictionaries.
                let _ = add_file(&mut b, n, &d.join(n));
            }
        }
    };
    feed(Engine::NATIVE_FILES, &custom_native, Builder::native, Builder::native_file);
    feed(Engine::SINO_FILES, &custom_sino, Builder::sino, Builder::sino_file);
    b.build(options)
}

#[derive(Default)]
pub struct Host {
    engine: RwLock<Option<Arc<Engine>>>,
    /// Rewrite rules from custom.json, applied around the engine.
    custom: RwLock<Option<Arc<Custom>>>,
    generation: AtomicU64,
}

impl Host {
    pub const fn new() -> Self {
        Host { engine: RwLock::new(None), custom: RwLock::new(None), generation: AtomicU64::new(0) }
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
        std::thread::spawn(move || {
            let custom = load_custom(user_dir.as_deref());
            let (engine, _errors) = load_engine_with(&dict_dir, user_dir.as_deref(), custom.as_ref(), options);
            if self.generation.load(Ordering::SeqCst) == generation {
                if let Ok(mut g) = self.custom.write() {
                    *g = custom.map(Arc::new);
                }
                self.set(engine);
            }
        });
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
    fn converter_passes_through_before_load() {
        let host = Host::new();
        let segs = host.segments("한글");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].candidates[0].text, "한글");
    }
}
