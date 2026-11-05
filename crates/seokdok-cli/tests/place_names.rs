//! Native place names from Japanese Wikipedia's list (dict/place-names.tsv).

use seokdok_core::dict::parse_native;
use seokdok_core::{Engine, Options};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn engine() -> Engine {
    let (e, errors) = Engine::from_dir(&root().join("dict"), Options::default()).expect("dict");
    assert!(!errors.iter().any(|m| m.contains("place-names.tsv")), "{errors:?}");
    e
}

fn first(e: &Engine, input: &str) -> String {
    e.convert(input).into_iter().next().map(|c| c.text).unwrap_or_default()
}

#[test]
fn a_name_with_no_other_spelling_takes_the_listed_one() {
    let e = engine();
    for (name, hanja) in [("진고개", "泥峴"), ("살곶다리", "箭串橋"), ("다락원", "樓院"), ("말섬", "馬島")] {
        assert_eq!(first(&e, name), hanja, "{name}");
    }
}

#[test]
fn an_ordinary_word_keeps_its_spelling() {
    let e = engine();
    for (word, usual) in [("서울", "京"), ("돌다리", "矼"), ("모래섬", "陼"), ("삼거리", "三거리"), ("한밭", "한밭")] {
        assert_eq!(first(&e, word), usual, "{word}");
    }
}

#[test]
fn the_tier_ranks_below_suggested() {
    let cost = |source: &str| parse_native(&["낱말", "語", "noun", "", source, ""]).unwrap().cost;
    assert!(cost("kun-attested") < cost("suggested"));
    assert!(cost("suggested") < cost("place-name"));
}

#[test]
fn every_row_cites_the_revision_and_quotes_the_romanization() {
    let text = std::fs::read_to_string(root().join("dict/place-names.tsv")).unwrap();
    let rows: Vec<Vec<&str>> = text.lines().filter(|l| !l.starts_with('#') && !l.starts_with("lemma\t")).map(|l| l.split('\t').collect()).collect();
    assert!(rows.len() > 200);
    for r in &rows {
        assert_eq!(r[4], "place-name", "{r:?}");
        assert!(r[6].contains("rev111083341 (secondary, Wikipedia)"), "{r:?}");
        // The note quotes the page's own form; the row writes it traditionally.
        assert!(r[6].contains("小藤・金澤: "), "{r:?}");
    }
}

#[test]
fn listed_hanja_are_still_read_by_sound() {
    let e = engine();
    for (hanja, sound) in [("高亭", "고정"), ("松洞", "송동"), ("炭洞", "탄동"), ("新院", "신원")] {
        assert_eq!(e.read(hanja), sound, "{hanja}");
    }
}
