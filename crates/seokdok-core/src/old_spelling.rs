//! Old spellings (옛한글) of Korean, offered after Hanja and accepted as input.
//!
//! The table of function syllables comes from the spellings the group's chat
//! uses, with their counts; see "Old spellings" in docs/orthography.md.

use crate::dict::is_han;
use crate::jamo::{is_syllable, join_syllable, SBASE};
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

fn is_cho(c: char) -> bool {
    matches!(c as u32, 0x1100..=0x115F | 0xA960..=0xA97C)
}
fn is_jung(c: char) -> bool {
    matches!(c as u32, 0x1160..=0x11A7 | 0xD7B0..=0xD7C6)
}
fn is_jong(c: char) -> bool {
    matches!(c as u32, 0x11A8..=0x11FF | 0xD7CB..=0xD7FB)
}

/// End of the conjoining syllable that starts at `i`: onsets, vowels, codas.
fn syllable_end(cs: &[char], i: usize) -> usize {
    let mut j = i;
    for part in [is_cho as fn(char) -> bool, is_jung, is_jong] {
        while j < cs.len() && part(cs[j]) {
            j += 1;
        }
    }
    j
}

/// One syllable of a Hangul run as typed, with the modern syllables it can be
/// read as.
pub(crate) struct Syl {
    pub typed: String,
    pub reads: Vec<char>,
}

/// Whether an old syllable is one of the function syllables in the table.
pub(crate) fn is_function_old(syllable: &str) -> bool {
    FUNCTION.iter().any(|(_, o)| *o == syllable)
}

/// Whether a run holds old letters (conjoining jamo) and so cannot go through
/// the converter as it is.
pub(crate) fn has_old(run: &str) -> bool {
    run.chars().any(|c| !is_syllable(c))
}

/// The syllables of a run, each with its modern readings. `None` if a letter
/// has no modern counterpart.
///
/// The folds, from old letters to the modern letters they became:
/// ㆍ (U+119E) to ㅏ or ㅡ, ᆡ (U+11A1, ㆍ+ㅣ) to ㅐ, ㅿ to ㅅ, ㆁ to ㅇ, ㅸ to
/// ㅂ, and the sibilant clusters ᄭ ᄯ ᄲ ᄶ ᄡ (ㅅ+ㄱ, ㅅ+ㄷ, ㅅ+ㅂ, ㅅ+ㅈ, ㅂ+ㅅ) to
/// ㄲ ㄸ ㅃ ㅉ ㅆ. They give the modern letter that the word's modern spelling
/// has in that place; the engine uses them only to look words up.
pub(crate) fn parse(run: &str) -> Option<Vec<Syl>> {
    let cs: Vec<char> = run.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        if is_syllable(cs[i]) {
            out.push(Syl { typed: cs[i].to_string(), reads: vec![cs[i]] });
            i += 1;
            continue;
        }
        let j = syllable_end(&cs, i);
        let span = &cs[i..j];
        let (l, rest) = span.split_first().filter(|(l, _)| is_cho(**l))?;
        let (v, rest) = rest.split_first().filter(|(v, _)| is_jung(**v))?;
        let t = match rest {
            [] => None,
            [t] => Some(fold_t(*t)?),
            _ => return None,
        };
        let l = fold_l(*l)?;
        let reads: Vec<char> = fold_v(*v)?.iter().filter_map(|&v| join_syllable(l, v, t)).collect();
        if reads.is_empty() {
            return None;
        }
        out.push(Syl { typed: span.iter().collect(), reads });
        i = j;
    }
    Some(out)
}

fn fold_l(l: char) -> Option<char> {
    Some(match l as u32 {
        0x1100..=0x1112 => l,
        0x1140 => '\u{1109}',
        0x114C => '\u{110B}',
        0x112B => '\u{1107}',
        0x112D => '\u{1101}',
        0x112F => '\u{1104}',
        0x1132 => '\u{1108}',
        0x1136 => '\u{110D}',
        0x1121 => '\u{110A}',
        _ => return None,
    })
}

