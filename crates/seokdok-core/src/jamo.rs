//! Hangul decomposition into role-tagged jamo components and recomposition.
//!
//! Matching works on a sequence of *components*: an onset (U+1100 block
//! initial), simple vowels (U+1161 block, with every compound vowel split, so
//! ㅘ = ᅩᅡ, ㅕ = ᅵᅥ, ㅐ = ᅡᅵ), and simple codas (U+11A8 block, with complex
//! codas split, so ㄻ = ᆯᆷ). Onset and coda consonants have distinct code
//! points, so a match can never treat an onset as a coda or the reverse.

pub const SBASE: u32 = 0xAC00;
const LBASE: u32 = 0x1100;
const VBASE: u32 = 0x1161;
const TBASE: u32 = 0x11A7;
const LCOUNT: u32 = 19;
const VCOUNT: u32 = 21;
const TCOUNT: u32 = 28;
const SCOUNT: u32 = LCOUNT * VCOUNT * TCOUNT;

pub fn is_syllable(c: char) -> bool {
    (SBASE..SBASE + SCOUNT).contains(&(c as u32))
}
pub fn is_l(c: char) -> bool {
    (0x1100..=0x1112).contains(&(c as u32))
}
pub fn is_v(c: char) -> bool {
    (0x1161..=0x1175).contains(&(c as u32))
}
pub fn is_t(c: char) -> bool {
    (0x11A8..=0x11C2).contains(&(c as u32))
}

fn ch(u: u32) -> char {
    char::from_u32(u).unwrap()
}

/// Split one vowel jamo (U+1161..U+1175) into simple vowel components.
fn split_v(v: char) -> &'static [char] {
    match v {
        'ᅡ' => &['ᅡ'],
        'ᅢ' => &['ᅡ', 'ᅵ'],
        'ᅣ' => &['ᅵ', 'ᅡ'],
        'ᅤ' => &['ᅵ', 'ᅡ', 'ᅵ'],
        'ᅥ' => &['ᅥ'],
        'ᅦ' => &['ᅥ', 'ᅵ'],
        'ᅧ' => &['ᅵ', 'ᅥ'],
        'ᅨ' => &['ᅵ', 'ᅥ', 'ᅵ'],
        'ᅩ' => &['ᅩ'],
        'ᅪ' => &['ᅩ', 'ᅡ'],
        'ᅫ' => &['ᅩ', 'ᅡ', 'ᅵ'],
        'ᅬ' => &['ᅩ', 'ᅵ'],
        'ᅭ' => &['ᅵ', 'ᅩ'],
        'ᅮ' => &['ᅮ'],
        'ᅯ' => &['ᅮ', 'ᅥ'],
        'ᅰ' => &['ᅮ', 'ᅥ', 'ᅵ'],
        'ᅱ' => &['ᅮ', 'ᅵ'],
        'ᅲ' => &['ᅵ', 'ᅮ'],
        'ᅳ' => &['ᅳ'],
        'ᅴ' => &['ᅳ', 'ᅵ'],
        'ᅵ' => &['ᅵ'],
        _ => &[],
    }
}

/// Rebuild one vowel jamo from simple components, if they form one.
fn join_v(parts: &[char]) -> Option<char> {
    (0x1161..=0x1175u32).map(ch).find(|&v| split_v(v) == parts)
}

fn split_t(t: char) -> &'static [char] {
    match t {
        'ᆪ' => &['ᆨ', 'ᆺ'],
        'ᆬ' => &['ᆫ', 'ᆽ'],
        'ᆭ' => &['ᆫ', 'ᇂ'],
        'ᆰ' => &['ᆯ', 'ᆨ'],
        'ᆱ' => &['ᆯ', 'ᆷ'],
        'ᆲ' => &['ᆯ', 'ᆸ'],
        'ᆳ' => &['ᆯ', 'ᆺ'],
        'ᆴ' => &['ᆯ', 'ᇀ'],
        'ᆵ' => &['ᆯ', 'ᇁ'],
        'ᆶ' => &['ᆯ', 'ᇂ'],
        'ᆹ' => &['ᆸ', 'ᆺ'],
        _ => &[],
    }
}

fn join_t(parts: &[char]) -> Option<char> {
    match parts {
        [t] => Some(*t),
        _ => (0x11A8..=0x11C2u32).map(ch).find(|&t| split_t(t) == parts),
    }
}

