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

/// The texts of a segment's candidates.
fn candidates(e: &Engine, input: &str) -> Vec<String> {
    e.segments(input)[0].candidates.iter().map(|c| c.text.clone()).collect()
}

/// Whether `want` appears in `list` in this order (other items may lie between).
fn in_order(list: &[String], want: &[&str]) -> bool {
    let mut at = 0;
    for w in want {
        match list[at..].iter().position(|c| c == w) {
            Some(i) => at += i + 1,
            None => return false,
        }
    }
    true
}

#[test]
fn other_okurigana_follow_the_dictionary_form() {
    let e = engine(Options::default());
    for (input, want) in [
        ("너무", &["超무", "超ㅜ", "超"][..]),
        ("처음", &["初음", "初ㅁ", "初"]),
        ("어떻게", &["何떻게", "何게", "何ㅔ", "何"]),
        ("말씀", &["言씀", "言ㅁ", "言"]),
        ("오히려", &["尙려", "尙히려", "尙ㅕ", "尙"]),
        // The particle follows the chosen form.
        ("너무도", &["超무도", "超ㅜ도", "超도"]),
        ("처음에", &["初음에", "初ㅁ에", "初에"]),
        ("말씀을", &["言씀을", "言ㅁ을", "言을"]),
        // An inflected stem keeps its ending; the rest of the stem comes back.
        ("가까웠다", &["近웠다", "近까웠다"]),
        ("가깝고", &["近고", "近깝고"]),
    ] {
        let got = candidates(&e, input);
        // Another entry may already write a form in the dictionary, which puts
        // it earlier, so presence is what holds here.
        assert!(want.iter().all(|w| got.iter().any(|c| c == w)), "{input}: {want:?} not all in {got:?}");
        // The dictionary form stays first and nothing is listed twice.
        assert_eq!(got[0], want[0], "{input}");
        let mut seen = std::collections::HashSet::new();
        assert!(got.iter().all(|c| seen.insert(c)), "{input}: duplicates in {got:?}");
        assert_eq!(e.convert(input)[0].text, want[0], "{input}");
    }
    // 過무 can be typed for 너무.
    assert!(candidates(&e, "너무").iter().any(|c| c == "過무"));
    assert!(candidates(&e, "너무도").iter().any(|c| c == "過무도"));
    // The dictionary forms of a word come before its other forms.
    let got = candidates(&e, "오히려");
    assert!(got.iter().position(|c| c == "尙려") < got.iter().position(|c| c == "尙히려"));
    // Attested spellings of every reading come before constructed ones.
    let got = candidates(&e, "이것");
    assert!(got.iter().position(|c| c == "此物") < got.iter().position(|c| c == "此ㅅ"), "{got:?}");
}

#[test]
fn one_syllable_words_have_the_letter_and_the_character() {
    let e = engine(Options::default());
    let got = candidates(&e, "나");
    assert!(in_order(&got, &["我", "我ㅏ"]), "{got:?}");
    let got = candidates(&e, "달");
    assert!(in_order(&got, &["月", "月ㄹ"]), "{got:?}");
    // The form that exists already is not offered twice.
    let got = candidates(&e, "좀");
    assert_eq!(got.iter().filter(|c| *c == "少ㅁ").count(), 1, "{got:?}");
}

#[test]
fn forms_of_lists_dictionary_forms_only() {
    let e = engine(Options::default());
    let find = |lemma: &str, hanja: &str| e.natives.iter().position(|n| n.lemma == lemma && n.hanja == hanja).unwrap_or_else(|| panic!("{lemma} {hanja}"));
    let neomu = e.forms_of(find("너무", "超"));
    assert!(!neomu.is_empty() && neomu.iter().all(|(w, _)| w.starts_with("超무")), "{neomu:?}");
    let chaeum = e.forms_of(find("처음", "初"));
    assert!(chaeum.iter().all(|(w, _)| w.starts_with("初음")), "{chaeum:?}");
    let gakkapda = e.forms_of(find("가깝다", "近"));
    assert!(gakkapda.iter().all(|(w, _)| !w.starts_with("近까") && !w.starts_with("近깝")), "{gakkapda:?}");
    // Nothing the strategies add is a dictionary form of these entries.
    for (forms, extra) in [(&neomu, ["超ㅜ", "超"]), (&chaeum, ["初ㅁ", "初"])] {
        assert!(forms.iter().all(|(w, _)| !extra.contains(&w.as_str())), "{forms:?}");
    }
}

