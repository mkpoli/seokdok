//! Old spellings (옛한글) of Korean, offered after Hanja and accepted as input.
//!
//! The table of function syllables comes from the spellings the group's chat
//! uses, with their counts; see "Old spellings" in docs/orthography.md.

use crate::dict::is_han;
use crate::fold::Syl;
use crate::jamo::is_syllable;
use std::borrow::Cow;

/// Modern syllable and the old spelling the chat writes for it (ㆍ is
/// U+119E, ᆡ is U+11A1).
const FUNCTION: [(char, &str); 10] = [
    ('하', "\u{1112}\u{119E}"),
    ('한', "\u{1112}\u{119E}\u{11AB}"),
    ('할', "\u{1112}\u{119E}\u{11AF}"),
    ('함', "\u{1112}\u{119E}\u{11B7}"),
    ('합', "\u{1112}\u{119E}\u{11B8}"),
    ('해', "\u{1112}\u{11A1}"),
    ('는', "\u{1102}\u{119E}\u{11AB}"),
    ('를', "\u{1105}\u{119E}\u{11AF}"),
    ('은', "\u{110B}\u{119E}\u{11AB}"),
    ('을', "\u{110B}\u{119E}\u{11AF}"),
];

fn old_of(c: char) -> Option<&'static str> {
    FUNCTION.iter().find(|(m, _)| *m == c).map(|(_, o)| *o)
}

/// A conjoining jamo of the Hangul Jamo blocks, modern or old.
pub(crate) fn is_jamo(c: char) -> bool {
    matches!(c as u32, 0x1100..=0x11FF | 0xA960..=0xA97C | 0xD7B0..=0xD7FB)
}

/// A letter that can follow Hanja inside one word: a syllable, a compatibility
/// jamo (ㅆ, ㄴ) or a conjoining jamo of an old syllable.
fn is_word_letter(c: char) -> bool {
    is_syllable(c) || ('\u{3131}'..='\u{318E}').contains(&c) || is_jamo(c)
}

/// The syllables of the light verb 하다 in the table (하 한 할 함 합 해).
fn is_ha_row(c: char) -> bool {
    matches!(c, '하' | '한' | '할' | '함' | '합' | '해')
}

/// `text` with the function syllables of its Hangul tail written the old way:
/// the letters after the last Hanja, or all of it when it is an inflection of
/// the light verb 하다 (`light`). With `ha_only`, only the 하 rows of the
/// table change. `None` when nothing changes.
fn old_tail(text: &str, light: bool, ha_only: bool) -> Option<String> {
    let mut out = String::with_capacity(text.len() + 8);
    let mut tail = light;
    let mut changed = false;
    for c in text.chars() {
        if is_han(c) {
            tail = true;
        } else if !is_word_letter(c) {
            tail = false;
        } else if let Some(old) = old_of(c).filter(|_| tail && (!ha_only || is_ha_row(c))) {
            out.push_str(old);
            changed = true;
            continue;
        }
        out.push(c);
    }
    changed.then_some(out)
}

/// The old spelling to offer for a segment's candidates, best first: that of
/// the best candidate, or for a light verb that of the first candidate it
/// changes (the Hangul 하세요 stands behind 爲세요 under `hada_kun`). A segment
/// that follows Hanja directly (the 를 of 字를) is all tail (`after_hanja`).
/// With `ha_only`, only the 하 rows are written the old way.
pub(crate) fn old_variant<'a>(texts: impl IntoIterator<Item = &'a str>, light: bool, after_hanja: bool, ha_only: bool) -> Option<String> {
    let mut texts = texts.into_iter();
    if light {
        texts.find_map(|t| old_tail(t, true, ha_only))
    } else {
        texts.next().and_then(|t| old_tail(t, after_hanja, ha_only))
    }
}

/// Whether text ends in Hanja, so that Hangul typed next to it is a tail.
pub(crate) fn ends_in_hanja(text: &str) -> bool {
    text.chars().next_back().is_some_and(is_han)
}

/// Function syllables written the old way, read as modern (ᄒᆞᆯ → 할), for
/// reading 釋讀 text back. Other old letters stay as they are.
pub(crate) fn modernize_function(text: &str) -> Cow<'_, str> {
    if !text.chars().any(is_jamo) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let cs: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let j = if is_cho(cs[i]) { syllable_end(&cs, i) } else { i + 1 };
        let span: String = cs[i..j].iter().collect();
        match FUNCTION.iter().find(|(_, o)| *o == span) {
            Some((m, _)) => out.push(*m),
            None => out.push_str(&span),
        }
        i = j;
    }
    Cow::Owned(out)
}

pub(crate) fn is_cho(c: char) -> bool {
    matches!(c as u32, 0x1100..=0x115F | 0xA960..=0xA97C)
}
pub(crate) fn is_jung(c: char) -> bool {
    matches!(c as u32, 0x1160..=0x11A7 | 0xD7B0..=0xD7C6)
}
pub(crate) fn is_jong(c: char) -> bool {
    matches!(c as u32, 0x11A8..=0x11FF | 0xD7CB..=0xD7FB)
}

/// End of the conjoining syllable that starts at `i`: onsets, vowels, codas.
pub(crate) fn syllable_end(cs: &[char], i: usize) -> usize {
    let mut j = i;
    for part in [is_cho as fn(char) -> bool, is_jung, is_jong] {
        while j < cs.len() && part(cs[j]) {
            j += 1;
        }
    }
    j
}

