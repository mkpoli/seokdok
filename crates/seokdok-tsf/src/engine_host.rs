//! Loads the conversion engine from the `dict` directory next to the DLL plus
//! the user's dictionaries, on a background thread. Until the load finishes,
//! the converter returns the Hangul unchanged.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use seokdok_core::{Candidate, Engine, Options, Segment};

use crate::session::Converter;

/// Read the dictionaries of `dict_dir` and the user files of `user_dir`
/// (`user.tsv`, `user-sino.tsv`), and build the engine. A user file comes
/// before the shipped file of the same name, so it takes priority.
pub fn load_engine(dict_dir: &Path, user_dir: Option<&Path>, options: Options) -> (Engine, Vec<String>) {
    let read = |names: &[&str]| -> Vec<(String, String)> {
        let mut v = Vec::new();
        for n in names {
            let user_file = n.starts_with("user");
            let dirs = [user_dir.filter(|_| user_file), Some(dict_dir)];
            for d in dirs.into_iter().flatten() {
                if let Ok(text) = std::fs::read_to_string(d.join(n)) {
                    v.push((n.to_string(), text));
                }
            }
        }
        v
    };
    let nat = read(Engine::NATIVE_FILES);
    let sin = read(Engine::SINO_FILES);
    let nat_ref: Vec<(&str, &str)> = nat.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let sin_ref: Vec<(&str, &str)> = sin.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    Engine::new(&nat_ref, &sin_ref, options)
}

#[derive(Default)]
pub struct Host {
    engine: RwLock<Option<Arc<Engine>>>,
    generation: AtomicU64,
}

impl Host {
    pub const fn new() -> Self {
        Host { engine: RwLock::new(None), generation: AtomicU64::new(0) }
    }

    pub fn is_ready(&self) -> bool {
        self.engine.read().map(|g| g.is_some()).unwrap_or(false)
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
            let (engine, _errors) = load_engine(&dict_dir, user_dir.as_deref(), options);
            if self.generation.load(Ordering::SeqCst) == generation {
                self.set(engine);
            }
        });
    }
}

impl Converter for Host {
    fn segments(&self, input: &str) -> Vec<Segment> {
        let engine = self.engine.read().ok().and_then(|g| g.clone());
        match engine {
            Some(e) => e.segments(input),
            None => vec![Segment {
                reading: input.to_string(),
                candidates: vec![Candidate { text: input.to_string(), note: String::new() }],
            }],
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
        assert!(!engine.sinos.is_empty());
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
    fn converter_passes_through_before_load() {
        let host = Host::new();
        let segs = host.segments("한글");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].candidates[0].text, "한글");
    }
}
