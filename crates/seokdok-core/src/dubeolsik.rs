//! 두벌식 (KS X 5002) Hangul composition, with old Hangul (옛한글).
//!
//! The composition is a plain string. Only its last syllable is ever "open":
//! one that can still take another jamo, or a lone jamo. Everything before it
//! is final, so the automaton needs no state beyond the string, and backspace
//! removes one typed letter at a time.
//!
//! A syllable is three runs of typed letters: initial, medial, final. Modern
//! syllables are written precomposed (한); anything else as conjoining jamo
//! (ᄒᆞ = U+1112 U+119E). The old letters ㅿ ㆁ ㆆ ㆍ and the sibilants
//! ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ sit on the Shift layer, as on Microsoft Old Hangul. Clusters
//! made only of modern letters (ㅅ+ㄱ → ᄭ, ㅂ+ㅇ → ㅸ) need the `old`
//! switch, because without it ordinary typing such as ㅎㅎ must stay apart.

use crate::old_jamo;

const CHO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
const JUNG: &str = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ";
const JONG: &str = "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ";
/// Modern letters that have a key; everything else typed is an old letter.
const MODERN_LETTERS: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎㅏㅐㅑㅒㅓㅔㅕㅖㅗㅛㅜㅠㅡㅣ";
const OLD_VOWELS: &str = "ㆍ";

const SBASE: u32 = 0xAC00;
const L_FILL: char = '\u{115F}';
const V_FILL: char = '\u{1160}';

/// Modern two-letter vowels and finals, as typed.
const VOWEL_PAIRS: [(&str, char); 7] =
    [("ㅗㅏ", 'ㅘ'), ("ㅗㅐ", 'ㅙ'), ("ㅗㅣ", 'ㅚ'), ("ㅜㅓ", 'ㅝ'), ("ㅜㅔ", 'ㅞ'), ("ㅜㅣ", 'ㅟ'), ("ㅡㅣ", 'ㅢ')];
const CODA_PAIRS: [(&str, char); 11] = [
    ("ㄱㅅ", 'ㄳ'),
    ("ㄴㅈ", 'ㄵ'),
    ("ㄴㅎ", 'ㄶ'),
    ("ㄹㄱ", 'ㄺ'),
    ("ㄹㅁ", 'ㄻ'),
    ("ㄹㅂ", 'ㄼ'),
    ("ㄹㅅ", 'ㄽ'),
    ("ㄹㅌ", 'ㄾ'),
    ("ㄹㅍ", 'ㄿ'),
    ("ㄹㅎ", 'ㅀ'),
    ("ㅂㅅ", 'ㅄ'),
];

/// The jamo on a 두벌식 key. Lowercase letters are the plain layer, uppercase
/// letters the Shift layer: ㅃㅉㄸㄲㅆㅒㅖ as usual, and the old letters
/// ㅿ ㆁ ㆆ ㆍ ᄼ ᄾ ᅎ ᅐ ᅔ ᅕ where Microsoft Old Hangul puts them.
pub fn key_to_jamo(c: char) -> Option<char> {
    Some(match c {
        'q' => 'ㅂ',
        'w' => 'ㅈ',
        'e' => 'ㄷ',
        'r' => 'ㄱ',
        't' => 'ㅅ',
        'y' | 'Y' => 'ㅛ',
        'u' | 'U' => 'ㅕ',
        'i' | 'I' => 'ㅑ',
        'o' => 'ㅐ',
        'p' => 'ㅔ',
        'Q' => 'ㅃ',
        'W' => 'ㅉ',
        'E' => 'ㄸ',
        'R' => 'ㄲ',
        'T' => 'ㅆ',
        'O' => 'ㅒ',
        'P' => 'ㅖ',
        'a' => 'ㅁ',
        'A' => 'ㅿ',
        's' | 'S' => 'ㄴ',
        'd' => 'ㅇ',
        'D' => 'ㆁ',
        'f' | 'F' => 'ㄹ',
        'g' => 'ㅎ',
        'G' => 'ㆆ',
        'h' | 'H' => 'ㅗ',
        'j' | 'J' => 'ㅓ',
        'k' => 'ㅏ',
        'K' => 'ㆍ',
        'l' | 'L' => 'ㅣ',
        'z' => 'ㅋ',
        'Z' => 'ᄼ',
        'x' => 'ㅌ',
        'X' => 'ᄾ',
        'c' => 'ㅊ',
        'C' => 'ᅎ',
        'v' => 'ㅍ',
        'V' => 'ᅐ',
        'b' => 'ㅠ',
        'B' => 'ᅔ',
        'n' => 'ㅜ',
        'N' => 'ᅕ',
        'm' | 'M' => 'ㅡ',
        _ => return None,
    })
}

