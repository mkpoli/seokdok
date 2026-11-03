//! The converter: a lattice over the jamo components of each Hangul run,
//! k-best search, and rendering of each analysis in the configured 送韓字
//! style.

use crate::conj::{ending_label, ending_surfaces, morph_ending, Ctx};
use crate::dict::{parse_native, parse_sino, tsv_lines, Native, Pos, Sino, Source, Tail};
use crate::jamo::{compose, decompose, is_l, is_syllable, is_t, is_v, resyllabify};
use crate::particles::{allowed, particles, Particle};
use std::collections::HashMap;

/// How the Hangul after a 訓讀 stem is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// 表音式: write what is pronounced after the stem (見ㅆ다 for 봤다,
    /// 近웠다 for 가까웠다). The 標準國語釋讀辭典 paradigms use this.
    Surface,
    /// 形態式: write the ending as attached to the underlying stem, without
    /// contraction (見았다, 難어요).
    Morpheme,
}

/// In the surface style, a vowel left over from a contraction (보+아 → 봐).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fragment {
    /// The Hanja absorbs the merged syllable: 見라 (봐라), 飮라 (마셔라).
    Absorb,
    /// The leftover vowel is written as a letter: 見ㅏ라, 飮ㅓ라.
    Letter,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub style: Style,
    pub fragment: Fragment,
    /// 末音添記 for every consonant-final 訓讀 stem (有ㅆ다), not only for
    /// entries whose written form asks for it (在ㅆ다).
    pub coda_always: bool,
    /// Write a written-out final consonant as the onset of a following ㅇ
    /// syllable (在쓰면 instead of 在ㅆ으면).
    pub resyllabify: bool,
    /// Prefer 爲 for the light verb 하다 (安寧爲세요 over 安寧하세요).
    pub hada_kun: bool,
    /// Maximum number of whole-phrase candidates.
    pub max_candidates: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            style: Style::Surface,
            fragment: Fragment::Absorb,
            coda_always: false,
            resyllabify: false,
            hada_kun: false,
            max_candidates: 12,
        }
    }
}

struct Node<T> {
    next: Vec<(char, u32)>,
    items: Vec<T>,
}

struct Trie<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Trie<T> {
    fn new() -> Self {
        Trie { nodes: vec![Node { next: Vec::new(), items: Vec::new() }] }
    }
    fn insert(&mut self, key: &[char], item: T) {
        let mut n = 0usize;
        for &c in key {
            let found = self.nodes[n].next.iter().find(|(k, _)| *k == c).map(|(_, i)| *i as usize);
            n = match found {
                Some(i) => i,
                None => {
                    self.nodes.push(Node { next: Vec::new(), items: Vec::new() });
                    let i = self.nodes.len() - 1;
                    self.nodes[n].next.push((c, i as u32));
                    i
                }
            };
        }
        self.nodes[n].items.push(item);
    }
    /// Every (end position, items) for keys that are prefixes of `input[from..]`.
    fn prefixes<'a>(&'a self, input: &[char], from: usize) -> Vec<(usize, &'a [T])> {
        let mut out = Vec::new();
        let mut n = 0usize;
        let mut pos = from;
        loop {
            if !self.nodes[n].items.is_empty() && pos > from {
                out.push((pos, self.nodes[n].items.as_slice()));
            }
            if pos >= input.len() {
                break;
            }
            match self.nodes[n].next.iter().find(|(k, _)| *k == input[pos]) {
                Some((_, i)) => {
                    n = *i as usize;
                    pos += 1;
                }
                None => break,
            }
        }
        out
    }
}

#[derive(Clone, Debug)]
enum Kind {
    Raw,
    Sino { entry: u32, particle: Option<u16> },
    Nominal { entry: u32, particle: Option<u16> },
    Verb { entry: u32, allo: u16, ending: u16 },
}

#[derive(Clone, Debug)]
struct Edge {
    start: usize,
    end: usize,
    cost: f32,
    kind: Kind,
}

/// One rendering of a segment.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub text: String,
    /// Short analysis for display, e.g. "讀 읽다 +아/어ㅆ+다 · community".
    pub note: String,
}

/// A span of the input with its alternative renderings, best first.
#[derive(Clone, Debug)]
pub struct Segment {
    /// The Hangul (or other) input this segment covers.
    pub reading: String,
    pub candidates: Vec<Candidate>,
}

