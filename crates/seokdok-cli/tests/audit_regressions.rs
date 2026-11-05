//! First candidates fixed by the spelling audit, with the shipped dictionaries.
//! Each case names the input and the text of the first candidate.

use seokdok_core::{Engine, Options};
use std::path::Path;

fn engine() -> Engine {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
    Engine::from_dir(&dir, Options::default()).expect("dict").0
}

/// Every input's first candidate must equal the wanted text.
fn first(cases: &[(&str, &str)]) {
    let e = engine();
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(|(input, want)| {
            let got = e.convert(input).into_iter().next().map(|c| c.text).unwrap_or_default();
            (got != *want).then(|| format!("{input}: want {want}, got {got}"))
        })
        .collect();
    assert!(wrong.is_empty(), "first candidates differ:\n{}", wrong.join("\n"));
}

#[test]
fn copula_after_a_vowel_final_noun() {
    first(&[
        ("하늘입니다", "天입니다"), ("학생입니다", "學生입니다"), ("하늘이다", "天이다"), ("학생이니까", "學生이니까"),
        ("학생이었다", "學生이었다"), ("사전이라면", "辭典이라면"), ("학생인데", "學生인데"), ("하늘일까요", "天일까요"),
        ("하나이다", "一이다"), ("견해입니다", "見解입니다"), ("표기입니다", "表記입니다"),
    ]);
}

#[test]
fn postposition_chains() {
    first(&[
        ("하늘에서만", "天에서만"), ("하늘까지만", "天까지만"), ("사전께서는", "辭典께서는"), ("자신께서는", "自身께서는"),
        ("사전과의", "辭典과의"), ("하늘이라니", "天이라니"), ("학생이래요", "學生이래요"), ("사전이라서는", "辭典이라서는"),
        ("일본에서만", "日本에서만"), ("일본만의", "日本만의"),
    ]);
}

#[test]
fn copula_does_not_follow_an_adverb_or_a_lone_pronoun() {
    // 아니+라니 is 아니다+라니; 나이+다 is a word, not 나+이다.
    first(&[("아니라니", "아니라니"), ("나이다", "年다"), ("저입니다", "彼人ㅂ니다")]);
}

#[test]
fn editorial_rows_for_common_sino_words() {
    first(&[
        ("꽤", "頗"), ("사실", "事實"), ("물론", "勿論"), ("일단", "一旦"), ("세기", "世紀"), ("매일", "每日"),
        ("시대", "時代"), ("고려", "高麗"), ("모음", "母音"), ("등", "等"), ("번", "番"), ("년", "年"),
        ("위해", "爲해"), ("위한", "爲한"), ("별로", "別로"), ("눈", "目"),
    ]);
}