/// Decompose a syllable into components.
pub fn decompose_syllable(s: char, out: &mut Vec<char>) {
    let i = s as u32 - SBASE;
    let l = i / (VCOUNT * TCOUNT);
    let v = (i % (VCOUNT * TCOUNT)) / TCOUNT;
    let t = i % TCOUNT;
    out.push(ch(LBASE + l));
    out.extend_from_slice(split_v(ch(VBASE + v)));
    if t != 0 {
        let tc = ch(TBASE + t);
        let parts = split_t(tc);
        if parts.is_empty() {
            out.push(tc);
        } else {
            out.extend_from_slice(parts);
        }
    }
}

/// Decompose a string of hangul syllables (other characters are skipped).
pub fn decompose(s: &str) -> Vec<char> {
    let mut out = Vec::new();
    for c in s.chars() {
        if is_syllable(c) {
            decompose_syllable(c, &mut out);
        } else if is_l(c) || is_t(c) {
            out.push(c);
        } else if is_v(c) {
            out.extend_from_slice(split_v(c));
        } else if let Some(j) = compat_to_component(c) {
            out.extend_from_slice(&j);
        }
    }
    out
}

/// Compatibility jamo (ㄱ, ㅏ …) as written in dictionary tails, read as a
/// coda (consonant) or a bare vowel.
fn compat_to_component(c: char) -> Option<Vec<char>> {
    let u = c as u32;
    if !(0x3131..=0x3163).contains(&u) {
        return None;
    }
    if u >= 0x314F {
        let v = ch(VBASE + (u - 0x314F));
        return Some(split_v(v).to_vec());
    }
    let t = compat_consonant_to_t(c)?;
    let parts = split_t(t);
    Some(if parts.is_empty() { vec![t] } else { parts.to_vec() })
}

const COMPAT_CONS: &str = "ㄱㄲㄳㄴㄵㄶㄷㄸㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅃㅄㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
// Coda equivalents, '\0' where no coda exists (ㄸ ㅃ ㅉ).
const COMPAT_T: [u32; 30] = [
    0x11A8, 0x11A9, 0x11AA, 0x11AB, 0x11AC, 0x11AD, 0x11AE, 0, 0x11AF, 0x11B0, 0x11B1, 0x11B2,
    0x11B3, 0x11B4, 0x11B5, 0x11B6, 0x11B7, 0x11B8, 0, 0x11B9, 0x11BA, 0x11BB, 0x11BC, 0x11BD,
    0, 0x11BE, 0x11BF, 0x11C0, 0x11C1, 0x11C2,
];

fn compat_consonant_to_t(c: char) -> Option<char> {
    let idx = COMPAT_CONS.chars().position(|x| x == c)?;
    match COMPAT_T[idx] {
        0 => None,
        u => Some(ch(u)),
    }
}

fn t_to_compat(t: char) -> char {
    let idx = COMPAT_T.iter().position(|&u| u == t as u32).unwrap();
    COMPAT_CONS.chars().nth(idx).unwrap()
}

fn v_to_compat(v: char) -> char {
    ch(0x314F + (v as u32 - VBASE))
}

/// The compatibility letter of an onset (ᄄ → ㄸ).
fn l_to_compat(l: char) -> char {
    "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ".chars().nth((l as u32 - LBASE) as usize).unwrap()
}

/// The coda that sounds the same as an onset (ᄋ has none).
pub fn l_to_t(l: char) -> Option<char> {
    let compat = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ".chars().nth((l as u32 - LBASE) as usize)?;
    if compat == 'ㅇ' {
        return None;
    }
    compat_consonant_to_t(compat)
}

/// The onset that sounds the same as a coda (ᆼ has none).
pub fn t_to_l(t: char) -> Option<char> {
    let compat = t_to_compat(t);
    if compat == 'ㅇ' {
        return None;
    }
    "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ".chars().position(|x| x == compat).map(|i| ch(LBASE + i as u32))
}

