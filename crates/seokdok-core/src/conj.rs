//! Korean verb and adjective inflection, split into stem allomorphs and
//! ending tables.
//!
//! A lemma yields a few *allomorphs* (읽-, or 하-/해-, or 다르-/달라-), each
//! tagged with a [`Ctx`] that names which ending surfaces may follow it. An
//! ending table is shared by every allomorph with the same context, so the
//! converter matches `allomorph ++ ending` against the input without
//! enumerating every inflected form.

use crate::jamo::{compose, decompose, is_l, is_t, is_v, last_vowel};
use std::collections::HashMap;

/// Inflection class of a stem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Class {
    Regular,
    /// ㅂ 불규칙: 춥다 → 추워
    B,
    /// ㄷ 불규칙: 듣다 → 들어
    D,
    /// ㅅ 불규칙: 낫다 → 나아
    S,
    /// ㅎ 불규칙: 그렇다 → 그래, 그런
    H,
    /// 르 불규칙: 다르다 → 달라
    Reu,
    /// 러 불규칙: 이르다 → 이르러
    Reo,
    /// 아니다
    Ani,
}

impl Class {
    pub fn parse(s: &str) -> Option<Class> {
        Some(match s {
            "" | "reg" => Class::Regular,
            "b" | "ㅂ" => Class::B,
            "d" | "ㄷ" => Class::D,
            "s" | "ㅅ" => Class::S,
            "h" | "ㅎ" => Class::H,
            "reu" | "르" => Class::Reu,
            "reo" | "러" => Class::Reo,
            "ani" => Class::Ani,
            _ => return None,
        })
    }
}

/// Guess the class of a stem from its shape and part of speech. Irregular
/// ㄷ/ㅅ verbs and the ㅂ verbs that alternate are listed; the dictionary can
/// override the guess with an explicit class.
pub fn guess_class(stem: &str, is_adj: bool) -> Class {
    const B_VERBS: &[&str] = &["돕", "굽", "눕", "줍", "깁"];
    const B_ADJ_REGULAR: &[&str] = &["좁", "굽", "수줍"];
    const D_IRR: &[&str] = &["듣", "걷", "묻", "싣", "깨닫", "붇", "일컫", "긷", "눋", "겯"];
    const S_IRR: &[&str] = &["낫", "짓", "잇", "붓", "긋", "젓", "잣"];
    const REU_REGULAR: &[&str] = &["따르", "치르", "들르", "우러르", "다다르"];
    const REO: &[&str] = &["푸르", "누르", "노르"];
    if stem == "아니" {
        return Class::Ani;
    }
    let c = decompose(stem);
    let last = *c.last().unwrap_or(&' ');
    if stem.ends_with('르') {
        if is_adj && REO.contains(&stem) {
            return Class::Reo;
        }
        if REU_REGULAR.iter().any(|s| stem.ends_with(s)) {
            return Class::Regular;
        }
        return Class::Reu;
    }
    if is_t(last) {
        // Compounds inherit the class of their last part (알아듣, 드러눕).
        let base_is = |list: &[&str]| list.iter().any(|s| stem.ends_with(s));
        match last {
            'ᆸ' if c.len() >= 2 && c[c.len() - 2] != 'ᆯ' => {
                if is_adj {
                    if B_ADJ_REGULAR.contains(&stem) {
                        return Class::Regular;
                    }
                    return Class::B;
                }
                if base_is(B_VERBS) {
                    return Class::B;
                }
            }
            'ᆮ' if base_is(D_IRR) => return Class::D,
            'ᆺ' if base_is(S_IRR) => return Class::S,
            'ᇂ' if is_adj && is_v(c[c.len() - 2]) && !stem.ends_with("좋") => {
                return Class::H
            }
            _ => {}
        }
    }
    Class::Regular
}

/// Ending morpheme kinds, by how they attach.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Kind {
    /// Consonant-initial: 고, 지, 다 (ㄹ drops before ᄂ ᄇ ᄉ).
    K,
    /// (으)-initial: (으)면, (으)ㄴ, (으)시.
    E,
    /// (스)ㅂ니다.
    S,
    /// 아/어-initial: 아서, 았.
    A,
    /// (느)ㄴ다.
    N,
}