pub struct Engine {
    pub natives: Vec<Native>,
    pub sinos: Vec<Sino>,
    pub options: Options,
    stems: Trie<(u32, u16)>,
    nominals: Trie<u32>,
    sino: Trie<u32>,
    endings: HashMap<Ctx, Trie<u16>>,
    particles: Vec<Particle>,
    particle_trie: Trie<u16>,
}

const RAW_COST: f32 = 4.5;
/// Extra cost for one-syllable words, which are the most ambiguous.
const SHORT_COST: f32 = 2.5;
const EDGE_COST: f32 = 1.0;

/// Built-in entries: Hangul-only light verb 하다, so 安寧하세요 is the
/// default and 爲 comes from the dictionary as an alternative.
const BUILTIN: &str = "하다\t하다\tverb\t\teditorial\t\n";

impl Engine {
    /// Build an engine from TSV texts. Returns the engine and the parse
    /// errors (file label, line, message), which are skipped.
    pub fn new(native_tsvs: &[(&str, &str)], sino_tsvs: &[(&str, &str)], options: Options) -> (Engine, Vec<String>) {
        let mut errors = Vec::new();
        let mut natives = Vec::new();
        for (label, text) in std::iter::once(&("builtin", BUILTIN)).chain(native_tsvs.iter()) {
            for (line, f) in tsv_lines(text) {
                match parse_native(&f) {
                    Ok(mut e) => {
                        e.origin = (label.to_string(), line);
                        natives.push(e)
                    }
                    Err(m) => errors.push(format!("{label}:{line}: {m}")),
                }
            }
        }
        let mut sinos = Vec::new();
        for (label, text) in sino_tsvs {
            for (line, f) in tsv_lines(text) {
                match parse_sino(&f) {
                    Ok(e) => sinos.push(e),
                    Err(m) => errors.push(format!("{label}:{line}: {m}")),
                }
            }
        }
        let mut eng = Engine {
            natives,
            sinos,
            options,
            stems: Trie::new(),
            nominals: Trie::new(),
            sino: Trie::new(),
            endings: HashMap::new(),
            particles: particles(),
            particle_trie: Trie::new(),
        };
        eng.index();
        (eng, errors)
    }