#[test]
fn okurigana_variants_read_back() {
    let e = engine(Options::default());
    for (written, read) in [
        ("超ㅜ", "너무"), ("過무", "너무"), ("超무", "너무"), ("初ㅁ에", "처음에"),
        ("尙히려", "오히려"), ("近까웠다", "가까웠다"), ("近깝고", "가깝고"), ("言씀을", "말씀을"),
        ("何ㅔ", "어떻게"),
    ] {
        assert_eq!(e.read(written), read, "{written}");
    }
    // The Hanja alone is read by sound, as before.
    assert_eq!(e.read("超"), "초");
    assert_eq!(e.read("初에"), "초에");
}

#[test]
fn demonstratives_of_place_side_and_time() {
    let e = engine(Options::default());
    assert_eq!(e.convert("여기")[0].text, "此處");
    assert_eq!(e.convert("거기")[0].text, "其處");
    assert_eq!(e.convert("저기")[0].text, "彼處");
    assert_eq!(e.convert("여기서")[0].text, "此處서");
    assert_eq!(e.convert("어디서")[0].text, "何處서");
    assert_eq!(e.convert("여기서도")[0].text, "此處서도");
    assert_eq!(e.convert("서")[0].text, "立");
    // The chat writes 此番 twelve times to 今番 once.
    assert_eq!(e.convert("이번")[0].text, "此番");
    assert!(offers(&e, "이번", "今番"));
    // Words ending in 서 keep their own reading.
    assert_eq!(e.convert("나서")[0].text, "出서");
    assert_eq!(e.convert("이어서")[0].text, "續어서");
    assert_eq!(e.convert("이쪽")[0].text, "此쪽");
    assert!(offers(&e, "저번", "這番"));
    assert_eq!(e.convert("거기에")[0].text, "其處에");
    for (input, want) in [
        ("여기", "此所"), ("여기", "此기"), ("거기", "其기"), ("저기", "彼所"), ("저기", "那"),
        ("이곳", "此處"), ("이곳", "此곳"), ("저곳", "彼處"), ("이쪽", "此側"), ("저쪽", "彼方"),
        ("그쪽", "其쪽"), ("저때", "彼時"), ("이날", "此日"), ("이날", "是日"), ("이번", "此番"),
        ("저번", "彼番"), ("이리", "此리"), ("저리", "彼리"),
    ] {
        assert!(offers(&e, input, want), "{input} → {want}");
    }
    // 여기다 and its forms are the verb.
    assert!(offers(&e, "여기다", "思다"));
}

#[test]
fn verb_variants_keep_the_syllables_of_the_word() {
    let e = engine(Options::default());
    // The syllables from the second on, joined as typed.
    for (input, want) in [("마신", "飮신"), ("마셔라", "飮셔라"), ("바빠", "忙빠"), ("바빴다", "忙빴다")] {
        let got = candidates(&e, input);
        assert!(got.iter().any(|c| c == want), "{input} → {want} in {got:?}");
    }
    // Nothing is made of a composed stem and a rendered ending.
    for bad in ["飮시ㄴ", "飮시라", "近깝ㅂ고", "近까았다"] {
        assert!(!candidates(&e, "마신").contains(&bad.to_string()) && !candidates(&e, "마셔라").contains(&bad.to_string()), "{bad}");
    }
    // The style options leave the variants as they are.
    for o in [
        Options { coda_always: true, ..Options::default() },
        Options { style: Style::Morpheme, ..Options::default() },
        Options { fragment: Fragment::Letter, ..Options::default() },
        Options { resyllabify: true, ..Options::default() },
    ] {
        let e = engine(o);
        for (input, want) in [("가깝고", "近깝고"), ("가까웠다", "近까웠다"), ("마셔라", "飮셔라"), ("바빠", "忙빠")] {
            let got = candidates(&e, input);
            assert!(got.iter().any(|c| c == want), "{input} → {want} in {got:?}");
        }
        for (input, bad) in [("가깝고", "近깝ㅂ고"), ("가까웠다", "近까았다")] {
            assert!(!candidates(&e, input).iter().any(|c| c == bad), "{input} → {bad}");
        }
    }
}

#[test]
fn reading_variants_does_not_depend_on_the_options() {
    for o in [
        Options::default(),
        Options { coda_always: true, ..Options::default() },
        Options { style: Style::Morpheme, ..Options::default() },
        Options { fragment: Fragment::Letter, ..Options::default() },
        Options { resyllabify: true, ..Options::default() },
    ] {
        let e = engine(o.clone());
        for (written, read) in [("近깝고", "가깝고"), ("近까웠다", "가까웠다"), ("忙빠", "바빠"), ("忙빴다", "바빴다")] {
            assert_eq!(e.read(written), read, "{written} with {o:?}");
        }
    }
}

