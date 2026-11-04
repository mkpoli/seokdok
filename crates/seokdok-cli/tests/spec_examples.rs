//! Every example in docs/orthography.md must be produced by the engine with
//! the shipped dictionaries, as the first candidate or an alternative.

use seokdok_core::{Engine, Fragment, Options, Style};
use std::path::Path;

fn engine(o: Options) -> Engine {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
    Engine::from_dir(&dir, o).expect("dict").0
}

fn offers(e: &Engine, input: &str, want: &str) -> bool {
    e.convert(input).iter().any(|c| c.text == want)
        || e.segments(input).iter().any(|s| s.candidates.iter().any(|c| c.text == want))
}

fn check(o: Options, cases: &[(&str, &str)]) {
    let e = engine(o);
    let missing: Vec<String> = cases
        .iter()
        .filter(|(i, w)| !offers(&e, i, w))
        .map(|(i, w)| format!("{i} → {w} (first: {})", e.convert(i)[0].text))
        .collect();
    assert!(missing.is_empty(), "missing:\n{}", missing.join("\n"));
}

#[test]
fn default_options() {
    check(Options::default(), &[
        ("다른", "異ㄴ"), ("전", "自ㄴ"), ("내", "我ㅣ"), ("좀", "少ㅁ"), ("그렇네요", "然ㅎ네요"),
        ("재밌네요", "滋味ㅆ네요"), ("갔다", "去ㅆ다"), ("봤다", "見ㅆ다"), ("했다", "爲ㅆ다"),
        ("달랐다", "異ㅆ다"), ("가까웠다", "近웠다"), ("걸었다", "步었다"), ("몰랐다", "䚳랐다"),
        ("아는", "知는"), ("그래서", "然서"), ("해요", "爲요"), ("봐라", "見라"), ("마셔라", "飮라"),
        ("와서", "來서"), ("절", "自ㄹ"), ("내가", "我ㅣ가"), ("처음에", "初음에"),
        ("오랜만에", "久ㄴ만에"), ("말씀을", "言씀을"), ("어떻게", "何떻게"), ("이런", "此런"),
        ("무슨", "何슨"), ("오히려", "尙려"), ("너무", "超무"), ("안녕하세요", "安寧하세요"),
        ("안녕하세요", "安寧爲세요"), ("하늘을", "天을"), ("있는", "在ㅆ는"), ("것을", "事ㅅ을"),
        ("규칙이", "規則이"), ("읽었다", "讀었다"), ("있네요", "有네요"), ("좋은", "好은"),
        // A stem ending in ㅏ/ㅓ takes 아/어 without a letter of its own.
        ("사", "買"), ("가", "去"), ("서", "立"),
        // Wiktionary etymologies and the interjection tier.
        ("게", "事ㅣ"), ("건", "事ㄴ"), ("대충", "大總"), ("재미", "滋味"), ("예", "唯"), ("아니요", "否"), ("이것저것", "此彼"),
    ]);
}

#[test]
fn morpheme_style() {
    check(Options { style: Style::Morpheme, ..Options::default() }, &[
        ("봤다", "見았다"), ("어려워요", "難어요"), ("쉬워서", "易어서"), ("가셨다", "去셨다"),
        ("썼다", "書었다"),
    ]);
}

#[test]
fn letter_fragments() {
    check(Options { fragment: Fragment::Letter, ..Options::default() }, &[
        ("봐라", "見ㅏ라"), ("마셔라", "飮ㅓ라"), ("와서", "來ㅏ서"),
    ]);
}

#[test]
fn coda_options() {
    check(Options { coda_always: true, ..Options::default() }, &[("있네요", "有ㅆ네요"), ("알고", "知ㄹ고")]);
    check(Options { resyllabify: true, ..Options::default() }, &[("있으면", "在쓰면"), ("있어서", "在써서")]);
    check(Options { hada_kun: true, ..Options::default() }, &[("안녕하세요", "安寧爲세요"), ("공부하다", "工夫爲다")]);
}