/// One ending morpheme. `body` is written without its alternating initial:
/// for K and N it is the whole form (N: the post-vowel form "ㄴ다"), for E the
/// form after 으 ("면", "ㄴ"), for S the post-vowel form ("ㅂ니다"), for A the
/// part after 아/어 ("서", "ㅆ").
#[derive(Clone, Copy, Debug)]
struct Morph {
    kind: Kind,
    body: &'static str,
}

const fn m(kind: Kind, body: &'static str) -> Morph {
    Morph { kind, body }
}

const FINALS: &[Morph] = &[
    // K
    m(Kind::K, "다"), m(Kind::K, "고"), m(Kind::K, "지"), m(Kind::K, "지만"), m(Kind::K, "지요"),
    m(Kind::K, "죠"), m(Kind::K, "게"), m(Kind::K, "기"), m(Kind::K, "기도"), m(Kind::K, "기에"),
    m(Kind::K, "기는"), m(Kind::K, "기로"), m(Kind::K, "기를"), m(Kind::K, "기가"), m(Kind::K, "긴"),
    m(Kind::K, "던"), m(Kind::K, "던데"), m(Kind::K, "던데요"), m(Kind::K, "네"), m(Kind::K, "네요"),
    m(Kind::K, "는"), m(Kind::K, "는데"), m(Kind::K, "는데요"), m(Kind::K, "는지"), m(Kind::K, "는구나"),
    m(Kind::K, "는군요"), m(Kind::K, "는가"), m(Kind::K, "는가요"), m(Kind::K, "는게"), m(Kind::K, "는걸"),
    m(Kind::K, "군요"), m(Kind::K, "군"), m(Kind::K, "구나"), m(Kind::K, "구요"), m(Kind::K, "고요"),
    m(Kind::K, "거나"), m(Kind::K, "거든"), m(Kind::K, "거든요"), m(Kind::K, "도록"), m(Kind::K, "다가"),
    m(Kind::K, "더라"), m(Kind::K, "더라고"), m(Kind::K, "더라고요"), m(Kind::K, "더라구요"), m(Kind::K, "더군요"),
    m(Kind::K, "더니"), m(Kind::K, "나"), m(Kind::K, "나요"), m(Kind::K, "냐"), m(Kind::K, "느냐"),
    m(Kind::K, "자"), m(Kind::K, "잖아"), m(Kind::K, "잖아요"), m(Kind::K, "다고"), m(Kind::K, "다는"),
    m(Kind::K, "다니"), m(Kind::K, "단"), m(Kind::K, "대요"), m(Kind::K, "대"), m(Kind::K, "고서"),
    m(Kind::K, "지는"), m(Kind::K, "진"), m(Kind::K, "지도"), m(Kind::K, "소"),
    m(Kind::K, "도다"), m(Kind::K, "노라"),
    // Adverbial -이 (많이, 같이) and -히 is lexical; -이 attaches like K.
    m(Kind::K, "이"), m(Kind::K, "다네요"), m(Kind::K, "다면"), m(Kind::K, "다니까"), m(Kind::K, "기엔"),
    m(Kind::K, "지마는"), m(Kind::K, "더라도"), m(Kind::K, "자마자"), m(Kind::K, "는데도"), m(Kind::K, "든"), m(Kind::K, "든지"),
    // N
    m(Kind::N, "ㄴ다"), m(Kind::N, "ㄴ다고"), m(Kind::N, "ㄴ다는"), m(Kind::N, "ㄴ대요"), m(Kind::N, "ㄴ다면"),
    // E
    m(Kind::E, "면"), m(Kind::E, "면서"), m(Kind::E, "며"), m(Kind::E, "니"), m(Kind::E, "니까"),
    m(Kind::E, "니까요"), m(Kind::E, "니깐"), m(Kind::E, "ㄴ"), m(Kind::E, "ㄴ데"), m(Kind::E, "ㄴ데요"),
    m(Kind::E, "ㄴ지"), m(Kind::E, "ㄴ가"), m(Kind::E, "ㄴ가요"), m(Kind::E, "ㄴ걸"), m(Kind::E, "ㄴ게"),
    m(Kind::E, "ㄴ적"), m(Kind::E, "ㄹ"), m(Kind::E, "ㄹ까"), m(Kind::E, "ㄹ까요"), m(Kind::E, "ㄹ게"),
    m(Kind::E, "ㄹ게요"), m(Kind::E, "ㄹ래"), m(Kind::E, "ㄹ래요"), m(Kind::E, "ㄹ지"), m(Kind::E, "ㄹ수록"),
    m(Kind::E, "ㄹ걸"), m(Kind::E, "ㄹ걸요"), m(Kind::E, "ㄹ텐데"), m(Kind::E, "ㄹ듯"), m(Kind::E, "ㄹ수"),
    m(Kind::E, "ㅁ"), m(Kind::E, "러"), m(Kind::E, "려고"), m(Kind::E, "려면"), m(Kind::E, "려"),
    m(Kind::E, "ㅂ시다"), m(Kind::E, "오"), m(Kind::E, "니라"), m(Kind::E, "리라"), m(Kind::E, "세요"),
    m(Kind::E, "십시오"), m(Kind::E, "소서"), m(Kind::E, "옵소서"), m(Kind::E, "ㄹ지라도"), m(Kind::E, "ㄴ데도"),
    // S
    m(Kind::S, "ㅂ니다"), m(Kind::S, "ㅂ니까"), m(Kind::S, "ㅂ디다"),
    // A
    m(Kind::A, ""), m(Kind::A, "서"), m(Kind::A, "요"), m(Kind::A, "도"), m(Kind::A, "야"),
    m(Kind::A, "라"), m(Kind::A, "야지"), m(Kind::A, "서는"), m(Kind::A, "서도"), m(Kind::A, "다가"),
    m(Kind::A, "서요"), m(Kind::A, "야겠다"), m(Kind::A, "야겠어요"), m(Kind::A, "야겠네요"),
    m(Kind::A, "야겠습니다"), m(Kind::A, "야지요"), m(Kind::A, "야죠"),
];

