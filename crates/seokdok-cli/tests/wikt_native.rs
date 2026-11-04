//! Common native words from English Wiktionary (dict/wikt-native.tsv), with the
//! shipped dictionaries.

use seokdok_core::{Engine, Options};
use std::path::{Path, PathBuf};

fn dict_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict")
}

fn first(e: &Engine, input: &str) -> String {
    e.convert(input).into_iter().next().map(|c| c.text).unwrap_or_default()
}

fn rows() -> Vec<Vec<String>> {
    std::fs::read_to_string(dict_dir().join("wikt-native.tsv"))
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("lemma\t"))
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect()
}

#[test]
fn a_common_native_word_leaves_hangul() {
    let (e, errors) = Engine::from_dir(&dict_dir(), Options::default()).expect("dict");
    assert!(!errors.iter().any(|m| m.contains("wikt-native.tsv")), "{errors:?}");
    for (word, spelled) in [
        ("눈앞", "目前"),
        ("여름철", "夏철"),
        ("밤낮", "夜晝"),
        ("새해", "新年"),
        ("앞쪽", "前쪽"),
        ("눈보라", "雪보라"),
        ("신발", "靴"),
        ("뜻밖", "意外"),
    ] {
        assert_eq!(first(&e, word), spelled, "{word}");
    }
}

#[test]
fn a_word_that_is_mostly_something_else_stays_as_it_was() {
    let (e, _) = Engine::from_dir(&dict_dir(), Options::default()).expect("dict");
    for word in ["들이", "지라", "이랑"] {
        assert_eq!(first(&e, word), word, "{word}");
    }
}

#[test]
fn every_row_names_its_source_and_ranks_as_suggested() {
    let rows = rows();
    assert!(rows.len() > 150);
    for r in &rows {
        assert_eq!(r.len(), 7, "{r:?}");
        assert_eq!(r[4], "suggested", "{r:?}");
        assert!(!r[6].is_empty(), "a source: {r:?}");
        assert!(r[5].is_empty() || r[5].parse::<u32>().is_ok(), "{r:?}");
    }
}

#[test]
fn no_row_repeats_a_better_tier() {
    let mut better = std::collections::HashSet::new();
    for f in ["corpus.tsv", "editorial.tsv", "community.tsv", "eonhae.tsv", "kun-attested.tsv", "suggested.tsv"] {
        for l in std::fs::read_to_string(dict_dir().join(f)).unwrap().lines() {
            let mut p = l.split('\t');
            if let (Some(a), Some(b)) = (p.next(), p.next()) {
                better.insert((a.to_string(), b.to_string()));
            }
        }
    }
    for r in rows() {
        assert!(!better.contains(&(r[0].clone(), r[1].clone())), "{r:?}");
    }
}
