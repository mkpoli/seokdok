//! Particles (조사) and copula forms that follow a nominal. They stay in
//! Hangul; contracted forms after a vowel (저+ㄴ → 전, 나+ㅣ → 내) are written
//! as the bare letter (自ㄴ, 我ㅣ).

use crate::jamo::decompose;

/// Which nominal endings a particle form may follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Cond {
    Cons,
    Vowel,
    /// Consonant-final but not ㄹ-final (으로 vs ㄹ+로).
    ConsNotL,
    /// Vowel-final or ㄹ-final (로).
    VowelOrL,
    Any,
    /// Vowel-final pronoun or bound noun only (나+ㅣ → 내, 거+ㅣ → 게).
    PronVowel,
}

/// (after consonant, after vowel) pairs. "" means the slot has no form.
const PAIRS: &[(&str, &str)] = &[
    ("이", "가"), ("을", "를"), ("은", "는"), ("과", "와"), ("이나", "나"), ("이랑", "랑"),
    ("이라고", "라고"), ("이라는", "라는"), ("이란", "란"), ("이야", "야"), ("이에요", "예요"),
    ("이야말로", "야말로"), ("이든", "든"), ("이든지", "든지"), ("이라도", "라도"), ("이며", "며"),
    ("이고", "고"), ("이지만", "지만"), ("이면", "면"), ("이니까", "니까"), ("이라서", "라서"),
    ("이다", "다"), ("입니다", "ㅂ니다"), ("입니다", ""), ("이었다", "였다"), ("이었는데", "였는데"),
    ("이었어요", "였어요"), ("이죠", "죠"), ("이네요", "네요"), ("인데", "ㄴ데"), ("인가", "ㄴ가"),
    ("인가요", "ㄴ가요"), ("인지", "ㄴ지"), ("인", ""), ("일", ""), ("이나마", "나마"), ("과는", "와는"),
    ("과도", "와도"), ("이라", "라"), ("이니라", "니라"), ("이로다", "로다"), ("이여", "여"), ("이시여", "시여"), ("이가", ""), ("이는", ""),
    ("이를", ""), ("인듯", "ㄴ듯"), ("일지도", "ㄹ지도"), ("일수도", "ㄹ수도"), ("이려나", "려나"), ("이었으려나", "였으려나"), ("이라면", "라면"), ("이라니", "라니"), ("이라니까", "라니까"), ("이면서", "면서"), ("이든가", "든가"), ("이거나", "거나"), ("이었던", "였던"), ("이었고", "였고"), ("이었지만", "였지만"), ("이었으면", "였으면"), ("이도", ""), ("이의", ""), ("이에게", ""),
];

const ANY: &[&str] = &[
    "의", "에", "에서", "에게", "에게서", "한테", "께", "께서", "도", "만", "까지", "부터", "보다",
    "처럼", "같이", "마다", "조차", "마저", "밖에", "뿐", "요", "대로", "쯤", "에는", "에서는", "에서도",
    "에게는", "에도", "에만", "까지는", "까지도", "부터는", "만은", "만큼", "보다는", "에선", "엔",
    // audit additions: full copula forms after any nominal, and common postposition chains
    "이다", "입니다", "이고", "이며", "이면", "인데", "인가", "인가요", "인지", "이지만", "이니까", "이라서", "이었다", "이었는데", "이었어요", "이죠", "이네요",
    "이면서", "이든가", "이거나", "이자", "이다가", "이라면", "이라니", "이라니까", "이었던", "이었고", "이었지만", "이었으면", "이던", "이던가", "입니까", "이십니다", "이세요", "이셨다",
    "이시다", "이래", "이래요", "이래서", "이니", "일지", "일까", "일까요", "이지", "이라서는",
    "에서만", "에게도", "에게만", "에게서는", "에다", "에다가", "한테서", "한테는", "쯤은", "쯤이", "쯤에", "쯤에는", "쯤으로", "뿐만", "뿐이", "뿐이다", "뿐인",
    "만요", "마냥", "처럼은", "처럼도", "처럼만", "보다도", "께서는", "께서도", "께서만", "치고", "치고는", "만의", "만으로", "만큼은", "마다는", "마다의", "부터도", "까지만",
    "과의", "와의", "에서의", "으로의", "로의", "으로서의", "로서의", "들만", "들까지", "들에서", "들로", "들께",
    "들한테", "들한테서", "들보다", "들이랑", "들처럼", "들이나", "들께서", "들마다", "들끼리", "들로는", "들에게는", "들에는", "들이다", "들이에요", "들입니다", "들이라",
    "이랑은", "들", "들이", "들을", "들은", "들의", "들도", "들에게", "들과", "만이", "만을", "에의", "씩",
];