fn fold_v(v: char) -> Option<Vec<char>> {
    Some(match v as u32 {
        0x1161..=0x1175 => vec![v],
        0x119E => vec!['\u{1161}', '\u{1173}'],
        0x11A1 => vec!['\u{1162}'],
        _ => return None,
    })
}

/// Codas: ᇫ (ㅿ) to ㅅ and ᇰ (ㆁ) to ㅇ.
fn fold_t(t: char) -> Option<char> {
    Some(match t as u32 {
        0x11A8..=0x11C2 => t,
        0x11EB => '\u{11BA}',
        0x11F0 => '\u{11BC}',
        _ => return None,
    })
}

/// Spellings of the era between the old letters and today's orthography that
/// are made of modern syllables, as the modern syllables they stand for:
/// ㄷ and ㅌ before ㅑ ㅕ ㅛ were ㅈ and ㅊ before ㅏ ㅓ ㅗ (텬하 = 천하, 뎌 = 저,
/// 됴 = 조, 댜 = 자), and a final ㅅ was a final ㅆ (잇다 = 있다, 갓다 = 갔다),
/// where the syllable can be a past tense.
fn era_reads(c: char) -> Vec<char> {
    let s = c as u32 - SBASE;
    let (l, v, t) = (s / 588, (s % 588) / 28, s % 28);
    let build = |l: u32, v: u32, t: u32| char::from_u32(SBASE + (l * 21 + v) * 28 + t).unwrap();
    // ㄷ (3) and ㅌ (16) before ㅑ (2), ㅕ (6), ㅛ (12).
    let palatal = match (l, v) {
        (3, 2 | 6 | 12) => Some((12, if v == 2 { 0 } else if v == 6 { 4 } else { 8 })),
        (16, 2 | 6 | 12) => Some((14, if v == 2 { 0 } else if v == 6 { 4 } else { 8 })),
        _ => None,
    };
    let mut out = Vec::new();
    if let Some((l2, v2)) = palatal {
        out.push(build(l2, v2, t));
    }
    // A final ㅅ (19) was a final ㅆ (20); only after the vowels of the past
    // tense (았 었 였 있) is that worth a lookup: ㅏ (0), ㅐ (1), ㅓ (4), ㅕ (6), ㅣ (20).
    // The noun 것 is never one.
    if t == 19 && matches!(v, 0 | 1 | 4 | 6 | 20) && c != '것' {
        out.push(build(l, v, 20));
        if let Some((l2, v2)) = palatal {
            out.push(build(l2, v2, 20));
        }
    }
    out
}

/// Whether the run has a syllable with an era spelling worth trying.
pub(crate) fn has_era(run: &str) -> bool {
    run.chars().any(|c| is_syllable(c) && !era_reads(c).is_empty())
}

/// At most this many modern readings are tried for a run.
const MAX_READINGS: usize = 8;

/// Modern readings of a run, best guess first. With `era`, the readings that
/// use an era spelling (and only those); without it, the folds of old letters.
pub(crate) fn readings(syls: &[Syl], era: bool) -> Vec<String> {
    // For each syllable the readings to choose from, the first being the
    // plain one, each flagged when it is an era spelling.
    let alts: Vec<Vec<(char, bool)>> = syls
        .iter()
        .map(|s| {
            let mut v: Vec<(char, bool)> = s.reads.iter().map(|&c| (c, false)).collect();
            if era {
                for &c in &s.reads {
                    v.extend(era_reads(c).into_iter().map(|e| (e, true)));
                }
            }
            v
        })
        .collect();
    let total = alts.iter().fold(1usize, |n, a| n.saturating_mul(a.len()));
    let typed: String = syls.iter().map(|s| s.typed.as_str()).collect();
    let mut found: Vec<(usize, String)> = Vec::new();
    let mut pick = vec![0usize; alts.len()];
    for _ in 0..total.min(1024) {
        let text: String = pick.iter().zip(&alts).map(|(&i, a)| a[i].0).collect();
        let used_era = pick.iter().zip(&alts).any(|(&i, a)| a[i].1);
        if text != typed && used_era == era {
            let moved = pick.iter().filter(|&&i| i != 0).count();
            found.push((moved, text));
        }
        // Next combination, the last syllable varying fastest.
        let mut k = pick.len();
        while k > 0 {
            k -= 1;
            pick[k] += 1;
            if pick[k] < alts[k].len() {
                break;
            }
            pick[k] = 0;
        }
    }
    // Fewest departures from the first choice first.
    found.sort_by_key(|(moved, _)| *moved);
    found.into_iter().map(|(_, t)| t).take(MAX_READINGS).collect()
}

