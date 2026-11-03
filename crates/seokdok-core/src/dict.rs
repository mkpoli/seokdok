//! Dictionary entries and their TSV format.
//!
//! `native.tsv` (訓讀): `lemma  written  pos  class  source  count  note`
//!   - `lemma`: the Hangul dictionary form (읽다, 하늘, 처음).
//!   - `written`: the 釋讀 dictionary form (讀다, 天, 初음, 囚두다, 在ㅆ다).
//!     Leading Hanja cover the start of the lemma; trailing Hangul repeats the
//!     uncovered rest. A trailing lone consonant (在ㅆ다) marks the stem's
//!     final consonant as always written out (末音添記).
//!   - `pos`: noun pron num bound adv det interj verb adj (or the Korean
//!     names 명사 대명사 수사 의존명사 부사 관형사 감탄사 동사 형용사).
//!   - `class`: inflection class (reg b d s h reu reo ani); blank = guessed.
//!   - `source`: provenance tier, see [`Source`]; `count`: corpus count.
//!
//! `sino.tsv` (音讀): `reading  hanja  count  source`.

use crate::conj::{allomorphs, guess_class, morph_ctx, Allomorph, Class, Ctx};
use crate::jamo::{decompose, is_syllable, is_t};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pos {
    Noun,
    Pron,
    Num,
    Bound,
    Adv,
    Det,
    Interj,
    Verb,
    Adj,
}

impl Pos {
    pub fn parse(s: &str) -> Option<Pos> {
        Some(match s {
            "noun" | "명사" => Pos::Noun,
            "pron" | "대명사" => Pos::Pron,
            "num" | "수사" | "조수사" => Pos::Num,
            "bound" | "의존명사" => Pos::Bound,
            "adv" | "부사" => Pos::Adv,
            "det" | "관형사" => Pos::Det,
            "interj" | "감탄사" => Pos::Interj,
            "verb" | "동사" => Pos::Verb,
            "adj" | "형용사" => Pos::Adj,
            _ => return None,
        })
    }
    pub fn inflects(self) -> bool {
        matches!(self, Pos::Verb | Pos::Adj)
    }
}

/// Where an entry comes from. Lower tiers are trusted more when ranking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The user's own dictionary.
    User,
    /// Attested in the 釋讀會 chat corpus (with a count).
    Corpus,
    /// 標準國語釋讀辭典, the 釋讀會 community dictionary.
    Community,
    /// Added by the project editors.
    Editorial,
    /// Proposed by a language model from word senses and 訓音 glosses;
    /// unreviewed.
    Suggested,
    /// Derived from a traditional 訓音 gloss (하늘 천 → 하늘 = 天).
    Gloss,
    /// Found through a Japanese 訓読み word with the same English gloss.
    Japanese,
    /// A weaker Japanese link (no main-sense match on both sides).
    JapaneseWeak,
    /// A Sino-Korean dictionary (libhangul).
    Lexicon,
}

impl Source {
    /// Trust rank used when several entries explain the same written form.
    pub fn rank(self) -> u8 {
        match self {
            Source::User => 0,
            Source::Corpus | Source::Editorial => 1,
            Source::Community => 2,
            Source::Suggested => 3,
            Source::Gloss | Source::Japanese | Source::JapaneseWeak | Source::Lexicon => 4,
        }
    }

    pub fn parse(s: &str) -> Source {
        match s {
            "user" => Source::User,
            "corpus" => Source::Corpus,
            "community" => Source::Community,
            "editorial" => Source::Editorial,
            "suggested" => Source::Suggested,
            "gloss" => Source::Gloss,
            "japanese" => Source::Japanese,
            "japanese-weak" => Source::JapaneseWeak,
            _ => Source::Lexicon,
        }
    }
}

/// How the written form relates to the lemma.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tail {
    /// Hanja cover the whole stem/word.
    None,
    /// The final consonant is written out (在ㅆ다, 事ㅅ).
    Coda,
    /// Hanja cover a prefix; the rest is Hangul (初음, 囚두다).
    Partial,
}

#[derive(Clone, Debug)]
pub struct Native {
    pub lemma: String,
    pub pos: Pos,
    /// The Hanja part of the written form ("" for Hangul-only helper entries).
    pub hanja: String,
    /// Lemma components without 다 for inflecting words.
    pub stem: Vec<char>,
    pub stem_str: String,
    /// Number of components covered by the Hanja.
    pub cover: usize,
    pub tail: Tail,
    pub class: Class,
    pub allos: Vec<Allomorph>,
    pub morph: Option<Ctx>,
    pub source: Source,
    pub count: u32,
    pub cost: f32,
    /// File label and line the entry came from.
    pub origin: (String, usize),
}