/// Recompose components into text. A syllable is an onset, one or more vowel
/// components, and up to two coda components. Material that does not start
/// with an onset is written as compatibility jamo (末音添記 letters such as
/// the ㄴ of 異ㄴ, or the bare vowel of 見ㅏ).
pub fn compose(comps: &[char]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < comps.len() {
        let c = comps[i];
        if is_l(c) {
            let mut j = i + 1;
            let vs = j;
            while j < comps.len() && is_v(comps[j]) {
                j += 1;
            }
            let vparts = &comps[vs..j];
            let vowel = if vparts.is_empty() { None } else { join_v(vparts) };
            match vowel {
                None => {
                    // Onset without a usable vowel: write the letter alone.
                    out.push(l_to_compat(c));
                    i += 1;
                    continue;
                }
                Some(v) => {
                    // Codas: take up to two, but leave one for a following vowel.
                    let ts = j;
                    let mut k = j;
                    while k < comps.len() && is_t(comps[k]) && k - ts < 2 {
                        k += 1;
                    }
                    let mut tparts = &comps[ts..k];
                    while !tparts.is_empty() && join_t(tparts).is_none() {
                        tparts = &tparts[..tparts.len() - 1];
                    }
                    let t = if tparts.is_empty() { 0 } else { join_t(tparts).unwrap() as u32 - TBASE };
                    let l = c as u32 - LBASE;
                    let vi = v as u32 - VBASE;
                    out.push(ch(SBASE + (l * VCOUNT + vi) * TCOUNT + t));
                    i = ts + tparts.len();
                }
            }
        } else if is_v(c) {
            // Bare vowel fragment: longest run that forms one vowel.
            let mut j = i + 1;
            while j < comps.len() && is_v(comps[j]) {
                j += 1;
            }
            let mut end = j;
            while end > i + 1 && join_v(&comps[i..end]).is_none() {
                end -= 1;
            }
            match join_v(&comps[i..end]) {
                Some(v) => out.push(v_to_compat(v)),
                None => out.push(v_to_compat(c)),
            }
            i = end;
        } else if is_t(c) {
            // Orphan coda: each written-out letter stays separate (ㄹㅁ, not ㄻ),
            // since adjacent letters come from different morphemes.
            out.push(t_to_compat(c));
            i += 1;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Rewrite a leading coda as the onset of a following ᄋ syllable
/// (ᆻ + 으면 → 쓰면), used for the resyllabified 末音添記 variant.
pub fn resyllabify(comps: &[char]) -> Vec<char> {
    let mut v = comps.to_vec();
    // A silent ㅎ (然ㅎ + 어서) is not carried over.
    if v.len() >= 2 && is_t(v[0]) && v[0] != 'ᇂ' && v[1] == 'ᄋ' {
        if let Some(l) = t_to_l(v[0]) {
            v.remove(0);
            v[0] = l;
        }
    }
    v
}

/// The last vowel component of a component string.
pub fn last_vowel(comps: &[char]) -> Option<char> {
    comps.iter().rev().copied().find(|&c| is_v(c))
}

/// Number of syllables (onsets) in a component string.
pub fn syllable_count(comps: &[char]) -> usize {
    comps.iter().filter(|&&c| is_l(c)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for w in ["읽었다", "마셔라", "봤다", "돼요", "앎", "삶을", "괜찮아요", "했습니다", "뭐"] {
            assert_eq!(compose(&decompose(w)), w, "{w}");
        }
    }

    #[test]
    fn fragments() {
        // 다른 minus 다르 → ㄴ
        let d = decompose("다른");
        let stem = decompose("다르");
        assert!(d.starts_with(&stem));
        assert_eq!(compose(&d[stem.len()..]), "ㄴ");
        // 봐 minus 보 → ㅏ
        let d = decompose("봐가");
        assert_eq!(compose(&d[decompose("보").len()..]), "ㅏ가");
        // 마셔라 minus 마시 → ㅓ라
        let d = decompose("마셔라");
        assert_eq!(compose(&d[decompose("마시").len()..]), "ㅓ라");
        // 그렇네요 minus 그러 → ㅎ네요
        let d = decompose("그렇네요");
        assert_eq!(compose(&d[decompose("그러").len()..]), "ㅎ네요");
    }

    #[test]
    fn compat_tail() {
        assert_eq!(decompose("ㅆ"), vec!['ᆻ']);
        assert_eq!(compose(&resyllabify(&decompose("ㅆ으면"))), "쓰면");
        assert_eq!(compose(&resyllabify(&decompose("ㅎ어서"))), "ㅎ어서");
        assert_eq!(compose(&['ᄄ']), "ㄸ");
        assert_eq!(compose(&['ᆯ', 'ᆸ']), "ㄹㅂ");
    }
}
