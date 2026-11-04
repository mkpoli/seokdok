//! The converter: a lattice over the jamo components of each Hangul run,
//! k-best search, and rendering of each analysis in the configured 末音添記
//! style.

use crate::conj::{ending_label, ending_surfaces, morph_ending, Ctx};
use crate::dict::{parse_native, parse_sino, tsv_lines, Native, Pos, Source, Tail};
use crate::jamo::{compose, decompose, is_l, is_syllable, is_t, is_v, resyllabify};
use crate::fold::{foldable, has_era_only, has_old, modern_mask, parse, readings, Reading, Syl, ALL, MAX_READINGS_MODERN, MAX_READINGS_OLD, STRONG};
use crate::old_spelling::{ends_in_hanja, is_function_old, is_jamo, keep_typed_tail, modernize_function, old_variant};
use crate::particles::{allowed, particles, Particle};
use std::collections::{HashMap, HashSet};

mod mixed;
mod okurigana;

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
    /// Put the old spelling of 하 (ᄒᆞ ᄒᆞᆫ ᄒᆞᆯ ᄒᆞᆷ ᄒᆞᆸ ᄒᆡ) first where it is
    /// offered, with the modern spelling right behind it. Without it the old
    /// spelling comes third.
    pub old_ha: bool,
    /// 國漢文混用: write only 音讀 words in Hanja and leave native words in
    /// Hangul. Native words still take part in segmentation, so 나라 stays
    /// 나라 instead of splitting into Sino syllables.
    pub sino_only: bool,
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
            old_ha: false,
            sino_only: false,
            max_candidates: 12,
        }
    }
}

const NONE: u32 = u32::MAX;

/// The honorific suffix 님 as conjoining jamo.
const NIM: [char; 3] = ['\u{1102}', '\u{1175}', '\u{11B7}'];

/// A prefix trie over jamo, flat from the start: each node is its key and
/// two links (first child, next sibling), with no allocation of its own; the
/// dictionaries hold hundreds of thousands of nodes. `freeze` gathers each
/// node's items into one contiguous slice.
struct Trie<T> {
    key: Vec<char>,
    first_child: Vec<u32>,
    next_sibling: Vec<u32>,
    /// While building: (node, item). After `freeze`: sorted by node.
    pending: Vec<(u32, T)>,
    /// Node n's items are items[item_start[n]..item_start[n + 1]].
    item_start: Vec<u32>,
    items: Vec<T>,
}

impl<T> Trie<T> {
    fn new() -> Self {
        Trie {
            key: vec!['\0'],
            first_child: vec![NONE],
            next_sibling: vec![NONE],
            pending: Vec::new(),
            item_start: Vec::new(),
            items: Vec::new(),
        }
    }

    fn child(&self, n: usize, c: char) -> Option<usize> {
        let mut i = self.first_child[n];
        while i != NONE {
            if self.key[i as usize] == c {
                return Some(i as usize);
            }
            i = self.next_sibling[i as usize];
        }
        None
    }

    fn insert(&mut self, key: &[char], item: T) {
        debug_assert!(self.item_start.is_empty(), "insert after freeze");
        let mut n = 0usize;
        for &c in key {
            n = match self.child(n, c) {
                Some(i) => i,
                None => {
                    let i = self.key.len();
                    self.key.push(c);
                    self.first_child.push(NONE);
                    self.next_sibling.push(self.first_child[n]);
                    self.first_child[n] = i as u32;
                    i
                }
            };
        }
        self.pending.push((n as u32, item));
    }

    fn freeze(&mut self) {
        let mut pending = std::mem::take(&mut self.pending);
        // Stable: items of a node keep their insertion order.
        pending.sort_by_key(|(n, _)| *n);
        let nodes = self.key.len();
        self.item_start = Vec::with_capacity(nodes + 1);
        self.items = Vec::with_capacity(pending.len());
        let mut it = pending.into_iter().peekable();
        for n in 0..nodes as u32 {
            self.item_start.push(self.items.len() as u32);
            while let Some((_, item)) = it.next_if(|(m, _)| *m == n) {
                self.items.push(item);
            }
        }
        self.item_start.push(self.items.len() as u32);
        self.key.shrink_to_fit();
        self.first_child.shrink_to_fit();
        self.next_sibling.shrink_to_fit();
    }

    fn node_items(&self, n: usize) -> &[T] {
        &self.items[self.item_start[n] as usize..self.item_start[n + 1] as usize]
    }

