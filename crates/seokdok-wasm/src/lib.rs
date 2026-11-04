//! JavaScript bindings for seokdok: the 釋讀 converter and a 두벌식 composer.
//!
//! ```js
//! import init, { Engine, Composer } from 'seokdok'
//! await init()
//! const engine = new Engine([['corpus.tsv', corpusText], …], [['sino.tsv', sinoText]])
//! engine.convert('하늘을 봤다')   // [{ text: '天을 見ㅆ다', note: '…' }, …]
//! engine.segments('하늘을 봤다')  // [{ reading: '하늘을', candidates: [{ text, note }, …] }, …]
//! engine.read('天을 見ㅆ다')      // '하늘을 봤다'
//! ```

use seokdok_core::dubeolsik;
use seokdok_core::{Engine as Core, Fragment, Itda, Options, Style};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

fn files(v: JsValue) -> Result<Vec<(String, String)>, JsError> {
    js_sys_like::Array::from(v).0
}

/// Minimal array reader without pulling in js-sys: expects `[[label, text], …]`.
mod js_sys_like {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = Array)]
        type JsArray;
        #[wasm_bindgen(method, getter)]
        fn length(this: &JsArray) -> u32;
        #[wasm_bindgen(method, indexing_getter)]
        fn get(this: &JsArray, i: u32) -> JsValue;
    }

    pub struct Array(pub Result<Vec<(String, String)>, JsError>);

    impl From<JsValue> for Array {
        fn from(v: JsValue) -> Self {
            let outer: JsArray = v.unchecked_into();
            let mut out = Vec::new();
            for i in 0..outer.length() {
                let pair: JsArray = outer.get(i).unchecked_into();
                let (Some(label), Some(text)) = (pair.get(0).as_string(), pair.get(1).as_string()) else {
                    return Array(Err(JsError::new("expected [[label, text], …]")));
                };
                out.push((label, text));
            }
            Array(Ok(out))
        }
    }
}

/// The converter. Dictionaries are passed as `[label, tsvText]` pairs in
/// priority order (see dict/README.md for the format).
#[wasm_bindgen]
pub struct Engine {
    core: Core,
    errors: Vec<String>,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(native: JsValue, sino: JsValue) -> Result<Engine, JsError> {
        let native = files(native)?;
        let sino = files(sino)?;
        let n: Vec<(&str, &str)> = native.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let s: Vec<(&str, &str)> = sino.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let (core, errors) = Core::new(&n, &s, Options::default());
        Ok(Engine { core, errors })
    }

    /// Lines skipped while loading, as a JSON array of strings.
    pub fn errors(&self) -> String {
        to_js_string(&json!(self.errors))
    }

    /// Set options from a JSON object: `{ style: "surface"|"morpheme",
    /// fragment: "absorb"|"letter", codaAlways, resyllabify, hadaKun,
    /// itda: "有"|"在"|"在ㅆ", oldHa,
    /// sinoOnly, maxCandidates }`. Missing keys keep their value.
    #[wasm_bindgen(js_name = setOptions)]
    pub fn set_options(&mut self, json: &str) -> Result<(), JsError> {
        let v: Value = serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let o = &mut self.core.options;
        if let Some(s) = v.get("style").and_then(Value::as_str) {
            o.style = if s == "morpheme" { Style::Morpheme } else { Style::Surface };
        }
        if let Some(s) = v.get("fragment").and_then(Value::as_str) {
            o.fragment = if s == "letter" { Fragment::Letter } else { Fragment::Absorb };
        }
        if let Some(b) = v.get("codaAlways").and_then(Value::as_bool) {
            o.coda_always = b;
        }
        if let Some(b) = v.get("resyllabify").and_then(Value::as_bool) {
            o.resyllabify = b;
        }
        if let Some(b) = v.get("hadaKun").and_then(Value::as_bool) {
            o.hada_kun = b;
        }
        if let Some(s) = v.get("itda").and_then(Value::as_str) {
            o.itda = match s {
                "在" => Itda::Jae,
                "在ㅆ" => Itda::JaeCoda,
                _ => Itda::Yu,
            };
        }
        if let Some(b) = v.get("oldHa").and_then(Value::as_bool) {
            o.old_ha = b;
        }
        if let Some(b) = v.get("sinoOnly").and_then(Value::as_bool) {
            o.sino_only = b;
        }
        if let Some(n) = v.get("maxCandidates").and_then(Value::as_u64) {
            o.max_candidates = n.clamp(2, 50) as usize;
        }
        Ok(())
    }

    /// Whole-input candidates as JSON: `[{ text, note }, …]`, best first.
    pub fn convert(&self, input: &str) -> String {
        let c: Vec<Value> = self.core.convert(input).into_iter().map(|c| json!({ "text": c.text, "note": c.note })).collect();
        to_js_string(&Value::Array(c))
    }

    /// Best segmentation as JSON: `[{ reading, candidates: [{ text, note }] }]`.
    pub fn segments(&self, input: &str) -> String {
        let s: Vec<Value> = self
            .core
            .segments(input)
            .into_iter()
            .map(|s| {
                json!({
                    "reading": s.reading,
                    "candidates": s.candidates.into_iter().map(|c| json!({ "text": c.text, "note": c.note })).collect::<Vec<_>>(),
                })
            })
            .collect();
        to_js_string(&Value::Array(s))
    }

    /// Read 釋讀 text back into Hangul.
    pub fn read(&self, text: &str) -> String {
        self.core.read(text)
    }

