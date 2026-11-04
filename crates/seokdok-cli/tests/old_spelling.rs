//! Old Hangul in the engine: the old spellings it offers and the old input it
//! accepts, with the shipped dictionaries.
//!
//! Old Hangul is written with conjoining jamo: ᄒᆞ is U+1112 U+119E, ᄂᆞᆫ is
//! U+1102 U+119E U+11AB, ᄅᆞᆯ is U+1105 U+119E U+11AF, ᄋᆞᆯ is U+110B U+119E
//! U+11AF.

use seokdok_core::{Engine, Options};
use std::path::Path;

const HA: &str = "\u{1112}\u{119E}";
const HAL: &str = "\u{1112}\u{119E}\u{11AF}";
const NEUN: &str = "\u{1102}\u{119E}\u{11AB}";
const REUL: &str = "\u{1105}\u{119E}\u{11AF}";
const EUL: &str = "\u{110B}\u{119E}\u{11AF}";

fn engine(o: Options) -> Engine {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
    Engine::from_dir(&dir, o).expect("dict").0
}

/// The whole-phrase candidates and, per segment, the candidates.
fn texts(e: &Engine, input: &str) -> (Vec<String>, Vec<Vec<String>>) {
    (
        e.convert(input).into_iter().map(|c| c.text).collect(),
        e.segments(input).into_iter().map(|s| s.candidates.into_iter().map(|c| c.text).collect()).collect(),
    )
}

#[test]
fn the_old_spelling_of_a_tail_comes_third() {
    let e = engine(Options::default());
    let (whole, segs) = texts(&e, "하고");
    assert_eq!(whole[..3], ["하고", "爲고", &format!("{HA}고")]);
    assert_eq!(segs[0][..3], ["하고", "爲고", &format!("{HA}고")]);
    let (whole, segs) = texts(&e, "추가할까요");
    assert_eq!(whole[0], "追加할까요");
    assert_eq!(whole[2], format!("追加{HAL}까요"));
    assert_eq!(segs[0][2], format!("追加{HAL}까요"));
    let (whole, segs) = texts(&e, "공부하는");
    assert_eq!(whole[2], format!("工夫{HA}{NEUN}"));
    assert_eq!(segs[1][2], format!("{HA}{NEUN}"));
    assert_eq!(texts(&e, "이것을").0[2], format!("此것{EUL}"));
    // Hanja typed in the input make the Hangul after them a tail.
    assert_eq!(texts(&e, "字를").0, ["字를", "字㔹", &format!("字{REUL}")]);
    assert_eq!(texts(&e, "해요").0[2], "\u{1112}\u{11A1}요");
    // The modern candidates keep their places, and 했 has no old spelling.
    assert_eq!(texts(&e, "했다").0[..2], ["했다", "爲ㅆ다"]);
    assert!(texts(&e, "했다").0.iter().all(|t| !t.contains('\u{1112}')));
    assert_eq!(texts(&e, "안녕하세요").0[..2], ["安寧하세요", "安寧爲세요"]);
}

#[test]
fn the_old_spelling_is_offered_in_kuk_han_mun_too() {
    let e = engine(Options { sino_only: true, ..Options::default() });
    let (whole, segs) = texts(&e, "공부하셨다");
    assert_eq!(whole[0], "工夫하셨다");
    assert_eq!(whole[2], format!("工夫{HA}셨다"));
    assert_eq!(segs[1][..2], ["하셨다", &format!("{HA}셨다")]);
    assert_eq!(texts(&e, "규칙을").0[..3], ["規則을", "규칙을", &format!("規則{EUL}")]);
}

const HAN: &str = "\u{1112}\u{119E}\u{11AB}";

#[test]
fn old_ha_puts_the_old_spelling_of_ha_first() {
    let e = engine(Options { old_ha: true, ..Options::default() });
    // The modern spelling stays right behind it.
    let (whole, segs) = texts(&e, "하고");
    assert_eq!(whole[..3], [format!("{HA}고"), "하고".to_string(), "爲고".to_string()]);
    assert_eq!(segs[0][..2], [format!("{HA}고"), "하고".to_string()]);
    let (whole, segs) = texts(&e, "추가할까요");
    assert_eq!(whole[..2], [format!("追加{HAL}까요"), "追加할까요".to_string()]);
    assert_eq!(segs[0][..2], [format!("追加{HAL}까요"), "追加할까요".to_string()]);
    assert_eq!(texts(&e, "공부한").0[..2], [format!("工夫{HAN}"), "工夫한".to_string()]);
    // Only the 하 rows: 는 and 를 stay modern in first place.
    let (whole, _) = texts(&e, "공부하는");
    assert_eq!(whole[..2], [format!("工夫{HA}는"), "工夫하는".to_string()]);
    assert!(whole.contains(&format!("工夫{HA}{NEUN}")));
    assert_eq!(texts(&e, "字를").0[0], "字를");
    assert_eq!(texts(&e, "규칙을").0[0], "規則을");
    // Nothing to swap in a word without 하.
    assert_eq!(texts(&e, "하늘").0, texts(&engine(Options::default()), "하늘").0);
}

#[test]
fn old_ha_off_changes_nothing() {
    let off = engine(Options::default());
    assert_eq!(texts(&off, "하고").0[0], "하고");
    assert_eq!(texts(&off, "공부한").0[0], "工夫한");
}