const HON: Morph = m(Kind::E, "시");
const PAST: Morph = m(Kind::A, "ㅆ");
const FUT: Morph = m(Kind::K, "겠");

/// The context an allomorph presents to the ending that follows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ctx {
    /// Consonant-final stem (읽-, 좋-). `kinds` restricts which morpheme kinds
    /// it hosts (irregular stems split the kinds between two allomorphs).
    Cons { bright: bool, kinds: KindSet },
    /// Vowel-final stem (가-, 보-, 마시-), last vowel component `v`.
    Vowel { v: char, pv: char, bright: bool, kinds: KindSet },
    /// Allomorph that already contains the 아/어 vowel (해-, 돼-, 그래-, 달라-).
    Fused,
    /// ㅡ-deleted allomorph ending in an onset (ᄊ-, 바ᄈ-).
    Deleted { bright: bool },
    /// 하-: 아/어 endings take 여 (하여, 하였다).
    Hada,
    /// ㄹ-stem keeping its ㄹ (알고, 알면, 알아).
    LKeep { bright: bool },
    /// ㄹ-stem with ㄹ dropped (아는, 압니다, 안).
    LDrop,
    /// ㅂ-irregular vowel allomorph (추-, 도-): 워/와, 운, 우면.
    BIrr { bright: bool },
    /// ㅎ-irregular nasal allomorph (그러-): 그런, 그러면.
    HNasal,
    /// 러-irregular: 이르러.
    Reo,
    /// 아니-: 아니어서, 아니에요, 아니야, 아니라.
    Ani,
    /// Transparent spelling for the morpheme style: the ending is written as
    /// attached to the lemma's underlying final, with no contraction.
    MorphCons { bright: bool, h: bool },
    MorphL { bright: bool },
    MorphV { bright: bool, hada: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KindSet(u8);
impl KindSet {
    pub const ALL: KindSet = KindSet(0b11111);
    pub const KSN: KindSet = KindSet(0b10101); // K, S, N
    pub const EA: KindSet = KindSet(0b01010); // E, A
    pub const KESN: KindSet = KindSet(0b10111); // all but A
    fn has(self, k: Kind) -> bool {
        let bit = match k {
            Kind::K => 0,
            Kind::E => 1,
            Kind::S => 2,
            Kind::A => 3,
            Kind::N => 4,
        };
        self.0 & (1 << bit) != 0
    }
}

fn c(s: &str) -> Vec<char> {
    decompose(s)
}

fn cat(a: &[char], b: &[char]) -> Vec<char> {
    let mut v = a.to_vec();
    v.extend_from_slice(b);
    v
}

/// Surfaces of one morpheme attached to a host context. Several surfaces are
/// returned when contraction is optional (보아 / 봐).
fn attach(ctx: Ctx, mo: Morph) -> Vec<Vec<char>> {
    let body = c(mo.body);
    let eu = |b: &[char]| -> Vec<char> {
        // 으 + body: a coda-initial body becomes the coda of 으.
        cat(&c("으"), b)
    };
    let aeo = |bright: bool| c(if bright { "아" } else { "어" });
    match ctx {
        Ctx::Cons { bright, kinds } | Ctx::Vowel { bright, kinds, .. }
            if !kinds.has(mo.kind) =>
        {
            let _ = bright;
            vec![]
        }
        Ctx::Cons { bright, .. } | Ctx::MorphCons { bright, h: false } => match mo.kind {
            Kind::K => vec![body],
            Kind::N => vec![cat(&c("는"), &body[1..])],
            Kind::E => vec![eu(&body)],
            Kind::S => vec![cat(&c("스"), &body)],
            Kind::A => vec![cat(&aeo(bright), &body)],
        },
        Ctx::MorphCons { bright, h: true } => match mo.kind {
            Kind::E => vec![body],
            _ => attach(Ctx::MorphCons { bright, h: false }, mo),
        },
        Ctx::Vowel { v, pv, bright, .. } => match mo.kind {
            Kind::K | Kind::E | Kind::S | Kind::N => vec![body],
            Kind::A => {
                let full = cat(&aeo(bright), &body);
                let bare_v = if bright { 'ᅡ' } else { 'ᅥ' };
                let fragment = {
                    let mut f = vec![bare_v];
                    f.extend_from_slice(&body);
                    f
                };
                match v {
                    // 가+아 → 가, 서+어 → 서 (the vowel is absorbed).
                    'ᅡ' | 'ᅥ' => vec![body],
                    // 보+아 → 봐 / 보아, 주+어 → 줘 / 주어, 마시+어 → 마셔.
                    'ᅩ' | 'ᅮ' if pv != 'ᅵ' => vec![fragment, full],
                    // 내+어 → 내 / 내어, 세+어 → 세 / 세어.
                    'ᅵ' if matches!(pv, 'ᅡ' | 'ᅥ') => vec![body, full],
                    // 쉬어, 띄어, 되어: ㅟ ㅢ ㅚ do not contract here.
                    'ᅵ' if matches!(pv, 'ᅮ' | 'ᅳ' | 'ᅩ') => vec![full],
                    'ᅵ' => vec![fragment, full],
                    _ => vec![full],
                }
            }
        },
        Ctx::Fused => match mo.kind {
            Kind::A => vec![body],
            _ => vec![],
        },
        Ctx::Deleted { bright } => match mo.kind {
            Kind::A => {
                let mut f = vec![if bright { 'ᅡ' } else { 'ᅥ' }];
                f.extend_from_slice(&body);
                vec![f]
            }
            _ => vec![],
        },
        Ctx::Hada => match mo.kind {
            Kind::A => vec![cat(&c("여"), &body)],
            _ => vec![body],
        },
        Ctx::LKeep { bright } => match mo.kind {
            Kind::K => {
                if matches!(body.first(), Some('ᄂ' | 'ᄇ' | 'ᄉ')) {
                    vec![]
                } else {
                    vec![body]
                }
            }
            // ㅁ keeps the ㄹ (앎); ᄆ ᄅ-initial bodies keep it (알면, 알려고).
            Kind::E => match body.first() {
                Some('ᆷ') => vec![body],
                Some('ᄆ' | 'ᄅ' | 'ᄋ') => vec![body],
                _ => vec![],
            },
            Kind::S | Kind::N => vec![],
            Kind::A => vec![cat(&aeo(bright), &body)],
        },
        Ctx::LDrop => match mo.kind {
            Kind::K => {
                if matches!(body.first(), Some('ᄂ' | 'ᄇ' | 'ᄉ')) {
                    vec![body]
                } else {
                    vec![]
                }
            }
            Kind::E => match body.first() {
                Some('ᆷ' | 'ᄆ' | 'ᄅ' | 'ᄋ') => vec![],
                _ => vec![body],
            },
            Kind::S | Kind::N => vec![body],
            Kind::A => vec![],
        },
        Ctx::MorphL { bright } => match mo.kind {
            Kind::A => vec![cat(&aeo(bright), &body)],
            _ => vec![body],
        },
        Ctx::BIrr { bright } => match mo.kind {
            Kind::E => vec![cat(&c("우"), &body)],
            Kind::A => vec![cat(&c(if bright { "와" } else { "워" }), &body)],
            _ => vec![],
        },
        Ctx::HNasal => match mo.kind {
            Kind::E => vec![body],
            _ => vec![],
        },
        Ctx::Reo => match mo.kind {
            Kind::A => vec![cat(&c("러"), &body)],
            _ => vec![],
        },
        Ctx::Ani => match mo.kind {
            Kind::A => {
                let mut v = vec![cat(&c("어"), &body)];
                match mo.body {
                    "" => v.push(c("야")),
                    "요" => v.push(c("에요")),
                    "라" => v.push(c("라")),
                    "서" => v.push(c("라서")),
                    _ => {}
                }
                v
            }
            _ => vec![body],
        },
        Ctx::MorphV { bright, hada } => match mo.kind {
            Kind::A => vec![cat(&c(if hada { "여" } else if bright { "아" } else { "어" }), &body)],
            _ => vec![body],
        },
    }
}

/// Context presented by an ending morpheme to the next one.
fn after(mo: Morph) -> Ctx {
    match mo.body {
        "시" => Ctx::Vowel { v: 'ᅵ', pv: ' ', bright: false, kinds: KindSet::ALL },
        _ => Ctx::Cons { bright: false, kinds: KindSet::ALL },
    }
}

/// A complete ending: a chain of morphemes (prefinals + final).
#[derive(Clone, Debug)]
pub struct Ending {
    /// Display form, e.g. "시+었+습니다".
    pub label: String,
    chain: Vec<Morph>,
}

fn chains() -> &'static Vec<Ending> {
    use std::sync::OnceLock;
    static CHAINS: OnceLock<Vec<Ending>> = OnceLock::new();
    CHAINS.get_or_init(|| {
        let mut out = Vec::new();
        for hon in [false, true] {
            for past in 0..=2 {
                for fut in [false, true] {
                    for f in FINALS {
                        let mut ch = Vec::new();
                        if hon {
                            ch.push(HON);
                        }
                        for _ in 0..past {
                            ch.push(PAST);
                        }
                        if fut {
                            ch.push(FUT);
                        }
                        // 세요/십시오/소서 already carry the honorific.
                        if hon && matches!(f.body, "세요" | "십시오") {
                            continue;
                        }
                        ch.push(*f);
                        let label = ch
                            .iter()
                            .map(|mo| {
                                let pre = match mo.kind {
                                    Kind::E => "(으)",
                                    Kind::S => "(스)",
                                    Kind::A => "아/어",
                                    _ => "",
                                };
                                format!("{pre}{}", mo.body)
                            })
                            .collect::<Vec<_>>()
                            .join("+");
                        out.push(Ending { label, chain: ch });
                    }
                }
            }
        }
        out
    })
}

