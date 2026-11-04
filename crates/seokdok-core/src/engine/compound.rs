//! A compound of nouns typed as one word (옛한글, 봄바람) offers every way of
//! writing it, as one list: each part's own choices are combined, including
//! readings that only appear when the part is analysed alone (한글 as 韓字,
//! from 한 + 글). The parts can still be split by resizing the segment.

use super::{Around, Candidate, Edge, Engine, Kind, Segment};
use crate::dict::is_han;
use crate::jamo::{compose, syllable_count};

/// Parts in a compound, the syllables it may span, the choices kept per part
/// and the combinations listed.
const MAX_PARTS: usize = 3;
const MAX_SYLLABLES: usize = 8;
const PER_PART: usize = 16;
const MAX_COMBINATIONS: usize = 64;
/// How far a part in Hangul alone ranks behind its Hanja choices.
const HANGUL_PENALTY: usize = 3;

impl Engine {
    /// The particle closing edge `e`, if any.
    fn edge_particle(e: &Edge) -> Option<u32> {
        match e.kind {
            Kind::Nominal { particle, .. } | Kind::Sino { particle, .. } => particle.map(u32::from),
            _ => None,
        }
    }

    /// Whether `path` is nouns only, with a particle at most on the last one.
    fn is_compound(&self, comps: &[char], edges: &[Edge], path: &[usize]) -> bool {
        if !(2..=MAX_PARTS).contains(&path.len()) || syllable_count(comps) > MAX_SYLLABLES {
            return false;
        }
        let mut words = 0;
        for (k, &ei) in path.iter().enumerate() {
            match edges[ei].kind {
                Kind::Raw => {}
                Kind::Nominal { .. } | Kind::Sino { .. } => {
                    words += 1;
                    if k + 1 < path.len() && Self::edge_particle(&edges[ei]).is_some() {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        words > 0
    }

    /// One part's choices: the segment's own, then what the part's letters
    /// give in pure Hanja when analysed alone, with its particle.
    fn part_choices(&self, comps: &[char], e: &Edge, own: &[Candidate]) -> Vec<Candidate> {
        let mut list: Vec<Candidate> = own.iter().take(PER_PART).cloned().collect();
        let tail = Self::edge_particle(e).map_or(String::new(), |p| compose(&self.particles[p as usize].comps));
        let stem_end = e.end - Self::edge_particle(e).map_or(0, |p| self.particles[p as usize].comps.len());
        let word = compose(&comps[e.start..stem_end]);
        for c in self.convert_plain(&word, false, false, Around::default()).whole {
            if list.len() >= PER_PART {
                break;
            }
            let text = format!("{}{tail}", c.text);
            if c.text.chars().all(is_han) && !list.iter().any(|x| x.text == text) {
                list.push(Candidate { text, note: c.note });
            }
        }
        list
    }

    /// The compound's combinations, best first: `path`'s own spelling leads.
    /// `None` if `path` is not a compound.
    pub(super) fn compound_list(&self, comps: &[char], edges: &[Edge], path: &[usize], segments: &[Segment]) -> Option<Vec<Candidate>> {
        if self.options.sino_only || segments.len() != path.len() || !self.is_compound(comps, edges, path) {
            return None;
        }
        let parts: Vec<Vec<Candidate>> = path.iter().zip(segments).map(|(&ei, s)| self.part_choices(comps, &edges[ei], &s.candidates)).collect();
        let weight = |rank: usize, c: &Candidate| rank + if c.text.chars().any(is_han) { 0 } else { HANGUL_PENALTY };
        // Every combination by its summed rank; a stable sort keeps the first part's order.
        let mut combos: Vec<(usize, Vec<usize>)> = vec![(0, Vec::new())];
        for part in &parts {
            let mut next = Vec::with_capacity(combos.len() * part.len());
            for (w, picks) in &combos {
                for (i, c) in part.iter().enumerate() {
                    let mut p = picks.clone();
                    p.push(i);
                    next.push((w + weight(i, c), p));
                }
            }
            next.sort_by_key(|(w, _)| *w);
            next.truncate(MAX_COMBINATIONS * 4);
            combos = next;
        }
        let mut out: Vec<Candidate> = Vec::new();
        for (_, picks) in combos {
            let text: String = picks.iter().zip(&parts).map(|(&i, p)| p[i].text.as_str()).collect();
            if !out.iter().any(|c| c.text == text) {
                let note = picks.iter().zip(&parts).map(|(&i, p)| p[i].note.as_str()).filter(|n| !n.is_empty()).collect::<Vec<_>>().join(" / ");
                out.push(Candidate { text, note });
                if out.len() == MAX_COMBINATIONS {
                    break;
                }
            }
        }
        Some(out)
    }
}