    /// Native dictionaries loaded by [`Engine::from_dir`], in order.
    pub const NATIVE_FILES: &'static [&'static str] = &["user.tsv", "corpus.tsv", "editorial.tsv", "community.tsv", "gloss.tsv"];
    pub const SINO_FILES: &'static [&'static str] = &["user-sino.tsv", "sino.tsv"];

    /// Load every known dictionary file present in `dir`.
    pub fn from_dir(dir: &std::path::Path, options: Options) -> std::io::Result<(Engine, Vec<String>)> {
        let read = |names: &[&str]| -> std::io::Result<Vec<(String, String)>> {
            let mut v = Vec::new();
            for n in names {
                let p = dir.join(n);
                if p.exists() {
                    v.push((n.to_string(), std::fs::read_to_string(&p)?));
                }
            }
            Ok(v)
        };
        let nat = read(Self::NATIVE_FILES)?;
        let sin = read(Self::SINO_FILES)?;
        let nat_ref: Vec<(&str, &str)> = nat.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let sin_ref: Vec<(&str, &str)> = sin.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        Ok(Engine::new(&nat_ref, &sin_ref, options))
    }

    fn index(&mut self) {
        for (i, n) in self.natives.iter().enumerate() {
            if n.pos.inflects() {
                for (a, al) in n.allos.iter().enumerate() {
                    self.stems.insert(&al.comps, (i as u32, a as u16));
                    self.endings.entry(al.ctx).or_insert_with(|| {
                        let mut t = Trie::new();
                        for (comps, idx) in ending_surfaces(al.ctx) {
                            if !comps.is_empty() {
                                t.insert(&comps, idx as u16);
                            }
                        }
                        t
                    });
                }
            } else {
                self.nominals.insert(&n.stem, i as u32);
            }
        }
        for (i, s) in self.sinos.iter().enumerate() {
            self.sino.insert(&decompose(&s.reading), i as u32);
        }
        for (i, p) in self.particles.iter().enumerate() {
            self.particle_trie.insert(&p.comps, i as u16);
        }
    }

    fn native_cost(&self, n: &Native) -> f32 {
        let mut c = n.cost;
        if n.lemma == "하다" {
            if n.hanja.is_empty() && self.options.hada_kun {
                c += 5.0;
            } else if !n.hanja.is_empty() && !self.options.hada_kun {
                c += 2.0;
            }
        }
        c
    }

    fn edges(&self, comps: &[char]) -> Vec<Edge> {
        let n = comps.len();
        let boundary = |p: usize| p == n || is_l(comps[p]);
        let mut edges = Vec::new();
        for s in 0..n {
            if !is_l(comps[s]) {
                continue;
            }
            let mut e = s + 1;
            while !boundary(e) {
                e += 1;
            }
            edges.push(Edge { start: s, end: e, cost: RAW_COST, kind: Kind::Raw });
            // Sino-Korean words, optionally with a particle.
            for (q, items) in self.sino.prefixes(comps, s) {
                if !boundary(q) {
                    continue;
                }
                for &i in items {
                    let short = if crate::jamo::syllable_count(&comps[s..q]) == 1 { SHORT_COST } else { 0.0 };
                    let base = EDGE_COST + self.sinos[i as usize].cost + short;
                    edges.push(Edge { start: s, end: q, cost: base, kind: Kind::Sino { entry: i, particle: None } });
                    for (r, pis) in self.particle_trie.prefixes(comps, q) {
                        if !boundary(r) {
                            continue;
                        }
                        for &pi in pis {
                            let p = &self.particles[pi as usize];
                            if allowed(p.cond, comps[q - 1], false) && !is_t(p.comps[0]) && !is_v(p.comps[0]) {
                                edges.push(Edge {
                                    start: s,
                                    end: r,
                                    cost: base + 0.2,
                                    kind: Kind::Sino { entry: i, particle: Some(pi) },
                                });
                            }
                        }
                    }
                }
            }
            // Native nominals (訓讀 nouns, pronouns, adverbs …).
            for (q, items) in self.nominals.prefixes(comps, s) {
                for &i in items {
                    let entry = &self.natives[i as usize];
                    let short = if crate::jamo::syllable_count(&entry.stem) > 1 {
                        0.0
                    } else {
                        match entry.source {
                            Source::Gloss | Source::Lexicon => SHORT_COST,
                            Source::Community => 1.0,
                            _ => 0.0,
                        }
                    };
                    let base = EDGE_COST + self.native_cost(entry) + short;
                    if boundary(q) {
                        edges.push(Edge { start: s, end: q, cost: base, kind: Kind::Nominal { entry: i, particle: None } });
                    }
                    let pron = entry.pos == Pos::Pron;
                    for (r, pis) in self.particle_trie.prefixes(comps, q) {
                        if !boundary(r) {
                            continue;
                        }
                        for &pi in pis {
                            let p = &self.particles[pi as usize];
                            if allowed(p.cond, comps[q - 1], pron) {
                                edges.push(Edge {
                                    start: s,
                                    end: r,
                                    cost: base + 0.2,
                                    kind: Kind::Nominal { entry: i, particle: Some(pi) },
                                });
                            }
                        }
                    }
                }
            }
            // Inflected verbs and adjectives: stem allomorph + ending.
            for (q, items) in self.stems.prefixes(comps, s) {
                for &(i, a) in items {
                    let entry = &self.natives[i as usize];
                    let ctx = entry.allos[a as usize].ctx;
                    let Some(et) = self.endings.get(&ctx) else { continue };
                    for (r, eis) in et.prefixes(comps, q) {
                        if !boundary(r) {
                            continue;
                        }
                        for &ei in eis {
                            let morphs = ending_label(ei as usize).matches('+').count() as f32;
                            edges.push(Edge {
                                start: s,
                                end: r,
                                cost: EDGE_COST + self.native_cost(entry) + 0.1 + 0.15 * morphs,
                                kind: Kind::Verb { entry: i, allo: a, ending: ei },
                            });
                        }
                    }
                }
            }
        }
        edges
    }

    /// k-best paths over a Hangul run (edge index lists, cheapest first).
    fn kbest(&self, comps: &[char], edges: &[Edge], k: usize) -> Vec<(f32, Vec<usize>)> {
        let n = comps.len();
        let mut by_start: Vec<Vec<usize>> = vec![Vec::new(); n + 1];
        for (i, e) in edges.iter().enumerate() {
            by_start[e.start].push(i);
        }
        let mut best: Vec<Vec<(f32, Vec<usize>)>> = vec![Vec::new(); n + 1];
        best[0].push((0.0, Vec::new()));
        for p in 0..n {
            if best[p].is_empty() {
                continue;
            }
            let here = std::mem::take(&mut best[p]);
            for &ei in &by_start[p] {
                let e = &edges[ei];
                for (c, path) in &here {
                    let mut np = path.clone();
                    np.push(ei);
                    best[e.end].push((c + e.cost, np));
                }
                best[e.end].sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                best[e.end].truncate(k);
            }
            best[p] = here;
        }
        std::mem::take(&mut best[n])
    }

    fn render(&self, comps: &[char], e: &Edge) -> Vec<Candidate> {
        let slice = &comps[e.start..e.end];
        let raw = || Candidate { text: compose(slice), note: String::new() };
        match &e.kind {
            Kind::Raw => vec![raw()],
            Kind::Sino { entry, particle } => {
                let s = &self.sinos[*entry as usize];
                let mut t = s.hanja.clone();
                if let Some(p) = particle {
                    t.push_str(&compose(&self.particles[*p as usize].comps));
                }
                vec![Candidate { text: t, note: format!("{} {} · 音讀", s.hanja, s.reading) }]
            }
            Kind::Nominal { entry, particle } => {
                let n = &self.natives[*entry as usize];
                if n.hanja.is_empty() {
                    return vec![raw()];
                }
                let mut t = n.hanja.clone();
                t.push_str(&compose(&n.stem[n.cover..]));
                if let Some(p) = particle {
                    t.push_str(&compose(&self.particles[*p as usize].comps));
                }
                vec![Candidate { text: t, note: note(n, None) }]
            }
            Kind::Verb { entry, allo, ending } => {
                let n = &self.natives[*entry as usize];
                if n.hanja.is_empty() {
                    return vec![raw()];
                }
                let al = &n.allos[*allo as usize];
                let tail = &comps[e.start + al.comps.len()..e.end];
                let o = &self.options;
                let mut out: Vec<String> = Vec::new();
                let surf = |frag: Fragment, coda: bool| self.surface(n, &al.comps, tail, frag, coda);
                let morph = |coda: bool| self.morpheme(n, *ending as usize, coda);
                let other_frag = if o.fragment == Fragment::Absorb { Fragment::Letter } else { Fragment::Absorb };
                let order: Vec<Option<String>> = match o.style {
                    Style::Surface => vec![
                        Some(surf(o.fragment, o.coda_always)),
                        morph(o.coda_always),
                        Some(surf(other_frag, o.coda_always)),
                        Some(surf(o.fragment, !o.coda_always)),
                        morph(!o.coda_always),
                    ],
                    Style::Morpheme => vec![
                        morph(o.coda_always),
                        Some(surf(o.fragment, o.coda_always)),
                        Some(surf(other_frag, o.coda_always)),
                        morph(!o.coda_always),
                        Some(surf(o.fragment, !o.coda_always)),
                    ],
                };
                for t in order.into_iter().flatten() {
                    if !out.contains(&t) {
                        out.push(t);
                    }
                }
                let nt = note(n, Some(*ending as usize));
                out.into_iter().map(|text| Candidate { text, note: nt.clone() }).collect()
            }
        }
    }

    /// Surface style: the Hanja stand for the stem as pronounced; what follows
    /// is the rest of the pronounced word.
    fn surface(&self, n: &Native, allo: &[char], ending: &[char], frag: Fragment, coda_always: bool) -> String {
        let strip_v = |v: &[char]| -> Vec<char> {
            if frag == Fragment::Absorb {
                v.iter().copied().skip_while(|&c| is_v(c)).collect()
            } else {
                v.to_vec()
            }
        };
        let codas = |s: &[char]| -> Vec<char> {
            let k = s.iter().rev().take_while(|&&c| is_t(c)).count();
            s[s.len() - k..].to_vec()
        };
        let mut written_coda = false;
        let rest: Vec<char> = match n.tail {
            Tail::None => {
                if coda_always && allo == n.stem.as_slice() && simple_coda(&n.stem) {
                    written_coda = true;
                    [codas(&n.stem), ending.to_vec()].concat()
                } else {
                    strip_v(ending)
                }
            }
            Tail::Coda => {
                if allo.starts_with(&n.stem) {
                    written_coda = true;
                    [n.stem[n.cover..].to_vec(), ending.to_vec()].concat()
                } else {
                    strip_v(ending)
                }
            }
            Tail::Partial => {
                if allo.starts_with(&n.stem[..n.cover]) {
                    let left: Vec<char> = allo[n.cover..].iter().copied().skip_while(|&c| is_t(c)).collect();
                    if left.is_empty() {
                        strip_v(ending)
                    } else {
                        [left, ending.to_vec()].concat()
                    }
                } else {
                    strip_v(ending)
                }
            }
        };
        let rest = if written_coda && self.options.resyllabify { resyllabify(&rest) } else { rest };
        format!("{}{}", n.hanja, compose(&rest))
    }

    /// Morpheme style: the ending written as attached to the lemma's
    /// underlying final, without contraction.
    fn morpheme(&self, n: &Native, ending: usize, coda_always: bool) -> Option<String> {
        let e = morph_ending(n.morph?, ending)?;
        let mut written_coda = false;
        let prefix: Vec<char> = match n.tail {
            Tail::None if coda_always && simple_coda(&n.stem) => {
                written_coda = true;
                let k = n.stem.iter().rev().take_while(|&&c| is_t(c)).count();
                n.stem[n.stem.len() - k..].to_vec()
            }
            Tail::None => Vec::new(),
            Tail::Coda => {
                written_coda = true;
                n.stem[n.cover..].to_vec()
            }
            Tail::Partial => n.stem[n.cover..].to_vec(),
        };
        // A written-out final consonant already carries the 으 slot.
        let rest = [prefix, e].concat();
        let rest = if written_coda && self.options.resyllabify { resyllabify(&rest) } else { rest };
        Some(format!("{}{}", n.hanja, compose(&rest)))
    }

    fn convert_run(&self, run: &str) -> (Vec<Candidate>, Vec<Segment>) {
        let comps = decompose(run);
        let edges = self.edges(&comps);
        let paths = self.kbest(&comps, &edges, self.options.max_candidates.max(4));
        let mut whole: Vec<Candidate> = Vec::new();
        let push = |c: Candidate, whole: &mut Vec<Candidate>| {
            if !whole.iter().any(|w| w.text == c.text) {
                whole.push(c);
            }
        };
        for (_, path) in &paths {
            let mut text = String::new();
            let mut notes = Vec::new();
            for &ei in path {
                let r = self.render(&comps, &edges[ei]);
                text.push_str(&r[0].text);
                if !r[0].note.is_empty() {
                    notes.push(r[0].note.clone());
                }
            }
            push(Candidate { text, note: notes.join(" / ") }, &mut whole);
        }
        // Second rendering of the best path (other 送韓字 variant).
        if let Some((_, path)) = paths.first() {
            let mut text = String::new();
            for &ei in path {
                let r = self.render(&comps, &edges[ei]);
                text.push_str(&r.get(1).unwrap_or(&r[0]).text);
            }
            push(Candidate { text, note: "variant".into() }, &mut whole);
        }
        push(Candidate { text: run.to_string(), note: "한글".into() }, &mut whole);
        whole.truncate(self.options.max_candidates.max(2));

        let mut segments = Vec::new();
        if let Some((_, path)) = paths.first() {
            for &ei in path {
                let e = &edges[ei];
                let mut alts: Vec<(f32, usize)> = edges
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.start == e.start && x.end == e.end)
                    .map(|(i, x)| (x.cost, i))
                    .collect();
                alts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                let mut cands: Vec<Candidate> = Vec::new();
                for (_, i) in alts {
                    for c in self.render(&comps, &edges[i]) {
                        if !cands.iter().any(|x| x.text == c.text) {
                            cands.push(c);
                        }
                    }
                }
                segments.push(Segment { reading: compose(&comps[e.start..e.end]), candidates: cands });
            }
        }
        (whole, segments)
    }

    /// Whole-input candidates, best first. Non-Hangul text passes through.
    pub fn convert(&self, input: &str) -> Vec<Candidate> {
        let mut per_run: Vec<Vec<Candidate>> = Vec::new();
        for (is_h, run) in runs(input) {
            if is_h {
                per_run.push(self.convert_run(&run).0);
            } else {
                per_run.push(vec![Candidate { text: run, note: String::new() }]);
            }
        }
        let k = per_run.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut out: Vec<Candidate> = Vec::new();
        for i in 0..k {
            let mut text = String::new();
            let mut notes = Vec::new();
            for r in &per_run {
                let c = r.get(i).unwrap_or(&r[0]);
                text.push_str(&c.text);
                if !c.note.is_empty() && c.note != "한글" {
                    notes.push(c.note.clone());
                }
            }
            if !out.iter().any(|c| c.text == text) {
                out.push(Candidate { text, note: notes.join(" / ") });
            }
        }
        out
    }

    /// Segmentation of the best analysis, each segment with alternatives.
    pub fn segments(&self, input: &str) -> Vec<Segment> {
        let mut out = Vec::new();
        for (is_h, run) in runs(input) {
            if is_h {
                out.extend(self.convert_run(&run).1);
            } else {
                out.push(Segment { reading: run.clone(), candidates: vec![Candidate { text: run, note: String::new() }] });
            }
        }
        out
    }
}