#[test]
fn the_group_reads_as_its_own_name() {
    let e = engine(Options::default());
    assert_eq!(e.convert("풀어읽기모임")[0].text, "釋讀會");
    assert_eq!(e.read("釋讀會"), "풀어읽기모임");
    assert_eq!(e.read("釋讀會의"), "풀어읽기모임의");
    assert_eq!(e.convert("석독회")[0].text, "釋讀會");
    assert_eq!(e.read("釋讀"), "석독");
}

#[test]
fn compound_verbs_take_the_meaning_of_each_part() {
    let e = engine(Options::default());
    assert_eq!(e.convert("내려받기")[0].text, "下려受기");
    assert_eq!(e.convert("내보내거나")[0].text, "出보내거나");
}

#[test]
fn interrogatives_in_hanja() {
    let e = engine(Options::default());
    for (input, first) in [
        ("왜", "何故"), ("누구", "誰"), ("누가", "誰가"), ("언제", "何時"), ("어디", "何處"), ("어디서", "何處서"),
        ("무엇", "何엇"), ("뭐", "何ㅓ"), ("어떻게", "何떻게"), ("어떤", "何떤"), ("어느", "何느"), ("무슨", "何슨"),
        ("어찌", "何찌"), ("얼마", "幾何"), ("얼마나", "幾何나"), ("어쩌죠", "何쩌죠"), ("어떡하죠", "何떡하죠"), ("뭐야", "何ㅓ야"), ("몇", "幾"),
    ] {
        assert_eq!(e.convert(input)[0].text, first, "{input}");
    }
    for (input, other) in [
        ("왜", "何以"), ("왜", "胡"), ("어디", "何所"), ("어떻게", "如何"), ("어찌", "奈何"), ("얼마나", "何마나"), ("얼마나", "幾나"),
        ("어째서", "何故"), ("어째서", "何째서"), ("누구", "孰"), ("언제", "何日"), ("무엇", "何物"),
    ] {
        assert!(offers(&e, input, other), "{input} → {other}");
    }
    assert_eq!(e.convert("왜요")[0].text, "何故요");
    assert_eq!(e.convert("언제쯤")[0].text, "何時쯤");
    assert_eq!(e.convert("누굴")[0].text, "誰ㄹ");
    assert_eq!(e.convert("이에요")[0].text, "乃요");
    assert_eq!(e.read("何故"), "왜");
}

#[test]
fn the_rest_of_the_demonstrative_family() {
    let e = engine(Options::default());
    for (input, first) in [
        ("그분", "其分"), ("이편", "此便"), ("저편", "彼便"), ("어느쪽", "何느쪽"),
        ("그이", "其人"), ("이이", "此人"), ("이달", "今月"), ("다음주", "來週"), ("이번주", "今週"),
        ("너희", "汝等"), ("이러다", "此러다"),
    ] {
        assert_eq!(e.convert(input)[0].text, first, "{input}");
    }
    for (input, other) in [("이분", "此分"), ("저들", "彼等"), ("저만큼", "彼만큼"), ("저대로", "彼대로"), ("저처럼", "彼처럼"), ("저희", "吾等"), ("저희", "僕等")] {
        assert!(offers(&e, input, other), "{input} → {other}");
    }
    // Words that merely contain 그이 or 저이 stay as they are.
    assert_eq!(e.convert("로그인")[0].text, "로그인");
    assert_eq!(e.convert("먼저일")[0].text, "먼저일");
    // 저 still reads as the humble pronoun on its own.
    assert_eq!(e.convert("저는")[0].text, "自는");
}

#[test]
fn bundle_is_written_with_dan() {
    let e = engine(Options::default());
    assert_eq!(e.convert("뭉치")[0].text, "團");
    assert_eq!(e.convert("말뭉치")[0].text, "言團");
    assert_eq!(e.read("言團"), "말뭉치");
    assert_eq!(e.read("團"), "뭉치");
}

