//! Text that is already partly or wholly 釋讀 or 國漢文混用.
//!
//! A word (a run without whitespace) that holds any Hanja is one unit: the
//! reader turns it into Hangul, and the unit's candidates are those of that
//! reading. The typed spelling is kept as a candidate, first in 釋讀 mode, so
//! converting text that is already converted never changes it. Words without
//! Hanja go through the ordinary Hangul path.

use super::{is_hanja, Candidate, Engine, Segment};
use crate::jamo::is_syllable;

/// Longest menu of one word.
const MENU: usize = 30;

/// A stretch of the input.
enum Piece {
    /// Whitespace.
    Space(String),
    /// Words without Hanja, with the spaces between them.
    Plain(String),
    /// One word holding Hanja.
    Word(String),
}

fn is_compat_jamo(c: char) -> bool {
    ('\u{3131}'..='\u{318E}').contains(&c)
}

/// Cut the input into whitespace, Hangul-only stretches and Hanja words.
/// `None` when no word holds Hanja.
fn pieces(input: &str) -> Option<Vec<Piece>> {
    if !input.chars().any(is_hanja) {
        return None;
    }
    let mut out = Vec::new();
    let mut plain = String::new();
    fn flush(plain: &mut String, out: &mut Vec<Piece>) {
        let all = std::mem::take(plain);
        let core = all.trim();
        if core.is_empty() {
            if !all.is_empty() {
                out.push(Piece::Space(all));
            }
            return;
        }
        let start = all.len() - all.trim_start().len();
        let end = all.trim_end().len();
        if start > 0 {
            out.push(Piece::Space(all[..start].to_string()));
        }
        out.push(Piece::Plain(core.to_string()));
        if end < all.len() {
            out.push(Piece::Space(all[end..].to_string()));
        }
    }
    let mut rest = input;
    while !rest.is_empty() {
        let ws = rest.chars().next().is_some_and(char::is_whitespace);
        let n = rest.find(|c: char| c.is_whitespace() != ws).unwrap_or(rest.len());
        let (token, tail) = rest.split_at(n);
        rest = tail;
        if !ws && token.chars().any(is_hanja) {
            flush(&mut plain, &mut out);
            out.push(Piece::Word(token.to_string()));
        } else {
            plain.push_str(token);
        }
    }
    flush(&mut plain, &mut out);
    Some(out)
}

impl Engine {
    /// Whole-input candidates for text holding Hanja words; `None` for the
    /// ordinary path.
    pub(super) fn mixed_convert(&self, input: &str) -> Option<Vec<Candidate>> {
        let pieces = pieces(input)?;
        let cap = self.options.max_candidates.max(2);
        let per: Vec<Vec<Candidate>> = pieces
            .into_iter()
            .map(|p| match p {
                Piece::Space(s) => vec![Candidate { text: s, note: String::new() }],
                Piece::Plain(s) => self.convert(&s),
                Piece::Word(w) => {
                    let mut menu = self.word_menu(&w);
                    menu.truncate(cap);
                    menu
                }
            })
            .collect();
        let k = per.iter().map(Vec::len).max().unwrap_or(0).min(cap);
        let mut out: Vec<Candidate> = Vec::new();
        for i in 0..k {
            let mut text = String::new();
            let mut notes = Vec::new();
            for r in &per {
                let c = r.get(i).unwrap_or(&r[0]);
                text.push_str(&c.text);
                if !c.note.is_empty() && c.note != "한글" {
                    notes.push(c.note.clone());
                }
            }
            if !out.iter().any(|c| c.text == text) {
                let note = if notes.len() == 1 { notes.remove(0) } else { notes.join(" / ") };
                out.push(Candidate { text, note });
            }
        }
        Some(out)
    }

