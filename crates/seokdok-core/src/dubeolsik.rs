//! 두벌식 (KS X 5002) Hangul composition.
//!
//! The composition is a plain string. Only its last character is ever
//! "open": a syllable that can still take another jamo, or a lone jamo. All
//! other characters are final, so the automaton needs no state beyond the
//! string and backspace removes one jamo at a time.

const CHO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
const JUNG: &str = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ";
const JONG: &str = "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ";

const SBASE: u32 = 0xAC00;

/// Two-jamo vowels: (first, second, result).
const VOWEL_PAIRS: [(char, char, char); 7] = [
    ('ㅗ', 'ㅏ', 'ㅘ'),
    ('ㅗ', 'ㅐ', 'ㅙ'),
    ('ㅗ', 'ㅣ', 'ㅚ'),
    ('ㅜ', 'ㅓ', 'ㅝ'),
    ('ㅜ', 'ㅔ', 'ㅞ'),
    ('ㅜ', 'ㅣ', 'ㅟ'),
    ('ㅡ', 'ㅣ', 'ㅢ'),
];

/// Two-jamo codas: (first, second, result).
const CODA_PAIRS: [(char, char, char); 11] = [
    ('ㄱ', 'ㅅ', 'ㄳ'),
    ('ㄴ', 'ㅈ', 'ㄵ'),
    ('ㄴ', 'ㅎ', 'ㄶ'),
    ('ㄹ', 'ㄱ', 'ㄺ'),
    ('ㄹ', 'ㅁ', 'ㄻ'),
    ('ㄹ', 'ㅂ', 'ㄼ'),
    ('ㄹ', 'ㅅ', 'ㄽ'),
    ('ㄹ', 'ㅌ', 'ㄾ'),
    ('ㄹ', 'ㅍ', 'ㄿ'),
    ('ㄹ', 'ㅎ', 'ㅀ'),
    ('ㅂ', 'ㅅ', 'ㅄ'),
];

/// The jamo on a 두벌식 key. Lowercase letters are the plain layer, uppercase
/// letters the Shift layer; only ㅃㅉㄸㄲㅆㅒㅖ differ between the layers.
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
        'a' | 'A' => 'ㅁ',
        's' | 'S' => 'ㄴ',
        'd' | 'D' => 'ㅇ',
        'f' | 'F' => 'ㄹ',
        'g' | 'G' => 'ㅎ',
        'h' | 'H' => 'ㅗ',
        'j' | 'J' => 'ㅓ',
        'k' | 'K' => 'ㅏ',
        'l' | 'L' => 'ㅣ',
        'z' | 'Z' => 'ㅋ',
        'x' | 'X' => 'ㅌ',
        'c' | 'C' => 'ㅊ',
        'v' | 'V' => 'ㅍ',
        'b' | 'B' => 'ㅠ',
        'n' | 'N' => 'ㅜ',
        'm' | 'M' => 'ㅡ',
        _ => return None,
    })
}

pub fn is_vowel(j: char) -> bool {
    JUNG.contains(j)
}

pub fn is_consonant(j: char) -> bool {
    CHO.contains(j) || JONG.contains(j)
}

/// A syllable block under construction. All fields are compatibility jamo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Syl {
    cho: Option<char>,
    jung: Option<char>,
    jong: Option<char>,
}

impl Syl {
    fn from_char(c: char) -> Option<Syl> {
        if is_vowel(c) {
            return Some(Syl { jung: Some(c), ..Syl::default() });
        }
        if is_consonant(c) {
            return Some(Syl { cho: Some(c), ..Syl::default() });
        }
        let s = (c as u32).checked_sub(SBASE)?;
        if s >= 19 * 21 * 28 {
            return None;
        }
        let (l, v, t) = ((s / 588) as usize, (s % 588 / 28) as usize, (s % 28) as usize);
        Some(Syl {
            cho: CHO.chars().nth(l),
            jung: JUNG.chars().nth(v),
            jong: if t == 0 { None } else { JONG.chars().nth(t - 1) },
        })
    }

    fn to_char(self) -> char {
        match (self.cho, self.jung) {
            (Some(l), Some(v)) => {
                let li = CHO.chars().position(|c| c == l).unwrap() as u32;
                let vi = JUNG.chars().position(|c| c == v).unwrap() as u32;
                let ti = self
                    .jong
                    .map_or(0, |t| JONG.chars().position(|c| c == t).unwrap() as u32 + 1);
                char::from_u32(SBASE + (li * 21 + vi) * 28 + ti).unwrap()
            }
            (Some(l), None) => l,
            (None, Some(v)) => v,
            (None, None) => unreachable!("empty syllable"),
        }
    }
}

