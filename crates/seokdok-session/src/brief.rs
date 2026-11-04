//! The short reading beside a candidate on a phone's one-line candidate bar.
//!
//! The engine's notes describe a candidate in full ("不 안 · 訓讀"). Beside the
//! candidate most of that repeats what the reader can already see: the Hanja
//! is the candidate, and the reading is what they typed. Left is what tells
//! the candidates apart: a dictionary word that differs from the input (行ㄹ is
//! 가다, 磨ㄹ is 갈다 for 갈), or 音 and 訓 when the list holds both.

use seokdok_core::jamo::is_hangul;

#[derive(Default)]
struct Reduced {
    /// Words the input does not already say, and the reader's own notes.
    words: Vec<String>,
    sound: bool,
    meaning: bool,
}

fn reduce(typed: &str, note: &str) -> Reduced {
    let mut out = Reduced::default();
    let word = |out: &mut Reduced, w: String| {
        if !out.words.contains(&w) {
            out.words.push(w);
        }
    };
    for piece in note.split(" / ").filter(|p| !p.is_empty()) {
        let mut parts = piece.split(" · ");
        let head = parts.next().unwrap_or("");
        let tails: Vec<&str> = parts.collect();
        if tails.is_empty() && matches!(head, "한글" | "옛한글" | "variant") {
            continue;
        }
        let described = tails.iter().any(|t| matches!(*t, "音讀" | "訓讀" | "訓音"));
        if !described {
            word(&mut out, piece.to_string());
            continue;
        }
        for t in &tails {
            match *t {
                "音讀" => out.sound = true,
                "訓讀" => out.meaning = true,
                "訓音" => {}
                other => word(&mut out, other.to_string()),
            }
        }
        // Hanja and the "+아/어" of an ending are left out; the Hangul is the reading.
        let reading: String = head.split(' ').filter(|w| !w.starts_with('+') && w.chars().any(is_hangul)).collect();
        if !reading.is_empty() && !typed.contains(&reading) {
            word(&mut out, reading);
        }
    }
    out
}

/// What to show beside each candidate of a list, in order, for input `typed`.
/// 音 and 訓 appear only on a list that holds both, and only where no word is
/// shown: a word differing from the input already marks a reading by meaning.
pub fn brief_notes<'a>(typed: &str, notes: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let reduced: Vec<Reduced> = notes.into_iter().map(|n| reduce(typed, n)).collect();
    let mixed = reduced.iter().any(|r| r.sound) && reduced.iter().any(|r| r.meaning);
    reduced
        .into_iter()
        .map(|r| {
            if !r.words.is_empty() {
                return r.words.join(" ");
            }
            if !mixed {
                return String::new();
            }
            let mut kinds = String::new();
            if r.sound {
                kinds.push('音');
            }
            if r.meaning {
                kinds.push('訓');
            }
            kinds
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::brief_notes;

    fn brief(typed: &str, notes: &[&str]) -> Vec<String> {
        brief_notes(typed, notes.iter().copied())
    }

    #[test]
    fn a_reading_the_input_already_says_is_left_out() {
        // The list as the engine gives it for 안녕.
        assert_eq!(
            brief("안녕", &["안녕 · 音讀", "", "不 안 · 訓讀", "安 안 · 音讀", "寧 녕 · 音讀"]),
            ["音", "", "訓", "音", "音"],
        );
    }

    #[test]
    fn a_list_of_one_kind_needs_no_mark() {
        assert_eq!(brief("사기", &["사기 · 音讀", "사기 · 音讀", ""]), ["", "", ""]);
    }

    #[test]
    fn a_dictionary_word_differing_from_the_input_stays() {
        assert_eq!(
            brief("갈", &["行 가다 +(으)ㄹ · 訓讀", "磨 갈다 +(으)ㄹ · 訓讀", ""]),
            ["가다", "갈다", ""],
        );
        assert_eq!(brief("읽어", &["讀 읽다 +아/어 · 訓讀"]), ["읽다"]);
        // 하다 for 하다 says nothing; 하다 for 하고 does.
        assert_eq!(brief("하다", &["爲 하다 +다 · 訓讀"]), [""]);
        assert_eq!(brief("하고", &["爲 하다 +고 · 訓讀"]), ["하다"]);
    }

    #[test]
    fn labels_the_candidate_shows_by_itself_are_dropped() {
        assert_eq!(brief("하다", &["", "옛한글", "한글", "variant"]), ["", "", "", ""]);
    }

    #[test]
    fn the_sound_and_meaning_of_a_single_hanja_repeat_the_input() {
        assert_eq!(brief("하늘천", &["하늘 천 · 訓音", "天 하늘 · 訓讀"]), ["", ""]);
    }

    #[test]
    fn pieces_of_a_longer_candidate_are_reduced_one_by_one() {
        assert_eq!(brief("학교에", &["學校 학교 · 音讀", "學校 학교 · 音讀 / 割 에다 +아/어 · 訓讀"]), ["音", "에다"]);
    }

    #[test]
    fn a_readers_own_note_is_kept() {
        assert_eq!(brief("집", &["내 말"]), ["내 말"]);
        assert_eq!(brief("집", &["家 집 · 訓讀 · 내 말"]), ["내 말"]);
    }
}
