//! 訓讀 readings attested in historical books (dict/eonhae.tsv), with the
//! shipped dictionaries.

use seokdok_core::dict::parse_native;
use seokdok_core::{Engine, Options};
use std::path::{Path, PathBuf};

fn dict_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict")
}

fn engine() -> Engine {
    let (e, errors) = Engine::from_dir(&dict_dir(), Options::default()).expect("dict");
    assert!(!errors.iter().any(|m| m.contains("eonhae.tsv")), "{errors:?}");
    e
}

fn candidates(e: &Engine, input: &str) -> Vec<String> {
    e.convert(input).into_iter().map(|c| c.text).collect()
}

fn rows() -> Vec<Vec<String>> {
    std::fs::read_to_string(dict_dir().join("eonhae.tsv"))
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("lemma\t"))
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect()
}

#[test]
fn mungchi_keeps_its_cited_spelling() {
    let e = engine();
    assert_eq!(candidates(&e, "뭉치")[0], "團");
    assert_eq!(candidates(&e, "말뭉치")[0], "言團");
}

#[test]
fn haneul_is_cheon() {
    assert_eq!(candidates(&engine(), "하늘")[0], "天");
}

#[test]
fn a_primer_reading_outranks_the_tiers_below_it() {
    let e = engine();
    // 倭語類解 盛 담을, 頗 ᄌᆞ몯; 光州千字文 千 즈믄; 訓蒙字會 菘 ᄇᆡᄎᆡ.
    assert_eq!(candidates(&e, "담고")[0], "盛고");
    assert_eq!(candidates(&e, "자못")[0], "頗");
    assert_eq!(candidates(&e, "즈믄")[0], "千");
    assert_eq!(candidates(&e, "배추")[0], "菘");
}

#[test]
fn a_hand_made_spelling_stays_ahead() {
    let e = engine();
    // 訓蒙字會 has 胷 and 倭語類解 膺, 臆 for 가슴; 釋讀辭典 has 胸. 개 is 犬 there, 狗 in 訓蒙字會.
    assert_eq!(candidates(&e, "가슴")[0], "胸");
    assert_eq!(candidates(&e, "개가")[0], "犬가");
    // The primer reading is still offered.
    assert!(candidates(&e, "가슴").iter().any(|c| c == "胷"));
    assert!(e.segments("가르치다").iter().any(|s| s.candidates.iter().any(|c| c.text == "訓다")));
}

#[test]
fn loanwords_and_particles_stay_as_they_were() {
    let e = engine();
    for word in ["버스", "컴퓨터", "버스를", "컴퓨터는"] {
        assert_eq!(candidates(&e, word)[0], word, "{word}");
    }
    assert_eq!(candidates(&e, "학교에서")[0], "學校에서");
}

#[test]
fn the_tier_sits_between_community_and_suggested() {
    let cost = |source: &str, count: &str| parse_native(&["낱말", "語", "noun", "", source, count]).unwrap().cost;
    assert!(cost("community", "") < cost("eonhae", ""));
    assert!(cost("eonhae", "") < cost("suggested", ""));
    // A chat count moves a row as it moves a suggested one, and more uses rank higher.
    assert_eq!(cost("eonhae", "137"), cost("suggested", "137"));
    assert!(cost("eonhae", "3") < cost("eonhae", "1"));
}

#[test]
fn every_row_says_where_to_check_it() {
    let rows = rows();
    assert!(rows.len() > 500);
    let mut primer = 0;
    for r in &rows {
        assert_eq!(r[4], "eonhae", "{r:?}");
        assert!(r[5].is_empty() || r[5].parse::<u32>().is_ok(), "the count is blank or a number: {r:?}");
        if r[6].starts_with("primer: ") {
            primer += 1;
            // Book year, page or folio: entry.
            assert!(r[6].contains(", ") && r[6].matches(": ").count() >= 2, "{r:?}");
        } else {
            assert!(r[6].starts_with("aligned ("), "{r:?}");
            assert!(r[6].contains(" → "), "a passage is quoted: {r:?}");
        }
    }
    assert!(primer > 500);
}

#[test]
fn a_gloss_from_namu_wiki_cites_its_page_and_licence() {
    let rows = rows();
    let row = |lemma: &str, written: &str| rows.iter().find(|r| r[0] == lemma && r[1] == written).unwrap_or_else(|| panic!("{lemma} {written}"));
    assert!(row("가", "際")[6].contains("新增類合 1576, 나무위키 신증유합/하권 2558 "));
    // 訓蒙字會's own 樹木 page writes {{크게|枳}}, with no {{Wikt}} inside; both transcriptions are cited on one row.
    assert!(row("탱자", "枳")[6].contains("訓蒙字會 1527, 樹木 상10A: "));
    assert!(row("탱자", "枳")[6].contains("나무위키 훈몽자회/상권 樹木 "));
    let mut cited = 0;
    for r in &rows {
        for citation in r[6].split("; ").filter(|c| c.contains("나무위키")) {
            cited += 1;
            assert!(citation.contains("CC BY-NC-SA 2.0 KR"), "{r:?}");
        }
    }
    assert!(cited > 1000);
}

#[test]
fn no_row_repeats_a_hand_made_pair() {
    let mut hand = std::collections::HashSet::new();
    for f in ["corpus.tsv", "editorial.tsv", "community.tsv"] {
        for l in std::fs::read_to_string(dict_dir().join(f)).unwrap().lines() {
            if l.starts_with('#') || l.starts_with("lemma\t") {
                continue;
            }
            let mut p = l.split('\t');
            if let (Some(a), Some(b)) = (p.next(), p.next()) {
                hand.insert((a.to_string(), b.to_string()));
            }
        }
    }
    for r in rows() {
        assert!(!hand.contains(&(r[0].clone(), r[1].clone())), "{r:?}");
    }
}
