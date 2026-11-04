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

/// Both the whole-phrase list and the one segment of a single run.
fn offers_all(e: &Engine, input: &str, want: &[&str]) {
    let (whole, segs) = texts(e, input);
    assert_eq!(segs.len(), 1, "{input}: one segment for the run");
    for w in want {
        assert!(whole.iter().any(|t| t == w), "{input}: {w} not in {whole:?}");
        assert!(segs[0].iter().any(|t| t == w), "{input}: {w} not in {:?}", segs[0]);
    }
}

#[test]
fn old_syllables_are_looked_up_by_their_modern_letters() {
    let e = engine(Options::default());
    // ᄉᆞ랑 = 사랑, 사ᄅᆞᆷ = 사람 (人), 말ᄊᆞᆷ = 말씀.
    let sarang = "\u{1109}\u{119E}랑";
    offers_all(&e, sarang, &["愛", sarang]);
    assert_eq!(texts(&e, sarang).0[0], "愛");
    let saram = "사\u{1105}\u{119E}\u{11B7}";
    offers_all(&e, saram, &["人", saram]);
    let malssam = "말\u{110A}\u{119E}\u{11B7}";
    offers_all(&e, malssam, &["言씀", malssam]);
    // ᄂᆞᆫ = 는, so 있ᄂᆞᆫ is 있는 with its tail as typed.
    let issneun = format!("있{NEUN}");
    offers_all(&e, &issneun, &["有\u{1102}\u{119E}\u{11AB}", "在ㅆ\u{1102}\u{119E}\u{11AB}", &issneun]);
    // ᄒᆞᆰ = 흙, ᄯᅩ = 또, ᅌᅵ = 이.
    offers_all(&e, "\u{1112}\u{119E}\u{11B0}", &["土"]);
}

#[test]
fn an_old_tail_is_kept_as_typed_in_first_place() {
    let e = engine(Options::default());
    let ha_go = format!("{HA}고");
    let (whole, segs) = texts(&e, &ha_go);
    assert_eq!(whole[..2], [ha_go.clone(), "爲고".to_string()]);
    assert_eq!(segs[0][..2], [ha_go.clone(), "爲고".to_string()]);
    let input = format!("추가{HAL}까요");
    let (whole, segs) = texts(&e, &input);
    assert_eq!(whole[0], format!("追加{HAL}까요"));
    assert_eq!(segs[0][0], whole[0]);
    assert!(whole.contains(&input));
    // Not read as the Sino-Korean word 하야.
    assert_eq!(texts(&e, &format!("{HA}야")).0[0], format!("{HA}야"));
    // 國漢文混用 too.
    let s = engine(Options { sino_only: true, ..Options::default() });
    assert_eq!(texts(&s, &format!("공부{HA}{NEUN}")).0[0], format!("工夫{HA}{NEUN}"));
}

#[test]
fn era_spellings_are_looked_up_behind_what_was_typed() {
    let e = engine(Options::default());
    // A syllable that modern words do not use gives its converted reading
    // first, then what was typed.
    offers_all(&e, "텬하", &["天下", "텬하"]);
    assert_eq!(texts(&e, "텬하").0[..2], ["天下", "텬하"]);
    assert_eq!(texts(&e, "됴타").0[..2], ["好다", "됴타"]);
    // A final ㅅ for ㅆ, only when the word was not converted as typed.
    offers_all(&e, "갓다", &["去ㅆ다", "갓다"]);
    assert_eq!(texts(&e, "갓다").0[0], "갓다");
    assert_eq!(texts(&e, "잇다").0[0], "續다");
    // Ordinary words are not looked up again.
    assert_eq!(texts(&e, "사랑").0[0], "愛");
}

#[test]
fn old_text_is_read_back_without_panicking() {
    let e = engine(Options::default());
    assert_eq!(e.read(&format!("追加{HAL}까요")), "추가할까요");
    assert_eq!(e.read(&format!("工夫{HA}{NEUN}")), "공부하는");
    assert_eq!(e.read(&format!("字{REUL}")), e.read("字를"));
    // Letters the reader does not know stay as they are.
    let sarang = "\u{1109}\u{119E}랑 天을";
    assert_eq!(e.read(sarang), "\u{1109}\u{119E}랑 하늘을");
    // Modern text reads as before.
    assert_eq!(e.read("天을 見ㅆ다"), "하늘을 봤다");
}

