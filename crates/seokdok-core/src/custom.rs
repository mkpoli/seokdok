//! The reader's customisation, in the `seokdok-custom` JSON format the web
//! page exports: own phrases (fixed text for a reading), own 訓讀 words (they
//! inflect), and rewrite rules after RIME's spelling algebra and filters.
//!
//! A rule is `{ on, stage, mode, regex, pattern, replace, flags }`:
//! - stage `input` rewrites the Hangul before conversion; `output` rewrites
//!   every candidate;
//! - mode `xform` replaces; `derive` keeps the original and adds the
//!   rewritten text right after it;
//! - `regex: false` matches `pattern` literally; `true` reads it as a regular
//!   expression (`flags` i, m, s; `$1` in `replace`).
//!
//! Rules run top to bottom, each on the output of the ones above.

use crate::engine::Candidate;
use regex::{Regex, RegexBuilder};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Rule {
    #[serde(default = "yes")]
    pub on: bool,
    #[serde(default = "output")]
    pub stage: String,
    #[serde(default = "xform")]
    pub mode: String,
    #[serde(default)]
    pub regex: bool,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub replace: String,
    #[serde(default)]
    pub flags: String,
    #[serde(default)]
    pub note: String,
}

fn yes() -> bool {
    true
}
fn output() -> String {
    "output".into()
}
fn xform() -> String {
    "xform".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Phrase {
    #[serde(default = "yes")]
    pub on: bool,
    pub reading: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Word {
    #[serde(default = "yes")]
    pub on: bool,
    pub lemma: String,
    pub written: String,
    #[serde(default)]
    pub pos: String,
}

#[derive(Debug, Default, Deserialize)]
struct File {
    #[serde(default)]
    format: String,
    #[serde(default)]
    phrases: Vec<Phrase>,
    #[serde(default)]
    words: Vec<Word>,
    #[serde(default)]
    rules: Vec<Rule>,
}

struct Compiled {
    rule: Rule,
    re: Regex,
}

/// A loaded customisation, ready to apply.
#[derive(Default)]
pub struct Custom {
    pub phrases: Vec<Phrase>,
    pub words: Vec<Word>,
    rules: Vec<Compiled>,
    /// Rules that failed to compile: (index, message).
    pub errors: Vec<(usize, String)>,
}

impl std::fmt::Debug for Custom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Custom").field("rules", &self.rules.len()).field("errors", &self.errors).finish()
    }
}

impl Custom {
    /// Parse a `seokdok-custom` JSON document.
    pub fn from_json(text: &str) -> Result<Custom, String> {
        let f: File = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if f.format != "seokdok-custom" {
            return Err("not a seokdok-custom file".into());
        }
        Ok(Custom::new(f.phrases, f.words, f.rules))
    }

    pub fn new(phrases: Vec<Phrase>, words: Vec<Word>, rules: Vec<Rule>) -> Custom {
        let mut compiled = Vec::new();
        let mut errors = Vec::new();
        for (i, r) in rules.into_iter().enumerate() {
            if !r.on || r.pattern.is_empty() {
                continue;
            }
            let src = if r.regex { r.pattern.clone() } else { regex::escape(&r.pattern) };
            let mut b = RegexBuilder::new(&src);
            if r.regex {
                b.case_insensitive(r.flags.contains('i'))
                    .multi_line(r.flags.contains('m'))
                    .dot_matches_new_line(r.flags.contains('s'));
            }
            match b.build() {
                Ok(re) => compiled.push(Compiled { rule: r, re }),
                Err(e) => errors.push((i, e.to_string())),
            }
        }
        Custom { phrases, words, rules: compiled, errors }
    }

    fn apply(c: &Compiled, text: &str) -> String {
        if c.rule.regex {
            // JavaScript writes "$1ㅏ"; Rust would read that as group "1ㅏ".
            let rep = js_replacement(&c.rule.replace);
            c.re.replace_all(text, rep.as_str()).into_owned()
        } else {
            c.re.replace_all(text, regex::NoExpand(&c.rule.replace)).into_owned()
        }
    }

    /// Rewrite the input Hangul with the input-stage `xform` rules.
    pub fn rewrite_input(&self, text: &str) -> String {
        let mut t = text.to_string();
        for c in self.rules.iter().filter(|c| c.rule.stage == "input" && c.rule.mode == "xform") {
            t = Self::apply(c, &t);
        }
        t
    }