/// Surface strings of every ending chain after a context: (components, chain index).
pub fn ending_surfaces(ctx: Ctx) -> Vec<(Vec<char>, usize)> {
    let mut out = Vec::new();
    for (idx, e) in chains().iter().enumerate() {
        let mut forms: Vec<Vec<char>> = attach(ctx, e.chain[0]);
        // 시 + 어요 is 세요; the chained form 셔요 is also produced.
        for (i, mo) in e.chain.iter().enumerate().skip(1) {
            let host = after(e.chain[i - 1]);
            let mut next = Vec::new();
            for f in &forms {
                for s in attach(host, *mo) {
                    next.push(cat(f, &s));
                }
            }
            forms = next;
        }
        for f in forms {
            out.push((f, idx));
        }
    }
    out
}

pub fn ending_label(idx: usize) -> &'static str {
    &chains()[idx].label
}

/// The morpheme-style spelling of ending `idx` after a lemma with morph
/// context `ctx` (one surface, no contraction at the stem boundary).
pub fn morph_ending(ctx: Ctx, idx: usize) -> Option<Vec<char>> {
    let e = &chains()[idx];
    let mut form = attach(ctx, e.chain[0]).into_iter().last()?;
    for (i, mo) in e.chain.iter().enumerate().skip(1) {
        let host = after(e.chain[i - 1]);
        let s = attach(host, *mo).into_iter().next()?;
        form = cat(&form, &s);
    }
    Some(form)
}