impl Engine {
    /// Every written form of native entry `i` with its Hangul surface, in all
    /// rendering variants (used to count attestations in a corpus).
    pub fn forms_of(&self, i: usize) -> Vec<(String, String)> {
        let n = &self.natives[i];
        let mut out = Vec::new();
        if n.hanja.is_empty() {
            return out;
        }
        if n.pos.inflects() {
            for (a, al) in n.allos.iter().enumerate() {
                for (e, idx) in ending_surfaces(al.ctx) {
                    let comps = [al.comps.clone(), e.clone()].concat();
                    let edge = Edge {
                        start: 0,
                        end: comps.len(),
                        cost: 0.0,
                        kind: Kind::Verb { entry: i as u32, allo: a as u16, ending: idx as u16 },
                    };
                    let surface = compose(&comps);
                    for c in self.render(&comps, &edge) {
                        out.push((c.text, surface.clone()));
                    }
                }
            }
        } else {
            let base = Edge { start: 0, end: n.stem.len(), cost: 0.0, kind: Kind::Nominal { entry: i as u32, particle: None } };
            out.push((self.render(&n.stem, &base)[0].text.clone(), compose(&n.stem)));
            let pron = n.pos == Pos::Pron;
            let last = *n.stem.last().unwrap();
            for (pi, p) in self.particles.iter().enumerate() {
                if allowed(p.cond, last, pron) {
                    let comps = [n.stem.clone(), p.comps.clone()].concat();
                    let e = Edge { start: 0, end: comps.len(), cost: 0.0, kind: Kind::Nominal { entry: i as u32, particle: Some(pi as u16) } };
                    out.push((self.render(&comps, &e)[0].text.clone(), compose(&comps)));
                }
            }
        }
        out
    }