/// 으로-type particles: 으로 after consonants except ㄹ, 로 elsewhere.
const RO: &[(&str, &str)] = &[
    ("으로", "로"), ("으로는", "로는"), ("으로도", "로도"), ("으로서", "로서"), ("으로써", "로써"),
    ("으로부터", "로부터"), ("으론", "론"), ("으로만", "로만"),
];

/// Contractions after a vowel-final nominal.
const CONTRACT: &[(&str, Cond)] = &[
    ("ㄴ", Cond::PronVowel), // 저는 → 전
    ("ㄹ", Cond::PronVowel), // 저를 → 절
    ("ㅣ", Cond::PronVowel), // 나의 → 내, 저의 → 제, 거+이 → 게
    ("ㅣ가", Cond::PronVowel), // 내가, 제가, 네가
    ("ㄴ데", Cond::PronVowel),
];

pub struct Particle {
    pub comps: Vec<char>,
    pub cond: Cond,
    /// A form of the copula 이다 (입니다, 이라면, 라니 …), which follows a
    /// noun, pronoun or number but not an adverb (아니+라니 is 아니다+라니).
    pub copular: bool,
}

impl Particle {
    /// A copula written out in full (이다, 입니다, 이라면), not contracted
    /// onto the noun (ㄴ가, 다).
    pub fn full_copula(&self) -> bool {
        self.copular && self.comps.starts_with(&['\u{110B}', '\u{1175}'])
    }
}

/// Copula forms after a consonant (the pairs' vowel forms are copular too).
const COPULA: &[&str] = &[
    "이다", "입니다", "이었다", "이었는데", "이었어요", "이죠", "이네요", "인데", "인가", "인가요", "인지", "인", "일",
    "이라", "이니라", "이로다", "이여", "이시여", "이고", "이며", "이면", "이지만", "이니까", "이라서", "이라면",
    "이라니", "이라니까", "이면서", "이든가", "이거나", "이었던", "이었고", "이었지만", "이었으면", "이자", "이다가",
    "이던", "이던가", "입니까", "이십니다", "이세요", "이셨다", "이시다", "이래", "이래요", "이래서", "이니", "일지",
    "일까", "일까요", "이지", "이라서는", "인듯", "일지도", "일수도", "이려나", "이었으려나",
];

pub fn particles() -> Vec<Particle> {
    let mut out = Vec::new();
    let mut add = |s: &str, cond: Cond, copular: bool| {
        if !s.is_empty() {
            out.push(Particle { comps: decompose(s), cond, copular });
        }
    };
    for (c, v) in PAIRS {
        let copular = COPULA.contains(c);
        add(c, Cond::Cons, copular);
        add(v, Cond::Vowel, copular);
    }
    for a in ANY {
        add(a, Cond::Any, COPULA.contains(a));
    }
    for (c, v) in RO {
        add(c, Cond::ConsNotL, false);
        add(v, Cond::VowelOrL, false);
    }
    for (s, cond) in CONTRACT {
        add(s, *cond, false);
    }
    // One entry per form: where a form that fits only one kind of noun (이다
    // after a consonant) is also listed for any noun, the any-noun entry
    // stands alone. A repeat would give every path through it twice and use
    // up the search's list of best paths. Ordered sets, because a hash set
    // frees its keys in an order that differs per process, which moved the
    // heap's large blocks between runs and the resident peak by 3.5 MB.
    let any: std::collections::BTreeSet<Vec<char>> = out.iter().filter(|p| p.cond == Cond::Any).map(|p| p.comps.clone()).collect();
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|p| (p.cond == Cond::Any || !any.contains(&p.comps)) && seen.insert((p.comps.clone(), p.cond)));
    out
}

/// Whether a particle form may follow a nominal whose last component is
/// `last` (`is_pron` for the ㅣ contraction).
pub fn allowed(cond: Cond, last: char, is_pron: bool) -> bool {
    let vowel = crate::jamo::is_v(last);
    let l = last == 'ᆯ';
    match cond {
        Cond::Any => true,
        Cond::Cons => !vowel,
        Cond::Vowel => vowel,
        Cond::ConsNotL => !vowel && !l,
        Cond::VowelOrL => vowel || l,
        Cond::PronVowel => vowel && is_pron,
    }
}