    /// Segments for text holding Hanja words; `None` for the ordinary path.
    pub(super) fn mixed_segments(&self, input: &str) -> Option<Vec<Segment>> {
        let pieces = pieces(input)?;
        let mut out = Vec::new();
        for p in pieces {
            match p {
                Piece::Space(s) => out.push(Segment { reading: s.clone(), candidates: vec![Candidate { text: s, note: String::new() }] }),
                Piece::Plain(s) => out.extend(self.segments(&s)),
                Piece::Word(w) => out.push(Segment { candidates: self.word_menu(&w), reading: w }),
            }
        }
        Some(out)
    }

    /// The menu of one word that holds Hanja: the spelling as typed, the
    /// candidates of its Hangul reading and that reading itself. In 國漢文混用
    /// the mode's own writing of the reading comes first.
    pub(crate) fn word_menu(&self, word: &str) -> Vec<Candidate> {
        let reading = self.read(word);
        // A Hanja the reader does not know, or a 送韓字 letter left unread
        // (見ㅆ다ㅋㅋ → 견ㅆ다ㅋㅋ): the reading cannot stand for the word.
        let poor = reading.chars().any(is_hanja) || unread_tail(word, &reading);
        let typed = |note: String| Candidate { text: word.to_string(), note };
        if poor {
            // Nothing derived from a failed reading is offered.
            return vec![typed(String::new())];
        }
        let all = self.reading_candidates(&reading);
        let first = all.first().cloned();
        let mut found = all;
        let mut menu: Vec<Candidate> = Vec::new();
        let push = |c: Candidate, menu: &mut Vec<Candidate>| {
            if !menu.iter().any(|m| m.text == c.text) {
                menu.push(c);
            }
        };
        let hangul = Candidate { text: reading.clone(), note: "한글".into() };
        found.retain(|c| c.text != reading);
        let typed = match found.iter().position(|c| c.text == word) {
            Some(i) => found.remove(i),
            None => typed(String::new()),
        };
        if self.options.sino_only {
            if let Some(first) = first {
                push(first, &mut menu);
            }
        }
        push(typed, &mut menu);
        for c in found {
            push(c, &mut menu);
        }
        // The old spelling of a tail (字ᄅᆞᆯ for 字를) is offered third, as for Hangul.
        if reading.chars().any(is_syllable) {
            let old = self.convert_unmixed(word).into_iter().filter(|c| c.note == "옛한글" && c.text != word);
            for (k, c) in old.enumerate() {
                if !menu.iter().any(|m| m.text == c.text) {
                    menu.insert((2 + k).min(menu.len()), c);
                }
            }
        }
        // The plain reading ends the menu, which stays short enough to show whole.
        menu.truncate(MENU - 1);
        push(hangul, &mut menu);
        menu
    }

    /// Candidates of a reading, dictionary forms before constructed ones: the
    /// alternatives of its one word, or the phrase readings of several.
    fn reading_candidates(&self, reading: &str) -> Vec<Candidate> {
        let segs = self.segments_unmixed(reading);
        let words: Vec<usize> = (0..segs.len()).filter(|&i| segs[i].reading.chars().any(is_syllable)).collect();
        match words.as_slice() {
            [] => Vec::new(),
            &[at] => {
                let side = |r: std::ops::Range<usize>| -> String { segs[r].iter().map(|s| s.candidates[0].text.as_str()).collect() };
                let (pre, post) = (side(0..at), side(at + 1..segs.len()));
                segs[at]
                    .candidates
                    .iter()
                    .map(|c| Candidate { text: format!("{pre}{}{post}", c.text), note: c.note.clone() })
                    .collect()
            }
            _ => self.convert_unmixed(reading),
        }
    }
}

/// A jamo run that follows Hanja in `word` and is still there in `reading`.
fn unread_tail(word: &str, reading: &str) -> bool {
    let cs: Vec<char> = word.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if is_compat_jamo(cs[i]) && i > 0 && is_hanja(cs[i - 1]) {
            let mut j = i;
            while j < cs.len() && is_compat_jamo(cs[j]) {
                j += 1;
            }
            let run: String = cs[i..j].iter().collect();
            if reading.contains(&run) {
                return true;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    false
}