/// One stem allomorph.
#[derive(Clone, Debug)]
pub struct Allomorph {
    pub comps: Vec<char>,
    pub ctx: Ctx,
}

fn bright_vowel(v: Option<char>) -> bool {
    matches!(v, Some('ᅡ' | 'ᅩ'))
}

/// Harmony of the syllable before the last one (for 르 and ㅡ stems).
fn prev_syllable_bright(stem: &[char]) -> bool {
    let onsets: Vec<usize> = stem.iter().enumerate().filter(|(_, &c)| is_l(c)).map(|(i, _)| i).collect();
    if onsets.len() < 2 {
        return false;
    }
    let prev = &stem[onsets[onsets.len() - 2]..onsets[onsets.len() - 1]];
    bright_vowel(last_vowel(prev))
}

/// Stem allomorphs of a lemma stem (dictionary form without 다).
pub fn allomorphs(stem: &str, class: Class) -> Vec<Allomorph> {
    let s = decompose(stem);
    let n = s.len();
    let last = s[n - 1];
    let lv = last_vowel(&s);
    let bright = bright_vowel(lv);
    let mut out = Vec::new();
    let mut push = |comps: Vec<char>, ctx: Ctx| out.push(Allomorph { comps, ctx });
    let replace_last_vowel = |s: &[char], nv: &[char]| -> Vec<char> {
        let mut v = s.to_vec();
        while v.last().map_or(false, |&c| is_v(c)) {
            v.pop();
        }
        v.extend_from_slice(nv);
        v
    };
    match class {
        Class::Ani => {
            push(s.clone(), Ctx::Ani);
        }
        Class::Regular if stem.ends_with('하') => {
            push(s.clone(), Ctx::Hada);
            push(replace_last_vowel(&s, &['ᅡ', 'ᅵ']), Ctx::Fused); // 해
        }
        Class::Regular | Class::Reu | Class::Reo if last == 'ᅳ' => {
            let kinds = if class == Class::Regular || class == Class::Reu || class == Class::Reo {
                KindSet::KESN
            } else {
                KindSet::ALL
            };
            push(s.clone(), Ctx::Vowel { v: 'ᅳ', pv: ' ', bright, kinds });
            let pb = prev_syllable_bright(&s);
            match class {
                Class::Reu => {
                    // 다르 → 달라: previous syllable takes ᆯ, then ᄅ + 아/어.
                    let mut base = s[..n - 2].to_vec(); // drop ᄅ ᅳ
                    base.push('ᆯ');
                    base.push('ᄅ');
                    base.push(if pb { 'ᅡ' } else { 'ᅥ' });
                    push(base, Ctx::Fused);
                }
                Class::Reo => push(s.clone(), Ctx::Reo),
                _ => {
                    // ㅡ deletion: 쓰 → ᄊ + ᅥ, 바쁘 → 바ᄈ + ᅡ.
                    let bright_del = if crate::jamo::syllable_count(&s) >= 2 { pb } else { false };
                    push(s[..n - 1].to_vec(), Ctx::Deleted { bright: bright_del });
                }
            }
        }
        Class::Regular if is_v(last) => {
            let pv = if n >= 2 && is_v(s[n - 2]) { s[n - 2] } else { ' ' };
            push(s.clone(), Ctx::Vowel { v: last, pv, bright, kinds: KindSet::ALL });
            if pv == 'ᅩ' && last == 'ᅵ' {
                // 되 → 돼 (fused) beside 되어.
                push(replace_last_vowel(&s, &['ᅩ', 'ᅡ', 'ᅵ']), Ctx::Fused);
            }
        }
        Class::Regular if last == 'ᆯ' => {
            push(s.clone(), Ctx::LKeep { bright });
            push(s[..n - 1].to_vec(), Ctx::LDrop);
        }
        Class::Regular | Class::Reu | Class::Reo => {
            push(s.clone(), Ctx::Cons { bright, kinds: KindSet::ALL });
        }
        Class::B => {
            push(s.clone(), Ctx::Cons { bright, kinds: KindSet::KSN });
            let b = stem == "돕" || stem == "곱";
            push(s[..n - 1].to_vec(), Ctx::BIrr { bright: b });
        }
        Class::D => {
            push(s.clone(), Ctx::Cons { bright, kinds: KindSet::KSN });
            let mut v = s[..n - 1].to_vec();
            v.push('ᆯ');
            push(v, Ctx::Cons { bright, kinds: KindSet::EA });
        }
        Class::S => {
            push(s.clone(), Ctx::Cons { bright, kinds: KindSet::KSN });
            push(s[..n - 1].to_vec(), Ctx::Cons { bright, kinds: KindSet::EA });
        }
        Class::H => {
            push(s.clone(), Ctx::Cons { bright, kinds: KindSet::KSN });
            let nasal = s[..n - 1].to_vec(); // 그러
            push(nasal.clone(), Ctx::HNasal);
            // 그러 → 그래, 하야 → 하얘, 누러 → 누레, 파라 → 파래.
            let v = last_vowel(&nasal);
            let fused_v: &[char] = if v == Some('ᅥ') && ["그렇", "이렇", "저렇", "어떻", "요렇", "고렇", "조렇"].iter().any(|x| stem.ends_with(x)) {
                &['ᅡ', 'ᅵ']
            } else {
                // Append ᅵ to whatever vowel is there.
                &[]
            };
            let fused = if fused_v.is_empty() {
                let mut x = nasal.clone();
                x.push('ᅵ');
                x
            } else {
                replace_last_vowel(&nasal, fused_v)
            };
            push(fused, Ctx::Fused);
        }
    }
    out
}

