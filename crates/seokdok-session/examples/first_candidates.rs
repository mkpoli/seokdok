//! The first candidates of the product being built, for comparing builds:
//!   cargo run --release -p seokdok-session --example first_candidates [--features eumdok] -- dict < tokens.txt
//! One line per input line: the input, the first whole-phrase candidate and the
//! segments' first candidates. On stderr: how many candidates anywhere in the
//! lists carry a 訓讀 reading.
use seokdok_session::config::Config;
use seokdok_session::engine_host::load_engine;
use std::io::{BufRead, Write};

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "dict".into());
    let (engine, _) = load_engine(std::path::Path::new(&dir), None, Config::default().engine_options());
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut kun = 0;
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let whole = engine.convert(&line);
        let segments = engine.segments(&line);
        kun += whole.iter().chain(segments.iter().flat_map(|s| &s.candidates)).filter(|c| c.note.contains("· 訓讀")).count();
        let first: String = segments.iter().filter_map(|s| s.candidates.first()).map(|c| c.text.as_str()).collect();
        writeln!(out, "{line}\t{}\t{first}", whole.first().map_or("", |c| c.text.as_str())).unwrap();
    }
    eprintln!("{kun} candidates carry a 訓讀 reading");
}