pub fn is_vowel(j: char) -> bool {
    JUNG.contains(j) || OLD_VOWELS.contains(j)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Cho,
    Jung,
    Jong,
}

fn table(p: Part) -> &'static [(&'static str, char)] {
    match p {
        Part::Cho => old_jamo::CHO,
        Part::Jung => old_jamo::JUNG,
        Part::Jong => old_jamo::JONG,
    }
}

/// The conjoining jamo for a run of typed letters.
fn jamo(p: Part, seq: &str) -> Option<char> {
    table(p).iter().find(|r| r.0 == seq).map(|r| r.1)
}

/// The typed letters of a conjoining jamo.
fn letters(p: Part, c: char) -> Option<&'static str> {
    table(p).iter().find(|r| r.1 == c).map(|r| r.0)
}

/// The modern compatibility jamo a run spells, if it is modern.
fn modern(p: Part, seq: &str) -> Option<char> {
    let one = |set: &str| {
        let mut cs = seq.chars();
        match (cs.next(), cs.next()) {
            (Some(c), None) if set.contains(c) => Some(c),
            _ => None,
        }
    };
    match p {
        Part::Cho => one(CHO),
        Part::Jung => one(JUNG).or_else(|| VOWEL_PAIRS.iter().find(|r| r.0 == seq).map(|r| r.1)),
        Part::Jong => one(JONG).or_else(|| CODA_PAIRS.iter().find(|r| r.0 == seq).map(|r| r.1)),
    }
}

fn modern_letters(p: Part, c: char) -> Option<String> {
    match p {
        Part::Jung => VOWEL_PAIRS.iter().find(|r| r.1 == c).map(|r| r.0.to_string()),
        Part::Jong => CODA_PAIRS.iter().find(|r| r.1 == c).map(|r| r.0.to_string()),
        Part::Cho => None,
    }
    .or_else(|| Some(c.to_string()))
}

/// A syllable under construction: the typed letters of each part.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Syl {
    l: String,
    v: String,
    t: String,
}

impl Syl {
    fn of(j: char) -> Syl {
        if is_vowel(j) {
            Syl { v: j.to_string(), ..Syl::default() }
        } else {
            Syl { l: j.to_string(), ..Syl::default() }
        }
    }

    fn render(&self) -> String {
        let compat = |c: char| old_jamo::COMPAT.iter().find(|r| r.0 == c).map(|r| r.1);
        if self.v.is_empty() {
            let l = jamo(Part::Cho, &self.l).expect("valid initial");
            return compat(l).map(String::from).unwrap_or_else(|| format!("{l}{V_FILL}"));
        }
        let v = jamo(Part::Jung, &self.v).expect("valid medial");
        if self.l.is_empty() {
            return compat(v).map(String::from).unwrap_or_else(|| format!("{L_FILL}{v}"));
        }
        let (ml, mv) = (modern(Part::Cho, &self.l), modern(Part::Jung, &self.v));
        let mt = if self.t.is_empty() { Some(None) } else { modern(Part::Jong, &self.t).map(Some) };
        if let (Some(l), Some(v), Some(t)) = (ml, mv, mt) {
            let li = CHO.chars().position(|c| c == l).unwrap() as u32;
            let vi = JUNG.chars().position(|c| c == v).unwrap() as u32;
            let ti = t.map_or(0, |t| JONG.chars().position(|c| c == t).unwrap() as u32 + 1);
            return char::from_u32(SBASE + (li * 21 + vi) * 28 + ti).unwrap().to_string();
        }
        let l = jamo(Part::Cho, &self.l).expect("valid initial");
        let t = if self.t.is_empty() { None } else { jamo(Part::Jong, &self.t) };
        let mut s = format!("{l}{v}");
        s.extend(t);
        s
    }
}