    /// The most attested 訓讀 entries as JSON:
    /// `[{ lemma, written, pos, source, count }]`, at most `limit`.
    pub fn entries(&self, limit: usize) -> String {
        let mut v: Vec<&seokdok_core::dict::Native> = self.core.natives.iter().filter(|n| !n.hanja.is_empty() && n.count > 0).collect();
        v.sort_by(|a, b| b.count.cmp(&a.count));
        let mut seen = std::collections::HashSet::new();
        let out: Vec<Value> = v
            .into_iter()
            .filter(|n| seen.insert((n.lemma.clone(), n.hanja.clone())))
            .take(limit)
            .map(|n| {
                let written = written_form(n);
                json!({ "lemma": n.lemma, "written": written, "pos": format!("{:?}", n.pos).to_lowercase(), "source": format!("{:?}", n.source).to_lowercase(), "count": n.count })
            })
            .collect();
        to_js_string(&Value::Array(out))
    }
}

/// The dictionary form as written (讀다, 初음, 在ㅆ다).
fn written_form(n: &seokdok_core::dict::Native) -> String {
    let rest = seokdok_core::jamo::compose(&n.stem[n.cover..]);
    let da = if n.pos.inflects() { "다" } else { "" };
    format!("{}{}{}", n.hanja, rest, da)
}

fn to_js_string(v: &Value) -> String {
    v.to_string()
}

/// A 두벌식 composition buffer: feed QWERTY keys, read Hangul.
#[wasm_bindgen]
pub struct Composer {
    inner: dubeolsik::Composer,
}

#[wasm_bindgen]
impl Composer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Composer {
        Composer { inner: dubeolsik::Composer::new() }
    }

    /// Feed one key (QWERTY letter, Shift as upper case). Returns false when
    /// the key is not a 두벌식 letter.
    pub fn input(&mut self, key: &str) -> bool {
        key.chars().next().is_some_and(|c| self.inner.input(c))
    }

    /// Feed one compatibility jamo as a Hangul keyboard sends it. Returns false
    /// when the character is not one.
    #[wasm_bindgen(js_name = inputLetter)]
    pub fn input_letter(&mut self, letter: &str) -> bool {
        letter.chars().next().is_some_and(|c| self.inner.input_letter(c))
    }

    /// Append a non-letter character (space, punctuation).
    #[wasm_bindgen(js_name = pushLiteral)]
    pub fn push_literal(&mut self, s: &str) {
        for c in s.chars() {
            self.inner.push_literal(c);
        }
    }

    /// Remove one jamo. Returns false when the buffer was empty.
    pub fn backspace(&mut self) -> bool {
        self.inner.backspace()
    }

    pub fn text(&self) -> String {
        self.inner.text().to_string()
    }

    /// Replace the buffer (for example after editing the text elsewhere).
    #[wasm_bindgen(js_name = setText)]
    pub fn set_text(&mut self, s: &str) {
        self.inner.set_text(s);
    }

    /// Let modern letters form old-Hangul clusters (ㅅ+ㄱ → ᄭ, ㅂ+ㅇ → ㅸ).
    #[wasm_bindgen(js_name = setOld)]
    pub fn set_old(&mut self, old: bool) {
        self.inner.set_old(old);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Default for Composer {
    fn default() -> Self {
        Self::new()
    }
}

/// What the reader keeps picking: the table the input methods use, with the
/// page holding its text (web/src/lib/learned.js).
#[wasm_bindgen]
pub struct Learned {
    inner: seokdok_session::learned::Learned,
}

#[wasm_bindgen]
impl Learned {
    /// The table in `text`, as [`Learned::to_text`] writes it; empty for
    /// anything else.
    #[wasm_bindgen(constructor)]
    pub fn new(text: &str) -> Learned {
        Learned { inner: seokdok_session::learned::Learned::parse(text).unwrap_or_default() }
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// `text` was committed for `reading`; `first` says it was the first
    /// candidate shown. True when the table changed.
    pub fn commit(&mut self, reading: &str, text: &str, first: bool) -> bool {
        self.inner.commit(reading, text, first)
    }

    /// Whether some form of this reading has been picked enough.
    pub fn promotes(&self, reading: &str) -> bool {
        self.inner.promotes(reading)
    }

    /// The index of the learned form in a JSON array of candidate texts, or -1.
    #[wasm_bindgen(js_name = rankIndex)]
    pub fn rank_index(&self, reading: &str, texts: &str) -> i32 {
        let texts: Vec<String> = serde_json::from_str(texts).unwrap_or_default();
        self.inner.learned_at(reading, texts.iter().map(String::as_str)).map_or(-1, |at| at as i32)
    }

    /// The runs of adjacent segments to join, from a JSON array of readings:
    /// `[[start, end], …]`.
    pub fn joins(&self, readings: &str) -> String {
        let readings: Vec<String> = serde_json::from_str(readings).unwrap_or_default();
        let readings: Vec<&str> = readings.iter().map(String::as_str).collect();
        let runs: Vec<Value> = self.inner.joins(&readings).into_iter().map(|r| json!([r.start, r.end])).collect();
        to_js_string(&Value::Array(runs))
    }

    /// The table as text, to store.
    #[wasm_bindgen(js_name = toText)]
    pub fn to_text(&self) -> String {
        self.inner.to_text()
    }
}

/// The Hangul jamo a QWERTY key types in 두벌식, or "" (for the on-screen keyboard).
#[wasm_bindgen(js_name = keyToJamo)]
pub fn key_to_jamo(key: &str) -> String {
    key.chars().next().and_then(dubeolsik::key_to_jamo).map(String::from).unwrap_or_default()
}