#[derive(Clone, Debug)]
pub struct Sino {
    pub reading: Box<str>,
    pub hanja: Box<str>,
    pub source: Source,
    pub count: u32,
    pub cost: f32,
}

fn native_cost(source: Source, count: u32) -> f32 {
    let base = match source {
        Source::User => 1.0,
        Source::Corpus => 2.5,
        Source::Community => 3.0,
        Source::Editorial => 3.0,
        Source::Suggested => 4.5,
        Source::Gloss => 6.0,
        Source::Japanese => 6.2,
        Source::JapaneseWeak => 7.2,
        Source::Lexicon => 6.0,
    };
    if count == 0 {
        return base;
    }
    // Attested entries rank by corpus frequency, whatever their tier.
    let attested = (3.2 - 0.45 * ((count as f32) + 1.0).ln()).max(0.2);
    match source {
        // Gloss entries share characters with better entries, so their
        // credited counts are noisy: never rank them above the community tier.
        Source::Suggested => attested.max(2.5),
        Source::Gloss | Source::Japanese | Source::Lexicon => attested.max(3.0),
        Source::JapaneseWeak => attested.max(4.0),
        _ => attested.min(base),
    }
}

fn sino_cost(reading: &str, source: Source, count: u32) -> f32 {
    let n = reading.chars().count();
    let base = match (source, n) {
        (Source::User, _) => 1.0,
        (_, 1) => 7.0,
        _ => 4.5,
    };
    (base - 0.35 * ((count as f32) + 1.0).ln()).max(0.3)
}

/// Map CJK compatibility ideographs (豈 U+F900 …, emitted by Korean IMEs
/// for alternative readings) to their unified ideographs (NFC).
pub fn unify(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfc().collect()
}

/// Han ideographs, including extensions and compatibility ideographs.
pub fn is_han(c: char) -> bool {
    matches!(c as u32, 0x2E80..=0x2FDF | 0x3005 | 0x3007 | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x3FFFF)
}

pub fn parse_native(fields: &[&str]) -> Result<Native, String> {
    let get = |i: usize| fields.get(i).map(|s| s.trim()).unwrap_or("");
    let lemma = get(0).to_string();
    let written = unify(get(1));
    let pos = Pos::parse(get(2)).ok_or_else(|| format!("unknown pos {:?}", get(2)))?;
    let source = Source::parse(get(4));
    let count: u32 = get(5).parse().unwrap_or(0);
    let (stem_str, wstem) = if pos.inflects() {
        let l = lemma.strip_suffix('다').ok_or("verb lemma must end in 다")?;
        let w = written.strip_suffix('다').ok_or("verb written form must end in 다")?;
        (l.to_string(), w.to_string())
    } else {
        (lemma.clone(), written.clone())
    };
    if stem_str.is_empty() || !stem_str.chars().all(is_syllable) {
        return Err(format!("lemma {lemma:?} must be Hangul syllables"));
    }
    // The Hanja part runs to the last Han character, so a compound verb's
    // written form (奪아去다 for 빼앗아가다) keeps its inner Hangul there.
    let split = wstem
        .char_indices()
        .filter(|(_, c)| is_han(*c))
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    let (hanja, tail_s) = wstem.split_at(split);
    let stem = decompose(&stem_str);
    let tail_c = decompose(tail_s);
    if hanja.is_empty() && tail_s != stem_str {
        return Err(format!("{written:?}: no Hanja"));
    }
    if let Some(c) = hanja.chars().next().filter(|&c| !is_han(c)) {
        return Err(format!("{written:?}: {c:?} is not a Han character"));
    }
    if let Some(c) = hanja.chars().find(|&c| !is_han(c) && !is_syllable(c)) {
        return Err(format!("{written:?}: {c:?} is not a Han character"));
    }
    // Inner Hangul (the 아 of 奪아去) must appear in the lemma, in order.
    let mut rest = stem_str.as_str();
    for seg in hanja.split(|c: char| is_han(c)).filter(|s| !s.is_empty()) {
        match rest.find(seg) {
            Some(i) if i > 0 => rest = &rest[i + seg.len()..],
            _ => return Err(format!("{written:?}: {seg:?} is not inside {stem_str:?}")),
        }
    }
    // The last character must cover something after the inner Hangul.
    if hanja.chars().any(is_syllable) && decompose(rest).len() <= tail_c.len() {
        return Err(format!("{written:?}: the last character covers nothing of {stem_str:?}"));
    }
    if !stem.ends_with(&tail_c) {
        return Err(format!("{written:?}: tail {tail_s:?} does not end {stem_str:?}"));
    }
    let cover = stem.len() - tail_c.len();
    let tail = if tail_c.is_empty() || hanja.is_empty() {
        Tail::None
    } else if tail_c.iter().all(|&c| is_t(c)) {
        Tail::Coda
    } else {
        Tail::Partial
    };
    let class = match get(3) {
        "" => {
            if pos.inflects() {
                guess_class(&stem_str, pos == Pos::Adj)
            } else {
                Class::Regular
            }
        }
        s => Class::parse(s).ok_or_else(|| format!("unknown class {s:?}"))?,
    };
    let (allos, morph) = if pos.inflects() {
        (allomorphs(&stem_str, class), Some(morph_ctx(&stem_str, class)))
    } else {
        (Vec::new(), None)
    };
    Ok(Native {
        lemma,
        pos,
        hanja: hanja.to_string(),
        stem,
        stem_str,
        cover,
        tail,
        class,
        allos,
        morph,
        source,
        count,
        cost: native_cost(source, count),
        origin: (String::new(), 0),
    })
}