#[test]
fn sino_only_leaves_native_words_in_hangul() {
    check(Options { sino_only: true, ..Options::default() }, &[
        ("나라를 사랑하는 마음", "나라를 사랑하는 마음"),
        ("선생님께서 공부하셨다", "先生님께서 工夫하셨다"),
        ("학교에 가서 친구를 만났다", "學校에 가서 親舊를 만났다"),
    ]);
}

#[test]
fn etymology_comes_before_homophones_but_after_kun() {
    let e = engine(Options::default());
    let first = |s: &str| e.convert(s)[0].text.clone();
    // 대충 is nativised from 大總 (Wiktionary); 代充 is an unrelated homophone.
    assert_eq!(first("대충"), "大總");
    // A native word's 訓讀 still wins over its etymological Sino spelling.
    assert_eq!(first("사랑"), "愛");
}

#[test]
fn reading_back_prefers_the_usual_reading() {
    let e = engine(Options::default());
    // sino.tsv credits 先生 with the chat count under both 선생 and 선상.
    assert_eq!(e.read("先生님"), "선생님");
    // The chat writes 一番 for 한번 (editorial.tsv).
    assert_eq!(e.read("一番씩"), "한번씩");
    // 音讀 + 하다/되다 stays 音讀 (not 讀하다 = 구독하다).
    assert_eq!(e.read("釋讀하면"), "석독하면");
    assert_eq!(e.read("飜譯해서"), "번역해서");
    assert_eq!(e.read("登載되어"), "등재되어");
    // 天 and 神 alone are 하늘 and 신; 하느님 is 天主.
    assert_eq!(e.read("天을 見ㅆ다"), "하늘을 봤다");
    assert_eq!(e.read("神이"), "신이");
}

#[test]
fn honorific_nim() {
    // Any noun takes 님, written 主 or left in Hangul.
    check(Options::default(), &[("두벌식", "두벌式"), ("해님", "日主"), ("선생님께서", "先生主께서"), ("부처님", "佛主"), ("아버님", "父主")]);
    let e = engine(Options::default());
    assert!(e.convert("해님").iter().any(|c| c.text == "日님"));
    assert!(e.convert("선생님").iter().any(|c| c.text == "先生님"));
    assert_eq!(e.read("佛主"), "부처님");
    assert_eq!(e.read("兄主"), "형님");
    assert_eq!(e.read("天主이"), "하느님이");
}

#[test]
fn determiners_attach_to_the_next_noun() {
    let e = engine(Options::default());
    assert_eq!(e.convert("옛한글")[0].text, "古ㅅ韓㐎");
    assert_eq!(e.convert("첫사랑")[0].text, "初ㅅ愛");
}

#[test]
fn notes_do_not_repeat_the_candidate() {
    let e = engine(Options::default());
    assert_eq!(e.convert("양심")[0].note, "양심 · 音讀");
    let seg = &e.segments("양심")[0].candidates[0];
    assert_eq!((seg.text.as_str(), seg.note.as_str()), ("良心", "양심 · 音讀"));
    // A phrase still names each piece.
    assert_eq!(e.convert("나라를 사랑")[0].note, "國 나라 · 訓讀 / 愛 사랑 · 訓讀");
}

#[test]
fn bare_stems_convert_alone() {
    // A stem typed alone, to be continued in Hangul: 不ㅎ + 고.
    let e = engine(Options::default());
    assert_eq!(e.convert("않")[0].text, "不ㅎ");
    assert_eq!(e.convert("읽")[0].text, "讀");
    assert_eq!(e.convert("어떻")[0].text, "何떻");
    assert!(offers(&e, "있", "在ㅆ"));
}

