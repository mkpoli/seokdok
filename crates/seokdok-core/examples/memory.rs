//! Resident memory before and after loading the dictionaries (Linux):
//!   cargo run --release -p seokdok-core --example memory -- dict
fn rss() -> String {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .filter(|l| l.starts_with("VmRSS") || l.starts_with("VmHWM"))
        .collect::<Vec<_>>()
        .join("  ")
}

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "dict".into());
    println!("start   {}", rss());
    let (engine, _) = seokdok_core::engine::Engine::from_dir(std::path::Path::new(&dir), Default::default()).expect("load");
    println!("loaded  {}", rss());
    extern "C" {
        fn malloc_trim(pad: usize) -> i32;
    }
    // SAFETY: glibc's malloc_trim only returns free memory to the system.
    unsafe { malloc_trim(0) };
    println!("trimmed {}", rss());
    let c = engine.convert("하늘을");
    println!("convert {} ({})", rss(), c[0].text);
}
