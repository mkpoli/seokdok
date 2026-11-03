//! Optional trace for diagnosing applications: written to
//! %LOCALAPPDATA%\seokdok\tsf.log only while a file named `debug` exists in
//! that folder, so normal use writes nothing.

use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

fn dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("seokdok"))
}

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| dir().is_some_and(|d| d.join("debug").exists()))
}

pub fn log(msg: impl FnOnce() -> String) {
    if !enabled() {
        return;
    }
    let Some(d) = dir() else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(d.join("tsf.log")) {
        let exe = std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned())).unwrap_or_default();
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let _ = writeln!(f, "{t} {exe} {}", msg());
    }
}
