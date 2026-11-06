//! Well-known people's names and their Hanja (dict/people.tsv).

use seokdok_core::{Engine, Options};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn engine() -> Engine {
    let (e, errors) = Engine::from_dir(&root().join("dict"), Options::default()).expect("dict");
    assert!(!errors.iter().any(|m| m.contains("people.tsv")), "{errors:?}");
    e
}

fn first(e: &Engine, input: &str) -> String {
    e.convert(input).into_iter().next().map(|c| c.text).unwrap_or_default()
}

#[test]
fn a_full_name_converts_whole() {
    let e = engine();
    for (name, hanja) in [
        ("김정은", "金正恩"),
        ("김일성", "金日成"),
        ("문재인", "文在寅"),
        ("노무현", "盧武鉉"),
        ("이세돌", "李世乭"),
        ("김정은동지", "金正恩同志"),
        ("김일성이", "金日成이"),
        ("손흥민과", "孫興慜과"),
    ] {
        assert_eq!(first(&e, name), hanja, "{name}");
    }
}

#[test]
fn a_name_stays_out_of_other_words() {
    let e = engine();
    for (word, unwanted) in [("아이소라", "李素羅"), ("마주앉다", "馬周"), ("제미나이인", "米娜"), ("이제", "李")] {
        assert!(!first(&e, word).contains(unwanted), "{word}: {}", first(&e, word));
    }
}

#[test]
fn a_two_syllable_name_follows_the_common_word() {
    let e = engine();
    assert_eq!(first(&e, "장면"), "場面");
    assert_eq!(first(&e, "고건"), "고건");
    assert!(e.convert("장면").iter().any(|c| c.text == "張勉"));
}

#[test]
fn a_word_of_the_same_reading_comes_first() {
    let e = engine();
    for (word, usual, name) in [("저수량", "貯水量", "褚遂良"), ("유용성", "有用性", "柳鏞成"), ("한학자", "漢學者", "韓鶴子")] {
        assert_eq!(first(&e, word), usual, "{word}");
        assert!(e.convert(word).iter().any(|c| c.text == name), "{word}: {name} not offered");
    }
}

#[test]
fn a_two_syllable_name_may_take_a_title() {
    let e = engine();
    assert!(e.convert("원빈씨").iter().any(|c| c.text.starts_with("元斌")));
}

#[test]
fn hanja_come_from_the_name_and_not_from_a_note_above_it() {
    let text = std::fs::read_to_string(root().join("dict/people.tsv")).unwrap();
    for row in ["성혼\t成婚", "손흠\t孫欽", "이구지\t離垢地", "전옥\t典獄", "유소연\t有所緣", "마조도일\t馬祖導一"] {
        assert!(!text.contains(row), "{row}");
    }
    assert!(text.contains("성혼\t成渾"));
}

#[test]
fn every_row_cites_its_item_and_revision() {
    let text = std::fs::read_to_string(root().join("dict/people.tsv")).unwrap();
    let rows: Vec<_> = text.lines().filter(|l| !l.starts_with('#') && !l.starts_with("reading\t")).collect();
    assert!(rows.len() > 4000, "{}", rows.len());
    for row in rows {
        let r: Vec<_> = row.split('\t').collect();
        assert_eq!(r[3], "person-name", "{row}");
        assert!(r[4].starts_with('Q') && r[4].contains(" ko:"), "{row}");
    }
}