#[test]
fn characters_found_by_their_gloss() {
    let e = engine(Options::default());
    let first = |s: &str| e.convert(s)[0].text.clone();
    // Nothing else reads these, so the character comes first, in either order.
    assert_eq!(first("살찔비"), "肥");
    assert_eq!(e.segments("살찔비")[0].candidates[0].text, "肥");
    assert_eq!(e.convert("살찔비")[0].note, "살찔 비 · 訓音");
    assert!(offers(&e, "비살찔", "肥"));
    assert!(offers(&e, "살찔 비", "肥"));
    assert!(offers(&e, "비 살찔", "肥"));
    // The other glosses of a character, and 두음법칙 forms of its reading.
    assert!(offers(&e, "재물용", "用"));
    assert!(offers(&e, "어질량", "良"));
    assert!(offers(&e, "어질양", "良"));
    // One span to pick from, inside a sentence too.
    assert_eq!(e.segments("비살찔").len(), 1);
    assert_eq!(e.segments("살찔 비").len(), 1);
    assert_eq!(e.convert("살찔비 사랑")[0].text, "肥 愛");
    // An ordinary reading stays first.
    assert_eq!(first("가집"), "家集");
    assert_eq!(first("날일"), "日일");
    assert!(offers(&e, "날일", "日"));
    // A short match may be a plain word, so it is only offered.
    assert_eq!(first("반짝"), "반짝");
    assert_eq!(first("둘이"), "둘이");
    assert!(offers(&e, "쓸용", "用"));
    assert_eq!(first("우물정"), "井");
    // Plain spaced words are untouched.
    assert_eq!(e.segments("나라를 사랑").len(), 3);
    // The keys are no words of their own: 肥 still reads 비, and 國漢文混用
    // keeps a native phrase in Hangul first.
    assert_eq!(e.read("肥"), "비");
    let sino = engine(Options { sino_only: true, ..Options::default() });
    assert_eq!(sino.convert("집가")[0].text, "집가");
    assert!(offers(&sino, "집가", "家"));
}

#[test]
fn the_bound_noun_su_is_written_with_hand() {
    let e = engine(Options::default());
    assert_eq!(e.convert("수")[0].text, "手");
    assert_eq!(e.convert("수도")[0].text, "手도");
    assert!(e.convert("할 수 있다")[0].text.contains('手'));
    // Typed without a space, 수 and 있다 or 없다 stay two choices.
    for (input, su, rest, other) in [
        ("수있다", "수", "有다", "在ㅆ다"),
        ("수없는", "수", "無는", "莫는"),
        ("수있겠네요", "수", "有겠네요", "在겠네요"),
        ("볼수없었던", "수", "無었던", "莫었던"),
    ] {
        let segs = e.segments(input);
        let seg = |w: &str| segs.iter().find(|s| s.reading == w).unwrap_or_else(|| panic!("{input}: {segs:?}"));
        assert_eq!(seg(su).candidates[0].text, "手", "{input}");
        assert!(seg(su).candidates.iter().any(|c| c.text == "수"), "{input}");
        let tail = segs.last().unwrap();
        assert!(tail.candidates.iter().any(|c| c.text == rest), "{input}");
        assert!(tail.candidates.iter().any(|c| c.text == other), "{input}");
    }
    assert_eq!(e.segments("수없다").len(), 1);
    assert!(e.segments("수없다")[0].candidates.iter().any(|c| c.text == "手無다"));
    // Whole words keep their own reading.
    assert_eq!(e.convert("옥수수")[0].text, "荳");
    assert_eq!(e.convert("수학")[0].text, "數學");
    assert_eq!(e.convert("선수")[0].text, "選手");
    // 手 on its own still reads 손 unless the bound noun fits.
    assert_eq!(e.read("手을"), "손을");
}

#[test]
fn demonstratives_take_every_spelling_of_geot() {
    let e = engine(Options::default());
    for (ko, han) in [("이", "此"), ("그", "其"), ("저", "彼")] {
        assert_eq!(e.convert(&format!("{ko}것"))[0].text, format!("{han}것"));
        assert_eq!(e.convert(&format!("{ko}거"))[0].text, format!("{han}거"));
        for w in ["事ㅅ", "事", "件", "物", "唟"] {
            assert!(offers(&e, &format!("{ko}것"), &format!("{han}{w}")), "{ko}것 → {han}{w}");
        }
        for w in ["事", "件", "物"] {
            assert!(offers(&e, &format!("{ko}거"), &format!("{han}{w}")), "{ko}거 → {han}{w}");
        }
        // With a particle too, and back again.
        assert!(offers(&e, &format!("{ko}것을"), &format!("{han}物을")));
        assert_eq!(e.read(&format!("{han}物을")), format!("{ko}것을"));
    }
}
