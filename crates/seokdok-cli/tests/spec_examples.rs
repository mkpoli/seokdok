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