#[test]
fn yeoreo_has_every_hanja_for_many() {
    let e = engine(Options::default());
    assert_eq!(e.convert("여러")[0].text, "諸");
    for want in ["庶", "庶러", "衆", "數", "多"] {
        assert!(offers(&e, "여러", want), "{want}");
    }
    assert!(offers(&e, "여러분", "庶러位"));
    assert!(offers(&e, "여러분", "諸公"));
    assert_eq!(e.convert("여러분")[0].text, "諸位");
    assert_eq!(e.convert("여러번")[0].text, "諸番");
}

#[test]
fn a_one_syllable_word_keeps_each_hanjas_forms_together() {
    let e = engine(Options::default());
    let got: Vec<String> = e.segments("좀")[0].candidates.iter().map(|c| c.text.clone()).collect();
    assert_eq!(&got[..2], ["少ㅁ", "少"], "{got:?}");
    let pos = |w: &str| got.iter().position(|c| c == w).unwrap_or_else(|| panic!("{w}: {got:?}"));
    assert_eq!(pos("聊ㅁ"), pos("聊") + 1, "{got:?}");
    assert_eq!(e.convert("좀")[0].text, "少ㅁ");
}

#[test]
fn the_bound_noun_teo() {
    let e = engine(Options::default());
    assert_eq!(e.convert("터")[0].text, "基");
    assert_eq!(e.convert("테니")[0].text, "址ㅣ니");
    assert_eq!(e.convert("텐데")[0].text, "址ㅣㄴ데");
    for (input, want) in [("테니", "테니"), ("테니", "基ㅣ니"), ("테니", "筈ㅣ니"), ("테니", "所ㅣ니"), ("텐데", "텐데"), ("텐데", "筈ㅣㄴ데")] {
        assert!(offers(&e, input, want), "{input} → {want}");
    }
    assert_eq!(e.read("基ㅣ니"), "테니");
    assert_eq!(e.read("址ㅣ니"), "테니");
    // 터 alone takes no particles, so 부터 is left whole.
    assert_eq!(e.convert("부터인데요")[0].text, "부터인데요");
    assert_eq!(e.convert("집터")[0].text, "垈");
    // 基 in a word stays Sino-Korean.
    assert_eq!(e.read("基礎"), "기초");
}

#[test]
fn spellings_from_the_chat_on_4_november() {
    let e = engine(Options::default());
    assert_eq!(e.convert("괜찮네요")[0].text, "不空然네요");
    assert!(offers(&e, "괜찮다", "不空然다"));
    assert!(offers(&e, "이런", "此樣"));
    assert_eq!(e.convert("이런")[0].text, "此런");
}

#[test]
fn readings_of_the_chat_of_2026_11_04() {
    let e = engine(Options::default());
    for (input, want) in [("귀찮다", "不貴다"), ("괜찮네요", "不公然네요"), ("가없다", "邊無다"), ("빈터", "空基"), ("빈터", "空墟"), ("터", "址"), ("터", "墟")] {
        assert!(offers(&e, input, want), "{input} → {want}");
    }
    assert_eq!(e.convert("괜찮다")[0].text, "不空然다");
    assert_eq!(e.convert("집터")[0].text, "垈");
}
#[test]
fn i_after_hanja_is_also_written_as_a_syllable() {
    let e = engine(Options::default());
    for (input, plain, spelled) in [
        ("테니", "址ㅣ니", "址이니"),
        ("텐데", "址ㅣㄴ데", "址인데"),
        ("내", "我ㅣ", "我이"),
        ("내가", "我ㅣ가", "我이가"),
        ("게", "事ㅣ", "事이"),
    ] {
        let list: Vec<String> = e.convert(input).into_iter().map(|c| c.text).collect();
        let at = list.iter().position(|t| t == plain).unwrap_or_else(|| panic!("{input}: {list:?}"));
        assert_eq!(list.get(at + 1).map(String::as_str), Some(spelled), "{input}: {list:?}");
    }
    assert_eq!(e.read("址이니"), "터이니");
    // Nothing else is pushed out, and a final ㅆ joins the syllable.
    assert!(offers(&e, "내", "吾ㅐ"));
    assert!(offers(&e, "냈다", "出있다"));
    assert!(!offers(&e, "냈다", "出이ㅆ다"));
    // A Hanja with the particle 이 still reads by its sound.
    assert_eq!(e.read("脚이"), "각이");
}