    /// Rewrite candidates with the output-stage rules, keeping order and
    /// dropping duplicates.
    pub fn rewrite_output(&self, cands: Vec<Candidate>) -> Vec<Candidate> {
        let rules: Vec<&Compiled> = self.rules.iter().filter(|c| c.rule.stage == "output").collect();
        if rules.is_empty() {
            return cands;
        }
        let mut list = cands;
        for c in rules {
            let mut next = Vec::with_capacity(list.len());
            for k in list {
                let t = Self::apply(c, &k.text);
                if c.rule.mode == "derive" {
                    let changed = t != k.text;
                    let note = if k.note.is_empty() { c.rule.note.clone() } else { format!("{} · {}", k.note, c.rule.note) };
                    next.push(k);
                    if changed {
                        next.push(Candidate { text: t, note });
                    }
                } else if t != k.text {
                    next.push(Candidate { text: t, note: k.note });
                } else {
                    next.push(k);
                }
            }
            list = next;
        }
        let mut seen = std::collections::HashSet::new();
        list.retain(|k| seen.insert(k.text.clone()));
        list
    }

    /// The user's words and phrases as engine TSV texts: (user.tsv, user-sino.tsv).
    pub fn user_tsv(&self) -> (String, String) {
        let clean = |s: &str| s.replace(['\t', '\n'], " ").trim().to_string();
        let native = self
            .words
            .iter()
            .filter(|w| w.on && !w.lemma.is_empty() && !w.written.is_empty())
            .map(|w| {
                let pos = if w.pos.is_empty() { "noun" } else { w.pos.as_str() };
                format!("{}\t{}\t{}\t\tuser\t\t\n", clean(&w.lemma), clean(&w.written), pos)
            })
            .collect();
        let sino = self
            .phrases
            .iter()
            .filter(|p| p.on && !p.reading.is_empty() && !p.text.is_empty())
            .map(|p| format!("{}\t{}\t\tuser\n", clean(&p.reading), clean(&p.text)))
            .collect();
        (native, sino)
    }
}

/// Turn JavaScript replacement syntax into Rust's: `$1` → `${1}`, `$$` → `$`
/// stays `$$`, and `$&` (whole match) → `${0}`.
fn js_replacement(s: &str) -> String {
    let mut out = String::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '$' && i + 1 < cs.len() {
            let n = cs[i + 1];
            if n.is_ascii_digit() {
                let mut j = i + 1;
                while j < cs.len() && cs[j].is_ascii_digit() && j < i + 3 {
                    j += 1;
                }
                out.push_str("${");
                out.extend(&cs[i + 1..j]);
                out.push('}');
                i = j;
                continue;
            }
            if n == '&' {
                out.push_str("${0}");
                i += 2;
                continue;
            }
            if n == '{' || n == '$' {
                out.push(cs[i]);
                out.push(n);
                i += 2;
                continue;
            }
        }
        out.push(cs[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(t: &[&str]) -> Vec<Candidate> {
        t.iter().map(|s| Candidate { text: s.to_string(), note: String::new() }).collect()
    }

    #[test]
    fn rules_match_the_web_page() {
        // The web page's own export: JavaScript replacement syntax ($1ㅏㅆ).
        let json = r#"{"format":"seokdok-custom","version":1,"phrases":[{"reading":"석독회","text":"釋讀會"}],
          "words":[{"lemma":"짓다","written":"作다","pos":"verb"}],
          "rules":[
            {"on":true,"stage":"output","mode":"xform","regex":false,"pattern":"爲","replace":"為"},
            {"on":true,"stage":"output","mode":"derive","regex":true,"pattern":"([\\p{Script=Han}])았","replace":"$1ㅏㅆ","flags":"gu"},
            {"on":true,"stage":"input","mode":"xform","regex":false,"pattern":"읍니다","replace":"습니다"},
            {"on":true,"stage":"output","mode":"xform","regex":true,"pattern":"(","replace":""}]}"#;
        let k = Custom::from_json(json).unwrap();
        assert_eq!(k.errors.len(), 1);
        let out: Vec<String> = k.rewrite_output(c(&["爲ㅆ다", "見았다", "봤다"])).into_iter().map(|x| x.text).collect();
        assert_eq!(out, ["為ㅆ다", "見았다", "見ㅏㅆ다", "봤다"]);
        assert_eq!(k.rewrite_input("있읍니다"), "있습니다");
        let (n, s) = k.user_tsv();
        assert!(n.starts_with("짓다\t作다\tverb"));
        assert!(s.starts_with("석독회\t釋讀會"));
    }

    #[test]
    fn literal_replacement_keeps_dollars() {
        let rule = Rule {
            on: true,
            stage: "output".into(),
            mode: "xform".into(),
            regex: false,
            pattern: "a.b".into(),
            replace: "$1".into(),
            flags: String::new(),
            note: String::new(),
        };
        let k = Custom::new(vec![], vec![], vec![rule]);
        assert_eq!(k.rewrite_output(c(&["a.b axb"]))[0].text, "$1 axb");
    }

    #[test]
    fn js_replacements() {
        assert_eq!(js_replacement("$1ㅏㅆ"), "${1}ㅏㅆ");
        assert_eq!(js_replacement("[$&]"), "[${0}]");
        assert_eq!(js_replacement("$$"), "$$");
    }
}