#[test]
fn no_jamo_makes_the_engine_panic() {
    let e = engine(Options::default());
    let jamo = ('\u{1100}'..='\u{11FF}').chain('\u{A960}'..='\u{A97C}').chain('\u{D7B0}'..='\u{D7FB}');
    for c in jamo {
        for input in [format!("{c}"), format!("가{c}"), format!("{c}가"), format!("{c}\u{1161}"), format!("\u{1109}{c}"), format!("\u{1109}\u{119E}{c}"), format!("天{c}")] {
            e.convert(&input);
            e.segments(&input);
            e.read(&input);
        }
    }
}

#[test]
fn hada_kun_still_offers_the_old_spelling_in_convert_and_segments() {
    let ha_go = format!("{HA}고");
    let e = engine(Options { hada_kun: true, ..Options::default() });
    let (whole, segs) = texts(&e, "하고");
    assert_eq!(whole[0], "爲고");
    assert_eq!(segs[0][0], "爲고");
    assert_eq!(whole.iter().position(|t| *t == ha_go), segs[0].iter().position(|t| *t == ha_go));
    assert_eq!(whole.iter().position(|t| *t == ha_go), Some(2));
    // With old_ha as well, both APIs put it first.
    let e = engine(Options { hada_kun: true, old_ha: true, ..Options::default() });
    let (whole, segs) = texts(&e, "하고");
    assert_eq!(whole[0], ha_go);
    assert_eq!(segs[0][0], ha_go);
    assert_eq!(texts(&e, "추가할까요").0[0], format!("追加{HAL}까요"));
}

#[test]
fn old_input_gets_the_same_old_spelling_as_modern_input() {
    let ha_go = format!("愛{HA}고");
    let sarang = "\u{1109}\u{119E}랑하고";
    // Setting off: third place, behind the typed run, as for 사랑하고.
    let e = engine(Options::default());
    let (whole, segs) = texts(&e, sarang);
    assert_eq!(whole[..3], ["愛하고".to_string(), sarang.to_string(), ha_go.clone()]);
    assert_eq!(segs[0][..3], whole[..3]);
    assert_eq!(texts(&e, "사랑하고").0[2], ha_go);
    // Setting on: first, with the modern spelling behind it.
    let e = engine(Options { old_ha: true, ..Options::default() });
    let (whole, segs) = texts(&e, sarang);
    assert_eq!(whole[..2], [ha_go.clone(), "愛하고".to_string()]);
    assert_eq!(segs[0][..2], whole[..2]);
    assert_eq!(texts(&e, "사랑하고").0[..2], whole[..2]);
    assert!(whole.contains(&sarang.to_string()));
}

#[test]
fn a_recognised_suffix_does_not_hide_the_era_reading_of_the_prefix() {
    let e = engine(Options::default());
    // 텬하고 reads 하고 as a word, and 天下고 is still looked up.
    let mixed = format!("텬{HA}고");
    for input in ["텬하고", mixed.as_str()] {
        let (whole, segs) = texts(&e, input);
        assert!(whole.iter().any(|t| t == input), "{input}: {whole:?}");
        assert!(whole.iter().any(|t| t == "天下고"), "{input}: {whole:?}");
        assert!(segs[0].iter().any(|t| t == "天下고"), "{input}: {:?}", segs[0]);
    }
}

fn offers(e: &Engine, input: &str, want: &str) -> bool {
    let (whole, segs) = texts(e, input);
    whole.iter().any(|t| t == want) || segs.iter().flatten().any(|t| t == want)
}

