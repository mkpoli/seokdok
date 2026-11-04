//! Text that is already partly or wholly 釋讀 or 國漢文混用 converts as well:
//! a word holding Hanja is one unit with the typed spelling among its
//! candidates, and pure Hangul words convert as before.

use seokdok_core::{Engine, Options};
use std::path::Path;
use std::sync::OnceLock;

fn engine(o: Options) -> Engine {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
    Engine::from_dir(&dir, o).expect("dict").0
}

fn seokdok() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| engine(Options::default()))
}

fn sino() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(|| engine(Options { sino_only: true, ..Options::default() }))
}

fn first(e: &Engine, s: &str) -> String {
    e.convert(s)[0].text.clone()
}

/// The readings of the segments, which must add up to the input.
fn readings(e: &Engine, s: &str) -> Vec<String> {
    let r: Vec<String> = e.segments(s).into_iter().map(|g| g.reading).collect();
    assert_eq!(r.concat(), s, "segments of {s:?} must cover it");
    r
}

/// The menu of the one word `s`.
fn menu(e: &Engine, s: &str) -> Vec<String> {
    let segs = e.segments(s);
    assert_eq!(segs.len(), 1, "{s:?} is one word: {:?}", segs.iter().map(|g| &g.reading).collect::<Vec<_>>());
    assert_eq!(segs[0].reading, s);
    let m: Vec<String> = segs[0].candidates.iter().map(|c| c.text.clone()).collect();
    for (i, c) in m.iter().enumerate() {
        assert!(!m[..i].contains(c), "{s:?}: {c:?} twice in {m:?}");
    }
    m
}

fn has_hanja(s: &str) -> bool {
    s.chars().any(seokdok_core::dict::is_han)
}

#[test]
fn already_converted_text_stays_in_seokdok_mode() {
    let e = seokdok();
    for s in [
        "初음 來신 事ㅅ을 歡迎합니다",
        "天을 見ㅆ다",
        "先生님께서",
        "良心",
        "天",
        "超ㅜ",
        "過무",
        "此處서",
        "古ㅅ韓㐎로 良心을 記다",
        "先生님께서 工夫하셨다",
    ] {
        assert_eq!(first(e, s), s);
        // Every Hanja word has a menu that opens with the typed spelling.
        for g in e.segments(s).iter().filter(|g| has_hanja(&g.reading)) {
            assert!(g.candidates.len() > 1, "{:?} has no alternatives", g.reading);
            assert_eq!(g.candidates[0].text, g.reading);
        }
    }
}

#[test]
fn the_example_line_is_cut_into_words_and_spaces() {
    let e = seokdok();
    assert_eq!(readings(e, "初음 來신 事ㅅ을 歡迎합니다"), ["初음", " ", "來신", " ", "事ㅅ을", " ", "歡迎합니다"]);
    assert_eq!(readings(e, "天을 見ㅆ다"), ["天을", " ", "見ㅆ다"]);
    // Whitespace is its own segment however it comes.
    assert_eq!(readings(e, "  天을  見ㅆ다 "), ["  ", "天을", "  ", "見ㅆ다", " "]);
    assert_eq!(readings(e, "天을\n見ㅆ다"), ["天을", "\n", "見ㅆ다"]);
    assert_eq!(e.segments("天을 見ㅆ다")[1].candidates.len(), 1);
}

#[test]
fn menus_hold_typed_dictionary_constructed_and_plain_forms_in_that_order() {
    let e = seokdok();
    let m = menu(e, "天을");
    assert_eq!(&m[..2], ["天을", "乾을"]);
    assert!(m.contains(&"宇을".to_string()));
    assert_eq!(m.last().unwrap(), "하늘을");
    // Attested forms (天을 乾을 …) come before constructed ones (天늘을).
    let pos = |w: &str| m.iter().position(|c| c == w).unwrap_or_else(|| panic!("{w} not in {m:?}"));
    assert!(pos("空을") < pos("天늘을"));

    let m = menu(e, "先生님께서");
    assert_eq!(m[0], "先生님께서");
    assert!(m.contains(&"先生主께서".to_string()));
    assert_eq!(m.last().unwrap(), "선생님께서");

    let m = menu(e, "良心");
    assert_eq!(&m[..2], ["良心", "兩心"]);
    assert_eq!(m.last().unwrap(), "양심");

    let m = menu(e, "天");
    assert_eq!(&m[..2], ["天", "乾"]);

    let m = menu(e, "來신");
    assert_eq!(m[0], "來신");
    assert_eq!(m.last().unwrap(), "오신");
}