/// `candidate`, a conversion of the modern `reading`, with the tail of the
/// typed run put back: the last syllables of the typed run, if they are
/// modern or function syllables with an old spelling, replace what the
/// reading wrote there (할까요 → ᄒᆞᆯ까요).
pub(crate) fn keep_typed_tail(candidate: &str, reading: &str, syls: &[Syl]) -> Option<String> {
    let rs: Vec<char> = reading.chars().collect();
    if rs.len() != syls.len() {
        return None;
    }
    for k in (1..=rs.len()).rev() {
        let tail: String = rs[rs.len() - k..].iter().collect();
        let Some(head) = candidate.strip_suffix(tail.as_str()) else { continue };
        let typed = &syls[syls.len() - k..];
        let usable = typed.iter().all(|s| s.typed.chars().all(is_syllable) || is_function_old(&s.typed));
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

    fn reads(run: &str, era: bool) -> Vec<String> {
        readings(&parse(run).unwrap(), era)
    }

    #[test]
    fn old_letters_fold_to_modern_ones() {
        // ᄉᆞ랑 = 사랑, ᄂᆞᆫ = 는, ᄒᆞᆰ = 흙; the ㅏ reading comes first.
        assert_eq!(reads("\u{1109}\u{119E}랑", false), ["사랑", "스랑"]);
        assert_eq!(reads("\u{1102}\u{119E}\u{11AB}", false), ["난", "는"]);
        assert_eq!(reads("\u{1112}\u{119E}\u{11B0}", false)[1], "흙");
        assert_eq!(reads("\u{1112}\u{11A1}", false), ["해"]);
        assert_eq!(reads("\u{1140}\u{1175}", false), ["시"]);
        assert_eq!(reads("\u{114C}\u{1175}", false), ["이"]);
        assert_eq!(reads("\u{112F}\u{1169}", false), ["또"]);
        assert_eq!(reads("\u{1109}\u{11A1}\u{11F0}", false), ["생"]);
        // A letter with no modern counterpart gives no reading.
        assert!(parse("\u{1113}\u{1161}").is_none());
    }

    #[test]
    fn era_spellings_are_modern_syllables() {
        assert_eq!(reads("텬하", true), ["천하"]);
        assert_eq!(reads("됴타", true), ["조타"]);
        assert_eq!(reads("뎌", true), ["저"]);
        assert_eq!(reads("댜", true), ["자"]);
        assert_eq!(reads("잇다", true), ["있다"]);
        assert_eq!(reads("갓다", true), ["갔다"]);
        // Nothing to try for ordinary words.
        assert!(reads("사랑", true).is_empty());
        assert!(!has_era("디자인") && !has_era("사랑") && has_era("텬하"));
    }

    #[test]
    fn at_most_eight_readings() {
        let run = "\u{1109}\u{119E}".repeat(6);
        assert_eq!(reads(&run, false).len(), 8);
    }

    #[test]
    fn the_typed_tail_is_put_back() {
        let syls = parse("추가\u{1112}\u{119E}\u{11AF}까요").unwrap();
        assert_eq!(keep_typed_tail("追加할까요", "추가할까요", &syls).unwrap(), "追加\u{1112}\u{119E}\u{11AF}까요");
        assert_eq!(keep_typed_tail("추가할까요", "추가할까요", &syls).unwrap(), "추가\u{1112}\u{119E}\u{11AF}까요");
        // Old letters in the head are not a tail.
        let syls = parse("\u{1109}\u{119E}랑\u{110B}\u{119E}\u{11AF}").unwrap();
        assert_eq!(keep_typed_tail("愛을", "사랑을", &syls).unwrap(), "愛\u{110B}\u{119E}\u{11AF}");
        let syls = parse("\u{1109}\u{119E}랑").unwrap();
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
