//! Modern readings of old and era spellings, found by a beam search over fold
//! classes. A run is split into units (onset, vowel, coda); each class
//! rewrites one or two neighbouring units, never changing their number, so a
//! reading lines up with the syllables as typed.
//!
//! The classes and the evidence for each are listed in docs/orthography.md.

use crate::jamo::{is_syllable, join_syllable, SBASE};
use crate::old_spelling::{is_cho, is_jung, syllable_end};

/// One syllable as its modern jamo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Unit {
    pub l: char,
    pub v: char,
    pub t: Option<char>,
}

impl Unit {
    fn of(c: char) -> Unit {
        let s = c as u32 - SBASE;
        let ch = |u: u32| char::from_u32(u).unwrap();
        let t = s % 28;
        Unit { l: ch(0x1100 + s / 588), v: ch(0x1161 + (s % 588) / 28), t: (t != 0).then(|| ch(0x11A7 + t)) }
    }
    fn ch(self) -> Option<char> {
        join_syllable(self.l, self.v, self.t)
    }
}

/// One syllable of a Hangul run as typed, with the units it can be read as:
/// each with a penalty and whether the fold is optional (a guess rather than
/// the one correspondence).
pub(crate) struct Syl {
    pub typed: String,
    pub reads: Vec<(Unit, f32, bool)>,
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
/// ㆍ (U+119E) to ㅏ or ㅡ, and as a guess ㅗ or ㅜ; ᆡ (U+11A1, ㆍ+ㅣ) to ㅐ,
/// as a guess ㅔ; ㅿ to ㅅ, ㆁ to ㅇ, ㅸ to ㅂ, as a guess ㅇ; and the initial
/// clusters of ㅅ or ㅂ with a consonant (ᄭ ᄯ ᄲ ᄶ ᄡ ᄧ ᄠ …) to the tense
/// consonant of the second. They give the modern letter that the word's
/// modern spelling has in that place; the engine uses them only to look
/// words up.
pub(crate) fn parse(run: &str) -> Option<Vec<Syl>> {
    let cs: Vec<char> = run.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        if is_syllable(cs[i]) {
            out.push(Syl { typed: cs[i].to_string(), reads: vec![(Unit::of(cs[i]), 0.0, false)] });
            i += 1;
            continue;
        }
        let j = syllable_end(&cs, i);
        out.push(parse_unit(&cs[i..j])?);
        i = j;
    }
    Some(out)
}

/// Whether the engine can read a unit (a syllable, or a run of L+ V+ T*).
pub(crate) fn foldable(unit: &str) -> bool {
    let cs: Vec<char> = unit.chars().collect();
    match cs.as_slice() {
        [c] if is_syllable(*c) => true,
        [] => false,
        _ => parse_unit(&cs).is_some(),
    }
}

/// One conjoining syllable: its initials, medials and finals, each a single
/// jamo or a sequence that makes one (ᄉ+ᄀ, ᅩ+ᅡ, ᆯ+ᆷ).
fn parse_unit(span: &[char]) -> Option<Syl> {
    let nl = span.iter().take_while(|c| is_cho(**c)).count();
    let nv = span[nl..].iter().take_while(|c| is_jung(**c)).count();
    let (ls, rest) = span.split_at(nl);
    let (vs, ts) = rest.split_at(nv);
    if ls.is_empty() || vs.is_empty() {
        return None;
    }
    let t = match ts {
        [] => None,
        [t] => Some(fold_t(*t)?),
        [a, b] => Some(fold_t(cluster(*a, *b)?)?),
        _ => return None,
    };
    let lead = match ls {
        [l] => *l,
        _ => compose_l(ls)?,
    };
    let vowels = fold_v(&match vs {
        [_] | ['\u{119E}', '\u{1175}'] => vs.to_vec(),
        _ => vec![compose_v(vs)?],
    })?;
    let mut reads = Vec::new();
    for (lc, lp, lo) in fold_l(lead)? {
        for &(v, vp, vo) in &vowels {
            if let Some(c) = join_syllable(lc, v, t) {
                reads.push((Unit::of(c), lp + vp, lo || vo));
            }
        }
    }
    (!reads.is_empty()).then(|| Syl { typed: span.iter().collect(), reads })
}

