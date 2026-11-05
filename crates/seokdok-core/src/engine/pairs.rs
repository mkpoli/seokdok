//! Word pairs: the verb next door decides between homophones (눈이 내린다 is
//! 雪, 눈이 아프다 is 目), which a run alone cannot.

use super::{Candidate, Segment};

const SUBJECT: &[&str] = &["", "이", "가", "은", "는", "도"];
const OBJECT: &[&str] = &["", "을", "를", "은", "는", "도"];
/// 을/를 only: 밤에 먹었다 is at night, and 손에 든 is in the hand.
const MARKED: &[&str] = &["을", "를"];

struct Pair {
    noun: &'static str,
    particles: &'static [&'static str],
    /// The next word's verb, as its first candidate's note names it.
    verb: &'static str,
    /// The noun's Hanja, or "" to leave the noun alone.
    hanja: &'static str,
    /// What the verb's spelling opens with, or "" to leave it alone.
    verb_opens: &'static str,
}

const fn pair(noun: &'static str, particles: &'static [&'static str], verb: &'static str, hanja: &'static str) -> Pair {
    Pair { noun, particles, verb, hanja, verb_opens: "" }
}

const PAIRS: &[Pair] = &[
    pair("눈", SUBJECT, "내리다", "雪"),
    pair("눈", SUBJECT, "오다", "雪"),
    pair("눈", SUBJECT, "쌓이다", "雪"),
    pair("눈", SUBJECT, "녹다", "雪"),
    pair("배", SUBJECT, "고프다", "腹"),
    pair("배", SUBJECT, "아프다", "腹"),
    pair("배", SUBJECT, "부르다", "腹"),
    pair("배", OBJECT, "타다", "舟"),
    pair("말", MARKED, "타다", "馬"),
    pair("말", SUBJECT, "달리다", "馬"),
    pair("다리", SUBJECT, "아프다", "脚"),
    pair("길", SUBJECT, "멀다", "道"),
    pair("길", SUBJECT, "막히다", "道"),
    pair("길", MARKED, "잃다", "道"),
    pair("길", MARKED, "걷다", "道"),
    pair("길", MARKED, "찾다", "道"),
    pair("해", MARKED, "끼치다", "害"),
    pair("해", MARKED, "입다", "害"),
    pair("사과", OBJECT, "먹다", "沙果"),
    pair("밤", MARKED, "먹다", "栗"),
    pair("밤", MARKED, "까다", "栗"),
    pair("차", OBJECT, "마시다", "茶"),
    pair("차", OBJECT, "타다", "車"),
    pair("차", MARKED, "몰다", "車"),
    Pair { noun: "예", particles: &["를"], verb: "들다", hanja: "例", verb_opens: "들" },
    Pair { noun: "손", particles: MARKED, verb: "들다", hanja: "", verb_opens: "擧" },
];

/// Whether a note names `verb`: "降 내리다 +(으)ㄴ다 · 訓讀".
fn names_verb(note: &str, verb: &str) -> bool {
    note.split(' ').any(|word| word == verb)
}

/// The pairs that `word` and a next word with this first note make.
fn rows<'a>(word: &'a str, note: &str) -> Vec<(&'a str, &'static Pair)> {
    PAIRS
        .iter()
        .filter_map(|p| {
            let rest = word.strip_prefix(p.noun)?;
            (p.particles.contains(&rest) && names_verb(note, p.verb)).then_some((rest, p))
        })
        .collect()
}

fn to_front(list: &mut [Candidate], fits: impl Fn(&str) -> bool) {
    if let Some(i) = list.iter().position(|c| fits(&c.text)) {
        list[..=i].rotate_right(1);
    }
}

/// Puts the spelling a pair asks for first on both sides. The noun's reading
/// may lie past the end of a short list, so it joins the list if it is missing.
fn apply(word: &str, left: &mut Vec<Candidate>, right: &mut [Candidate]) {
    let note = right.first().map_or(String::new(), |c| c.note.clone());
    for (rest, p) in rows(word, &note) {
        if !p.hanja.is_empty() {
            let want = format!("{}{rest}", p.hanja);
            if !left.iter().any(|c| c.text == want) {
                let note = format!("{} {} · 訓讀", p.hanja, p.noun);
                left.push(Candidate { text: want.clone(), note });
            }
            to_front(left, |t| t == want);
        }
        if !p.verb_opens.is_empty() {
            to_front(right, |t| t.starts_with(p.verb_opens));
        }
    }
}

fn blank(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c == ' ')
}

/// Applies the pairs to segments of a whole input.
pub(super) fn apply_to_segments(segs: &mut [Segment]) {
    for i in 2..segs.len() {
        if blank(&segs[i - 1].reading) {
            let word = segs[i - 2].reading.clone();
            let (head, tail) = segs.split_at_mut(i);
            apply(&word, &mut head[i - 2].candidates, &mut tail[0].candidates);
        }
    }
}

/// The same for whole words, one candidate list per run.
pub(super) fn apply_to_runs(runs: &[String], lists: &mut [Vec<Candidate>]) {
    for i in 2..runs.len() {
        if blank(&runs[i - 1]) {
            let (head, tail) = lists.split_at_mut(i);
            apply(&runs[i - 2], &mut head[i - 2], &mut tail[0]);
        }
    }
}