/// Every input must offer the wanted text among its candidates.
fn offers(cases: &[(&str, &str)]) {
    let e = engine();
    let missing: Vec<String> = cases
        .iter()
        .filter(|(input, want)| !e.convert(input).iter().any(|c| &c.text == want))
        .map(|(input, want)| format!("{input}: {want} not offered"))
        .collect();
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn a_whole_noun_beats_a_bare_verb_form() {
    first(&[
        ("눈", "目"), ("나라", "國"), ("길", "道"), ("꿈", "夢"), ("봄", "春"), ("불", "火"), ("쌀", "米"),
        ("곰", "熊"), ("풀", "草"), ("술", "酒"), ("사실", "事實"), ("물론", "勿論"), ("일단", "一旦"),
        ("세기", "世紀"), ("매일", "每日"), ("시대", "時代"), ("자기", "自己"), ("사진", "寫眞"),
        ("거의", "殆"), ("아니라", "非라"), ("잘", "良ㄹ"),
    ]);
}

#[test]
fn adnominal_verb_forms_keep_working() {
    first(&[("작은", "小은"), ("남은", "殘은"), ("긴", "長ㄴ"), ("빈", "空ㄴ"), ("간", "去ㄴ"), ("갈", "去ㄹ"), ("마실", "飮ㄹ")]);
    offers(&[("자신", "自身"), ("눈이", "目이"), ("눈이", "雪이")]);
}

#[test]
fn loanwords_stay_in_hangul() {
    let words = [
        "페이지", "파일", "메시지", "버전", "에러", "이모지", "바나나", "토큰", "모니터", "피자", "링크", "코드", "사이트",
        "로그인", "빌드",
    ];
    let mut cases: Vec<(String, String)> = Vec::new();
    for w in words {
        cases.push((w.to_string(), w.to_string()));
        for p in ["를", "이", "은", "에서", "와", "입니다"] {
            // A particle that agrees with the final letter of the word.
            let vowel = !matches!(w, "파일" | "버전" | "토큰" | "로그인");
            let ok = match p {
                "를" | "은" => if vowel { p == "를" } else { p == "은" },
                "이" => !vowel,
                "와" => vowel,
                _ => true,
            };
            if ok {
                cases.push((format!("{w}{p}"), format!("{w}{p}")));
            }
        }
    }
    cases.push(("로그인합니다".into(), "로그인합니다".into()));
    cases.push(("빌드하면".into(), "빌드하면".into()));
    let cases: Vec<(&str, &str)> = cases.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    first(&cases);
}

#[test]
fn strong_address_and_several_kinds() {
    first(&[
        ("강한", "強한"), ("강하다", "強하다"), ("강해서", "強해서"), ("강하고", "強하고"), ("주소", "住所"), ("주소를", "住所를"),
        ("여러가지", "여러 가지"), ("여러가지를", "여러 가지를"),
    ]);
}

#[test]
fn a_particle_typed_alone_stays_hangul() {
    first(&[
        ("에", "에"), ("에서", "에서"), ("까지", "까지"), ("는", "는"), ("와", "와"), ("인데", "인데"), ("입니다", "입니다"),
        ("이라면", "이라면"), ("이라니", "이라니"), ("조차", "조차"), ("에는", "에는"),
    ]);
}

#[test]
fn a_particle_that_is_also_a_word_converts_as_the_word() {
    first(&[
        ("이", "此"), ("나", "我"), ("보다", "見다"), ("들", "入ㄹ"), ("같이", "如이"), ("대로", "如"), ("만큼", "許"),
        ("들은", "聽은"),
    ]);
}

#[test]
fn an_editorial_whole_word_beats_a_pronoun_with_a_particle() {
    first(&[("저들", "彼等"), ("저만큼", "彼만큼"), ("저대로", "彼대로"), ("저처럼", "彼처럼")]);
    // 저 with a particle that no editorial word spells stays the humble 自.
    first(&[("저는", "自는"), ("저도", "自도"), ("저를", "自를"), ("저의", "自의"), ("저만", "自만")]);
}

#[test]
fn endings_the_chat_runs_into_the_next_word() {
    first(&[
        ("그런듯", "然ㄴ듯"), ("있다란", "有다란"), ("재밌는거", "滋味ㅆ는거"), ("같은거", "如은거"), ("말고는", "勿고는"),
        ("먹고도", "食고도"), ("있을지도", "有을지도"), ("만드는건", "作는건"), ("잠시만요", "暫時만요"), ("적었을때", "記었을때"),
        ("같으나", "如으나"), ("없으려나", "無으려나"), ("먹냐고", "食냐고"), ("보지마", "見지마"), ("그럴지도", "然ㄹ지도"),
        ("그럴수도", "然ㄹ수도"), ("모르겠지만요", "不知겠지만요"), ("읽는것", "讀는것"), ("괜찮은듯", "不空然은듯"),
        ("있는데요", "有는데요"),
    ]);
    // The same letters inside a word stay that word.
    first(&[("근거", "根據"), ("바다란", "海란"), ("바다", "海")]);
}

#[test]
fn an_adverbial_ending_takes_a_particle() {
    first(&[
        ("안타깝게도", "惜깝게도"), ("아쉽게도", "惜게도"), ("슬프게도", "悲게도"), ("좋게도", "好게도"),
        ("크게는", "大게는"), ("많게만", "多게만"),
    ]);
}

#[test]
fn a_helper_verb_runs_on_from_a_connective() {
    first(&[
        ("보고싶어요", "見고欲어요"), ("먹고싶은", "食고欲은"), ("보고있어요", "見고有어요"), ("하고있어요", "하고有어요"),
        ("먹지않아", "食지不ㅎ아"), ("하지않아", "하지不ㅎ아"), ("먹어봤어요", "食어見ㅆ어요"), ("써보세요", "書見세요"),
        ("해주세요", "해授세요"), ("적어놓은", "記어置은"),
    ]);
    // The same letters elsewhere stay as they were.
    first(&[("보고", "見고"), ("마중물", "마중물"), ("했나보네요", "했나보네요"), ("해놨어", "해놨어"), ("지도", "指導")]);
}

#[test]
fn words_that_look_like_a_loanword_or_an_ending_keep_their_reading() {
    // 올리고 and 알리고 are verb forms, not the loanwords oligo- and Aligot.
    first(&[("올리고", "昇고"), ("알리고", "告리고"), ("가자", "去자"), ("그러나", "其러나")]);
    // The copula typed as its own word stays Hangul.
    first(&[("인듯", "인듯"), ("일지도", "일지도"), ("뜻인듯", "意인듯"), ("의미인가요", "意味인가요")]);
}

#[test]
fn example_is_written_in_hanja_with_a_particle() {
    first(&[("예를", "例를"), ("예가", "例가"), ("예로는", "例로는"), ("예로부터", "예로부터"), ("예에서", "例에서"), ("예의", "禮儀")]);
}

#[test]
fn the_pronoun_with_a_particle_is_not_a_particle_alone() {
    first(&[
        ("이를", "此를"), ("이는", "此는"), ("이가", "此가"), ("이도", "此도"), ("이의", "此의"), ("이에게", "此에게"),
        ("이에", "此에"), ("이와", "此와"), ("그를", "其를"), ("그는", "其는"), ("저를", "自를"), ("저는", "自는"),
    ]);
}

#[test]
fn nouns_that_a_bare_stem_would_take() {
    first(&[("돌", "石"), ("발", "足"), ("털", "毛"), ("감", "柿"), ("실", "絲")]);
    // No chat attestation of 蜂, and 살 stays the verb stem (生 22× in the chat against 歲 11×, 肉 8×, 膚 1×).
    first(&[("벌", "벌"), ("살", "生")]);
}

#[test]
fn a_copula_form_is_listed_once() {
    // Listed twice, it would fill the search's best paths with copies and push 第이다 out.
    offers(&[("집이다", "第이다"), ("집이다", "宙이다")]);
}

#[test]
fn context_decides_between_a_verb_and_a_noun_or_particle() {
    // 무어 is an interjection in the dictionary and still takes 라.
    first(&[("무어라", "何라"), ("무어라 했니", "何라 했니")]);
    // A particle that is also a verb form keeps the verb inside a sentence, and is offered both ways alone.
    first(&[("빨리 와", "速 來"), ("공을 치고", "공을 打고"), ("와", "와"), ("치고", "치고")]);
    offers(&[("와", "來"), ("치고", "打고")]);
    // An adnominal verb form before a bound noun keeps its verb; one word alone keeps the noun.
    first(&[("바람이 불 때", "風이 吹ㄹ 時"), ("길 것 같다", "長ㄹ 事ㅅ 如다"), ("불", "火"), ("길", "道"), ("불 바람", "火 風")]);
}

#[test]
fn book_is_written_in_hanja_with_or_without_a_particle() {
    first(&[("책", "冊"), ("책을", "冊을"), ("책이", "冊이"), ("책임", "責任"), ("책상", "冊床")]);
}

#[test]
fn contractions_stay() {
    first(&[("전", "自ㄴ"), ("뭔가", "何ㅓㄴ가"), ("무언가", "何ㄴ가"), ("날", "我ㄹ")]);
}

#[test]
fn an_is_negation_on_its_own() {
    let e = engine();
    assert_eq!(e.convert("안 돼요")[0].text, "不 成요");
    assert_eq!(e.convert("안")[0].text, "不");
    assert!(e.convert("안").iter().chain(e.segments("안")[0].candidates.iter()).any(|c| c.text == "內"));
    // With a particle it is still the inside of something.
    assert_eq!(e.convert("안에")[0].text, "內에");
    assert_eq!(e.convert("무언가")[0].text, "何ㄴ가");
}

#[test]
fn the_common_word_leads_a_homophone_set() {
    first(&[
        ("전역", "轉役"), ("의사", "醫師"), ("신상", "身上"), ("기물", "器物"), ("진사", "進士"),
        ("호두", "胡桃"), ("호두과자", "胡桃菓子"),
    ]);
    // 지도 stays open to -을지도.
    first(&[("먹을지도", "食을지도"), ("있을지도", "有을지도"), ("을지도", "을지도")]);
}