#[test]
fn okurigana_variants_and_homographs_keep_the_typed_form_first() {
    let e = seokdok();
    for (typed, hangul, other) in [("超ㅜ", "너무", "超무"), ("過무", "너무", "超무"), ("此處서", "여기서", "此所서"), ("事ㅅ을", "것을", "件을")] {
        let m = menu(e, typed);
        assert_eq!(m[0], typed);
        assert!(m.contains(&other.to_string()), "{typed}: {other} not in {m:?}");
        assert_eq!(m.last().unwrap(), hangul);
    }
}

#[test]
fn tails_stay_with_their_word() {
    let e = seokdok();
    // 先生 + 님께서 and 見ㅆ + 다 are no separate pieces.
    assert_eq!(readings(e, "先生님께서"), ["先生님께서"]);
    assert_eq!(readings(e, "天을 見ㅆ다 先生님께서"), ["天을", " ", "見ㅆ다", " ", "先生님께서"]);
}

#[test]
fn pure_hangul_words_in_a_mixed_line_convert_as_before() {
    let e = seokdok();
    assert_eq!(first(e, "天을 봤다"), "天을 見ㅆ다");
    assert_eq!(first(e, "하늘을 見ㅆ다"), "天을 見ㅆ다");
    assert_eq!(first(e, "하늘을 봤다"), "天을 見ㅆ다");
    let segs = e.segments("天을 봤다");
    assert_eq!(segs.iter().map(|g| g.reading.as_str()).collect::<Vec<_>>(), ["天을", " ", "봤다"]);
    assert_eq!(segs[2].candidates[0].text, "見ㅆ다");
    let segs = e.segments("하늘을 見ㅆ다");
    assert_eq!(segs[0].candidates[0].text, "天을");
    assert_eq!(segs[2].candidates[0].text, "見ㅆ다");
    // The Hangul word is converted whichever side the Hanja word stands on.
    assert_eq!(first(e, "良心을 봤다 하늘"), "良心을 見ㅆ다 天");
}

#[test]
fn input_without_hanja_goes_the_old_way() {
    let e = seokdok();
    assert_eq!(first(e, "하늘을 봤다"), "天을 見ㅆ다");
    assert_eq!(first(e, "ㅋㅋ 하늘"), "ㅋㅋ 天");
    assert_eq!(first(e, "abc 123, 하늘!"), "abc 123, 天!");
    assert_eq!(first(e, "abc"), "abc");
}

#[test]
fn latin_digits_and_punctuation_pass_through() {
    let e = seokdok();
    assert_eq!(first(e, "abc 123, 天을!"), "abc 123, 天을!");
    assert_eq!(first(e, "「天을」 2020年에"), "「天을」 2020年에");
    let m = menu(e, "「天을」");
    assert_eq!(m[0], "「天을」");
    assert_eq!(m.last().unwrap(), "「하늘을」");
    let m = menu(e, "2020年에");
    assert_eq!(m[0], "2020年에");
    assert_eq!(m.last().unwrap(), "2020년에");
    assert_eq!(readings(e, "abc 123, 天을!"), ["abc 123,", " ", "天을!"]);
}

#[test]
fn words_the_reader_cannot_read_are_kept() {
    for e in [seokdok(), sino()] {
        // 𠀀 has no reading, and 見ㅆ다ㅋㅋ is not a form the reader knows.
        for s in ["𠀀을", "見ㅆ다ㅋㅋ"] {
            assert_eq!(first(e, s), s);
            let m = menu(e, s);
            assert_eq!(m[0], s);
            assert!(!m.contains(&e.read(s)) || e.read(s) == s, "{s}: the broken reading must not be offered");
        }
        assert_eq!(first(e, "𠀀을 abc"), "𠀀을 abc");
    }
}

#[test]
fn mixed_input_in_old_hangul_does_not_break() {
    let e = seokdok();
    for s in ["ᄒᆞᆫ글 天을", "天을 ᄒᆞ늘", "古ㅅ한글 ᄒᆞᆫ글"] {
        readings(e, s);
        assert!(!e.convert(s).is_empty());
    }
    assert_eq!(e.segments("ᄒᆞᆫ글 天을").last().unwrap().candidates[0].text, "天을");
}