/// Initials written as a sequence of jamo: ㅅㄱ is ᄭ, ㅂㅅㄷ is ᄣ.
fn compose_l(ls: &[char]) -> Option<char> {
    Some(match ls.iter().map(|c| *c as u32).collect::<Vec<_>>()[..] {
        [0x1109, 0x1100] => '\u{112D}',
        [0x1109, 0x1103] => '\u{112F}',
        [0x1109, 0x1107] => '\u{1132}',
        [0x1109, 0x110C] => '\u{1136}',
        [0x1107, 0x1109] => '\u{1121}',
        [0x1107, 0x1100] => '\u{111E}',
        [0x1107, 0x1103] => '\u{1120}',
        [0x1107, 0x110C] => '\u{1127}',
        [0x1107, 0x110A] => '\u{1125}',
        [0x1107, 0x1109, 0x1100] => '\u{1122}',
        [0x1107, 0x1109, 0x1103] => '\u{1123}',
        [0x1107, 0x1109, 0x1107] => '\u{1124}',
        [0x1107, 0x1109, 0x110C] => '\u{1126}',
        [0x1100, 0x1100] => '\u{1101}',
        [0x1103, 0x1103] => '\u{1104}',
        [0x1107, 0x1107] => '\u{1108}',
        [0x1109, 0x1109] => '\u{110A}',
        [0x110C, 0x110C] => '\u{110D}',
        _ => return None,
    })
}

/// Medials written as a sequence of jamo: ᅩ+ᅡ is ᅪ.
fn compose_v(vs: &[char]) -> Option<char> {
    Some(match vs.iter().map(|c| *c as u32).collect::<Vec<_>>()[..] {
        [0x1169, 0x1161] => '\u{116A}',
        [0x1169, 0x1162] | [0x1169, 0x1161, 0x1175] => '\u{116B}',
        [0x1169, 0x1175] => '\u{116C}',
        [0x116E, 0x1165] => '\u{116F}',
        [0x116E, 0x1166] | [0x116E, 0x1165, 0x1175] => '\u{1170}',
        [0x116E, 0x1175] => '\u{1171}',
        [0x1173, 0x1175] => '\u{1174}',
        [0x1161, 0x1175] => '\u{1162}',
        [0x1165, 0x1175] => '\u{1166}',
        [0x1175, 0x1161] => '\u{1163}',
        [0x1175, 0x1165] => '\u{1167}',
        [0x1175, 0x1169] => '\u{116D}',
        [0x1175, 0x116E] => '\u{1172}',
        [0x1175, 0x1162] => '\u{1164}',
        [0x1175, 0x1166] => '\u{1168}',
        _ => return None,
    })
}

fn fold_l(l: char) -> Option<Vec<(char, f32, bool)>> {
    let one = |c: char| Some(vec![(c, 0.0, false)]);
    match l as u32 {
        0x1100..=0x1112 => one(l),
        // ㅿ mostly fell away (거ᅀᅵ 거의, ᄆᆞᅀᆞᆷ 마음), and is ㅅ in a few words.
        0x1140 => Some(vec![('\u{1109}', 0.0, false), ('\u{110B}', 0.0, true)]),
        0x114C => one('\u{110B}'),
        0x112B => Some(vec![('\u{1107}', 0.0, false), ('\u{110B}', 0.4, true)]),
        // ㅅ or ㅂ + ㄱ, ㅅ+ㄷ, ㅂ+ㄷ, ㅅ+ㅂ, ㅂ+ㅅㄱ, ㅅ+ㅈ, ㅂ+ㅈ: the tense consonant.
        0x112D | 0x111E | 0x1122 => one('\u{1101}'),
        0x112F | 0x1120 | 0x1123 => one('\u{1104}'),
        0x1132 | 0x1124 => one('\u{1108}'),
        0x1136 | 0x1127 | 0x1126 => one('\u{110D}'),
        0x1121 | 0x1125 => one('\u{110A}'),
        _ => None,
    }
}