/// The last syllable of `buf` that may still grow: its byte offset and parts.
fn open_syllable(buf: &str) -> Option<(usize, Syl)> {
    let mut it = buf.char_indices().rev();
    let (i, last) = it.next()?;
    // A precomposed syllable.
    if let Some(s) = (last as u32).checked_sub(SBASE).filter(|s| *s < 19 * 21 * 28) {
        let (l, v, t) = ((s / 588) as usize, (s % 588 / 28) as usize, (s % 28) as usize);
        let v = JUNG.chars().nth(v).unwrap();
        let t = if t == 0 { String::new() } else { modern_letters(Part::Jong, JONG.chars().nth(t - 1).unwrap()).unwrap() };
        return Some((i, Syl { l: CHO.chars().nth(l).unwrap().to_string(), v: modern_letters(Part::Jung, v).unwrap(), t }));
    }
    // A lone compatibility jamo.
    if ('\u{3131}'..='\u{318E}').contains(&last) {
        let c = old_jamo::COMPAT.iter().find(|r| r.1 == last).map(|r| r.0)?;
        if let Some(l) = letters(Part::Cho, c) {
            return Some((i, Syl { l: l.into(), ..Syl::default() }));
        }
        let v = letters(Part::Jung, c)?;
        return Some((i, Syl { v: v.into(), ..Syl::default() }));
    }
    // Conjoining jamo: [initial or filler] [medial or filler] [final].
    let mut start = i;
    let mut s = Syl::default();
    let mut cur = Some((i, last));
    if let Some(t) = letters(Part::Jong, last) {
        s.t = t.into();
        cur = it.next();
    }
    if let Some((j, c)) = cur {
        if c == V_FILL || letters(Part::Jung, c).is_some() {
            if c != V_FILL {
                s.v = letters(Part::Jung, c).unwrap().into();
            }
            start = j;
            cur = it.next();
        } else if !s.t.is_empty() {
            return None;
        }
    }
    if let Some((j, c)) = cur {
        if c == L_FILL || letters(Part::Cho, c).is_some() {
            if c != L_FILL {
                s.l = letters(Part::Cho, c).unwrap().into();
            }
            start = j;
        }
    }
    // A final belongs to a syllable only with both an initial and a medial.
    let valid = (!s.l.is_empty() || !s.v.is_empty()) && (s.t.is_empty() || (!s.l.is_empty() && !s.v.is_empty()));
    valid.then_some((start, s))
}

/// The Hangul composition buffer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Composer {
    buf: String,
    old: bool,
}

