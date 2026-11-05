//! Word pairs: the word next door decides between homophones (눈이 내린다 is
//! 雪, 눈이 아프다 is 目), which a run alone cannot.

use super::{Candidate, Segment};

/// (left word, opening of the right word, left Hanja, right spelling's opening).
/// The left word may carry a particle; an empty spelling leaves that side alone.
const PAIRS: &[(&str, &str, &str, &str)] = &[
    ("눈", "내리", "雪", ""), ("눈", "내린", "雪", ""), ("눈", "내렸", "雪", ""), ("눈", "내려", "雪", ""), ("눈", "온", "雪", ""),
    ("눈", "왔", "雪", ""), ("눈", "오", "雪", ""), ("눈", "와", "雪", ""), ("눈", "쌓", "雪", ""), ("눈", "녹", "雪", ""),
    ("배", "고프", "腹", ""), ("배", "고파", "腹", ""), ("배", "고팠", "腹", ""), ("배", "아프", "腹", ""), ("배", "아파", "腹", ""),
    ("배", "아팠", "腹", ""), ("배", "부르", "腹", ""), ("배", "불러", "腹", ""), ("배", "불렀", "腹", ""),
    ("말", "타", "馬", ""), ("말", "탔", "馬", ""), ("말", "탄", "馬", ""), ("말", "달리", "馬", ""),
    ("다리", "아프", "脚", ""), ("다리", "아파", "脚", ""), ("다리", "아팠", "脚", ""), ("다리", "길", "脚", ""),
    ("길", "멀", "道", ""), ("길", "먼", "道", ""), ("길", "막히", "道", ""), ("길", "막혀", "道", ""), ("길", "잃", "道", ""),
    ("길", "걷", "道", ""), ("길", "걸어", "道", ""), ("길", "떠나", "道", ""), ("길", "찾", "道", ""),
    ("해", "끼치", "害", ""), ("해", "끼쳐", "害", ""), ("해", "끼쳤", "害", ""), ("해", "입", "害", ""),
    ("사과", "먹", "沙果", ""), ("사과", "맛있", "沙果", ""), ("사과", "깎", "沙果", ""),
    ("밤", "먹", "栗", ""), ("밤", "까", "栗", ""), ("차", "마시", "茶", ""), ("차", "마셔", "茶", ""), ("차", "마셨", "茶", ""),
    ("차", "타", "車", ""), ("차", "탔", "車", ""), ("차", "몰", "車", ""), ("배", "타", "舟", ""), ("배", "탔", "舟", ""),
    ("예를", "들", "例를", "들"), ("손", "들", "", "擧"),
];

/// The left word of a pair that `word` opens, and what follows it.
fn pair_rows<'a>(word: &'a str, next: &'a str) -> impl Iterator<Item = (&'a str, &'static str, &'static str)> {
    PAIRS.iter().filter_map(move |&(l, r, lo, ro)| {
        let rest = word.strip_prefix(l)?;
        // A particle at most: 눈썹이 is another word.
        (rest.chars().count() <= 2 && next.starts_with(r)).then_some((rest, lo, ro))
    })
}

fn to_front(list: &mut [Candidate], fits: impl Fn(&str) -> bool) {
    if let Some(i) = list.iter().position(|c| fits(&c.text)) {
        list[..=i].rotate_right(1);
    }
}

/// Moves the spelling a pair asks for to the front of both words' candidates.
pub(super) fn apply_to_segments(segs: &mut [Segment]) {
    for i in 2..segs.len() {
        if segs[i - 1].reading != " " {
            continue;
        }
        let (left, right) = (segs[i - 2].reading.clone(), segs[i].reading.clone());
        let rows: Vec<_> = pair_rows(&left, &right).collect();
        for (rest, lo, ro) in rows {
            if !lo.is_empty() {
                let want = format!("{}{rest}", lo);
                to_front(&mut segs[i - 2].candidates, |t| t == want);
            }
            if !ro.is_empty() {
                to_front(&mut segs[i].candidates, |t| t.starts_with(ro));
            }
        }
    }
}

/// The same for whole words, one candidate list per run.
pub(super) fn apply_to_runs(runs: &[String], lists: &mut [Vec<Candidate>]) {
    for i in 2..runs.len() {
        if runs[i - 1] != " " {
            continue;
        }
        let rows: Vec<_> = pair_rows(&runs[i - 2], &runs[i]).collect();
        for (rest, lo, ro) in rows {
            if !lo.is_empty() {
                let want = format!("{}{rest}", lo);
                to_front(&mut lists[i - 2], |t| t == want);
            }
            if !ro.is_empty() {
                to_front(&mut lists[i], |t| t.starts_with(ro));
            }
        }
    }
}