#[test]
fn kokanmun_mode_writes_the_mode_first_and_keeps_the_typed_form_selectable() {
    let e = sino();
    assert_eq!(first(e, "天을 見ㅆ다"), "하늘을 봤다");
    assert_eq!(first(e, "天을 봤다"), "하늘을 봤다");
    assert_eq!(first(e, "하늘을 見ㅆ다"), "하늘을 봤다");
    let m = menu(e, "天을");
    assert_eq!(&m[..2], ["하늘을", "天을"]);
    let m = menu(e, "見ㅆ다");
    assert_eq!(&m[..2], ["봤다", "見ㅆ다"]);
    assert_eq!(first(e, "先生님께서 工夫하셨다"), "先生님께서 工夫하셨다");
    assert_eq!(first(e, "良心"), "良心");
    assert_eq!(first(e, "超ㅜ"), "너무");
    assert_eq!(menu(e, "超ㅜ")[..2], ["너무", "超ㅜ"]);
    let m = menu(e, "先生님께서");
    assert_eq!(m[0], "先生님께서");
    assert!(m.contains(&"선생님께서".to_string()));
    // Already in the mode's writing, and still no duplicate.
    assert_eq!(menu(e, "良心").iter().filter(|c| *c == "良心").count(), 1);
}

#[test]
fn the_reader_gives_hangul_in_every_mode() {
    for e in [seokdok(), sino()] {
        assert_eq!(e.read("天을 見ㅆ다"), "하늘을 봤다");
        assert_eq!(e.read("初음 來신 事ㅅ을 歡迎합니다"), "처음 오신 것을 환영합니다");
        assert_eq!(e.read("先生님께서 工夫하셨다"), "선생님께서 공부하셨다");
    }
}

// Already-converted text, from the README and the site, is left alone.

fn read(rel: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// `{漢字|한자}` marks shown as their Hanja.
fn hanja_side(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let tail = &rest[i + 1..];
        match (tail.find('|'), tail.find('}')) {
            (Some(bar), Some(end)) if bar < end => {
                out.push_str(&tail[..bar]);
                rest = &tail[end + 1..];
            }
            _ => {
                out.push('{');
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

#[test]
fn text_the_readme_and_the_site_show_is_not_changed() {
    let e = seokdok();
    let (mut lines, mut words) = (std::collections::HashSet::new(), std::collections::HashSet::new());
    for file in ["README.md", "web/src/lib/i18n.svelte.js", "web/src/lib/samples.js", "web/src/lib/pages.js"] {
        for line in hanja_side(&read(file)).lines().filter(|l| has_hanja(l)) {
            if !lines.insert(line.to_string()) {
                continue;
            }
            // A line keeps every Hanja word; Hangul words around them may convert.
            for g in e.segments(line).iter().filter(|g| has_hanja(&g.reading)) {
                assert_eq!(g.candidates[0].text, g.reading, "{file}: {line}");
            }
            for w in line.split_whitespace().filter(|w| has_hanja(w)) {
                if !words.insert(w.to_string()) {
                    continue;
                }
                assert_eq!(first(e, w), w, "{file}: {w}");
            }
        }
    }
    assert!(lines.len() > 100 && words.len() > 300, "{} lines, {} words", lines.len(), words.len());
    // The README's own examples, as whole lines.
    for line in read("README.md").lines().filter(|l| l.contains(" → ")) {
        let Some((_, out)) = line.split_once("→") else { continue };
        let out = out.trim();
        if out.split_whitespace().all(has_hanja) {
            assert_eq!(first(e, out), out);
        }
    }
}

#[test]
fn words_mixing_hanja_and_old_letters_stay_first_and_read_back() {
    for e in [seokdok(), sino()] {
        let m = menu(e, "追加ᄒᆞᆯ까요");
        assert!(m.contains(&"追加ᄒᆞᆯ까요".to_string()));
        assert!(m.contains(&"追加할까요".to_string()), "{m:?}");
        assert!(m.contains(&"추가할까요".to_string()));
        let m = menu(e, "字ᄅᆞᆯ");
        assert!(m.contains(&"字ᄅᆞᆯ".to_string()));
        assert!(m.len() > 3, "字ᄅᆞᆯ offers alternatives: {m:?}");
        assert!(m.contains(&"자를".to_string()));
        assert_eq!(readings(e, "字ᄅᆞᆯ 追加ᄒᆞᆯ까요"), ["字ᄅᆞᆯ", " ", "追加ᄒᆞᆯ까요"]);
    }
    assert_eq!(first(seokdok(), "追加ᄒᆞᆯ까요"), "追加ᄒᆞᆯ까요");
    assert_eq!(first(seokdok(), "字ᄅᆞᆯ 追加ᄒᆞᆯ까요"), "字ᄅᆞᆯ 追加ᄒᆞᆯ까요");
    assert!(menu(seokdok(), "字ᄅᆞᆯ").contains(&"切ㄹ".to_string()));
    assert_eq!(seokdok().read("追加ᄒᆞᆯ까요"), "추가할까요");
    assert_eq!(seokdok().read("字ᄅᆞᆯ"), "자를");
}