#[test]
fn the_plural_is_also_written_with_hanja() {
    let e = engine(Options::default());
    for (input, first, more) in [
        ("사람들", "人들", ["人等", "人達"]),
        ("사람들은", "人들은", ["人等은", "人們은"]),
        ("학생들", "學生들", ["學生等", "學生們"]),
        ("친구들이", "親舊들이", ["親舊等이", "親舊們이"]),
    ] {
        assert_eq!(e.convert(input)[0].text, first, "{input}");
        for want in more {
            assert!(offers(&e, input, want), "{input} → {want}");
            assert!(e.segments(input).iter().any(|s| s.candidates.iter().any(|c| c.text == want)), "{input} → {want}");
        }
    }
    // A word that is already plural comes first without a second one.
    assert_eq!(e.convert("우리들")[0].text, "我等");
    assert_eq!(e.convert("너희들")[0].text, "汝等");
    assert!(offers(&e, "우리들", "我等들"));
    // 들 of the verb 들다 and of a word's own last syllable is left alone.
    assert!(!offers(&e, "손들어", "手等어"));
    assert!(!offers(&e, "아들", "子等"));
    assert!(!offers(&e, "물들이", "染等이"));
    // 平等 is not plural, and other words stay in the list.
    assert_eq!(e.convert("평등들")[0].text, "平等들");
    assert!(offers(&e, "길들이", "馴"));
    assert!(offers(&e, "뎌들", "伊들"));
    // Reading back.
    for (written, reading) in [("學生等", "학생들"), ("學生們은", "학생들은"), ("學生達", "학생들"), ("平等", "평등"), ("到達", "도달")] {
        assert_eq!(e.read(written), reading, "{written}");
    }
    assert_eq!(e.read("人等은"), e.read("人들은"));
}

#[test]
fn the_adverb_kkwae_is_hanja_inside_a_phrase() {
    let e = engine(Options::default());
    for (input, want) in [("꽤", "頗"), ("꽤많다", "頗多다"), ("꽤좋다", "頗好다"), ("꽤나", "頗나")] {
        assert_eq!(e.convert(input)[0].text, want, "{input}");
    }
    assert!(offers(&e, "꽤많다", "꽤多다"));
}

#[test]
fn sino_words_with_their_own_row_convert_in_the_sino_mode() {
    let e = engine(Options { sino_only: true, ..Options::default() });
    assert_eq!(e.convert("물론")[0].text, "勿論");
    assert_eq!(e.convert("그건물론")[0].text, "그건勿論");
    assert_eq!(e.convert("사실")[0].text, "事實");
    assert_eq!(e.convert("일단은")[0].text, "一旦은");
    // Native words stay in Hangul.
    assert_eq!(e.convert("하늘")[0].text, "하늘");
    assert_eq!(e.convert("수")[0].text, "수");
}

#[test]
fn the_past_is_also_written_with_ssang_siot() {
    let e = engine(Options::default());
    for (input, want) in [("있었다", "在ㅆ다"), ("있었어요", "在ㅆ어요"), ("먹었다", "食ㅆ다"), ("먹었습니다", "食ㅆ습니다"), ("받았다", "受ㅆ다")] {
        assert!(e.segments(input).iter().any(|s| s.candidates.iter().any(|c| c.text == want)), "{input} → {want}");
    }
    // The dictionary's form stays first, and 在ㅆ다 for 있다 stays.
    assert_eq!(e.convert("먹었다")[0].text, "食었다");
    assert!(offers(&e, "있다", "在ㅆ다"));
    assert_eq!(e.read("在ㅆ다"), "있다");
    // It comes right after the form it spells.
    let list: Vec<String> = e.segments("마음먹었다")[0].candidates.iter().map(|c| c.text.clone()).collect();
    let at = list.iter().position(|t| t == "決었다").unwrap();
    assert_eq!(list[at + 1], "決ㅆ다", "{list:?}");
    // The phrase list keeps the form with the tense in it.
    assert!(e.convert("있었다").iter().any(|c| c.text == "有ㅆ었다"));
}

#[test]
fn sino_suffixes_attach_to_sino_words() {
    let e = engine(Options::default());
    for (input, want) in [("표시용", "表示用"), ("표시용으로", "表示用으로"), ("업무용", "業務用"), ("현실적", "現實的"), ("현실적으로", "現實的으로")] {
        assert_eq!(e.convert(input)[0].text, want, "{input}");
    }
    // Homophones with their own use are not suffixes, and a greeting takes no suffix.
    for (input, wrong) in [("친구형", "親舊型"), ("전주성", "全州性"), ("가정식", "家庭式"), ("은행론", "銀行論"), ("안녕용", "安寧用"), ("죄송용", "罪悚用"), ("가세용", "加勢用")] {
        assert_ne!(e.convert(input)[0].text, wrong, "{input}");
    }
    // 用 before 龍 alone too, and particles stay particles.
    let list: Vec<String> = e.convert("용").into_iter().map(|c| c.text).collect();
    let at = |t: &str| list.iter().position(|x| x == t);
    assert!(at("用").unwrap() < at("龍").unwrap(), "{list:?}");
    assert_eq!(e.convert("학교가")[0].text, "學校가");
    assert_eq!(e.convert("학생인")[0].text, "學生인");
    assert_eq!(e.convert("하세용")[0].text, "하세용");
}

