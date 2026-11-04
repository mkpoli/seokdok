//! Native words the chat types most, with a sourced spelling (tools/chat_gaps.tsv
//! merged into dict/suggested.tsv).

use seokdok_core::{Engine, Options};
use std::path::{Path, PathBuf};

fn dict_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict")
}

fn first(e: &Engine, input: &str) -> String {
    e.convert(input).into_iter().next().map(|c| c.text).unwrap_or_default()
}

#[test]
fn a_word_with_no_good_spelling_gets_the_sourced_one() {
    let (e, _) = Engine::from_dir(&dict_dir(), Options::default()).expect("dict");
    for (word, spelled) in [
        ("그러게요", "然게요"),
        ("아뇨", "否"),
        ("그대", "汝"),
        ("특히", "特히"),
        ("우리말", "我等言"),
        ("어서오세요", "速來세요"),
        ("마땅히", "當히"),
        ("일훔", "名"),
    ] {
        assert_eq!(first(&e, word), spelled, "{word}");
    }
}

#[test]
fn every_row_names_its_source_and_matches_the_merged_file() {
    let root = dict_dir();
    let merged = std::fs::read_to_string(root.join("suggested.tsv")).unwrap();
    let rows = std::fs::read_to_string(root.join("../tools/chat_gaps.tsv")).unwrap();
    for l in rows.lines().filter(|l| !l.starts_with('#') && !l.starts_with("lemma\t")) {
        let f: Vec<&str> = l.split('\t').collect();
        assert_eq!(f.len(), 7, "{l}");
        assert_eq!(f[4], "suggested", "{l}");
        assert!(!f[6].is_empty(), "a source: {l}");
        let row = format!("{}\t{}\t", f[0], f[1]);
        assert!(merged.lines().any(|m| m.starts_with(&row) && m.ends_with(f[6])), "{l}");
    }
}