/// Context used for the morpheme-style spelling of a lemma.
pub fn morph_ctx(stem: &str, class: Class) -> Ctx {
    let s = decompose(stem);
    let last = *s.last().unwrap();
    // ㅡ-final stems (다르, 바쁘) take their harmony from the syllable before.
    let bright = if last == 'ᅳ' { prev_syllable_bright(&s) } else { bright_vowel(last_vowel(&s)) };
    if class == Class::Ani {
        return Ctx::Ani;
    }
    if is_v(last) {
        return Ctx::MorphV { bright, hada: stem.ends_with('하') };
    }
    if last == 'ᆯ' && class == Class::Regular {
        return Ctx::MorphL { bright };
    }
    Ctx::MorphCons { bright, h: class == Class::H }
}

/// Every inflected surface of a lemma (for tests and dictionary checks):
/// surface string → ending label.
pub fn paradigm(stem: &str, class: Class) -> HashMap<String, Vec<&'static str>> {
    let mut out: HashMap<String, Vec<&'static str>> = HashMap::new();
    for a in allomorphs(stem, class) {
        for (e, idx) in ending_surfaces(a.ctx) {
            out.entry(compose(&cat(&a.comps, &e))).or_default().push(ending_label(idx));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(stem: &str, class: Class, forms: &[&str]) {
        let p = paradigm(stem, class);
        for f in forms {
            assert!(p.contains_key(*f), "{stem}: missing {f}");
        }
    }

    #[test]
    fn regular() {
        has("가", Class::Regular, &["갔었다", "가야겠다"]);
        has("읽", Class::Regular, &["읽었었다", "읽더라도", "읽자마자", "읽을지라도", "읽어", "읽었다", "읽으면", "읽는", "읽습니다", "읽은", "읽으셨습니다", "읽겠네요", "읽는다"]);
        has("좋", Class::Regular, &["좋아", "좋았다", "좋은", "좋네요", "좋겠네요"]);
        has("가", Class::Regular, &["가", "갔다", "가요", "간", "갈", "감", "갑니다", "가셨다", "가세요", "간다"]);
        has("보", Class::Regular, &["봐", "보아", "봤다", "보았는데", "봐서", "볼", "봅니다"]);
        has("마시", Class::Regular, &["마셔", "마셨다", "마셔라", "마실"]);
        has("되", Class::Regular, &["돼", "됐다", "되어", "될", "됩니다"]);
        has("쓰", Class::Regular, &["써", "썼다", "쓴", "쓸", "씁니다", "쓰고"]);
        has("바쁘", Class::Regular, &["바빠", "바빴다", "바쁜"]);
        has("하", Class::Regular, &["해", "했다", "해요", "하여", "하였다", "한", "합니다", "하세요", "하셨습니다"]);
        has("안녕하", Class::Regular, &["안녕하세요", "안녕해요"]);
    }

    #[test]
    fn l_stems() {
        has("알", Class::Regular, &["알아", "알았다", "아는", "압니다", "안", "알", "앎", "알면", "아세요", "알고", "안다"]);
        has("만들", Class::Regular, &["만들어", "만든", "만드신", "만듭니다", "만들면"]);
    }

    #[test]
    fn irregular() {
        has("춥", Class::B, &["추워", "추웠다", "추운", "추우면", "춥다", "춥습니다"]);
        has("돕", Class::B, &["도와", "도왔다", "도운", "돕고"]);
        has("듣", Class::D, &["들어", "들었다", "들으면", "듣고", "듣는"]);
        has("낫", Class::S, &["나아", "나은", "나으면", "낫다"]);
        has("그렇", Class::H, &["그래", "그랬다", "그런", "그러면", "그렇네요", "그렇습니다", "그럴"]);
        has("하얗", Class::H, &["하얘", "하얀"]);
        has("다르", Class::Reu, &["달라", "달랐다", "다른", "다를", "다르면", "다릅니다"]);
        has("모르", Class::Reu, &["몰라", "몰랐네요", "모르겠네요", "모르는"]);
        has("이르", Class::Reo, &["이르러", "이르렀다"]);
        has("아니", Class::Ani, &["아닌", "아니라", "아니었다", "아닙니다"]);
    }

    #[test]
    fn guess() {
        assert_eq!(guess_class("춥", true), Class::B);
        assert_eq!(guess_class("잡", false), Class::Regular);
        assert_eq!(guess_class("듣", false), Class::D);
        assert_eq!(guess_class("그렇", true), Class::H);
        assert_eq!(guess_class("좋", true), Class::Regular);
        assert_eq!(guess_class("많", true), Class::Regular);
        assert_eq!(guess_class("괜찮", true), Class::Regular);
        assert_eq!(guess_class("다르", true), Class::Reu);
        assert_eq!(guess_class("따르", false), Class::Regular);
        assert_eq!(guess_class("알아듣", false), Class::D);
        assert_eq!(guess_class("누르", false), Class::Reu);
        assert_eq!(guess_class("수줍", true), Class::Regular);
    }
}
