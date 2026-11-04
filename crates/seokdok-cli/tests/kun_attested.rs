//! 訓讀 printed in books of 1882-1945 (dict/kun-attested.tsv), with the shipped
//! dictionaries, and the usage file behind them (data/kun-usage.tsv).

use seokdok_core::dict::parse_native;
use seokdok_core::{Engine, Options};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn engine() -> Engine {
    let (e, errors) = Engine::from_dir(&root().join("dict"), Options::default()).expect("dict");
    assert!(!errors.iter().any(|m| m.contains("kun-attested.tsv")), "{errors:?}");
    e
}

fn first(e: &Engine, input: &str) -> String {
    e.convert(input).into_iter().next().map(|c| c.text).unwrap_or_default()
}

fn rows(path: &str, header: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(root().join(path))
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with(header))
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect()
}

#[test]
fn a_word_with_no_good_spelling_takes_the_printed_one() {
    let e = engine();
    // Each rests on a ruby or a parenthesis in the file's notes.
    assert_eq!(first(&e, "우리나라"), "我國");
    assert_eq!(first(&e, "나래"), "翼");
    assert_eq!(first(&e, "사나이"), "男兒");
}

#[test]
fn a_common_word_keeps_its_usual_hanja() {
    let e = engine();
    for (word, usual) in [("가락", "錘"), ("이끌어", "導어"), ("다다라", "到다라"), ("갖고", "갖고"), ("된장이면", "된醬이면"), ("쓰임이", "쓰임이")] {
        assert_eq!(first(&e, word), usual, "{word}");
    }
    assert_eq!(first(&e, "하늘"), "天");
    assert_eq!(first(&e, "사람"), "人");
}

#[test]
fn the_tier_ranks_below_eonhae_and_above_suggested() {
    let cost = |source: &str, count: &str| parse_native(&["낱말", "語", "noun", "", source, count]).unwrap().cost;
    assert!(cost("eonhae", "") < cost("kun-attested", ""));
    assert!(cost("kun-attested", "") < cost("suggested", ""));
    assert_eq!(cost("kun-attested", "137"), cost("eonhae", "137"));
}

#[test]
fn every_row_cites_book_year_and_page() {
    let rows = rows("dict/kun-attested.tsv", "lemma\t");
    assert!(rows.len() > 200);
    for r in &rows {
        assert_eq!(r[4], "kun-attested", "{r:?}");
        assert!(r[6].contains(", ") && r[6].contains("/") && r[6].contains(": "), "book year, page: quote: {r:?}");
        assert!(r[6].chars().any(|c| c.is_ascii_digit()), "a year: {r:?}");
    }
}

#[test]
fn no_row_repeats_a_better_tier() {
    let mut better = std::collections::HashSet::new();
    for f in ["corpus.tsv", "editorial.tsv", "community.tsv", "eonhae.tsv"] {
        for r in rows(&format!("dict/{f}"), "lemma\t") {
            if r.len() > 1 {
                better.insert((r[0].clone(), r[1].clone()));
            }
        }
    }
    for r in rows("dict/kun-attested.tsv", "lemma\t") {
        assert!(!better.contains(&(r[0].clone(), r[1].clone())), "{r:?}");
    }
}

#[test]
fn the_usage_file_stays_out_of_the_apps_and_gives_each_use_a_source() {
    // The iOS keyboard copies dict/ whole, so the usage file must live elsewhere.
    assert!(!root().join("dict/kun-usage.tsv").exists());
    let usage = rows("data/kun-usage.tsv", "hanja\t");
    assert!(usage.len() > 1000);
    let mut seen = std::collections::HashSet::new();
    for r in &usage {
        assert_eq!(r.len(), 11, "{r:?}");
        assert!(r[8].starts_with("페이지:"), "{r:?}");
        assert!(!r[10].is_empty(), "{r:?}");
        assert!(seen.insert((r[0].clone(), r[1].clone(), r[5].clone())), "one row per distinct use: {r:?}");
    }
    for (hanja, reading) in [("軍浦場", "군포내"), ("烏山", "오믜"), ("人", "사람"), ("力", "힘")] {
        assert!(usage.iter().any(|r| r[0] == hanja && r[1] == reading), "{hanja} {reading}");
    }
    // Readings by sound are no part of it.
    assert!(!usage.iter().any(|r| r[0] == "勞動" && r[1] == "로동"));
}