fn pair(table: &[(char, char, char)], a: char, b: char) -> Option<char> {
    table.iter().find(|p| p.0 == a && p.1 == b).map(|p| p.2)
}

fn split(table: &[(char, char, char)], r: char) -> Option<(char, char)> {
    table.iter().find(|p| p.2 == r).map(|p| (p.0, p.1))
}

fn can_be_coda(j: char) -> bool {
    JONG.contains(j)
}

/// The Hangul composition buffer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Composer {
    buf: String,
}

impl Composer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_text(text: &str) -> Self {
        Composer { buf: text.to_string() }
    }

    pub fn text(&self) -> &str {
        &self.buf
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

    /// Feed one compatibility jamo.
    pub fn input_jamo(&mut self, j: char) {
        let open = self.buf.chars().next_back().and_then(Syl::from_char);
        let Some(mut s) = open else {
            self.buf.push(j);
            return;
        };
        self.buf.pop();
        if is_vowel(j) {
            match (s.cho, s.jung, s.jong) {
                // 갃 + ㅏ → 각사, 갂 + ㅏ → 가까
                (Some(_), Some(_), Some(t)) => {
                    let (keep, moved) = match split(&CODA_PAIRS, t) {
                        Some((a, b)) => (Some(a), b),
                        None => (None, t),
                    };
                    s.jong = keep;
                    self.buf.push(s.to_char());
                    let next = Syl { cho: Some(moved), jung: Some(j), jong: None };
                    self.buf.push(next.to_char());
                }
                (cho, Some(v), None) => match pair(&VOWEL_PAIRS, v, j) {
                    Some(w) => {
                        self.buf.push(Syl { cho, jung: Some(w), jong: None }.to_char());
                    }
                    None => {
                        self.buf.push(s.to_char());
                        self.buf.push(j);
                    }
                },
                (Some(l), None, _) => {
                    self.buf.push(Syl { cho: Some(l), jung: Some(j), jong: None }.to_char());
                }
                _ => {
                    self.buf.push(s.to_char());
                    self.buf.push(j);
                }
            }
        } else {
            match (s.cho, s.jung, s.jong) {
                (Some(_), Some(_), None) if can_be_coda(j) => {
                    s.jong = Some(j);
                    self.buf.push(s.to_char());
                }
                (Some(_), Some(_), Some(t)) => match pair(&CODA_PAIRS, t, j) {
                    Some(c) => {
                        s.jong = Some(c);
                        self.buf.push(s.to_char());
                    }
                    None => {
                        self.buf.push(s.to_char());
                        self.buf.push(j);
                    }
                },
                _ => {
                    self.buf.push(s.to_char());
                    self.buf.push(j);
                }
            }
        }
    }

    /// Remove one jamo (or one literal character). Returns false when empty.
    pub fn backspace(&mut self) -> bool {
        let Some(last) = self.buf.pop() else {
            return false;
        };
        let Some(mut s) = Syl::from_char(last) else {
            return true;
        };
        if let Some(t) = s.jong {
            s.jong = split(&CODA_PAIRS, t).map(|p| p.0);
        } else if let Some(v) = s.jung {
            match split(&VOWEL_PAIRS, v) {
                Some((a, _)) => s.jung = Some(a),
                None => s.jung = None,
            }
        } else {
            s.cho = None;
        }
        if s.cho.is_some() || s.jung.is_some() {
            self.buf.push(s.to_char());
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_keys(keys: &str) -> String {
        let mut c = Composer::new();
        for k in keys.chars() {
            if !c.input(k) {
                c.push_literal(k);
            }
        }
        c.text().to_string()
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
        assert_eq!(key_to_jamo('K'), Some('ㅏ'));
        assert_eq!(key_to_jamo('m'), Some('ㅡ'));
        assert_eq!(key_to_jamo('1'), None);
        assert_eq!(key_to_jamo(' '), None);
    }

    #[test]
    fn every_letter_key_has_a_jamo() {
        for c in 'a'..='z' {
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
        assert_eq!(type_keys("rkrt"), "갃");
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
        // ㄸ ㅃ ㅉ cannot close a syllable.
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
        for ch in ['가', '힣', '값', '뷁', '꽁', '의'] {
            assert_eq!(Syl::from_char(ch).unwrap().to_char(), ch);
        }
    }
}