fn fold_v(vs: &[char]) -> Option<Vec<(char, f32, bool)>> {
    Some(match vs {
        [v] if (0x1161..=0x1175).contains(&(*v as u32)) => vec![(*v, 0.0, false)],
        [v] if *v == '\u{119E}' => vec![('\u{1161}', 0.0, false), ('\u{1173}', 0.05, false), ('\u{1169}', 0.5, true), ('\u{116E}', 0.5, true)],
        [v] if *v == '\u{11A1}' => vec![('\u{1162}', 0.0, false), ('\u{1166}', 0.3, true)],
        ['\u{119E}', '\u{1175}'] => vec![('\u{1162}', 0.0, false), ('\u{1166}', 0.3, true)],
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

// Classes. Each rewrites units that look like an era spelling; the first four
// are spellings of the era, the rest are guesses that a dictionary hit must
// confirm.
/// ㄷ ㅌ before ㅑ ㅕ ㅛ were ㅈ ㅊ before ㅏ ㅓ ㅗ (텬하 = 천하, 뎌 = 저, 됴 = 조).
pub(crate) const PALATAL: u16 = 1;
/// 디 티 were 지 치 (only for words left unconverted: loanwords have them).
pub(crate) const PALATAL_I: u16 = 2;
/// ㅅ ㅈ ㅊ before ㅑ ㅕ ㅛ ㅠ: 셔 = 서, 죠 = 조, 슈 = 수.
pub(crate) const GLIDE: u16 = 4;
/// A final ㅅ was a final ㅆ after the vowels of the past tense (갓다 = 갔다).
pub(crate) const PAST_S: u16 = 8;
/// The seven finals: ㅂ for ㅄ ㅍ (업시 = 없이, 놉다 = 높다), ㅅ ㄷ for ㅈ ㅊ ㅌ ㅆ.
pub(crate) const FINAL: u16 = 16;
/// ㅎ and a following ㄷ ㄱ ㅈ written as ㅌ ㅋ ㅊ (됴타 = 좋다, 만타 = 많다).
pub(crate) const FUSION: u16 = 32;
/// A coda written as the onset of the next syllable (사르미 = 사람이).
pub(crate) const CARRY: u16 = 64;
/// ㄴ for ㅇ before ㅣ ㅑ ㅕ ㅛ ㅠ at the start of a word (녀름 = 여름).
pub(crate) const INITIAL: u16 = 128;
/// ㅡ for ㅜ after ㅁ ㅂ ㅍ (믈 = 물).
pub(crate) const LABIAL: u16 = 256;
pub(crate) const ALL: u16 = 511;
/// Classes tried for every run that shows them, converted or not.
pub(crate) const STRONG: u16 = PALATAL | PAST_S | FUSION;

const CLASSES: [u16; 9] = [PALATAL, PALATAL_I, GLIDE, PAST_S, FINAL, FUSION, CARRY, INITIAL, LABIAL];

struct Op {
    changes: Vec<(usize, Unit)>,
    pen: f32,
    class: u16,
}

fn plain_of(v: char) -> Option<char> {
    Some(match v {
        '\u{1163}' => '\u{1161}',
        '\u{1167}' => '\u{1165}',
        '\u{116D}' => '\u{1169}',
        '\u{1172}' => '\u{116E}',
        _ => return None,
    })
}

/// The coda that an onset becomes (ㅎ too: 됴흔 = 좋은).
fn coda_of(l: char) -> Option<char> {
    Some(match l {
        '\u{1100}' => '\u{11A8}',
        '\u{1102}' => '\u{11AB}',
        '\u{1103}' => '\u{11AE}',
        '\u{1105}' => '\u{11AF}',
        '\u{1106}' => '\u{11B7}',
        '\u{1107}' => '\u{11B8}',
        '\u{1109}' => '\u{11BA}',
        '\u{110C}' => '\u{11BD}',
        '\u{110E}' => '\u{11BE}',
        '\u{110F}' => '\u{11BF}',
        '\u{1110}' => '\u{11C0}',
        '\u{1111}' => '\u{11C1}',
        '\u{1112}' => '\u{11C2}',
        _ => return None,
    })
}

/// A coda followed by a carried one, as one cluster coda.
fn cluster(t: char, c: char) -> Option<char> {
    Some(match (t, c) {
        ('\u{11AF}', '\u{11A8}') => '\u{11B0}',
        ('\u{11AF}', '\u{11B7}') => '\u{11B1}',
        ('\u{11AF}', '\u{11B8}') => '\u{11B2}',
        ('\u{11AF}', '\u{11BA}') => '\u{11B3}',
        ('\u{11AF}', '\u{11C0}') => '\u{11B4}',
        ('\u{11AF}', '\u{11C1}') => '\u{11B5}',
        ('\u{11AB}', '\u{11BD}') => '\u{11AC}',
        ('\u{11AB}', '\u{11C2}') => '\u{11AD}',
        ('\u{11AF}', '\u{11C2}') => '\u{11B6}',
        ('\u{11B8}', '\u{11BA}') => '\u{11B9}',
        ('\u{11A8}', '\u{11BA}') => '\u{11AA}',
        _ => return None,
    })
}

/// The ops of the enabled classes that start at unit `i`. `strict` narrows
/// the guesses to forms that look like the era's, for runs typed in modern
/// letters.
fn ops(us: &[Unit], i: usize, mask: u16, strict: bool, out: &mut Vec<Op>) {
    let u = us[i];
    let one = |to: Unit, pen: f32, class: u16| Op { changes: vec![(i, to)], pen, class };
    if mask & PALATAL != 0 {
        if let (Some(l), Some(v)) = (match u.l { '\u{1103}' => Some('\u{110C}'), '\u{1110}' => Some('\u{110E}'), _ => None }, plain_of(u.v).filter(|v| *v != '\u{116E}')) {
            out.push(one(Unit { l, v, ..u }, 0.6, PALATAL));
        }
    }
    if mask & PALATAL_I != 0 && u.v == '\u{1175}' {
        let l = match u.l { '\u{1103}' => Some('\u{110C}'), '\u{1110}' => Some('\u{110E}'), _ => None };
        if let Some(l) = l {
            out.push(one(Unit { l, ..u }, 0.8, PALATAL_I));
        }
    }
    if mask & GLIDE != 0 && matches!(u.l, '\u{1109}' | '\u{110C}' | '\u{110E}') {
        if let Some(v) = plain_of(u.v) {
            out.push(one(Unit { v, ..u }, 0.8, GLIDE));
        }
    }
    if mask & PAST_S != 0 && u.t == Some('\u{11BA}') && matches!(u.v, '\u{1161}' | '\u{1162}' | '\u{1165}' | '\u{1167}' | '\u{1175}') && !(u.l == '\u{1100}' && u.v == '\u{1165}') {
        out.push(one(Unit { t: Some('\u{11BB}'), ..u }, 0.5, PAST_S));
    }
    if mask & FINAL != 0 {
        if let Some(t) = u.t {
            let alts: &[(u32, f32)] = match t as u32 {
                0x11BA => &[(0x11BD, 0.8), (0x11BE, 0.8), (0x11C0, 0.8)],
                0x11AE => &[(0x11BA, 0.6), (0x11BB, 0.6), (0x11BD, 0.8), (0x11BE, 0.8), (0x11C0, 0.8)],
                0x11B8 => &[(0x11B9, 0.6), (0x11C1, 0.7)],
                0x11A8 if !strict => &[(0x11BF, 0.7), (0x11A9, 0.7), (0x11AA, 0.8)],
                0x11AB if !strict => &[(0x11AC, 0.8), (0x11AD, 0.8)],
                0x11AF if !strict => &[(0x11B0, 0.8), (0x11B1, 0.8), (0x11B6, 0.8)],
                _ => &[],
            };
            for &(to, pen) in alts {
                out.push(one(Unit { t: char::from_u32(to), ..u }, pen, FINAL));
            }
            if !strict && t == '\u{11BA}' {
                out.push(one(Unit { t: Some('\u{11BB}'), ..u }, 0.6, FINAL));
            }
        }
    }
    if let Some(&n) = us.get(i + 1) {
        if mask & FUSION != 0 && (!strict || continues(&us[i + 2..])) {
            let l = match n.l { '\u{1110}' => Some('\u{1103}'), '\u{110F}' => Some('\u{1100}'), '\u{110E}' => Some('\u{110C}'), '\u{1111}' => Some('\u{1107}'), _ => None };
            let t = match u.t { None => Some('\u{11C2}'), Some('\u{11AB}') => Some('\u{11AD}'), Some('\u{11AF}') => Some('\u{11B6}'), _ => None };
            if let (Some(l), Some(t)) = (l, t) {
                out.push(Op { changes: vec![(i, Unit { t: Some(t), ..u }), (i + 1, Unit { l, ..n })], pen: 0.5, class: FUSION });
            }
        }
        if mask & CARRY != 0 && n.l != '\u{110B}' && (!strict || i + 2 == us.len() || (n.l == '\u{1112}' && i + 3 == us.len())) {
            let ok = !strict
                || (matches!(n.v, '\u{1175}' | '\u{1166}' | '\u{1161}' | '\u{1165}') && n.t.is_none())
                || (n.v == '\u{1173}' && matches!(n.t, Some('\u{11AB}' | '\u{11AF}')));
            if let (true, Some(c)) = (ok, coda_of(n.l)) {
                let t = match u.t { None => Some(c), Some(t) => cluster(t, c) };
                if let Some(t) = t {
                    out.push(Op { changes: vec![(i, Unit { t: Some(t), ..u }), (i + 1, Unit { l: '\u{110B}', ..n })], pen: 0.6, class: CARRY });
                    // 사르미 = 사람이: ㆍ was written ㅡ.
                    if u.v == '\u{1173}' {
                        out.push(Op { changes: vec![(i, Unit { t: Some(t), v: '\u{1161}', ..u }), (i + 1, Unit { l: '\u{110B}', ..n })], pen: 0.8, class: CARRY });
                    }
                }
            }
        }
    }
    if mask & INITIAL != 0 && i == 0 && u.l == '\u{1102}' && matches!(u.v, '\u{1175}' | '\u{1163}' | '\u{1167}' | '\u{116D}' | '\u{1172}') {
        out.push(one(Unit { l: '\u{110B}', ..u }, 0.6, INITIAL));
    }
    if mask & LABIAL != 0 && matches!(u.l, '\u{1106}' | '\u{1107}' | '\u{1111}') && u.v == '\u{1173}' {
        out.push(one(Unit { v: '\u{116E}', ..u }, 0.7, LABIAL));
    }
}

/// What may follow a fused ending in a modern-letter run: nothing, or one or
/// two of the common endings and particles (됴코요, 됴타고, 됴치만).
fn continues(rest: &[Unit]) -> bool {
    rest.len() <= 2 && rest.iter().all(|u| u.ch().is_some_and(|c| "요고는은도만지게서며면네다라나니죠야까".contains(c)))
}

fn units_of(run: &str) -> Vec<Unit> {
    run.chars().filter(|c| is_syllable(*c)).map(Unit::of).collect()
}

/// The classes that have something to rewrite in a run of modern syllables:
/// the strong ones always, the others only for a run left unconverted.
pub(crate) fn modern_mask(run: &str, unconverted: bool) -> u16 {
    // Most runs have nothing to try: look at them without allocating.
    if !run.chars().any(|c| is_syllable(c) && (strong_marker(Unit::of(c)) || unconverted)) {
        return 0;
    }
    let us = units_of(run);
    // Era spellings are short words; a long unconverted run is a name or a
    // compound, and trying it costs more than it finds.
    let unconverted = unconverted && us.len() <= MAX_WEAK_UNITS;
    let mut found = 0;
    let mut sink = Vec::new();
    for i in 0..us.len() {
        for c in CLASSES {
            if found & c == 0 && (c & STRONG != 0 || unconverted) {
                ops(&us, i, c, true, &mut sink);
                if !sink.is_empty() {
                    found |= c;
                    sink.clear();
                }
            }
        }
    }
    found
}

/// The strong classes' marks: ㄷ ㅌ before ㅑ ㅕ ㅛ, ㅅ after a past-tense vowel, or a ㅌ ㅋ ㅊ ㅍ onset (the ending of 좋다, 많다 …).
fn strong_marker(u: Unit) -> bool {
    matches!(u.l, '\u{1110}' | '\u{110F}' | '\u{110E}' | '\u{1111}') || (matches!(u.l, '\u{1103}' | '\u{1110}') && matches!(u.v, '\u{1163}' | '\u{1167}' | '\u{116D}')) || (u.t == Some('\u{11BA}') && matches!(u.v, '\u{1161}' | '\u{1162}' | '\u{1165}' | '\u{1167}' | '\u{1175}'))
}

/// Longest run of modern syllables whose weak classes are tried.
const MAX_WEAK_UNITS: usize = 6;

/// A syllable of the era that modern words do not use: ㄷ ㅌ before ㅑ ㅛ, or
/// before ㅕ with a final other than ㅆ (텬 됴 댜 뎡; but 견뎌 and 버텼다 are
/// modern).
pub(crate) fn has_era_only(run: &str) -> bool {
    run.chars().any(|c| {
        is_syllable(c) && {
            let u = Unit::of(c);
            matches!(u.l, '\u{1103}' | '\u{1110}') && (matches!(u.v, '\u{1163}' | '\u{116D}') || (u.v == '\u{1167}' && !matches!(u.t, None | Some('\u{11BB}'))))
        }
    })
}

/// At most this many readings are tried for a run.
pub(crate) const MAX_READINGS_OLD: usize = 24;
pub(crate) const MAX_READINGS_MODERN: usize = 12;
const BEAM: usize = 12;
const DEPTH: usize = 3;

pub(crate) struct Reading {
    pub text: String,
    pub penalty: f32,
    /// A guess was used (a class or an optional fold), so the conversion must
    /// cover the whole reading to count.
    pub optional: bool,
    /// The classes that rewrote it (read by the tests).
    pub classes: u16,
}

/// Modern readings of a run, best guess first.
pub(crate) fn readings(syls: &[Syl], mask: u16, strict: bool, cap: usize) -> Vec<Reading> {
    let typed: String = syls.iter().map(|s| s.typed.as_str()).collect();
    // Every combination of the folds of each syllable, the likeliest first.
    let mut base: Vec<(Vec<Unit>, f32, bool)> = vec![(Vec::new(), 0.0, false)];
    for s in syls {
        let mut next = Vec::new();
        for (us, p, o) in &base {
            for &(u, up, uo) in &s.reads {
                let mut v = us.clone();
                v.push(u);
                next.push((v, p + up, *o || uo));
            }
        }
        next.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        next.truncate(BEAM);
        base = next;
    }
    let text_of = |us: &[Unit]| -> Option<String> { us.iter().map(|u| u.ch()).collect() };
    let mut seen = std::collections::HashSet::new();
    let mut all: Vec<Reading> = Vec::new();
    let mut frontier: Vec<(Vec<Unit>, f32, u16)> = Vec::new();
    for (us, p, o) in base {
        let Some(text) = text_of(&us) else { continue };
        if seen.insert(text.clone()) {
            if text != typed {
                all.push(Reading { text, penalty: p, optional: o, classes: 0 });
            }
            frontier.push((us, p, 0));
        }
    }
    let mut buf = Vec::new();
    for _ in 0..DEPTH {
        let mut next: Vec<(Vec<Unit>, f32, u16)> = Vec::new();
        for (us, p, used) in &frontier {
            for i in 0..us.len() {
                buf.clear();
                ops(us, i, mask, strict, &mut buf);
                for op in buf.drain(..) {
                    let mut v = us.clone();
                    for (at, u) in &op.changes {
                        v[*at] = *u;
                    }
                    let Some(text) = text_of(&v) else { continue };
                    if seen.insert(text.clone()) {
                        all.push(Reading { text, penalty: p + op.pen, optional: true, classes: used | op.class });
                        next.push((v, p + op.pen, used | op.class));
                    }
                }
            }
        }
        next.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        next.truncate(BEAM);
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }
    all.sort_by(|a, b| a.penalty.partial_cmp(&b.penalty).unwrap());
    all.truncate(cap);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reads(run: &str, mask: u16, strict: bool) -> Vec<String> {
        readings(&parse(run).unwrap(), mask, strict, 24).into_iter().map(|r| r.text).collect()
    }
    fn has(run: &str, mask: u16, strict: bool, want: &str) {
        let r = reads(run, mask, strict);
        assert!(r.iter().any(|t| t == want), "{run}: {want} not in {r:?}");
    }

    #[test]
    fn old_letters_fold_to_modern_ones() {
        // ᄉᆞ랑 = 사랑, ᄂᆞᆫ = 는, ᄒᆞᆰ = 흙; the ㅏ reading comes first.
        assert_eq!(reads("\u{1109}\u{119E}랑", 0, false)[..2], ["사랑", "스랑"]);
        assert_eq!(reads("\u{1102}\u{119E}\u{11AB}", 0, false)[..2], ["난", "는"]);
        has("\u{1112}\u{119E}\u{11B0}", 0, false, "흙");
        assert_eq!(reads("\u{1112}\u{11A1}", 0, false)[0], "해");
        assert_eq!(reads("\u{1140}\u{1175}", 0, false), ["시", "이"]);
        assert_eq!(reads("\u{114C}\u{1175}", 0, false), ["이"]);
        assert_eq!(reads("\u{112F}\u{1169}", 0, false), ["또"]);
        assert_eq!(reads("\u{1109}\u{11A1}\u{11F0}", 0, false)[0], "생");
        // The clusters of ㅅ or ㅂ with a consonant read as the tense one.
        has("\u{1127}\u{1161}\u{11A8}", 0, false, "짝");
        has("\u{1120}\u{1173}\u{11BA}", 0, false, "뜻");
        has("\u{112D}\u{1169}\u{11BA}", 0, false, "꼿");
        // ㆍ after ㅁ is ㅜ in 아ᄆᆞ리 = 아무리.
        has("아\u{1106}\u{119E}리", 0, false, "아무리");
        // A letter with no modern counterpart gives no reading.
        assert!(parse("\u{1113}\u{1161}").is_none());
    }

    #[test]
    fn classes_read_pronounced_spellings() {
        has("텬하", ALL, true, "천하");
        has("됴타", ALL, true, "좋다");
        has("만타", ALL, true, "많다");
        has("안코", ALL, true, "않고");
        has("뎌", ALL, true, "저");
        has("댜", ALL, true, "자");
        has("갓다", ALL, true, "갔다");
        has("잇다", ALL, true, "있다");
        has("업시", ALL, true, "없이");
        has("갑시", ALL, true, "값이");
        has("그르슬", ALL, true, "그릇을");
        has("사르미", ALL, true, "사람이");
        has("마으미", ALL, true, "마음이");
        has("니르다", ALL, true, "이르다");
        has("닙다", ALL, true, "입다");
        has("녀름", ALL, true, "여름");
        has("믈", ALL, true, "물");
        has("블", ALL, true, "불");
        has("플", ALL, true, "풀");
        has("셔울", ALL, true, "서울");
        has("됴흔", ALL, true, "좋은");
        has("됴하셔", ALL, true, "좋아서");
        // Before the common endings and particles too.
        has("됴코요", ALL, true, "좋고요");
        has("됴타고", ALL, true, "좋다고");
        has("됴치만", ALL, true, "좋지만");
        has("죠션", ALL, true, "조선");
        // Old letters combine with the classes.
        has("사\u{1105}\u{119E}미", ALL, false, "사람이");
        has("마\u{110B}\u{119E}미", ALL, false, "마음이");
        has("둏\u{110B}\u{119E}\u{11AB}", ALL, false, "좋은");
        has("업\u{1102}\u{119E}\u{11AB}", ALL, false, "없는");
        has("놉다", ALL, false, "높다");
        has("노\u{1111}\u{119E}\u{11AB}", ALL, false, "높은");
    }

    #[test]
    fn modern_input_without_a_marker_has_no_readings() {
        assert_eq!(modern_mask("사랑", true), 0);
        assert_eq!(modern_mask("텬하", false), PALATAL);
        assert_ne!(modern_mask("갓다", false) & PAST_S, 0);
        assert_eq!(modern_mask("갓다", false) & FINAL, 0);
        assert_ne!(modern_mask("업시", true) & FINAL, 0);
        assert!(has_era_only("텬하") && has_era_only("됴타") && !has_era_only("견뎌") && !has_era_only("버텼다") && has_era_only("뎡쥬"));
    }

    #[test]
    fn at_most_the_cap() {
        let run = "\u{1109}\u{119E}".repeat(8);
        assert_eq!(readings(&parse(&run).unwrap(), ALL, false, 24).len(), 24);
    }
}