    /// Every (end position, items) for keys that are prefixes of `input[from..]`.
    fn prefixes<'a>(&'a self, input: &[char], from: usize) -> Vec<(usize, &'a [T])> {
        let mut out = Vec::new();
        let mut n = 0usize;
        let mut pos = from;
        loop {
            let items = self.node_items(n);
            if !items.is_empty() && pos > from {
                out.push((pos, items));
            }
            if pos >= input.len() {
                break;
            }
            match self.child(n, input[pos]) {
                Some(i) => {
                    n = i;
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

/// What surrounds the run being converted in the input.
#[derive(Clone, Copy, Debug)]
struct Around {
    /// The run is the only Hangul in the input.
    alone: bool,
    /// The next word of the input is a bound noun (때, 것, 수 …), which an
    /// adnominal verb form before it (불 때, 길 것) needs.
    bound_next: bool,
}

impl Default for Around {
    fn default() -> Self {
        Around { alone: true, bound_next: false }
    }
}

/// Bound nouns that follow an adnominal verb form.
const BOUND_NEXT: &[&str] = &["때", "것", "수", "줄", "데", "적", "바", "뿐", "듯", "리"];

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
    sinos: SinoTable,
    hun: HunTable,
    loans: HashSet<String>,
    pub options: Options,
    stems: Trie<(u32, u16)>,
    nominals: Trie<u32>,
    sino: Trie<u32>,
    endings: HashMap<Ctx, Trie<u16>>,
    /// Endings that leave no letters of their own after a stem: 가+아 → 가,
    /// 서+어 → 서, so 사 (주세요) is 사다 in its 아/어 form.
    zero_endings: HashMap<Ctx, Vec<u16>>,
    particles: Vec<Particle>,
    particle_trie: Trie<u16>,
    /// Reverse indexes for [`Engine::read`], built on first use.
    reverse: std::sync::OnceLock<Reverse>,
    /// Written forms of entries, kept by [`Engine::read`]: listing them costs
    /// more than the lookup (the second key is `variant_forms`). The forms
    /// depend on the spelling options, so the cache starts over when they change.
    forms_cache: std::sync::Mutex<(u8, HashMap<(usize, bool), std::sync::Arc<Vec<(String, String)>>>)>,
}

/// The Sino-Korean entries, packed: every reading and Hanja form lives in one
/// string, so an entry costs a dozen bytes and no allocation of its own.
#[derive(Default)]
struct SinoTable {
    text: String,
    entries: Vec<SinoEntry>,
}

struct SinoEntry {
    reading: (u32, u16),
    hanja: (u32, u16),
    cost: f32,
    /// Counts summed over the files, to break ties between the readings of
    /// one Hanja word (sino.tsv credits the chat count to each reading).
    count: u32,
}

impl SinoTable {
    fn push(&mut self, reading: &str, hanja: &str, cost: f32, count: u32) {
        let mut put = |s: &str| {
            let at = self.text.len() as u32;
            self.text.push_str(s);
            (at, s.len() as u16)
        };
        let (reading, hanja) = (put(reading), put(hanja));
        self.entries.push(SinoEntry { reading, hanja, cost, count });
    }
    fn str(&self, (at, len): (u32, u16)) -> &str {
        &self.text[at as usize..at as usize + len as usize]
    }
    fn reading(&self, i: usize) -> &str {
        self.str(self.entries[i].reading)
    }
    fn hanja(&self, i: usize) -> &str {
        self.str(self.entries[i].hanja)
    }
    fn cost(&self, i: usize) -> f32 {
        self.entries[i].cost
    }
    fn count(&self, i: usize) -> u32 {
        self.entries[i].count
    }
    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// 訓音 keys (살찔비 → 肥), sorted, for finding a character by its gloss. They
/// are looked up whole and stay out of the lattice and the reader.
#[derive(Default)]
struct HunTable {
    /// `key\thanja` rows, back to back.
    text: String,
    ends: Vec<u32>,
}

impl HunTable {
    fn new(mut rows: Vec<(String, String)>) -> HunTable {
        // Stable: a key's characters keep their order in the file.
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        let mut t = HunTable { text: String::with_capacity(rows.iter().map(|r| r.0.len() + r.1.len() + 1).sum()), ends: Vec::with_capacity(rows.len()) };
        for (key, hanja) in &rows {
            t.text.push_str(key);
            t.text.push('\t');
            t.text.push_str(hanja);
            t.ends.push(t.text.len() as u32);
        }
        t
    }
    fn row(&self, i: usize) -> (&str, &str) {
        let start = if i == 0 { 0 } else { self.ends[i - 1] as usize };
        self.text[start..self.ends[i] as usize].split_once('\t').unwrap_or_default()
    }
    fn get(&self, key: &str) -> Vec<&str> {
        let (mut lo, mut hi) = (0, self.ends.len());
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.row(mid).0 < key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        (lo..self.ends.len()).map(|i| self.row(i)).take_while(|r| r.0 == key).map(|r| r.1).collect()
    }
}

struct Reverse {
    /// Hanja word → (reading, cost) of its best Sino-Korean entry.
    sino: HashMap<String, (String, f32)>,
    /// Hanja string → native entries written with it, cheapest first.
    native: HashMap<String, Vec<usize>>,
    max_sino: usize,
    /// Native written keys that contain Hangul between characters.
    compounds: Vec<String>,
    /// Inflected forms of the verbs that make Sino-Korean verbs (하다, 되다
    /// …): 釋讀하면 reads 석독 + 하면.
    light: std::collections::HashSet<String>,
}

/// What a run converts to: the whole-run candidates, the segments, and the
/// cost of the best path when it uses a dictionary word.
struct Converted {
    whole: Vec<Candidate>,
    segments: Vec<Segment>,
    cost: Option<f32>,
    /// No raw Hangul is left in the best path.
    covered: bool,
    /// Every word of the best path is an entry with a count of at least `ATTESTED`.
    attested: bool,
    /// The old spelling of the best reading, and with `old_ha` the one that
    /// writes only the 하 rows that way (built when `full`).
    old: Option<String>,
    ha: Option<String>,
}

/// A modern reading of old input is kept when its best path costs at most
/// this much more than the best reading's.
const READING_MARGIN: f32 = 4.0;

/// The count an entry needs for its reading to go first for an era-only syllable
/// (the counts are the chat's, or FusionIME's frequencies where the chat has none).
const ATTESTED: u32 = 3;

/// How many era readings are added behind the candidates of a run.
const ERA_EXTRA: usize = 8;

const RAW_COST: f32 = 4.5;
/// The ending index of a bare stem (않 → 不ㅎ), typed alone to be continued
/// in Hangul.
const BARE: u16 = u16::MAX;
/// Extra cost for one-syllable words, which are the most ambiguous.
const SHORT_COST: f32 = 2.5;
const EDGE_COST: f32 = 1.0;
/// Added to a bare adnominal verb form that a whole noun also spells.
const WHOLE_NOUN_PENALTY: f32 = 2.5;
/// How close a noun must come to a bound noun with a particle (거+의) to replace it.
const BOUND_MARGIN: f32 = 0.5;
/// Endings that run on into a bound noun or conjunction, as the final morpheme's label.
const TAIL_ENDINGS: &[&str] = &[
    "듯", "는듯", "다란", "는거", "는건", "는것", "고는", "고도", "고만", "냐고", "지마", "지만요", "(으)ㄴ듯", "(으)ㄴ거",
    "(으)ㄴ건", "(으)ㄴ것", "(으)ㄹ지도", "(으)ㄹ수도", "(으)ㄹ때", "(으)나", "(으)려나",
];
/// Added to a pronoun plus particle that an editorial whole word also spells.
const EDITORIAL_WORD_MARGIN: f32 = 2.5;
/// A loanword kept in Hangul (loanwords.tsv); cheaper than its Hanja homophones.
const LOAN_COST: f32 = 2.0;
/// A particle typed alone, kept in Hangul.
const ALONE_COST: f32 = 0.5;
/// Particles that are also words (이 this, 나 I, 보다 to see, 들 field, 같이, 대로, 만큼, 들은 heard) and 이에요, which the editors write 乃요, and 이를, 이는, 이가, 이도, 이의, 이에게, which are the pronoun 이 and a particle: typed alone they convert as words.
const HOMOGRAPH_PARTICLES: &[&str] = &["이", "나", "보다", "들", "같이", "대로", "만큼", "들은", "들을", "보다는", "이에요", "이를", "이는", "이가", "이도", "이의", "이에게"];
/// Added to verb forms in the archaic -니라.
const ARCHAIC_ENDING: f32 = 1.0;

/// Built-in entries: Hangul-only light verb 하다, so 安寧하세요 is the
/// default and 爲 comes from the dictionary as an alternative.
const BUILTIN: &str = "하다\t하다\tverb\t\teditorial\t\n";

/// Collects dictionary files one at a time, then builds the [`Engine`].
pub struct Builder {
    natives: Vec<Native>,
    sinos: SinoTable,
    hun: Vec<(String, String)>,
    /// Loanwords (loanwords.tsv), which stay in Hangul.
    loans: HashSet<String>,
    /// Hash of a (reading, Hanja) pair → its entry; the Sino files overlap
    /// (fusion.tsv and sino.tsv share most words).
    index: HashMap<u64, u32>,
    errors: Vec<String>,
}

impl Builder {
    pub fn new() -> Builder {
        let mut b = Builder { natives: Vec::new(), sinos: SinoTable::default(), hun: Vec::new(), loans: HashSet::new(), index: HashMap::new(), errors: Vec::new() };
        b.native("builtin", BUILTIN);
        b
    }

    /// Add a 訓讀 dictionary (`native.tsv` format).
    pub fn native(&mut self, label: &str, text: &str) {
        for (line, f) in tsv_lines(text) {
            match parse_native(&f) {
                Ok(mut e) => {
                    e.origin = (label.to_string(), line);
                    self.natives.push(e)
                }
                Err(m) => self.errors.push(format!("{label}:{line}: {m}")),
            }
        }
    }

    /// Add a 音讀 dictionary (`sino.tsv` format). A pair already present
    /// keeps its first entry, with the lowest cost any file gives it.
    pub fn sino(&mut self, label: &str, text: &str) {
        use std::hash::{BuildHasher, Hash, Hasher};
        for (line, f) in tsv_lines(text) {
            match parse_sino(&f) {
                Ok(e) if f.get(3) == Some(&"hun") => self.hun.push((e.reading.into(), e.hanja.into())),
                Ok(e) if f.get(3) == Some(&"loanword") => {
                    self.loans.insert(e.reading.into());
                }
                Ok(e) => {
                    let mut h = self.index.hasher().build_hasher();
                    (&*e.reading, &*e.hanja).hash(&mut h);
                    let key = h.finish();
                    match self.index.get(&key).map(|&i| i as usize) {
                        Some(i) if self.sinos.reading(i) == &*e.reading && self.sinos.hanja(i) == &*e.hanja => {
                            let kept = &mut self.sinos.entries[i];
                            kept.cost = kept.cost.min(e.cost);
                            kept.count = kept.count.saturating_add(e.count);
                        }
                        _ => {
                            self.index.entry(key).or_insert(self.sinos.len() as u32);
                            self.sinos.push(&e.reading, &e.hanja, e.cost, e.count);
                        }
                    }
                }
                Err(m) => self.errors.push(format!("{label}:{line}: {m}")),
            }
        }
    }

    /// The engine and the parse errors (file label, line, message).
    pub fn build(self, options: Options) -> (Engine, Vec<String>) {
        let Builder { natives, mut sinos, hun, loans, index, errors } = self;
        drop(index);
        sinos.text.shrink_to_fit();
        sinos.entries.shrink_to_fit();
        let mut eng = Engine {
            natives,
            sinos,
            hun: HunTable::new(hun),
            loans,
            options,
            stems: Trie::new(),
            nominals: Trie::new(),
            sino: Trie::new(),
            endings: HashMap::new(),
            zero_endings: HashMap::new(),
            particles: particles(),
            particle_trie: Trie::new(),
            reverse: std::sync::OnceLock::new(),
            forms_cache: Default::default(),
        };
        eng.index();
        (eng, errors)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Build an engine from TSV texts. Returns the engine and the parse
    /// errors (file label, line, message), which are skipped.
    pub fn new(native_tsvs: &[(&str, &str)], sino_tsvs: &[(&str, &str)], options: Options) -> (Engine, Vec<String>) {
        let mut b = Builder::new();
        for (label, text) in native_tsvs {
            b.native(label, text);
        }
        for (label, text) in sino_tsvs {
            b.sino(label, text);
        }
        b.build(options)
    }

    /// Native dictionaries loaded by [`Engine::from_dir`], in order.
    pub const NATIVE_FILES: &'static [&'static str] = &["user.tsv", "corpus.tsv", "editorial.tsv", "community.tsv", "suggested.tsv", "gloss.tsv", "kun-ja.tsv", "kun-ja-weak.tsv"];
    pub const SINO_FILES: &'static [&'static str] = &["user-sino.tsv", "editorial-sino.tsv", "fusion.tsv", "sino.tsv", "etym.tsv", "loanwords.tsv", "hun.tsv"];

    /// Load every known dictionary file present in `dir`.
    /// One file is read at a time and dropped once parsed, which keeps the
    /// peak low (the iOS keyboard extension has little memory).
    pub fn from_dir(dir: &std::path::Path, options: Options) -> std::io::Result<(Engine, Vec<String>)> {
        let mut b = Builder::new();
        for n in Self::NATIVE_FILES {
            let p = dir.join(n);
            if p.exists() {
                b.native(n, &std::fs::read_to_string(&p)?);
            }
        }
        for n in Self::SINO_FILES {
            let p = dir.join(n);
            if p.exists() {
                b.sino(n, &std::fs::read_to_string(&p)?);
            }
        }
        Ok(b.build(options))
    }

    fn index(&mut self) {
        for (i, n) in self.natives.iter().enumerate() {
            if n.pos.inflects() {
                for (a, al) in n.allos.iter().enumerate() {
                    self.stems.insert(&al.comps, (i as u32, a as u16));
                    if !self.endings.contains_key(&al.ctx) {
                        let mut t = Trie::new();
                        let mut zero = Vec::new();
                        for (comps, idx) in ending_surfaces(al.ctx) {
                            if comps.is_empty() {
                                zero.push(idx as u16);
                            } else {
                                t.insert(&comps, idx as u16);
                            }
                        }
                        t.freeze();
                        self.endings.insert(al.ctx, t);
                        self.zero_endings.insert(al.ctx, zero);
                    }
                }
            } else {
                self.nominals.insert(&n.stem, i as u32);
            }
        }
        for i in 0..self.sinos.len() {
            self.sino.insert(&decompose(self.sinos.reading(i)), i as u32);
        }
        for (i, p) in self.particles.iter().enumerate() {
            self.particle_trie.insert(&p.comps, i as u16);
        }
        self.stems.freeze();
        self.nominals.freeze();
        self.sino.freeze();
        self.particle_trie.freeze();
    }

    fn native_cost(&self, n: &Native) -> f32 {
        // Hangul-only helper entries (the light verb 하다) are cheap.
        let mut c = if n.hanja.is_empty() { 0.5 } else { n.cost };
        if n.lemma == "하다" {
            if n.hanja.is_empty() && self.options.hada_kun {
                c += 5.0;
            } else if !n.hanja.is_empty() && !self.options.hada_kun {
                c += 2.0;
            }
        }
        c
    }

    fn edges(&self, comps: &[char], around: Around) -> Vec<Edge> {
        let n = comps.len();
        let boundary = |p: usize| p == n || is_l(comps[p]);
        // Particles and endings close a word: inside a run they are misparses.
        let mid = |r: usize| if r == n { 0.0 } else { 8.0 };
        // Short copula forms after a vowel (라, 다, 고, 야 …) are as often the
        // end of an unknown word (나라, 바다) as a particle.
        let weak = |p: &Particle| {
            let short = crate::jamo::syllable_count(&p.comps) == 1;
            let common = matches!(compose(&p.comps).as_str(), "가" | "를" | "는" | "와" | "도" | "만" | "의" | "에" | "로" | "랑" | "나" | "요");
            if p.cond == crate::particles::Cond::Vowel && short && !common { 2.5 } else { 0.0 }
        };
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
            // The whole run left in Hangul, cheaper than syllable by syllable.
            if s == 0 && crate::jamo::syllable_count(comps) >= 2 {
                let k = crate::jamo::syllable_count(comps) as f32;
                edges.push(Edge { start: 0, end: n, cost: RAW_COST + 2.5 * (k - 1.0), kind: Kind::Raw });
            }
            // An unknown word left in Hangul, followed by a particle (따름+이니라).
            if s == 0 {
                for q in (1..n).filter(|&q| is_l(comps[q])) {
                    for (r, pis) in self.particle_trie.prefixes(comps, q) {
                        if r != n {
                            continue;
                        }
                        let ok = pis.iter().any(|&pi| {
                            let p = &self.particles[pi as usize];
                            allowed(p.cond, comps[q - 1], false) && is_l(p.comps[0])
                        });
                        // A single syllable before a copula (나+이다) is more often a
                        // word that ends in it (나이+다) than an unknown word.
                        let lone_copula = crate::jamo::syllable_count(&comps[..q]) == 1 && pis.iter().all(|&pi| self.particles[pi as usize].full_copula());
                        if ok {
                            let k = crate::jamo::syllable_count(&comps[..q]) as f32;
                            edges.push(Edge { start: 0, end: n, cost: RAW_COST + 2.5 * (k - 1.0) + 0.4 + if lone_copula { 2.5 } else { 0.0 }, kind: Kind::Raw });
                        }
                    }
                }
            }
            // Sino-Korean words, optionally with a particle.
            for (q, items) in self.sino.prefixes(comps, s) {
                if !boundary(q) {
                    continue;
                }
                for &i in items {
                    let short = if crate::jamo::syllable_count(&comps[s..q]) == 1 { SHORT_COST } else { 0.0 };
                    let base = EDGE_COST + self.sinos.cost(i as usize) + short;
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
                                    cost: base + 0.2 + mid(r) + weak(p),
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
                            Source::Gloss | Source::Japanese | Source::JapaneseWeak | Source::Lexicon | Source::Suggested => SHORT_COST,
                            Source::Community => 1.0,
                            _ => 0.0,
                        }
                    };
                    let base = EDGE_COST + self.native_cost(entry) + short;
                    if boundary(q) {
                        // A bare native word inside a longer run is usually a misparse
                        // of an unknown word (나라 ≠ 吾+라).
                        // The honorific suffix 님 attaches to any noun (해님, 달님), and
                        // a determiner always precedes one (옛한글, 첫사랑).
                        // The bound noun 수 runs straight into 있다 and 없다 when typed
                        // without a space (수있다, 볼수없었던).
                        let inside = if q == n || comps[q..].starts_with(&NIM) || entry.pos == Pos::Det || is_su_before_existence(entry, &comps[q..]) {
                            0.0
                        } else if crate::jamo::syllable_count(&entry.stem) == 1 {
                            5.0
                        } else {
                            3.0
                        };
                        edges.push(Edge { start: s, end: q, cost: base + inside, kind: Kind::Nominal { entry: i, particle: None } });
                    }
                    // Pronouns and bound nouns contract with a following particle:
                    // 저는 → 전, 거+이 → 게, 할 수는 → 할 순.
                    // Interjections stand alone: 안 不 takes no particle, 안에 is 內에.
                    if entry.pos == Pos::Interj {
                        continue;
                    }
                    let pron = matches!(entry.pos, Pos::Pron | Pos::Bound);
                    for (r, pis) in self.particle_trie.prefixes(comps, q) {
                        if !boundary(r) {
                            continue;
                        }
                        for &pi in pis {
                            let p = &self.particles[pi as usize];
                            // The copula follows nouns, pronouns and numbers, not
                            // adverbs (아니+라니 is 아니다+라니; 무어라 and 무어+ㄴ가 are
                            // interjections that take it), and a one-syllable
                            // pronoun with 이다 is rarer than a word that ends
                            // in it (나이+다).
                            if p.copular && !is_t(p.comps[0]) && entry.pos == Pos::Adv {
                                continue;
                            }
                            let lone_pron = if p.full_copula() && entry.pos == Pos::Pron && crate::jamo::syllable_count(&entry.stem) == 1 { 5.0 } else { 0.0 };
                            if allowed(p.cond, comps[q - 1], pron) {
                                edges.push(Edge {
                                    start: s,
                                    end: r,
                                    cost: base + 0.2 + mid(r) + weak(p) + lone_pron,
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
                    // A bare stem at the end of the input: the stem as the
                    // inflected forms write it, with the ending still to come.
                    if q == n && entry.allos[a as usize].comps == entry.stem {
                        edges.push(Edge {
                            start: s,
                            end: q,
                            cost: EDGE_COST + self.native_cost(entry) + 1.0,
                            kind: Kind::Verb { entry: i, allo: a, ending: BARE },
                        });
                    }
                    if boundary(q) {
                        for &ei in self.zero_endings.get(&ctx).into_iter().flatten() {
                            edges.push(Edge {
                                start: s,
                                end: q,
                                cost: EDGE_COST + self.native_cost(entry) + 0.1 + mid(q),
                                kind: Kind::Verb { entry: i, allo: a, ending: ei },
                            });
                        }
                    }
                    for (r, eis) in et.prefixes(comps, q) {
                        if !boundary(r) {
                            continue;
                        }
                        for &ei in eis {
                            let label = ending_label(ei as usize);
                            let morphs = label.matches('+').count() as f32;
                            // An archaic ending (알+니라) is rare next to a word that
                            // ends in the same letters (아니+라).
                            let archaic = if matches!(label, "(으)니라" | "니라") { ARCHAIC_ENDING } else { 0.0 };
                            edges.push(Edge {
                                start: s,
                                end: r,
                                cost: EDGE_COST + self.native_cost(entry) + 0.1 + 0.8 * morphs + mid(r) + archaic,
                                kind: Kind::Verb { entry: i, allo: a, ending: ei },
                            });
                        }
                    }
                }
            }
        }
        if around.alone {
            self.alone_edges(comps, &mut edges);
        }
        self.loan_edges(comps, &mut edges);
        if !around.bound_next {
            self.prefer_whole_noun(comps, &mut edges);
        }
        self.prefer_editorial_word(comps, &mut edges);
        self.prefer_word_over_tail(comps, &mut edges);
        edges
    }

    /// Endings that end in a bound noun or a conjunction (는거, ㄴ듯, 고도, ㄹ때,
    /// 나) turn up inside words: 근거 is 根據, not 書+ㄴ거, and 바다란 is 海+란.
    /// A whole token that a dictionary noun or Sino-Korean word spells, alone or
    /// with a particle, keeps its place over those verb forms.
    fn prefer_word_over_tail(&self, comps: &[char], edges: &mut [Edge]) {
        let n = comps.len();
        if n == 0 || crate::jamo::syllable_count(comps) > 3 {
            return;
        }
        let word_cost = edges
            .iter()
            .filter(|e| e.start == 0 && e.end == n)
            .filter(|e| match e.kind {
                Kind::Nominal { entry, .. } => {
                    let nat = &self.natives[entry as usize];
                    matches!(nat.source, Source::Community | Source::Corpus | Source::Editorial | Source::Suggested) && matches!(nat.pos, Pos::Noun | Pos::Pron | Pos::Num | Pos::Adv)
                }
                Kind::Sino { entry, .. } => self.sinos.count(entry as usize) > 0,
                _ => false,
            })
            .map(|e| e.cost)
            .fold(f32::INFINITY, f32::min);
        for e in edges.iter_mut().filter(|e| e.start == 0 && e.end == n) {
            if let Kind::Verb { ending, .. } = e.kind {
                if ending != BARE && word_cost < e.cost + WHOLE_NOUN_PENALTY {
                    let label = ending_label(ending as usize);
                    if TAIL_ENDINGS.contains(&label.rsplit('+').next().unwrap_or("")) {
                        e.cost += WHOLE_NOUN_PENALTY;
                    }
                }
            }
        }
    }

    /// A whole token that an editor entered as one word (저들 彼等, 저만큼 彼만큼)
    /// beats the same letters read as a pronoun and a particle (自+들, 自+만큼),
    /// which the pronoun's chat count would otherwise win.
    fn prefer_editorial_word(&self, comps: &[char], edges: &mut [Edge]) {
        let n = comps.len();
        let word_cost = edges
            .iter()
            .filter(|e| e.start == 0 && e.end == n)
            .filter_map(|e| match e.kind {
                Kind::Nominal { entry, particle: None } if self.natives[entry as usize].source == Source::Editorial => Some(e.cost),
                _ => None,
            })
            .fold(f32::INFINITY, f32::min);
        for e in edges.iter_mut().filter(|e| e.start == 0 && e.end == n) {
            if let Kind::Nominal { entry, particle: Some(_) } = e.kind {
                let nat = &self.natives[entry as usize];
                if nat.pos == Pos::Pron && nat.source != Source::Editorial && word_cost < e.cost + EDITORIAL_WORD_MARGIN {
                    e.cost += EDITORIAL_WORD_MARGIN;
                }
            }
        }
    }

    /// A particle or copula form typed on its own (에서, 까지, 는, 입니다) stays
    /// Hangul, except the particles that are also words of their own.
    fn alone_edges(&self, comps: &[char], edges: &mut Vec<Edge>) {
        let n = comps.len();
        if n == 0 || !is_l(comps[0]) {
            return;
        }
        let alone = self.particle_trie.prefixes(comps, 0).into_iter().any(|(r, pis)| r == n && pis.iter().any(|&pi| is_l(self.particles[pi as usize].comps[0])));
        if alone && !HOMOGRAPH_PARTICLES.contains(&compose(comps).as_str()) {
            edges.push(Edge { start: 0, end: n, cost: ALONE_COST, kind: Kind::Raw });
        }
    }

    /// A loanword (페이지, 파일, 피자), alone or with a particle or the light
    /// verb 하다, is kept in Hangul: it has no Hanja, and its syllables are not
    /// Sino-Korean words.
    fn loan_edges(&self, comps: &[char], edges: &mut Vec<Edge>) {
        let n = comps.len();
        if self.loans.is_empty() || n == 0 || !is_l(comps[0]) {
            return;
        }
        for q in (1..=n).filter(|&q| q == n || is_l(comps[q])) {
            if crate::jamo::syllable_count(&comps[..q]) < 2 || !self.loans.contains(&compose(&comps[..q])) {
                continue;
            }
            if q == n {
                edges.push(Edge { start: 0, end: n, cost: LOAN_COST, kind: Kind::Raw });
                continue;
            }
            // 로그인합니다, 빌드하면: the loanword with the light verb 하다 after it.
            if comps[q] == '\u{1112}' && matches!(comps.get(q + 1), Some('\u{1161}' | '\u{1162}')) {
                edges.push(Edge { start: 0, end: q, cost: LOAN_COST, kind: Kind::Raw });
            }
            for (r, pis) in self.particle_trie.prefixes(comps, q) {
                let ok = r == n && pis.iter().any(|&pi| {
                    let p = &self.particles[pi as usize];
                    allowed(p.cond, comps[q - 1], false) && is_l(p.comps[0])
                });
                if ok {
                    edges.push(Edge { start: 0, end: n, cost: LOAN_COST + 0.2, kind: Kind::Raw });
                }
            }
        }
    }

    /// A whole one- or two-syllable token that is a noun of the community,
    /// corpus or editorial dictionaries, or a chat-attested Sino-Korean word,
    /// is far more often that noun than a bare adnominal, nominalised or
    /// imperative verb form: 눈 is 目, not 누+ㄴ, 나라 is 國, not 나+아라, 길
    /// is 道, not 길+ㄹ. Longer tokens and endings with letters of their own
    /// (작은, 남은, 마실 …) are left alone.
    fn prefer_whole_noun(&self, comps: &[char], edges: &mut [Edge]) {
        let n = comps.len();
        if n == 0 || crate::jamo::syllable_count(comps) > 2 {
            return;
        }
        // The cheapest whole-token noun, and the cheapest that is not a bound noun.
        let (mut noun_cost, mut lexical_cost) = (f32::INFINITY, f32::INFINITY);
        for e in edges.iter().filter(|e| e.start == 0 && e.end == n) {
            if let Kind::Nominal { entry, particle: None } = e.kind {
                let nat = &self.natives[entry as usize];
                if matches!(nat.source, Source::Community | Source::Corpus | Source::Editorial) && matches!(nat.pos, Pos::Noun | Pos::Pron | Pos::Num | Pos::Bound | Pos::Adv) {
                    noun_cost = noun_cost.min(e.cost);
                    if nat.pos != Pos::Bound {
                        lexical_cost = lexical_cost.min(e.cost);
                    }
                }
            }
        }
        for e in edges.iter_mut().filter(|e| e.start == 0 && e.end == n) {
            // Only where the penalty can change the outcome: a verb form far
            // cheaper than the noun (긴, 빈, 간) stays first.
            let penalise = match e.kind {
                Kind::Verb { ending, .. } => {
                    ending != BARE
                        && noun_cost < e.cost + WHOLE_NOUN_PENALTY
                        && matches!(ending_label(ending as usize), "(으)ㄴ" | "(으)ㄹ" | "(으)ㅁ" | "아/어라" | "기" | "진" | "소" | "(으)니라")
                }
                // A bound noun never opens a phrase on its own (거+의, 수+ㄹ):
                // 거의 is the adverb 殆, 술 is 酒.
                Kind::Nominal { entry, particle: Some(_) } => self.natives[entry as usize].pos == Pos::Bound && lexical_cost < e.cost + BOUND_MARGIN,
                _ => false,
            };
            if penalise {
                e.cost += WHOLE_NOUN_PENALTY;
            }
        }
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

    /// The dictionary's written forms of an edge, then the other 末音添記
    /// choices for the same word (see [`Engine::okurigana`]).
    fn render(&self, comps: &[char], e: &Edge) -> Vec<Candidate> {
        let mut out = self.render_dict(comps, e);
        if !self.options.sino_only {
            let more = self.okurigana(comps, e, &out, true);
            out.extend(more);
        }
        out
    }

    fn render_dict(&self, comps: &[char], e: &Edge) -> Vec<Candidate> {
        self.render_dict_as(comps, e, !self.options.sino_only)
    }

    /// [`Engine::render_dict`], with 訓讀 words in Hanja (`kun`) or in Hangul.
    /// The reader needs the Hanja whatever the conversion mode.
    fn render_dict_as(&self, comps: &[char], e: &Edge, kun: bool) -> Vec<Candidate> {
        let slice = &comps[e.start..e.end];
        let raw = || Candidate { text: compose(slice), note: String::new() };
        match &e.kind {
            Kind::Raw => vec![raw()],
            Kind::Sino { entry, particle } => {
                let (hanja, reading) = (self.sinos.hanja(*entry as usize), self.sinos.reading(*entry as usize));
                let mut t = hanja.to_string();
                if let Some(p) = particle {
                    t.push_str(&compose(&self.particles[*p as usize].comps));
                }
                vec![Candidate { text: t, note: format!("{hanja} {reading} · 音讀") }]
            }
            Kind::Nominal { entry, particle } => {
                let n = &self.natives[*entry as usize];
                if n.hanja.is_empty() || !kun {
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
                if n.hanja.is_empty() || !kun {
                    return vec![raw()];
                }
                let al = &n.allos[*allo as usize];
                let tail = &comps[e.start + al.comps.len()..e.end];
                let o = &self.options;
                let mut out: Vec<String> = Vec::new();
                let rs = o.resyllabify;
                let surf = |frag: Fragment, coda: bool| self.surface(n, &al.comps, tail, frag, coda, rs);
                let morph = |coda: bool| if *ending == BARE { None } else { self.morpheme(n, *ending as usize, coda, rs) };
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
                // The other resyllabification last, so the reader knows it too.
                let other_rs = [
                    Some(self.surface(n, &al.comps, tail, o.fragment, o.coda_always, !rs)),
                    Some(self.surface(n, &al.comps, tail, o.fragment, !o.coda_always, !rs)),
                    if *ending == BARE { None } else { self.morpheme(n, *ending as usize, o.coda_always, !rs) },
                ];
                for t in order.into_iter().chain(other_rs).flatten() {
                    if !out.contains(&t) {
                        out.push(t);
                    }
                }
                let nt = note(n, (*ending != BARE).then_some(*ending as usize));
                out.into_iter().map(|text| Candidate { text, note: nt.clone() }).collect()
            }
        }
    }

    /// Surface style: the Hanja stand for the stem as pronounced; what follows
    /// is the rest of the pronounced word.
    fn surface(&self, n: &Native, allo: &[char], ending: &[char], frag: Fragment, coda_always: bool, resyll: bool) -> String {
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
        let rest = if written_coda && resyll { resyllabify(&rest) } else { rest };
        format!("{}{}", n.hanja, compose(&rest))
    }

    /// Morpheme style: the ending written as attached to the lemma's
    /// underlying final, without contraction.
    fn morpheme(&self, n: &Native, ending: usize, coda_always: bool, resyll: bool) -> Option<String> {
        let e = morph_ending(n.morph?, ending)?;
        // ㄹ is not pronounced before ᄂ ᄇ ᄉ or a coda ending (아는, 압니다),
        // and no written consonant goes before a coda ending.
        let blocked = match e.first() {
            Some(&c) if is_t(c) => true,
            Some('ᄂ' | 'ᄇ' | 'ᄉ') => n.stem.last() == Some(&'ᆯ'),
            _ => false,
        };
        let mut written_coda = false;
        let prefix: Vec<char> = match n.tail {
            _ if blocked => Vec::new(),
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
        let rest = if written_coda && resyll { resyllabify(&rest) } else { rest };
        Some(format!("{}{}", n.hanja, compose(&rest)))
    }

    /// Whole-run candidates and segments of one run of Hangul. A run with old
    /// letters is looked up by its modern readings; one that conversion left
    /// as it was may be an era spelling.
    fn convert_run(&self, run: &str, after_hanja: bool, around: Around) -> (Vec<Candidate>, Vec<Segment>) {
        if has_old(run) {
            return self.convert_old(run);
        }
        let Converted { mut whole, mut segments, .. } = self.convert_plain(run, after_hanja, true, around);
        // Spellings of the era: the strong classes whenever the run shows
        // one, the others when it was left as typed.
        let unconverted = whole.first().is_some_and(|c| c.text == run);
        let mask = modern_mask(run, unconverted);
        if mask != 0 {
            if let Some(syls) = parse(run) {
                let found = self.read_as(&syls, mask, true, MAX_READINGS_MODERN);
                // One candidate of each reading in turn, so that the likely
                // readings all show before the long tail of one. They add to
                // the list: every modern candidate stays.
                let mut era: Vec<Candidate> = Vec::new();
                for k in 0..found.iter().map(|f| f.2.len()).max().unwrap_or(0) {
                    for (_, reading, cands, _) in &found {
                        if let Some(c) = cands.get(k) {
                            if c.text != *reading && !era.iter().any(|x| x.text == c.text) && !whole.iter().any(|x| x.text == c.text) {
                                era.push(c.clone());
                            }
                        }
                    }
                }
                era.truncate(ERA_EXTRA);
                if !era.is_empty() {
                    // A syllable that modern words do not use puts the best
                    // reading first, then what was typed: when the reading is
                    // two syllables or more and made of whole entries the
                    // chat uses. Otherwise the readings come behind it.
                    let first = has_era_only(run)
                        && found.first().is_some_and(|(_, r, c, attested)| *attested && r.chars().count() >= 2 && c.first().is_some_and(|c| c.text == era[0].text && c.text.chars().any(crate::dict::is_han)));
                    let lead = if first { Some(era.remove(0)) } else { None };
                    let place = |list: &mut Vec<Candidate>| {
                        list.splice(1.min(list.len())..1.min(list.len()), era.clone());
                        if let Some(l) = &lead {
                            list.insert(0, l.clone());
                        }
                    };
                    place(&mut whole);
                    match segments.as_mut_slice() {
                        [one] => place(&mut one.candidates),
                        _ => {
                            segments = vec![Segment { reading: run.to_string(), candidates: whole.clone() }];
                        }
                    }
                }
            }
        }
        (whole, segments)
    }

    /// A run with old letters: what its modern readings convert to, with the
    /// tail as typed when it is made of function syllables, and the run itself
    /// kept as a choice.
    fn convert_old(&self, run: &str) -> (Vec<Candidate>, Vec<Segment>) {
        let syls = parse(run).unwrap_or_default();
        let found = self.read_as(&syls, ALL, false, MAX_READINGS_OLD);
        // The old spelling of the best reading, as for modern input.
        let (old, ha) = match found.first() {
            Some((_, reading, _, _)) => {
                let c = self.convert_plain(reading, false, true, Around::default());
                (c.old, c.ha)
            }
            None => (None, None),
        };
        let restored = |reading: &str, cands: Vec<Candidate>| -> Vec<Candidate> {
            cands
                .into_iter()
                .map(|c| Candidate { text: keep_typed_tail(&c.text, reading, &syls).unwrap_or(c.text), note: c.note })
                .collect()
        };
        let mut list: Vec<Candidate> = Vec::new();
        for (_, reading, cands, _) in found {
            for c in restored(&reading, cands) {
                if !list.iter().any(|x| x.text == c.text) {
                    list.push(c);
                }
            }
        }
        // Old letters that are all function syllables stay as typed: the
        // candidates that keep them come first.
        let old_syls: Vec<&str> = syls.iter().map(|s| s.typed.as_str()).filter(|t| !t.chars().all(is_syllable)).collect();
        let mut at = list.len().min(1);
        // A syllable that modern words do not use puts its reading first.
        if !old_syls.is_empty() && old_syls.iter().all(|o| is_function_old(o)) && !has_era_only(run) {
            let (kept, rest): (Vec<Candidate>, Vec<Candidate>) = list.into_iter().partition(|c| old_syls.iter().any(|o| c.text.contains(o)));
            at = kept.len().min(1);
            list = kept;
            list.extend(rest);
        }
        let typed = Candidate { text: run.to_string(), note: "한글".into() };
        if !list.iter().any(|c| c.text == run) {
            list.insert(at, typed.clone());
        }
        offer_old(&mut list, old);
        first_old(&mut list, ha);
        let keep = self.options.max_candidates.max(2);
        if list.len() > keep {
            let had = list[..keep].iter().any(|c| c.text == run);
            list.truncate(keep);
            if !had {
                list[keep - 1] = typed;
            }
        }
        let segments = vec![Segment { reading: run.to_string(), candidates: list.clone() }];
        (list, segments)
    }

    /// The conversions of the modern readings of a run, best path first, only
    /// those that use a dictionary word and cost little more than the best.
    fn read_as(&self, syls: &[Syl], mask: u16, strict: bool, cap: usize) -> Vec<(f32, String, Vec<Candidate>, bool)> {
        let mut found: Vec<(f32, String, Vec<Candidate>, bool)> = Vec::new();
        for r in readings(syls, mask, strict, cap) {
            if strict && !self.worth_converting(syls, &r) {
                continue;
            }
            let c = self.convert_plain(&r.text, false, false, Around::default());
            // A guess has to be confirmed by words that cover all of it.
            if let Some(cost) = c.cost.filter(|_| c.covered || !r.optional) {
                found.push((cost + r.penalty, r.text, c.whole, c.attested));
            }
        }
        found.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        if let Some(best) = found.first().map(|f| f.0) {
            found.retain(|f| f.0 <= best + READING_MARGIN);
        }
        found
    }

    /// A reading that guesses with a weak class is converted only if some
    /// dictionary word covers the first syllable it changed (a stem, a noun or
    /// a Sino-Korean word; what follows may be an ending or a particle). This
    /// keeps the lookups of unconverted words cheap.
    fn worth_converting(&self, syls: &[Syl], r: &Reading) -> bool {
        if r.classes & !STRONG == 0 {
            return true;
        }
        let Some(first) = r.text.chars().zip(syls).position(|(c, s)| s.typed.chars().next() != Some(c)) else { return true };
        let comps = decompose(&r.text);
        let mut starts = vec![0usize];
        for c in r.text.chars() {
            starts.push(starts.last().unwrap() + decompose(&c.to_string()).len());
        }
        let end = starts[first + 1];
        (0..=first).any(|s| {
            let from = starts[s];
            self.nominals.prefixes(&comps, from).iter().any(|(q, _)| *q >= end)
                || self.sino.prefixes(&comps, from).iter().any(|(q, _)| *q >= end)
                || self.stems.prefixes(&comps, from).iter().any(|(q, _)| *q >= end)
        })
    }

    /// `full`: also build the segments and the old spellings, which the
    /// readings tried for another run do not need.
    fn convert_plain(&self, run: &str, after_hanja: bool, full: bool, around: Around) -> Converted {
        let comps = decompose(run);
        let edges = self.edges(&comps, around);
        let paths = self.kbest(&comps, &edges, self.options.max_candidates.max(4));
        let mut whole: Vec<Candidate> = Vec::new();
        let push = |c: Candidate, whole: &mut Vec<Candidate>| {
            if !whole.iter().any(|w| w.text == c.text) {
                whole.push(c);
            }
        };
        // The best path's renderings are used three times.
        let best: Vec<Vec<Candidate>> = paths.first().map(|(_, path)| path.iter().map(|&ei| self.render(&comps, &edges[ei])).collect()).unwrap_or_default();
        for (n, (_, path)) in paths.iter().enumerate() {
            let mut text = String::new();
            let mut notes = Vec::new();
            for (k, &ei) in path.iter().enumerate() {
                let rendered;
                let r = if n == 0 {
                    &best[k]
                } else {
                    rendered = self.render(&comps, &edges[ei]);
                    &rendered
                };
                text.push_str(&r[0].text);
                if !r[0].note.is_empty() {
                    notes.push(r[0].note.clone());
                }
            }
            push(Candidate { text, note: notes.join(" / ") }, &mut whole);
        }
        // Second rendering of the best path (other 末音添記 variant).
        if !best.is_empty() {
            let mut text = String::new();
            for r in &best {
                text.push_str(&r.get(1).unwrap_or(&r[0]).text);
            }
            push(Candidate { text, note: "variant".into() }, &mut whole);
        }
        push(Candidate { text: run.to_string(), note: "한글".into() }, &mut whole);

        let cost = paths.first().filter(|(_, path)| path.iter().any(|&ei| !matches!(edges[ei].kind, Kind::Raw))).map(|(c, _)| *c);
        let covered = paths.first().is_some_and(|(_, path)| path.iter().all(|&ei| !matches!(edges[ei].kind, Kind::Raw)));
        let attested = paths.first().is_some_and(|(_, path)| {
            path.iter().all(|&ei| match &edges[ei].kind {
                Kind::Raw => false,
                Kind::Sino { entry, .. } => self.sinos.entries[*entry as usize].count >= ATTESTED,
                Kind::Nominal { entry, .. } | Kind::Verb { entry, .. } => self.natives[*entry as usize].count >= ATTESTED,
            })
        });
        let mut segments = Vec::new();
        // The old spelling of each segment's tail, for the best path.
        let mut old_seg: Vec<Option<String>> = Vec::new();
        let mut old_whole = None;
        // The same with only the 하 rows, for `old_ha`.
        let ha = self.options.old_ha;
        let mut ha_seg: Vec<Option<String>> = Vec::new();
        let mut ha_whole = None;
        if let Some((_, path)) = paths.first().filter(|_| full) {
            let mut pieces: Vec<String> = Vec::new();
            let mut ha_pieces: Vec<String> = Vec::new();
            let mut any = false;
            let mut ha_any = false;
            for (n, &ei) in path.iter().enumerate() {
                let e = &edges[ei];
                let tail = after_hanja && n == 0;
                let light = matches!(&e.kind, Kind::Verb { entry, .. } if self.natives[*entry as usize].lemma == "하다");
                let mut alts: Vec<(f32, usize)> = edges
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.start == e.start && x.end == e.end)
                    .map(|(i, x)| (x.cost, i))
                    .collect();
                alts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                // Dictionary forms of every reading first, then the constructed ones.
                let mut cands: Vec<Candidate> = Vec::new();
                // A one-syllable word has only its Hanja and its last letter to choose
                // between, so each Hanja keeps its forms together: 少ㅁ, 少, 聊, 聊ㅁ.
                let one_syllable = |e: &Edge| {
                    matches!(e.kind, Kind::Nominal { entry, .. } if crate::jamo::syllable_count(&self.natives[entry as usize].stem) == 1)
                };
                for &(_, i) in &alts {
                    let forms = if one_syllable(&edges[i]) { self.render(&comps, &edges[i]) } else { self.render_dict(&comps, &edges[i]) };
                    for c in forms {
                        if !cands.iter().any(|x| x.text == c.text) {
                            cands.push(c);
                        }
                    }
                }
                for &(_, i) in &alts {
                    for c in self.render(&comps, &edges[i]) {
                        if !cands.iter().any(|x| x.text == c.text) {
                            cands.push(c);
                        }
                    }
                }
                // The same alternatives serve the segment and the phrase, so
                // `convert` and `segments` offer the same spelling: under
                // `hada_kun` the Hangul 하고 behind 爲고 still gives ᄒᆞ고.
                let v = old_variant(cands.iter().map(|c| c.text.as_str()), light, tail, false);
                any |= v.is_some();
                pieces.push(v.clone().unwrap_or_else(|| best[n][0].text.clone()));
                old_seg.push(v);
                if ha {
                    let h = old_variant(cands.iter().map(|c| c.text.as_str()), light, tail, true);
                    ha_any |= h.is_some();
                    ha_pieces.push(h.clone().unwrap_or_else(|| best[n][0].text.clone()));
                    ha_seg.push(h);
                }
                segments.push(Segment { reading: compose(&comps[e.start..e.end]), candidates: cands });
            }
            if any {
                old_whole = Some(pieces.concat());
            }
            if ha_any {
                ha_whole = Some(ha_pieces.concat());
            }
        }
        // One word still offers the phrase readings: 수없다 is 數없다 or 手無다.
        if let [one] = segments.as_mut_slice() {
            for c in &whole {
                if !one.candidates.iter().any(|x| x.text == c.text) {
                    one.candidates.push(c.clone());
                }
            }
        }
        let hun = self.hun_candidates(run);
        if hun.is_empty() {
            if full {
                if segments.len() == old_seg.len() {
                    for (s, v) in segments.iter_mut().zip(old_seg) {
                        offer_old(&mut s.candidates, v);
                    }
                    for (s, v) in segments.iter_mut().zip(ha_seg) {
                        first_old(&mut s.candidates, v);
                    }
                }
                offer_old(&mut whole, old_whole.clone());
                first_old(&mut whole, ha_whole.clone());
            }
        } else {
            // A key spans the run, so the run becomes one segment to pick from.
            let cands = match segments.as_mut_slice() {
                [one] => std::mem::take(&mut one.candidates),
                _ => whole.clone(),
            };
            segments = vec![Segment { reading: run.to_string(), candidates: self.with_hun(cands, run, hun.clone()) }];
            whole = self.with_hun(whole, run, hun);
        }
        for s in &mut segments {
            okurigana::add_i_spellings(&mut s.candidates);
            self.add_plural_spellings(&mut s.candidates);
        }
        okurigana::add_i_spellings(&mut whole);
        self.add_plural_spellings(&mut whole);
        // The literal input always stays selectable.
        let keep = self.options.max_candidates.max(2);
        if whole.len() > keep {
            let typed = whole.iter().find(|c| c.text == run).cloned();
            whole.truncate(keep);
            if let Some(t) = typed.filter(|t| !whole.iter().any(|c| c.text == t.text)) {
                whole[keep - 1] = t;
            }
        }
        Converted { whole, segments, cost, covered, attested, old: old_whole, ha: ha_whole }
    }

    /// Characters whose gloss is `word`, in either order: 살찔비 or 비살찔 → 肥.
    fn hun_candidates(&self, word: &str) -> Vec<Candidate> {
        let mut chars = word.chars();
        let Some(first) = chars.next() else { return Vec::new() };
        let rest = chars.as_str();
        let mut out: Vec<Candidate> = Vec::new();
        for key in [word.to_string(), format!("{rest}{first}")] {
            let Some((last, _)) = key.char_indices().last() else { continue };
            for hanja in self.hun.get(&key) {
                if !out.iter().any(|c| c.text == hanja) {
                    out.push(Candidate { text: hanja.to_string(), note: format!("{} {} · 訓音", &key[..last], &key[last..]) });
                }
            }
        }
        out
    }

    /// Add gloss matches to `list`, behind the best reading. They go ahead of
    /// it only when conversion left the input as it was and the input is a
    /// full gloss then reading of three syllables or more: shorter ones are
    /// often plain words (반짝 = 짝 반, 둘이 = 둘 이), and 國漢文混用 leaves native
    /// words in Hangul on purpose.
    fn with_hun(&self, mut list: Vec<Candidate>, input: &str, hun: Vec<Candidate>) -> Vec<Candidate> {
        list.retain(|c| !hun.iter().any(|h| h.text == c.text));
        let key: String = input.split_whitespace().collect();
        let first = !self.options.sino_only
            && list.first().is_some_and(|c| c.text == input)
            && key.chars().count() >= 3
            && !self.hun.get(&key).is_empty();
        let at = if first { 0 } else { list.len().min(1) };
        list.splice(at..at, hun);
        list
    }

    /// Gloss matches for an input spaced as 살찔 비.
    fn spaced_hun(&self, input: &str) -> Option<Vec<Candidate>> {
        let words: Vec<&str> = input.split_whitespace().collect();
        if words.len() < 2 || !words.iter().all(|w| w.chars().all(is_syllable)) {
            return None;
        }
        Some(self.hun_candidates(&words.concat())).filter(|h| !h.is_empty())
    }

    /// The surroundings of run `i` among the pieces of an input.
    fn around(&self, pieces: &[(bool, String)], i: usize) -> Around {
        let hangul = pieces.iter().filter(|(h, _)| *h).count();
        // The next Hangul word, with only blanks between.
        let next = match pieces.get(i + 1) {
            Some((false, gap)) if !gap.is_empty() && gap.chars().all(char::is_whitespace) => pieces.get(i + 2).filter(|(h, _)| *h).map(|(_, w)| w.as_str()),
            _ => None,
        };
        let bound_next = next.is_some_and(|w| {
            let mut cs = w.chars();
            let first = cs.next().map(String::from).unwrap_or_default();
            let rest = cs.as_str();
            // The bound noun alone or with a particle: 때, 때는, 것을 — not 바람.
            BOUND_NEXT.contains(&first.as_str()) && (rest.is_empty() || self.particle_trie.prefixes(&decompose(rest), 0).iter().any(|(r, _)| *r == decompose(rest).len()))
        });
        Around { alone: hangul == 1, bound_next }
    }

    /// Whole-input candidates, best first. Non-Hangul text passes through.
    /// Number of Sino-Korean (reading, Hanja) pairs loaded.
    pub fn sino_count(&self) -> usize {
        self.sinos.len()
    }

    pub fn convert(&self, input: &str) -> Vec<Candidate> {
        self.mixed_convert(input).unwrap_or_else(|| self.convert_unmixed(input))
    }

    /// [`Engine::convert`] for text without Hanja words.
    fn convert_unmixed(&self, input: &str) -> Vec<Candidate> {
        let mut per_run: Vec<Vec<Candidate>> = Vec::new();
        let mut after_hanja = false;
        let pieces = runs(input);
        for (i, (is_h, run)) in pieces.iter().enumerate() {
            let (is_h, run) = (*is_h, run.clone());
            if is_h {
                per_run.push(self.convert_run(&run, after_hanja, self.around(&pieces, i)).0);
            } else {
                after_hanja = ends_in_hanja(&run);
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
                let note = whole_note(&text, &notes);
                out.push(Candidate { text, note });
            }
        }
        // Old letters the engine cannot read split the input into runs: the
        // input as typed stays one choice for the whole phrase.
        if input.chars().any(is_jamo) && !out.iter().any(|c| c.text == input) {
            out.push(Candidate { text: input.to_string(), note: String::new() });
        }
        match self.spaced_hun(input) {
            Some(hun) => self.with_hun(out, input, hun),
            None => out,
        }
    }

    /// Segmentation of the best analysis, each segment with alternatives.
    pub fn segments(&self, input: &str) -> Vec<Segment> {
        self.mixed_segments(input).unwrap_or_else(|| self.segments_unmixed(input))
    }

    /// [`Engine::segments`] for text without Hanja words.
    fn segments_unmixed(&self, input: &str) -> Vec<Segment> {
        if self.spaced_hun(input).is_some() {
            return vec![Segment { reading: input.to_string(), candidates: self.convert(input) }];
        }
        let mut out = Vec::new();
        let mut after_hanja = false;
        let pieces = runs(input);
        for (i, (is_h, run)) in pieces.iter().enumerate() {
            let (is_h, run) = (*is_h, run.clone());
            if is_h {
                out.extend(self.convert_run(&run, after_hanja, self.around(&pieces, i)).1.into_iter().map(|mut s| {
                    for c in &mut s.candidates {
                        c.note = whole_note(&c.text, std::slice::from_ref(&c.note));
                    }
                    s
                }));
            } else {
                after_hanja = ends_in_hanja(&run);
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
                    for c in self.render_dict_as(&comps, &edge, true) {
                        out.push((c.text, surface.clone()));
                    }
                }
            }
        } else {
            let base = Edge { start: 0, end: n.stem.len(), cost: 0.0, kind: Kind::Nominal { entry: i as u32, particle: None } };
            out.push((self.render_dict_as(&n.stem, &base, true)[0].text.clone(), compose(&n.stem)));
            let pron = matches!(n.pos, Pos::Pron | Pos::Bound);
            let last = *n.stem.last().unwrap();
            for (pi, p) in self.particles.iter().enumerate() {
                if allowed(p.cond, last, pron) {
                    let comps = [n.stem.clone(), p.comps.clone()].concat();
                    let e = Edge { start: 0, end: comps.len(), cost: 0.0, kind: Kind::Nominal { entry: i as u32, particle: Some(pi as u16) } };
                    out.push((self.render_dict_as(&comps, &e, true)[0].text.clone(), compose(&comps)));
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

fn is_hanja(c: char) -> bool {
    crate::dict::is_han(c)
}

/// A letter of the Hangul tail after Hanja: a syllable, a compatibility jamo
/// or any jamo of an old syllable (which then stays whole in its token).
fn is_hangulish(c: char) -> bool {
    is_syllable(c) || ('\u{3131}'..='\u{318E}').contains(&c) || is_jamo(c)
}

impl Engine {
    fn reverse(&self) -> &Reverse {
        self.reverse.get_or_init(|| {
            let mut sino: HashMap<String, (String, f32, u32)> = HashMap::new();
            let mut max_sino = 1;
            for i in 0..self.sinos.len() {
                let (hanja, reading, cost) = (self.sinos.hanja(i), self.sinos.reading(i), self.sinos.cost(i));
                let count = self.sinos.entries[i].count;
                max_sino = max_sino.max(hanja.chars().count());
                let e = sino.entry(hanja.to_string()).or_insert((reading.to_string(), f32::MAX, 0));
                if cost < e.1 || (cost == e.1 && count > e.2) {
                    *e = (reading.to_string(), cost, count);
                }
            }
            let mut native: HashMap<String, Vec<usize>> = HashMap::new();
            for (i, n) in self.natives.iter().enumerate() {
                if !n.hanja.is_empty() {
                    native.entry(n.hanja.clone()).or_default().push(i);
                }
            }
            for v in native.values_mut() {
                v.sort_by(|&a, &b| self.native_cost(&self.natives[a]).partial_cmp(&self.native_cost(&self.natives[b])).unwrap());
            }
            let compounds = native.keys().filter(|k| k.chars().any(is_syllable)).cloned().collect();
            let light = (0..self.natives.len())
                .filter(|&i| matches!(self.natives[i].lemma.as_str(), "하다" | "되다" | "시키다" | "당하다" | "스럽다"))
                .flat_map(|i| self.forms_of(i).into_iter().map(|(_, s)| s))
                .collect();
            Reverse { sino: sino.into_iter().map(|(k, (r, c, _))| (k, (r, c))).collect(), native, max_sino, compounds, light }
        })
    }

    /// Read Hanja by sound with the fewest dictionary words; returns the
    /// reading and its cost (1 per word, 0.8 more for one character, 5 for
    /// a character with no reading).
    fn read_sino(&self, han: &str) -> (String, f32) {
        let r = self.reverse();
        let cs: Vec<char> = han.chars().collect();
        let n = cs.len();
        let mut best: Vec<Option<(f32, String)>> = vec![None; n + 1];
        best[0] = Some((0.0, String::new()));
        for i in 0..n {
            let Some((c0, s0)) = best[i].clone() else { continue };
            let relax = |j: usize, cost: f32, text: &str, best: &mut Vec<Option<(f32, String)>>| {
                let c = c0 + cost;
                if best[j].as_ref().map_or(true, |(bc, _)| c < *bc) {
                    best[j] = Some((c, format!("{s0}{text}")));
                }
            };
            relax(i + 1, 5.0, &cs[i].to_string(), &mut best);
            for len in 1..=r.max_sino.min(n - i) {
                let w: String = cs[i..i + len].iter().collect();
                if let Some((reading, _)) = r.sino.get(&w) {
                    relax(i + len, if len == 1 { 1.8 } else { 1.0 }, reading, &mut best);
                }
            }
        }
        let (c, t) = best[n].clone().unwrap();
        (t, c)
    }

    fn is_particle_tail(&self, tail: &str) -> bool {
        if tail.is_empty() {
            return true;
        }
        let comps = decompose(tail);
        self.particles.iter().any(|p| p.comps == comps && !is_t(p.comps[0]) && !is_v(p.comps[0]))
    }

    /// One token: a Hanja run and the Hangul that follows it. The reading
    /// is either all 音讀 (tail = particles), or 音讀 for a prefix and a
    /// 訓讀 entry for the rest, whichever costs less.
    fn read_token(&self, han: &str, tail: &str) -> String {
        let r = self.reverse();
        let (sino, sc) = self.read_sino(han);
        let plain = self.is_particle_tail(tail) || r.light.contains(tail);
        let mut best = (sc + if plain { 0.0 } else { 4.0 }, format!("{sino}{tail}"));
        let cs: Vec<char> = han.chars().collect();
        let written_tail = |k: usize| format!("{}{tail}", cs[k..].iter().collect::<String>());
        for k in 0..cs.len() {
            let suffix: String = cs[k..].iter().collect();
            let Some(entries) = r.native.get(&suffix) else { continue };
            let written = written_tail(k);
            for &i in entries {
                let n = &self.natives[i];
                let cost = self.kun_cost(n);
                let (pre, pc) = self.read_sino(&cs[..k].iter().collect::<String>());
                if pc + cost >= best.0 {
                    continue;
                }
                if let Some((_, surface)) = self.cached_forms(i, false).iter().find(|(w, _)| *w == written) {
                    let surface = surface.clone();
                    best = (pc + cost, format!("{pre}{surface}"));
                }
            }
            // Not a dictionary form: a 末音添記 variant (超ㅜ, 近까웠다) of an entry.
            if !tail.is_empty() {
                self.read_variant(entries, &written, &cs[..k], &mut best);
            }
        }
        best.1
    }

    /// [`Engine::forms_of`] (or `variant_forms`) of entry `i`, listed once.
    fn cached_forms(&self, i: usize, variants: bool) -> std::sync::Arc<Vec<(String, String)>> {
        let o = &self.options;
        let spelling = (o.style == Style::Morpheme) as u8
            | (o.fragment == Fragment::Letter) as u8 * 2
            | (o.coda_always as u8) * 4
            | (o.resyllabify as u8) * 8
            | (o.hada_kun as u8) * 16
            | (o.old_ha as u8) * 32;
        {
            let mut cache = self.forms_cache.lock().unwrap();
            if cache.0 != spelling {
                *cache = (spelling, HashMap::new());
            }
            if let Some(hit) = cache.1.get(&(i, variants)) {
                return hit.clone();
            }
        }
        let list = std::sync::Arc::new(if variants { self.variant_forms(i) } else { self.forms_of(i) });
        let mut cache = self.forms_cache.lock().unwrap();
        if cache.0 == spelling {
            cache.1.insert((i, variants), list.clone());
        }
        list
    }

    /// Offer the reading of `written` as a 末音添記 variant of one of `entries`
    /// (the 訓讀 entries of its Hanja after `prefix`), unless some entry
    /// has it as a dictionary form.
    fn read_variant(&self, entries: &[usize], written: &str, prefix: &[char], best: &mut (f32, String)) {
        let (pre, pc) = self.read_sino(&prefix.iter().collect::<String>());
        for &i in entries {
            let cost = self.kun_cost(&self.natives[i]);
            if pc + cost >= best.0 {
                continue;
            }
            if let Some((_, surface)) = self.cached_forms(i, true).iter().find(|(w, _)| w == written) {
                if !entries.iter().any(|&j| self.cached_forms(j, false).iter().any(|(w, _)| w == written)) {
                    let surface = surface.clone();
                    *best = (pc + cost, format!("{pre}{surface}"));
                }
            }
        }
    }

    /// The cost of reading Hanja through a 訓讀 entry: hand-made spellings
    /// win a tie with a whole-word 音讀 reading (一番 = 한번, not 일번).
    fn kun_cost(&self, n: &Native) -> f32 {
        let tier = match n.source {
            Source::User | Source::Corpus | Source::Editorial => -0.5,
            Source::Community => 0.5,
            Source::Suggested => 2.0,
            Source::Gloss | Source::Japanese | Source::Lexicon => 3.0,
            Source::JapaneseWeak => 3.5,
        };
        1.0 + tier - 0.1 * ((n.count as f32) + 1.0).ln()
    }

    /// Read 釋讀 text back into Hangul (釋讀文 → 한글). Sino-Korean words
    /// are read by sound, 訓讀 words through the dictionary entries that
    /// produce them; anything unknown is left as written.
    pub fn read(&self, text: &str) -> String {
        let cs: Vec<char> = crate::dict::unify(&modernize_function(text)).chars().collect();
        let mut out = String::new();
        let mut i = 0;
        while i < cs.len() {
            if !is_hanja(cs[i]) {
                out.push(cs[i]);
                i += 1;
                continue;
            }
            let mut j = i;
            while j < cs.len() && is_hanja(cs[j]) {
                j += 1;
            }
            // A compound verb's written form spans inner Hangul (奪아去).
            let rest: String = cs[i..].iter().collect();
            if let Some(key) = self.reverse().compounds.iter().filter(|k| rest.starts_with(k.as_str())).max_by_key(|k| k.len()) {
                j = i + key.chars().count();
            }
            let mut k = j;
            while k < cs.len() && is_hangulish(cs[k]) {
                k += 1;
            }
            let han: String = cs[i..j].iter().collect();
            let tail: String = cs[j..k].iter().collect();
            out.push_str(&self.read_token(&han, &tail));
            i = k;
        }
        out
    }
}

/// A stem ending in exactly one coda component (있, 그렇; not 읽, 않).
fn simple_coda(stem: &[char]) -> bool {
    let k = stem.iter().rev().take_while(|&&c| is_t(c)).count();
    k == 1
}

fn is_su_before_existence(entry: &Native, rest: &[char]) -> bool {
    entry.pos == Pos::Bound && entry.lemma == "수" && ["있", "없"].iter().any(|w| rest.starts_with(&decompose(w)))
}

/// The notes of a candidate's pieces. A piece that makes up the whole
/// candidate is not named again: 良心 is noted "양심 · 音讀".
fn whole_note(text: &str, notes: &[String]) -> String {
    match notes {
        [one] => one.strip_prefix(text).and_then(|r| r.strip_prefix(' ')).unwrap_or(one).to_string(),
        _ => notes.join(" / "),
    }
}

/// What a 訓讀 candidate is: "讀 읽다 +아/어 · 訓讀".
fn note(n: &Native, ending: Option<usize>) -> String {
    match ending {
        Some(e) => format!("{} {} +{} · 訓讀", n.hanja, n.lemma, ending_label(e)),
        None => format!("{} {} · 訓讀", n.hanja, n.lemma),
    }
}

/// Put the old spelling of a tail right behind the best candidates: third
/// place, or last in a shorter list. It does not take first place.
fn offer_old(list: &mut Vec<Candidate>, old: Option<String>) {
    let Some(text) = old else { return };
    if list.is_empty() || list.iter().any(|c| c.text == text) {
        return;
    }
    let at = list.len().min(2);
    list.insert(at, Candidate { text, note: "옛한글".into() });
}

/// With `old_ha`: the old spelling of 하 in first place, the candidate it
/// was made from right behind it.
fn first_old(list: &mut Vec<Candidate>, old: Option<String>) {
    let Some(text) = old else { return };
    if list.is_empty() {
        return;
    }
    list.retain(|c| c.text != text);
    list.insert(0, Candidate { text, note: "옛한글".into() });
}

/// Split input into runs of Hangul (syllables and conjoining jamo) and
/// everything else.
fn runs(input: &str) -> Vec<(bool, String)> {
    let mut out: Vec<(bool, String)> = Vec::new();
    let mut push = |h: bool, piece: &str| match out.last_mut() {
        Some((k, s)) if *k == h => s.push_str(piece),
        _ => out.push((h, piece.to_string())),
    };
    if !input.chars().any(is_jamo) {
        for c in input.chars() {
            push(is_syllable(c), c.encode_utf8(&mut [0; 4]));
        }
        return out;
    }
    // Old letters: a unit the engine can read joins the run; one it cannot
    // (a stray jamo, an initial with no vowel, a letter with no modern
    // counterpart) stays as it is, and the text around it is converted.
    for unit in crate::jamo::units(input) {
        let mut cs = unit.chars();
        let h = match (cs.next(), cs.next()) {
            (Some(c), None) if is_syllable(c) => true,
            (Some(c), _) if is_jamo(c) => foldable(unit),
            _ => false,
        };
        push(h, unit);
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
    const SINO: &str = "reading\thanja\tcount\tsource\n안녕\t安寧\t\t\n규칙\t規則\t\t\n풀어읽기모임\t釋讀會\t\t\n환영\t歡迎\t\t\n";

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
    fn compound_verb() {
        let (e, errs) = Engine::new(&[("t", "빼앗아가다\t奪아去다\tverb\n일어나다\t起어出다\tverb\n")], &[], Options::default());
        assert!(errs.is_empty(), "{errs:?}");
        assert_eq!(first(&e, "빼앗아갔다"), "奪아去ㅆ다");
        assert_eq!(first(&e, "일어났어요"), "起어出ㅆ어요");
        assert_eq!(e.read("起어出ㅆ어요"), "일어났어요");
    }

    #[test]
    fn read_back() {
        let e = eng(Options::default());
        assert_eq!(e.read("天을 見ㅆ다"), "하늘을 봤다");
        assert_eq!(e.read("規則을 讀었다"), "규칙을 읽었다");
        assert_eq!(e.read("自ㄴ 在ㅆ으면 好겠어요"), "전 있으면 好겠어요");
        assert_eq!(e.read("安寧爲요"), "안녕해요");
    }

    #[test]
    fn read_back_ignores_the_conversion_mode() {
        let e = eng(Options { sino_only: true, ..Options::default() });
        assert_eq!(e.read("天을 見ㅆ다"), "하늘을 봤다");
        assert_eq!(e.read("初음 來신 事ㅅ을 歡迎"), "처음 오신 것을 환영");
    }

    #[test]
    fn read_back_follows_options_changed_later() {
        // The site changes the options of a running engine.
        let mut e = eng(Options { resyllabify: true, ..Options::default() });
        let fresh = eng(Options::default()).read("難ㅂ어");
        assert_eq!(fresh, "어려워");
        let _ = e.read("難ㅂ어");
        e.options.resyllabify = false;
        assert_eq!(e.read("難ㅂ어"), fresh);
        e.options.resyllabify = true;
        e.options.style = Style::Morpheme;
        let morph = eng(Options { resyllabify: true, style: Style::Morpheme, ..Options::default() });
        assert_eq!(e.read("難ㅂ어"), morph.read("難ㅂ어"));
    }

    #[test]
    fn hada_kun() {
        let e = eng(Options { hada_kun: true, ..Options::default() });
        assert_eq!(first(&e, "안녕해요"), "安寧爲요");
    }
}