pub fn parse_sino(fields: &[&str]) -> Result<Sino, String> {
    let get = |i: usize| fields.get(i).map(|s| s.trim()).unwrap_or("");
    let reading = get(0).to_string();
    let hanja = unify(get(1));
    if reading.is_empty() || !reading.chars().all(is_syllable) || hanja.is_empty() {
        return Err(format!("bad sino line {fields:?}"));
    }
    let count = get(2).parse().unwrap_or(0);
    let source = Source::parse(if get(3).is_empty() { "lexicon" } else { get(3) });
    // "hangul": the chat writes this reading in Hangul and never this Hanja
    // form, so it is most likely a native word (사랑 ≠ 舍廊).
    let native_homophone = get(3) == "hangul";
    let cost = sino_cost(&reading, source, count) + if native_homophone { 4.0 } else { 0.0 };
    Ok(Sino { reading: reading.into(), hanja: hanja.into(), source, count, cost })
}

/// Read a TSV file's data lines (skipping blanks, `#` comments and a header
/// line starting with `lemma` or `reading`).
pub fn tsv_lines(text: &str) -> impl Iterator<Item = (usize, Vec<&str>)> {
    text.lines().enumerate().filter_map(|(i, l)| {
        let t = l.trim_end_matches('\r');
        if t.trim().is_empty() || t.starts_with('#') || t.starts_with("lemma\t") || t.starts_with("reading\t") {
            None
        } else {
            Some((i + 1, t.split('\t').collect()))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tails() {
        let e = parse_native(&["있다", "在ㅆ다", "adj"]).unwrap();
        assert_eq!(e.tail, Tail::Coda);
        assert_eq!(e.hanja, "在");
        let e = parse_native(&["가두다", "囚두다", "verb"]).unwrap();
        assert_eq!(e.tail, Tail::Partial);
        assert_eq!(e.cover, decompose("가").len());
        let e = parse_native(&["처음", "初음", "noun"]).unwrap();
        assert_eq!(e.tail, Tail::Partial);
        let e = parse_native(&["것", "事ㅅ", "bound"]).unwrap();
        assert_eq!(e.tail, Tail::Coda);
        let e = parse_native(&["읽다", "讀다", "동사"]).unwrap();
        assert_eq!(e.tail, Tail::None);
        assert!(parse_native(&["읽다", "讀어다", "verb"]).is_err());
        assert!(parse_native(&["잘못", "X", "noun"]).is_err());
        let e = parse_native(&["빼앗아가다", "奪아去다", "verb"]).unwrap();
        assert_eq!((e.hanja.as_str(), &e.tail), ("奪아去", &Tail::None));
        assert!(parse_native(&["빼앗아가다", "奪어去다", "verb"]).is_err());
        assert!(parse_native(&["일어나다", "起어出나다", "verb"]).is_err());
    }
}
