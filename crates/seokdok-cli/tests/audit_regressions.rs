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
fn contractions_stay() {
    first(&[("전", "自ㄴ"), ("뭔가", "何ㅓㄴ가"), ("무언가", "何ㄴ가"), ("날", "吾ㄹ")]);
}
