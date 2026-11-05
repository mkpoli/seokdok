//! A compound of nouns typed as one word (옛한글, 봄바람) lists the ways of
//! writing it together: each part's leading choices are combined, including
//! readings that only appear when the part is analysed alone (한글 as 韓字,
//! from 한 + 글). Resizing the segment splits the parts, with all their choices.

use super::{Around, Candidate, Edge, Engine, Kind, Segment};
use crate::dict::{is_han, Pos};
use crate::jamo::{compose, syllable_count};

/// Parts in a compound, the syllables it may span, the choices kept per part
/// and the combinations listed.
const MAX_PARTS: usize = 3;
const MAX_SYLLABLES: usize = 8;
const PER_PART: usize = 16;
const MAX_COMBINATIONS: usize = 64;
/// Determiners that form compounds. 그, 이, 한, 두 start phrases (그 사람).
const COMPOUND_DETERMINERS: &[&str] = &["옛", "첫", "새", "맨", "헌", "온"];
/// How far a part in Hangul alone ranks behind its Hanja choices.
const HANGUL_PENALTY: usize = 3;

/// Demonstratives, which run straight into the word they point at (그화를).
const DEMONSTRATIVES: [(char, char); 3] = [('그', '其'), ('이', '此'), ('저', '彼')];
/// How many of the word's own choices follow one.
const AFTER_DEMONSTRATIVE: usize = 12;

impl Engine {
    /// A run left whole that starts with a demonstrative also reads as it plus
    /// a word: 그화를 as 其化를, 其話를. The word's choices come from its own analysis.
    pub(super) fn demonstrative_choices(&self, run: &str) -> Vec<Candidate> {
        let mut chars = run.chars();
        let Some(first) = chars.next() else { return Vec::new() };
        let Some(&(_, hanja)) = DEMONSTRATIVES.iter().find(|(d, _)| *d == first) else { return Vec::new() };
        let rest = chars.as_str();
        if self.options.sino_only || rest.is_empty() {
            return Vec::new();
        }
        self.convert_plain(rest, false, false, Around::default())
            .whole
            .into_iter()
            .filter(|c| c.text.chars().any(is_han))
            .take(AFTER_DEMONSTRATIVE)
            .map(|c| Candidate { text: format!("{hanja}{}", c.text), note: c.note })
            .collect()
    }

    /// The particle closing edge `e`, if any.
    fn edge_particle(e: &Edge) -> Option<u32> {
        match e.kind {
            Kind::Nominal { particle, .. } | Kind::Sino { particle, .. } => particle.map(u32::from),
            _ => None,
        }
    }

    /// Whether a word that starts phrases also spells edge `e`'s span: a
    /// determiner, pronoun, adverb, interjection, numeral or bound noun (그, 두, 또).
    fn spells_phrase_word(&self, edges: &[Edge], e: &Edge) -> bool {
        edges.iter().any(|x| {
            let Kind::Nominal { entry, particle: None } = x.kind else { return false };
            let n = &self.natives[entry as usize];
            x.start == e.start
                && x.end == e.end
                && n.pos != Pos::Noun
                && !(n.pos == Pos::Det && COMPOUND_DETERMINERS.contains(&n.lemma.as_str()))
        })
    }

    /// Whether edge `e` can be a part of a compound: a noun, a Sino-Korean word,
    /// or Hangul left over a span that a noun also spells. Letters that make a
    /// particle (에, 를) are not one.
    fn is_compound_part(&self, comps: &[char], edges: &[Edge], e: &Edge) -> bool {
        if self.spells_phrase_word(edges, e) {
            return false;
        }
        match e.kind {
            Kind::Nominal { entry, .. } => {
                let n = &self.natives[entry as usize];
                n.pos == Pos::Noun || (n.pos == Pos::Det && COMPOUND_DETERMINERS.contains(&n.lemma.as_str()))
            }
            Kind::Sino { .. } => true,
            Kind::Raw => {
                let particle = self.particle_trie.prefixes(comps, e.start).iter().any(|(r, _)| *r == e.end);
                !particle
                    && edges.iter().any(|x| {
                        x.start == e.start
                            && x.end == e.end
                            && Self::edge_particle(x).is_none()
                            && matches!(x.kind, Kind::Nominal { .. } | Kind::Sino { .. })
                            && self.is_compound_part(comps, edges, x)
                    })
            }
            _ => false,
        }
    }

    /// Whether `path` is nouns only, with a particle at most on the last one.
    fn is_compound(&self, comps: &[char], edges: &[Edge], path: &[usize]) -> bool {
        (2..=MAX_PARTS).contains(&path.len())
            && syllable_count(comps) <= MAX_SYLLABLES
            && path.iter().enumerate().all(|(k, &ei)| {
                self.is_compound_part(comps, edges, &edges[ei]) && (k + 1 == path.len() || Self::edge_particle(&edges[ei]).is_none())
            })
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
        // The readings sharing the part's first character lead (韓字 before 漢字, for 韓㐎).
        let lead = own.first().and_then(|c| c.text.chars().next());
        list[own.len().min(PER_PART)..].sort_by_key(|c| c.text.chars().next() != lead);
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
