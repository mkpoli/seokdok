//! Old spellings (옛한글) of the syllables that make up function words.
//!
//! The table comes from the spellings the group's chat uses, with their
//! counts; see "Old spellings" in docs/orthography.md.

use crate::dict::is_han;
use crate::jamo::is_syllable;

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

/// A letter that can follow Hanja inside one word: a syllable, a compatibility
/// jamo (ㅆ, ㄴ) or a conjoining jamo of an old syllable.
fn is_word_letter(c: char) -> bool {
    is_syllable(c) || ('\u{3131}'..='\u{318E}').contains(&c) || ('\u{1100}'..='\u{11FF}').contains(&c) || ('\u{A960}'..='\u{A97C}').contains(&c) || ('\u{D7B0}'..='\u{D7FB}').contains(&c)
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
}