/// Whether an old syllable is one of the function syllables in the table.
pub(crate) fn is_function_old(syllable: &str) -> bool {
    FUNCTION.iter().any(|(_, o)| *o == syllable)
}

/// `candidate`, a conversion of the modern `reading`, with the tail of the
/// typed run put back: the last syllables of the typed run, if each is the
/// reading's own or the old spelling of a function syllable, replace what
/// the reading wrote there (할까요 → ᄒᆞᆯ까요).
pub(crate) fn keep_typed_tail(candidate: &str, reading: &str, syls: &[Syl]) -> Option<String> {
    let rs: Vec<char> = reading.chars().collect();
    if rs.len() != syls.len() {
        return None;
    }
    for k in (1..=rs.len()).rev() {
        let tail: String = rs[rs.len() - k..].iter().collect();
        let Some(head) = candidate.strip_suffix(tail.as_str()) else { continue };
        let typed = &syls[syls.len() - k..];
        // Each typed syllable is the reading's own or the old spelling of it.
        let usable = typed.iter().zip(&rs[rs.len() - k..]).all(|(s, r)| s.typed == r.to_string() || FUNCTION.iter().any(|(m, o)| m == r && *o == s.typed));
        let typed: String = typed.iter().map(|s| s.typed.as_str()).collect();
        if usable && typed != tail {
            return Some(format!("{head}{typed}"));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tails_after_hanja() {
        assert_eq!(old_tail("字를", false, false).unwrap(), "字\u{1105}\u{119E}\u{11AF}");
        assert_eq!(old_tail("此것을", false, false).unwrap(), "此것\u{110B}\u{119E}\u{11AF}");
        assert_eq!(old_tail("在ㅆ는", false, false).unwrap(), "在ㅆ\u{1102}\u{119E}\u{11AB}");
        assert_eq!(old_tail("追加할까요", false, false).unwrap(), "追加\u{1112}\u{119E}\u{11AF}까요");
    }

    #[test]
    fn hangul_without_hanja_is_left_alone() {
        assert_eq!(old_tail("하는", false, false), None);
        assert_eq!(old_tail("天 하는", false, false), None);
        assert!(old_tail("字를", false, false).is_some());
    }

    #[test]
    fn light_verb_changes_every_function_syllable() {
        assert_eq!(old_tail("하는", true, false).unwrap(), "\u{1112}\u{119E}\u{1102}\u{119E}\u{11AB}");
        assert_eq!(old_tail("해요", true, false).unwrap(), "\u{1112}\u{11A1}요");
        // 했 is not in the table.
        assert_eq!(old_tail("했다", true, false), None);
    }

    #[test]
    fn the_light_verb_offers_its_first_changing_candidate() {
        assert_eq!(old_variant(["爲고", "하고"], true, false, false).unwrap(), "\u{1112}\u{119E}고");
        assert_eq!(old_variant(["爲고", "하고"], false, false, false), None);
        assert_eq!(old_variant(["를", "㔹"], false, true, false).unwrap(), "\u{1105}\u{119E}\u{11AF}");
        assert!(ends_in_hanja("漢字") && !ends_in_hanja("字 "));
    }

    #[test]
    fn the_ha_rows_alone() {
        assert_eq!(old_tail("工夫하는", false, true).unwrap(), "工夫\u{1112}\u{119E}는");
        assert_eq!(old_tail("하는", true, true).unwrap(), "\u{1112}\u{119E}는");
        assert_eq!(old_tail("追加한", false, true).unwrap(), "追加\u{1112}\u{119E}\u{11AB}");
        assert_eq!(old_tail("字를", false, true), None);
        assert_eq!(old_variant(["工夫", "하는"], true, false, true).unwrap(), "\u{1112}\u{119E}는");
    }

    #[test]
    fn the_typed_tail_is_put_back() {
        let syls = crate::fold::parse("추가\u{1112}\u{119E}\u{11AF}까요").unwrap();
        assert_eq!(keep_typed_tail("追加할까요", "추가할까요", &syls).unwrap(), "追加\u{1112}\u{119E}\u{11AF}까요");
        assert_eq!(keep_typed_tail("추가할까요", "추가할까요", &syls).unwrap(), "추가\u{1112}\u{119E}\u{11AF}까요");
        // Old letters in the head are not a tail.
        let syls = crate::fold::parse("\u{1109}\u{119E}랑\u{110B}\u{119E}\u{11AF}").unwrap();
        assert_eq!(keep_typed_tail("愛을", "사랑을", &syls).unwrap(), "愛\u{110B}\u{119E}\u{11AF}");
        let syls = crate::fold::parse("\u{1109}\u{119E}랑").unwrap();
        assert_eq!(keep_typed_tail("愛", "사랑", &syls), None);
    }

    #[test]
    fn function_syllables_read_back() {
        assert_eq!(modernize_function("工夫\u{1112}\u{119E}\u{1102}\u{119E}\u{11AB}"), "工夫하는");
        assert_eq!(modernize_function("字\u{1105}\u{119E}\u{11AF} 읽다"), "字를 읽다");
        // Other old syllables stay, and a coda keeps ᄒᆞ from matching.
        let other = "\u{1112}\u{119E}\u{11BC}";
        assert_eq!(modernize_function(other), other);
        assert!(matches!(modernize_function("하늘을"), Cow::Borrowed(_)));
    }
}
