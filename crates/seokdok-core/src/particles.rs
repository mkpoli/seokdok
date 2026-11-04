//! Particles (조사) and copula forms that follow a nominal. They stay in
//! Hangul; contracted forms after a vowel (저+ㄴ → 전, 나+ㅣ → 내) are written
//! as the bare letter (自ㄴ, 我ㅣ).

use crate::jamo::decompose;

/// Which nominal endings a particle form may follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    ("이를", ""), ("이도", ""), ("이의", ""), ("이에게", ""),
];

const ANY: &[&str] = &[
    "의", "에", "에서", "에게", "에게서", "한테", "께", "께서", "도", "만", "까지", "부터", "보다",
    "처럼", "같이", "마다", "조차", "마저", "밖에", "뿐", "요", "대로", "쯤", "에는", "에서는", "에서도",
    "에게는", "에도", "에만", "까지는", "까지도", "부터는", "만은", "만큼", "보다는", "에선", "엔",
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
}

pub fn particles() -> Vec<Particle> {
    let mut out = Vec::new();
    let mut add = |s: &str, cond: Cond| {
        if !s.is_empty() {
            out.push(Particle { comps: decompose(s), cond });
        }
    };
    for (c, v) in PAIRS {
        add(c, Cond::Cons);
        add(v, Cond::Vowel);
    }
    for a in ANY {
        add(a, Cond::Any);
    }
    for (c, v) in RO {
        add(c, Cond::ConsNotL);
        add(v, Cond::VowelOrL);
    }
    for (s, cond) in CONTRACT {
        add(s, *cond);
    }
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
