//! 送韓字 strategies: other ways to write the Hangul after a word's Hanja.
//!
//! A 訓讀 entry has one dictionary form (超무, 初음, 近다). The same word is
//! also written with more or less of its reading after the Hanja. The
//! alternatives are made when a segment is rendered, so the dictionaries and
//! the engine's memory do not grow.
//!
//! Nouns, adverbs and pronouns whose Hanja is a single character get, in this
//! order: the last syllable (何게), every syllable but the first (何떻게), the
//! last letter alone (何ㅔ) and the Hanja alone (何). Inflected stems keep
//! their ending and, when the Hanja stand for two syllables or more, write the
//! typed word's own syllables from the last covered one, or from the second,
//! after the Hanja: 近웠다 also reads 近까웠다, 飮ㄴ also reads 飮신. Particles
//! follow whichever form is chosen.

use super::{Candidate, Edge, Engine, Kind};
use crate::conj::ending_surfaces;
use crate::dict::{is_han, Native, Pos, Tail};
use crate::jamo::{compose, is_l, is_syllable, is_t, is_v};
use crate::particles::allowed;

/// Whether `n` has variants: one Hanja character covers the word.
fn eligible(n: &Native) -> bool {
    let mut han = n.hanja.chars();
    let one = han.next().is_some_and(is_han) && han.next().is_none();
    one && (n.pos.inflects() || matches!(n.pos, Pos::Noun | Pos::Adv | Pos::Pron))
}

/// The components of each syllable of `comps`, which starts with an onset.
fn syllables(comps: &[char]) -> Vec<&[char]> {
    let starts: Vec<usize> = (0..comps.len()).filter(|&i| is_l(comps[i])).collect();
    (0..starts.len()).map(|k| &comps[starts[k]..starts.get(k + 1).copied().unwrap_or(comps.len())]).collect()
}

/// The last letter: the final consonant if there is one, else the whole last
/// vowel (the ㅜ of 무, the ㅔ of 게).
fn last_letter(comps: &[char]) -> Vec<char> {
    match comps.last() {
        Some(&c) if is_t(c) => vec![c],
        _ => comps[comps.len() - comps.iter().rev().take_while(|&&c| is_v(c)).count()..].to_vec(),
    }
}

impl Engine {
    /// The 送韓字 variants of an edge whose dictionary forms are `dict`, new
    /// texts only, in the order of the module docs. `comps` is the whole
    /// input the edge lies in. `hanja_only` adds the form with the Hanja
    /// alone (nouns); [`Engine::read`] leaves it out and keeps reading such
    /// forms as 音讀. The result does not depend on the style options.
    pub(super) fn okurigana(&self, comps: &[char], e: &Edge, dict: &[Candidate], hanja_only: bool) -> Vec<Candidate> {
        let (n, tails): (&Native, Vec<String>) = match &e.kind {
            Kind::Nominal { entry, particle } => {
                let n = &self.natives[*entry as usize];
                if !eligible(n) {
                    return Vec::new();
                }
                let p: &[char] = particle.map_or(&[], |p| &self.particles[p as usize].comps);
                let syl = syllables(&n.stem);
                let mut ks: Vec<&[char]> = Vec::new();
                if syl.len() >= 2 {
                    ks.push(syl[syl.len() - 1]);
                    ks.push(&n.stem[syl[0].len()..]);
                }
                let letter = last_letter(&n.stem);
                ks.push(&letter);
                if hanja_only {
                    ks.push(&[]);
                }
                (n, ks.into_iter().map(|k| compose(&[k, p].concat())).collect())
            }
            Kind::Verb { entry, allo, .. } => {
                let n = &self.natives[*entry as usize];
                if !eligible(n) {
                    return Vec::new();
                }
                let al = &n.allos[*allo as usize];
                // The stem as pronounced here, as far as the Hanja cover it.
                let covered: &[char] = match n.tail {
                    Tail::None => &al.comps,
                    Tail::Partial if al.comps.starts_with(&n.stem[..n.cover]) => &n.stem[..n.cover],
                    _ => return Vec::new(),
                };
                // The Hanja stand for the first `m` syllables of the word. What
                // follows them is a tail of the typed word's own syllables, so
                // the variant reads back as the word: the syllables from the
                // last covered one on (飮셔라), or from the second on.
                let m = covered.iter().filter(|&&c| is_l(c)).count();
                let word: Vec<char> = compose(&comps[e.start..e.end]).chars().collect();
                if m < 2 || word.len() < m || !word.iter().all(|&c| is_syllable(c)) {
                    return Vec::new();
                }
                (n, [m - 1, 1].iter().map(|&from| word[from..].iter().collect()).collect())
            }
            _ => return Vec::new(),
        };
        let note = dict.first().map(|c| c.note.clone()).unwrap_or_default();
        let mut out: Vec<Candidate> = Vec::new();
        for t in tails {
            let text = format!("{}{t}", n.hanja);
            if !dict.iter().chain(&out).any(|c| c.text == text) {
                out.push(Candidate { text, note: note.clone() });
            }
        }
        out
    }

    /// Every variant of native entry `i` with the Hangul it reads as, the way
    /// [`Engine::forms_of`] lists the dictionary's forms, for [`Engine::read`].
    pub(super) fn variant_forms(&self, i: usize) -> Vec<(String, String)> {
        let n = &self.natives[i];
        let mut out = Vec::new();
        if !eligible(n) {
            return out;
        }
        let mut add = |comps: Vec<char>, kind: Kind| {
            let edge = Edge { start: 0, end: comps.len(), cost: 0.0, kind };
            let surface = compose(&comps);
            for c in self.okurigana(&comps, &edge, &[], false) {
                out.push((c.text, surface.clone()));
            }
        };
        if n.pos.inflects() {
            for (a, al) in n.allos.iter().enumerate() {
                for (e, idx) in ending_surfaces(al.ctx) {
                    add([al.comps.clone(), e].concat(), Kind::Verb { entry: i as u32, allo: a as u16, ending: idx as u16 });
                }
            }
        } else {
            add(n.stem.clone(), Kind::Nominal { entry: i as u32, particle: None });
            let pron = matches!(n.pos, Pos::Pron | Pos::Bound);
            let last = *n.stem.last().unwrap();
            for (pi, p) in self.particles.iter().enumerate() {
                if allowed(p.cond, last, pron) {
                    add([n.stem.clone(), p.comps.clone()].concat(), Kind::Nominal { entry: i as u32, particle: Some(pi as u16) });
                }
            }
        }
        out
    }
}