#[test]
fn a_compound_of_nouns_offers_every_combination() {
    let e = engine(Options::default());
    assert_eq!(e.convert("옛한글")[0].text, "古ㅅ韓㐎");
    let seg = e.segments("옛한글");
    assert_eq!(seg.len(), 1, "one list");
    for want in ["古ㅅ韓㐎", "古韓㐎", "古韓字", "古ㅅ韓字", "舊韓㐎", "옛韓㐎", "옛한글"] {
        assert!(seg[0].candidates.iter().any(|c| c.text == want), "{want}: {:?}", seg[0].candidates.iter().map(|c| &c.text).collect::<Vec<_>>());
    }
    assert!(e.segments("옛한글을")[0].candidates.iter().any(|c| c.text == "古韓字을"));
    // Phrases keep their segments: a verb, a determiner, a pronoun or adverb, a
    // particle in the middle, an unknown syllable.
    for phrase in ["사람이좋다", "그사람", "두사람", "매우빨리", "오늘내가", "파일에한글", "뷁한글"] {
        assert!(e.segments(phrase).len() >= 2, "{phrase}");
    }
    assert_eq!(e.segments("봄바람").len(), 1);
    // The old spelling stays in the phrase list and in third place.
    assert!(e.convert("친구시간을").iter().any(|c| c.text.ends_with("時間ᄋᆞᆯ")));
    assert!(e.segments("친구시간을")[0].candidates.iter().position(|c| c.text.ends_with("時間ᄋᆞᆯ")).unwrap() <= 2);
}

#[test]
fn finding_out_and_its_misspelling() {
    let e = engine(Options::default());
    assert_eq!(e.convert("찾아냈어요")[0].text, "尋出ㅆ어요");
    assert_eq!(e.convert("찿아냈어요")[0].text, "尋出ㅆ어요");
    assert!(offers(&e, "찾아냈어요", "探아出ㅆ어요"));
    assert_eq!(e.convert("찿아")[0].text, "探아");
}

#[test]
fn more_readings_from_the_chat_of_2026_11_04() {
    let e = engine(Options::default());
    for (input, want) in [("쥐락펴락", "握락伸락"), ("셈치고", "算打고"), ("곧", "直"), ("내려받기", "下受")] {
        assert!(offers(&e, input, want), "{input} → {want}");
    }
    assert_eq!(e.convert("내려받았다")[0].text, "下려受았다");
}

#[test]
fn the_plural_takes_more_particles() {
    let e = engine(Options::default());
    for (input, first, more) in [("사람들한테", "人들한테", "人等한테"), ("학생들보다", "學生들보다", "學生們보다"), ("친구들이랑", "親舊들이랑", "親舊達이랑"), ("선생님들처럼", "先生主들처럼", "")] {
        assert_eq!(e.convert(input)[0].text, first, "{input}");
        assert!(more.is_empty() || offers(&e, input, more), "{input} → {more}");
    }
}

#[test]
fn a_compound_part_keeps_its_own_character_first() {
    let e = engine(Options::default());
    let list: Vec<String> = e.segments("옛한글")[0].candidates.iter().map(|c| c.text.clone()).collect();
    let at = |t: &str| list.iter().position(|x| x == t).unwrap();
    assert!(at("古ㅅ韓字") < at("古ㅅ漢字"), "{list:?}");
}

#[test]
fn the_bound_noun_de_is_a_place() {
    let e = engine(Options::default());
    assert_eq!(e.convert("만드는 데도").into_iter().next().unwrap().text, "作는 處도");
    assert!(offers(&e, "데도", "所도"));
    assert!(offers(&e, "데", "燒"));
    // Inside a word it stays the ending.
    for (input, want) in [("만드는데도", "作는데도"), ("쓰는데", "書는데"), ("하는데", "하는데"), ("그런데", "然ㄴ데"), ("인데", "인데")] {
        assert_eq!(e.convert(input)[0].text, want, "{input}");
    }
}