    /// All (written, surface, lemma) triples.
    pub fn written_forms(&self) -> Vec<(String, String, String)> {
        let mut out = Vec::new();
        for i in 0..self.natives.len() {
            for (w, s) in self.forms_of(i) {
                out.push((w, s, self.natives[i].lemma.clone()));
            }
        }
        out
    }
}

/// A stem ending in exactly one coda component (있, 그렇; not 읽, 않).
fn simple_coda(stem: &[char]) -> bool {
    let k = stem.iter().rev().take_while(|&&c| is_t(c)).count();
    k == 1
}

fn note(n: &Native, ending: Option<usize>) -> String {
    let src = match n.source {
        Source::User => "user",
        Source::Corpus => "corpus",
        Source::Community => "community",
        Source::Editorial => "editorial",
        Source::Gloss => "gloss",
        Source::Lexicon => "lexicon",
    };
    match ending {
        Some(e) => format!("{} {} +{} · 訓讀 {}", n.hanja, n.lemma, ending_label(e), src),
        None => format!("{} {} · 訓讀 {}", n.hanja, n.lemma, src),
    }
}

/// Split input into runs of Hangul syllables and everything else.
fn runs(input: &str) -> Vec<(bool, String)> {
    let mut out: Vec<(bool, String)> = Vec::new();
    for c in input.chars() {
        let h = is_syllable(c);
        match out.last_mut() {
            Some((k, s)) if *k == h => s.push(c),
            _ => out.push((h, c.to_string())),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATIVE: &str = "\
lemma\twritten\tpos\tclass\tsource\tcount
읽다\t讀다\tverb\t\tcommunity\t
보다\t見다\tverb\t\tcommunity\t
있다\t在ㅆ다\tadj\t\teditorial\t
있다\t有다\tadj\t\tcommunity\t
그렇다\t然다\tadj\t\tcorpus\t
다르다\t異다\tadj\t\tcommunity\t
어렵다\t難다\tadj\t\tcorpus\t
하다\t爲다\tverb\t\tcommunity\t
오다\t來다\tverb\t\tcommunity\t
마시다\t飮다\tverb\t\tcommunity\t
모르다\t䚳르다\tverb\t\tcommunity\t
알다\t知다\tverb\t\tcommunity\t
가깝다\t近다\tadj\t\tcommunity\t
하늘\t天\tnoun\t\tgloss\t
것\t事ㅅ\tbound\t\tcorpus\t
처음\t初음\tnoun\t\tcorpus\t
저\t自\tpron\t\tcorpus\t
나\t我\tpron\t\tcorpus\t
";
    const SINO: &str = "reading\thanja\tcount\tsource\n안녕\t安寧\t\t\n규칙\t規則\t\t\n석독회\t釋讀會\t\t\n환영\t歡迎\t\t\n";

    fn eng(o: Options) -> Engine {
        let (e, errs) = Engine::new(&[("t", NATIVE)], &[("t", SINO)], o);
        assert!(errs.is_empty(), "{errs:?}");
        e
    }

    fn first(e: &Engine, s: &str) -> String {
        e.convert(s)[0].text.clone()
    }

    fn any(e: &Engine, s: &str, want: &str) {
        let c: Vec<String> = e.segments(s).iter().flat_map(|g| g.candidates.iter().map(|c| c.text.clone())).collect();
        let w: Vec<String> = e.convert(s).iter().map(|c| c.text.clone()).collect();
        assert!(c.contains(&want.to_string()) || w.contains(&want.to_string()), "{s}: {want} not in {c:?} / {w:?}");
    }

    #[test]
    fn surface_style() {
        let e = eng(Options::default());
        assert_eq!(first(&e, "하늘을 봤다"), "天을 見ㅆ다");
        assert_eq!(first(&e, "읽었다"), "讀었다");
        assert_eq!(first(&e, "다른"), "異ㄴ");
        assert_eq!(first(&e, "달랐다"), "異ㅆ다");
        assert_eq!(first(&e, "가까웠다"), "近웠다");
        assert_eq!(first(&e, "몰랐다"), "䚳랐다");
        assert_eq!(first(&e, "아는"), "知는");
        assert_eq!(first(&e, "안녕하세요"), "安寧하세요");
        assert_eq!(first(&e, "규칙을"), "規則을");
        assert_eq!(first(&e, "처음"), "初음");
        assert_eq!(first(&e, "것을"), "事ㅅ을");
        assert_eq!(first(&e, "전"), "自ㄴ");
        assert_eq!(first(&e, "내가"), "我ㅣ가");
        assert_eq!(first(&e, "그런"), "然ㄴ");
        any(&e, "그렇네요", "然네요");
        any(&e, "있으면", "在ㅆ으면");
        any(&e, "마셔라", "飮라");
        any(&e, "마셔라", "飮ㅓ라");
        any(&e, "안녕해요", "安寧爲요");
    }

    #[test]
    fn morpheme_style() {
        let e = eng(Options { style: Style::Morpheme, ..Options::default() });
        assert_eq!(first(&e, "어려워요"), "難어요");
        assert_eq!(first(&e, "봤다"), "見았다");
        any(&e, "있어서", "在ㅆ어서");
    }

    #[test]
    fn resyllabified_coda() {
        let e = eng(Options { resyllabify: true, ..Options::default() });
        any(&e, "있으면", "在쓰면");
        any(&e, "있어서", "在써서");
    }

    #[test]
    fn hada_kun() {
        let e = eng(Options { hada_kun: true, ..Options::default() });
        assert_eq!(first(&e, "안녕해요"), "安寧爲요");
    }
}