impl Composer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_text(text: &str) -> Self {
        Composer { buf: text.to_string(), old: false }
    }

    /// Let modern letters form old clusters (ㅅ+ㄱ → ᄭ, ㅂ+ㅇ → ㅸ, ㆍ+ㆍ is
    /// always allowed since it holds an old letter).
    pub fn set_old(&mut self, old: bool) {
        self.old = old;
    }

    pub fn old(&self) -> bool {
        self.old
    }

    pub fn text(&self) -> &str {
        &self.buf
    }

    pub fn set_text(&mut self, text: &str) {
        self.buf = text.to_string();
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.buf)
    }

    /// Append a character that is not a jamo key (space, digit, punctuation).
    pub fn push_literal(&mut self, c: char) {
        self.buf.push(c);
    }

    /// Feed a key. Returns false when the key is not a 두벌식 letter.
    pub fn input(&mut self, key: char) -> bool {
        match key_to_jamo(key) {
            Some(j) => {
                self.input_jamo(j);
                true
            }
            None => false,
        }
    }

    /// Feed a compatibility jamo as a Hangul keyboard sends it: ㅎ, ㅘ, ㄳ, ㆍ,
    /// ㅸ. A joined letter goes in letter by letter, as typed on the keys. An old
    /// one (ㅸ) stays joined whatever the `old` switch says, since the keyboard
    /// joined it. Returns false for anything else.
    pub fn input_letter(&mut self, compat: char) -> bool {
        let Some(c) = old_jamo::COMPAT.iter().find(|r| r.1 == compat).map(|r| r.0) else {
            return false;
        };
        let Some(seq) = letters(Part::Cho, c).or_else(|| letters(Part::Jung, c)).or_else(|| letters(Part::Jong, c)) else {
            return false;
        };
        let keep = modern(Part::Jung, seq).is_none() && modern(Part::Jong, seq).is_none();
        let old = self.old;
        for (i, j) in seq.chars().enumerate() {
            self.old = old || (keep && i > 0);
            self.input_jamo(j);
        }
        self.old = old;
        true
    }

    /// Whether a run of letters may form this part of a syllable.
    fn allowed(&self, p: Part, seq: &str) -> bool {
        jamo(p, seq).is_some()
            && (self.old || modern(p, seq).is_some() || seq.chars().any(|c| !MODERN_LETTERS.contains(c)))
    }

    /// Feed one jamo (as returned by [`key_to_jamo`]).
    pub fn input_jamo(&mut self, j: char) {
        let Some((at, mut s)) = open_syllable(&self.buf) else {
            self.buf.push_str(&Syl::of(j).render());
            return;
        };
        self.buf.truncate(at);
        let push = |buf: &mut String, a: &Syl, b: Option<&Syl>| {
            buf.push_str(&a.render());
            if let Some(b) = b {
                buf.push_str(&b.render());
            }
        };
        let lone = Syl::of(j);
        if is_vowel(j) {
            if !s.t.is_empty() {
                // 갃 + ㅏ → 각사: the last final letter starts the next syllable.
                let mut t: Vec<char> = s.t.chars().collect();
                let moved = t.pop().unwrap().to_string();
                let rest: String = t.into_iter().collect();
                if self.allowed(Part::Cho, &moved) && (rest.is_empty() || self.allowed(Part::Jong, &rest)) {
                    s.t = rest;
                    let next = Syl { l: moved, v: j.to_string(), t: String::new() };
                    push(&mut self.buf, &s, Some(&next));
                } else {
                    push(&mut self.buf, &s, Some(&lone));
                }
            } else if !s.v.is_empty() {
                let v = format!("{}{j}", s.v);
                if self.allowed(Part::Jung, &v) {
                    s.v = v;
                    push(&mut self.buf, &s, None);
                } else {
                    push(&mut self.buf, &s, Some(&lone));
                }
            } else {
                s.v = j.to_string();
                push(&mut self.buf, &s, None);
            }
        } else if s.v.is_empty() {
            let l = format!("{}{j}", s.l);
            if self.allowed(Part::Cho, &l) {
                s.l = l;
                push(&mut self.buf, &s, None);
            } else {
                push(&mut self.buf, &s, Some(&lone));
            }
        } else if s.l.is_empty() {
            push(&mut self.buf, &s, Some(&lone));
        } else {
            let t = format!("{}{j}", s.t);
            if self.allowed(Part::Jong, &t) {
                s.t = t;
                push(&mut self.buf, &s, None);
            } else {
                push(&mut self.buf, &s, Some(&lone));
            }
        }
    }

    /// Remove one typed letter (or one literal character). Returns false when empty.
    pub fn backspace(&mut self) -> bool {
        let Some((at, mut s)) = open_syllable(&self.buf) else {
            return self.buf.pop().is_some();
        };
        self.buf.truncate(at);
        let part = if !s.t.is_empty() {
            &mut s.t
        } else if !s.v.is_empty() {
            &mut s.v
        } else {
            &mut s.l
        };
        part.pop();
        if !s.l.is_empty() || !s.v.is_empty() {
            self.buf.push_str(&s.render());
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_keys(keys: &str) -> String {
        type_with(keys, false)
    }

    fn type_with(keys: &str, old: bool) -> String {
        let mut c = Composer::new();
        c.set_old(old);
        for k in keys.chars() {
            if !c.input(k) {
                c.push_literal(k);
            }
        }
        c.text().to_string()
    }

    #[test]
    fn a_stray_final_is_not_swallowed() {
        for head in ["\u{119E}", "\u{D7B0}", "\u{115F}\u{119E}"] {
            let text = format!("{head}\u{11AF}");
            let mut c = Composer::from_text(&text);
            c.input('r');
            assert_eq!(c.text(), format!("{text}ㄱ"));
            let mut c = Composer::from_text(&text);
            c.backspace();
            assert_eq!(c.text(), head);
        }
    }

    #[test]
    fn layout_plain_and_shift() {
        assert_eq!(key_to_jamo('q'), Some('ㅂ'));
        assert_eq!(key_to_jamo('Q'), Some('ㅃ'));
        assert_eq!(key_to_jamo('W'), Some('ㅉ'));
        assert_eq!(key_to_jamo('E'), Some('ㄸ'));
        assert_eq!(key_to_jamo('R'), Some('ㄲ'));
        assert_eq!(key_to_jamo('T'), Some('ㅆ'));
        assert_eq!(key_to_jamo('O'), Some('ㅒ'));
        assert_eq!(key_to_jamo('P'), Some('ㅖ'));
        assert_eq!(key_to_jamo('m'), Some('ㅡ'));
        assert_eq!(key_to_jamo('1'), None);
        assert_eq!(key_to_jamo(' '), None);
    }

    #[test]
    fn old_letters_on_the_shift_layer() {
        let got: String = "ADGKZXCVBN".chars().map(|c| key_to_jamo(c).unwrap()).collect();
        assert_eq!(got, "ㅿㆁㆆㆍᄼᄾᅎᅐᅔᅕ");
    }

    #[test]
    fn every_letter_key_has_a_jamo() {
        for c in ('a'..='z').chain('A'..='Z') {
            assert!(key_to_jamo(c).is_some(), "{c}");
        }
    }

    #[test]
    fn simple_syllables() {
        assert_eq!(type_keys("gks"), "한");
        assert_eq!(type_keys("rmf"), "글");
        assert_eq!(type_keys("rk"), "가");
        assert_eq!(type_keys("dk"), "아");
        assert_eq!(type_keys("Tk"), "싸");
    }

    #[test]
    fn coda_migrates_to_the_next_syllable() {
        assert_eq!(type_keys("rkr"), "각");
        assert_eq!(type_keys("rkrk"), "가가");
        assert_eq!(type_keys("gksrmf"), "한글");
        assert_eq!(type_keys("rkrt"), "갃");
        assert_eq!(type_keys("rkrtk"), "각사");
        assert_eq!(type_keys("rkRk"), "가까");
        assert_eq!(type_keys("rkTk"), "가싸");
    }

    #[test]
    fn complex_vowels() {
        assert_eq!(type_keys("rhk"), "과");
        assert_eq!(type_keys("rho"), "괘");
        assert_eq!(type_keys("rhl"), "괴");
        assert_eq!(type_keys("rnj"), "궈");
        assert_eq!(type_keys("rnp"), "궤");
        assert_eq!(type_keys("rnl"), "귀");
        assert_eq!(type_keys("rml"), "긔");
        assert_eq!(type_keys("hk"), "ㅘ");
    }

    #[test]
    fn complex_vowel_then_coda() {
        assert_eq!(type_keys("rhks"), "관");
        assert_eq!(type_keys("rhksk"), "과나");
    }

    #[test]
    fn complex_codas() {
        assert_eq!(type_keys("rksw"), "갅");
        assert_eq!(type_keys("rksg"), "갆");
        assert_eq!(type_keys("rkfr"), "갉");
        assert_eq!(type_keys("rkfa"), "갊");
        assert_eq!(type_keys("rkfq"), "갋");
        assert_eq!(type_keys("rkft"), "갌");
        assert_eq!(type_keys("rkfx"), "갍");
        assert_eq!(type_keys("rkfv"), "갎");
        assert_eq!(type_keys("rkfg"), "갏");
        assert_eq!(type_keys("rkqt"), "값");
    }

    #[test]
    fn complex_coda_splits_when_a_vowel_follows() {
        assert_eq!(type_keys("rkqtk"), "갑사");
        assert_eq!(type_keys("dkfrk"), "알가");
        assert_eq!(type_keys("rkfgk"), "갈하");
    }

    #[test]
    fn non_coda_consonants_start_a_new_syllable() {
        assert_eq!(type_keys("rkE"), "가ㄸ");
        assert_eq!(type_keys("rkEk"), "가따");
        assert_eq!(type_keys("rkQ"), "가ㅃ");
        assert_eq!(type_keys("rkW"), "가ㅉ");
    }

    #[test]
    fn coda_does_not_combine_when_not_a_pair() {
        assert_eq!(type_keys("rkrr"), "각ㄱ");
        assert_eq!(type_keys("rksr"), "간ㄱ");
    }

    #[test]
    fn lone_jamo() {
        assert_eq!(type_keys("r"), "ㄱ");
        assert_eq!(type_keys("rr"), "ㄱㄱ");
        assert_eq!(type_keys("gg"), "ㅎㅎ");
        assert_eq!(type_keys("k"), "ㅏ");
        assert_eq!(type_keys("kk"), "ㅏㅏ");
        assert_eq!(type_keys("rk k"), "가 ㅏ");
    }

    #[test]
    fn a_vowel_after_a_closed_vowel_syllable_is_alone() {
        assert_eq!(type_keys("rkk"), "가ㅏ");
        assert_eq!(type_keys("rkl"), "가ㅣ");
    }

    #[test]
    fn spaces_and_literals_close_the_syllable() {
        assert_eq!(type_keys("gks rmf"), "한 글");
        assert_eq!(type_keys("rk1"), "가1");
        assert_eq!(type_keys("rk,k"), "가,ㅏ");
    }

    #[test]
    fn sentence() {
        assert_eq!(type_keys("dlfmf dkfma"), "이를 아름");
        assert_eq!(type_keys("tjsrl"), "선기");
    }

    #[test]
    fn arae_a_and_the_old_letters() {
        assert_eq!(type_keys("gK"), "\u{1112}\u{119E}"); // ᄒᆞ
        assert_eq!(type_keys("gKdu"), "\u{1112}\u{119E}여"); // ᄒᆞ여
        assert_eq!(type_keys("gKs"), "\u{1112}\u{119E}\u{11AB}"); // ᄒᆞᆫ
        assert_eq!(type_keys("gKsk"), "\u{1112}\u{119E}나");
        assert_eq!(type_keys("rKl"), "\u{1100}\u{11A1}"); // ᄀᆡ
        assert_eq!(type_keys("rKK"), "\u{1100}\u{11A2}"); // ᄀᆢ
        assert_eq!(type_keys("Ak"), "\u{1140}\u{1161}"); // ᅀᅡ
        assert_eq!(type_keys("Dk"), "\u{114C}\u{1161}"); // ᅌᅡ
        assert_eq!(type_keys("rkD"), "\u{1100}\u{1161}\u{11F0}"); // 가 + final ㆁ
        assert_eq!(type_keys("K"), "ㆍ");
        assert_eq!(type_keys("A"), "ㅿ");
        assert_eq!(type_keys("Zk"), "\u{113C}\u{1161}");
    }

    fn send_letters(letters: &str) -> String {
        let mut c = Composer::new();
        for l in letters.chars() {
            assert!(c.input_letter(l), "{l}");
        }
        c.text().to_string()
    }

    #[test]
    fn letters_compose_as_the_keys_do() {
        assert_eq!(send_letters("ㅇㅏㄴㄴㅕㅇ"), type_keys("dkssud"));
        assert_eq!(send_letters("ㅎㅏㄴㄱㅡㄹ"), type_keys("gksrmf"));
        assert_eq!(send_letters("ㄱㅗㅏ"), type_keys("rhk"));
        assert_eq!(send_letters("ㄱㅏㅂㅅㅏ"), type_keys("rkqtk"));
    }

    #[test]
    fn old_letters_compose_as_the_keys_do() {
        assert_eq!(send_letters("ㅎㆍ"), type_keys("gK"));
        assert_eq!(send_letters("ㅎㆍㄴ"), type_keys("gKs"));
        assert_eq!(send_letters("ㅿㅏ"), type_keys("Ak"));
        assert_eq!(send_letters("ㆁㅏ"), type_keys("Dk"));
        assert_eq!(send_letters("ㆆㅏ"), type_keys("Gk"));
        assert_eq!(send_letters("ㄱㅏㆁ"), type_keys("rkD"));
    }

    #[test]
    fn a_joined_letter_goes_in_as_its_letters() {
        assert_eq!(send_letters("ㄱㅘ"), "과");
        assert_eq!(send_letters("ㄱㅏㄳ"), "갃");
        assert_eq!(send_letters("ㄱㅏㄳㅏ"), "각사");
        assert_eq!(send_letters("ㅸㅏ"), "\u{112B}\u{1161}");
        assert_eq!(send_letters("ㄱㅏㅄ"), "값");
        // Not onto the syllable before it, where the keys would not join either.
        assert_eq!(send_letters("ㄱㅏㅘ"), type_keys("rkhk"));
        assert_eq!(send_letters("ㅇㅡㅢ"), type_keys("dmml"));
        assert_eq!(send_letters("ㅄㅏ"), type_keys("qtk"));
    }

    #[test]
    fn not_every_character_is_a_letter() {
        let mut c = Composer::new();
        for ch in ['a', '1', '가', '\u{3164}', '\u{1100}'] {
            assert!(!c.input_letter(ch), "{ch}");
        }
        assert_eq!(c.text(), "");
    }

    #[test]
    fn modern_letters_cluster_only_when_old_is_on() {
        assert_eq!(type_keys("trna"), "ㅅ굼"); // plain: ᄭ is not formed
        assert_eq!(type_with("trna", true), "\u{112D}\u{116E}\u{11B7}"); // ᄭᅮᆷ
        assert_eq!(type_with("qdk", true), "\u{112B}\u{1161}"); // ᄫᅡ
        assert_eq!(type_with("qtrk", true), "\u{1122}\u{1161}"); // ᄢᅡ
        assert_eq!(type_with("gg", true), "ㆅ"); // ᅘ alone, in its compatibility form
        // Modern syllables stay precomposed.
        assert_eq!(type_with("gksrmf", true), "한글");
    }

    #[test]
    fn old_text_reopens_and_backspaces_by_letter() {
        let mut c = Composer::from_text("ᄒᆞᆫ");
        c.backspace();
        assert_eq!(c.text(), "\u{1112}\u{119E}");
        c.backspace();
        assert_eq!(c.text(), "ㅎ");
        let mut c = Composer::from_text("\u{1100}\u{11A2}");
        c.backspace();
        assert_eq!(c.text(), "\u{1100}\u{119E}");
        let mut c = Composer::from_text("\u{1112}\u{119E}");
        c.input('s');
        assert_eq!(c.text(), "\u{1112}\u{119E}\u{11AB}");
    }

    #[test]
    fn backspace_is_jamo_level() {
        let mut c = Composer::new();
        for k in "rhksk".chars() {
            c.input(k);
        }
        assert_eq!(c.text(), "과나");
        c.backspace();
        assert_eq!(c.text(), "과ㄴ");
        c.backspace();
        assert_eq!(c.text(), "과");
        c.backspace();
        assert_eq!(c.text(), "고");
        c.backspace();
        assert_eq!(c.text(), "ㄱ");
        c.backspace();
        assert_eq!(c.text(), "");
        assert!(!c.backspace());
    }

    #[test]
    fn backspace_splits_complex_coda() {
        let mut c = Composer::from_text("값");
        c.backspace();
        assert_eq!(c.text(), "갑");
        c.backspace();
        assert_eq!(c.text(), "가");
        c.backspace();
        assert_eq!(c.text(), "ㄱ");
    }

    #[test]
    fn backspace_crosses_syllables_and_literals() {
        let mut c = Composer::from_text("한 글");
        c.backspace();
        assert_eq!(c.text(), "한 그");
        c.backspace();
        assert_eq!(c.text(), "한 ㄱ");
        c.backspace();
        assert_eq!(c.text(), "한 ");
        c.backspace();
        assert_eq!(c.text(), "한");
        c.backspace();
        assert_eq!(c.text(), "하");
    }

    #[test]
    fn backspace_then_retype() {
        let mut c = Composer::new();
        for k in "rkr".chars() {
            c.input(k);
        }
        c.backspace();
        c.input('s');
        assert_eq!(c.text(), "간");
    }

    #[test]
    fn syllable_round_trip() {
        for ch in ["가", "힣", "값", "뷁", "꽁", "의", "\u{1112}\u{119E}", "\u{112D}\u{116E}\u{11B7}"] {
            let (at, s) = open_syllable(ch).unwrap();
            assert_eq!(at, 0);
            assert_eq!(s.render(), ch);
        }
    }
}