#[test]
fn spellings_as_pronounced_are_read_through_fold_classes() {
    let e = engine(Options::default());
    // A syllable that modern words do not use: the converted reading first, then what was typed.
    for (typed, first) in [("됴타", "好다"), ("됴코", "好고"), ("됴치", "好지"), ("텬하", "天下"), ("됴하셔", "好아서")] {
        let (whole, segs) = texts(&e, typed);
        assert_eq!(whole[..2], [first, typed], "{typed}");
        assert_eq!(segs[0][..2], [first, typed], "{typed}");
    }
    // The other classes offer their reading behind the first candidate.
    for (typed, want) in [
        ("만타", "多다"), ("안코", "不ㅎ고"), ("사르미", "人이"), ("그르슬", "器을"), ("니르다", "至다"), ("닙다", "服다"),
        ("녀름", "夏"), ("갓다", "去ㅆ다"), ("업시", "無이"), ("놉다", "高다"), ("믈", "水"), ("블", "火"), ("플", "草"),
        ("셔울", "京"), ("잇다", "有다"),
    ] {
        assert!(offers(&e, typed, want), "{typed} → {want}: {:?}", texts(&e, typed).0);
    }
    // Old letters with the classes.
    for (typed, want) in [
        ("\u{1109}\u{119E}랑", "愛"), ("사\u{1105}\u{119E}미", "人이"), ("마\u{110B}\u{119E}미", "心이"), ("둏\u{110B}\u{119E}\u{11AB}", "好\u{110B}\u{119E}\u{11AB}"),
        ("잇\u{1102}\u{119E}\u{11AB}", "有\u{1102}\u{119E}\u{11AB}"), ("업\u{1102}\u{119E}\u{11AB}", "無\u{1102}\u{119E}\u{11AB}"), ("노\u{1111}\u{119E}\u{11AB}", "高은"),
        ("\u{112F}\u{1169}", "亦"), ("\u{112D}\u{1169}\u{11BA}", "花"), ("\u{1121}\u{1165}", "書"), ("\u{1120}\u{1173}\u{11AE}", "意"),
    ] {
        assert!(offers(&e, typed, want), "{typed} → {want}: {:?}", texts(&e, typed).0);
        assert!(texts(&e, typed).0.iter().any(|t| t == typed), "{typed} stays selectable");
    }
    // Initial-sound forms that the dictionary already reads stay first.
    for (typed, first) in [("량심", "良心"), ("로동", "勞動"), ("력사", "歷史")] {
        assert_eq!(texts(&e, typed).0[0], first);
    }
    // Words that are modern too keep their reading first.
    assert_eq!(texts(&e, "곳").0[0], "處");
    assert_eq!(texts(&e, "잇다").0[0], "續다");
    assert_eq!(texts(&e, "믈").0[0], "믈");
}

#[test]
fn reading_keeps_old_syllables_whole() {
    let e = engine(Options::default());
    // The function syllables become modern.
    assert_eq!(e.read(&format!("追加{HAL}까요")), "추가할까요");
    // Other old syllables stay whole in their token, whatever the Hanja before them.
    for han in ["讀", "天", "追加", "字"] {
        for old in ["\u{110A}\u{119E}\u{11B7}", "\u{1109}\u{119E}", "\u{1112}\u{119E}\u{11AF}\u{11BC}", "\u{112F}\u{1169}"] {
            let out = e.read(&format!("{han}{old}이"));
            assert!(out.contains(old), "{han}{old}: {out:?}");
            assert!(out.ends_with('이'), "{han}{old}: {out:?}");
        }
    }
    // Text round-trips through the reader when it holds no Hanja.
    let plain = "\u{1109}\u{119E}랑 나랏말\u{110A}\u{119E}미 \u{112F}\u{1169}";
    assert_eq!(e.read(plain), plain);
}

#[test]
fn random_old_and_modern_text_never_panics_or_loses_a_run() {
    let e = engine(Options::default());
    let pieces = [
        "사", "랑", "하", "텬", "됴", "타", "잇", "업", "믈", "미", "르", "슬", "니", "녀", "름", "\u{1112}\u{119E}", "\u{1112}\u{119E}\u{11AF}", "\u{110A}\u{119E}\u{11B7}",
        "\u{1109}\u{119E}", "\u{112F}\u{1169}", "\u{1121}\u{1165}", "\u{1120}\u{1173}\u{11AE}", "\u{1105}\u{119E}\u{11AF}", "\u{110B}\u{119E}", "\u{1140}\u{1175}",
        "\u{11A1}", "\u{1100}", "\u{1161}", "\u{11AB}", "\u{A960}", "\u{D7B0}", " ", "天", "ㅆ",
    ];
    let mut x: u64 = 0x2545F4914F6CDD1D;
    let mut next = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    for _ in 0..500 {
        let n = 1 + (next() % 10) as usize;
        let input: String = (0..n).map(|_| pieces[(next() % pieces.len() as u64) as usize]).collect();
        let whole = e.convert(&input);
        assert!(!whole.is_empty() || input.is_empty(), "{input:?}");
        let segs = e.segments(&input);
        assert_eq!(segs.iter().map(|s| s.reading.as_str()).collect::<String>(), input, "segments lose text");
        assert!(segs.iter().all(|s| !s.candidates.is_empty()), "{input:?}");
        e.read(&input);
    }
}
