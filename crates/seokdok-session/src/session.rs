//! The input session: a platform-independent state machine from [`Key`] to
//! [`Effect`]s. The TSF layer applies the effects to the document and the
//! candidate window; nothing here touches Windows.
//!
//! Phases: idle (no composition), composing (Hangul in the buffer), and
//! converted (the buffer shown as 釋讀 segments, one of them focused).

use seokdok_core::dubeolsik::Composer;
use seokdok_core::{Candidate, Segment};

use crate::keys::Key;

/// Candidates shown per page of the candidate window; digits 1-9 pick one.
pub const PAGE_SIZE: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    /// Type Hangul, convert to 釋讀.
    #[default]
    Seokdok,
    /// Type Hangul, no conversion.
    Hangul,
    /// Pass keys through.
    Latin,
}

/// Source of conversions; the engine in production, a stub in tests.
pub trait Converter {
    fn segments(&self, input: &str) -> Vec<Segment>;
    /// Candidates for exactly this span (used when a segment is resized).
    fn whole(&self, input: &str) -> Vec<Candidate> {
        self.segments(input).into_iter().flat_map(|s| s.candidates).collect()
    }
}

/// Display style of a run of preedit text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attr {
    /// Hangul as typed.
    Input,
    /// Converted segment.
    Converted,
    /// The converted segment that candidate keys act on.
    Focus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub attr: Attr,
}

/// One page of the candidate window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Popup {
    /// (text, note) of the candidates on this page.
    pub items: Vec<(String, String)>,
    /// Index of the selected candidate within `items`.
    pub selected: usize,
    /// 1-based position of the selected candidate and the candidate count.
    pub position: usize,
    pub total: usize,
    /// Lay the candidates out in a row instead of a column.
    pub horizontal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Replace the composition text (starting a composition if none is open).
    Preedit(Vec<Run>),
    /// Replace the composition by final text and end it.
    Commit(String),
    /// Remove the composition and its text.
    Cancel,
    /// Show or hide the candidate window.
    Popup(Option<Popup>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Output {
    pub eaten: bool,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug)]
struct Seg {
    reading: String,
    cands: Vec<Candidate>,
    sel: usize,
}

impl Seg {
    fn text(&self) -> &str {
        &self.cands[self.sel].text
    }
}

#[derive(Clone, Debug)]
struct Conv {
    segs: Vec<Seg>,
    focus: usize,
    open: bool,
    /// Started with Space: confirming it marks a word break.
    spaced: bool,
}

/// What a key does in the current state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Act {
    Insert(char),
    Literal(char),
    /// Convert; true when started by Space (a word break follows).
    Convert(bool),
    /// Commit the buffer or the conversion, then append this character.
    CommitWith(Option<char>),
    /// Commit, then start a new composition with this jamo key.
    CommitThenInsert(char),
    /// Commit, then start a new composition with this literal.
    CommitThenLiteral(char),
    Cancel,
    Backspace,
    Revert,
    Next,
    Prev,
    FocusLeft,
    FocusRight,
    /// Pick a candidate on the page and confirm through this segment.
    Pick(usize),
    /// Next / previous page of candidates.
    Page(isize),
    /// Shrink (-1) or extend (+1) the focused segment by one syllable.
    Resize(isize),
    /// Close the candidate window, keeping the conversion.
    CloseWindow,
    ToggleLatin,
    Noop,
}

#[derive(Clone, Debug)]
pub struct Session {
    mode: Mode,
    /// Mode to return to from Latin.
    hangul_mode: Mode,
    space_converts: bool,
    /// 韓㐎 專用: nothing is converted, so the converting mode does not exist.
    hangul_only: bool,
    /// Candidate window in a row; ← → then walk the candidates.
    horizontal: bool,
    composer: Composer,
    conv: Option<Conv>,
    /// A Space conversion was confirmed: the next Hangul typed gets a space first.
    pending_break: bool,
}

impl Session {
    pub fn new(mode: Mode, space_converts: bool) -> Self {
        let hangul_mode = if mode == Mode::Latin { Mode::Seokdok } else { mode };
        Session {
            mode,
            hangul_mode,
            space_converts,
            hangul_only: false,
            horizontal: false,
            composer: Composer::new(),
            conv: None,
            pending_break: false,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_space_converts(&mut self, v: bool) {
        self.space_converts = v;
    }

    /// 韓㐎 專用: type Hangul and never convert. Space inserts a space and
    /// Enter commits; the 釋讀 mode becomes the plain Hangul mode.
    pub fn set_hangul_only(&mut self, v: bool) {
        self.hangul_only = v;
        self.mode = self.allowed(self.mode);
        self.hangul_mode = self.allowed(self.hangul_mode);
    }

    fn allowed(&self, mode: Mode) -> Mode {
        if self.hangul_only && mode == Mode::Seokdok { Mode::Hangul } else { mode }
    }

    pub fn set_horizontal(&mut self, v: bool) {
        self.horizontal = v;
    }

    /// Let modern letters form old-Hangul clusters (ㅅ+ㄱ → ᄭ, ㅂ+ㅇ → ㅸ).
    pub fn set_old_clusters(&mut self, v: bool) {
        self.composer.set_old(v);
    }

    /// True while a composition exists.
    pub fn is_composing(&self) -> bool {
        self.conv.is_some() || !self.composer.is_empty()
    }

    /// The Hangul typed so far.
    pub fn buffer(&self) -> &str {
        self.composer.text()
    }

    /// Would [`Session::handle`] consume this key?
    pub fn eats(&self, key: Key) -> bool {
        self.plan(key).is_some()
    }

    fn plan(&self, key: Key) -> Option<Act> {
        if key == Key::HangulToggle {
            return Some(Act::ToggleLatin);
        }
        if self.mode == Mode::Latin {
            return None;
        }
        let jamo = key.jamo_key();
        if let Some(conv) = &self.conv {
            let walk = conv.open && self.horizontal;
            return Some(match key {
                Key::Letter { .. } => Act::CommitThenInsert(jamo.unwrap()),
                Key::Printable(c @ '1'..='9') if conv.open => {
                    let n = c as usize - '1' as usize;
                    let page = conv.segs[conv.focus].sel / PAGE_SIZE;
                    if page * PAGE_SIZE + n < conv.segs[conv.focus].cands.len() {
                        Act::Pick(n)
                    } else {
                        Act::Noop
                    }
                }
                Key::Printable(c) => Act::CommitThenLiteral(c),
                Key::Space { shift: false } if !self.space_converts && !conv.open => Act::CommitWith(Some(' ')),
                Key::Space { shift: true } | Key::Up => Act::Prev,
                Key::Space { shift: false } | Key::Down | Key::Convert => Act::Next,
                Key::Tab { shift } => Act::Page(if shift { -1 } else { 1 }),
                Key::PageDown => Act::Page(1),
                Key::PageUp => Act::Page(-1),
                Key::Left { shift: true } => Act::Resize(-1),
                Key::Right { shift: true } => Act::Resize(1),
                Key::Left { .. } if walk => Act::Prev,
                Key::Right { .. } if walk => Act::Next,
                Key::Left { .. } => Act::FocusLeft,
                Key::Right { .. } => Act::FocusRight,
                Key::Enter => Act::CommitWith(None),
                Key::Esc if conv.open => Act::CloseWindow,
                Key::Esc | Key::Backspace => Act::Revert,
                Key::Other => Act::Noop,
                Key::HangulToggle => unreachable!(),
            });
        }
        if self.composer.is_empty() {
            return match key {
                Key::Letter { .. } => Some(Act::Insert(jamo.unwrap())),
                _ => None,
            };
        }
        let converts = self.mode == Mode::Seokdok;
        Some(match key {
            Key::Letter { .. } => Act::Insert(jamo.unwrap()),
            Key::Space { shift: false } if converts && self.space_converts => Act::Convert(true),
            Key::Space { shift: false } if !converts => Act::CommitWith(Some(' ')),
            Key::Space { .. } if converts => Act::Literal(' '),
            Key::Space { .. } => Act::CommitWith(Some(' ')),
            Key::Printable(c) if converts => Act::Literal(c),
            Key::Printable(c) => Act::CommitWith(Some(c)),
            Key::Enter => Act::CommitWith(None),
            Key::Esc => Act::Cancel,
            Key::Backspace => Act::Backspace,
            Key::Tab { .. } | Key::Down | Key::Convert if converts => Act::Convert(false),
            Key::Tab { .. }
            | Key::Down
            | Key::Convert
            | Key::Up
            | Key::Left { .. }
            | Key::Right { .. }
            | Key::PageUp
            | Key::PageDown
            | Key::Other => Act::Noop,
            Key::HangulToggle => unreachable!(),
        })
    }

    pub fn handle(&mut self, key: Key, conv: &dyn Converter) -> Output {
        let Some(act) = self.plan(key) else {
            self.pending_break = false;
            return Output { eaten: false, effects: Vec::new() };
        };
        let brk = std::mem::take(&mut self.pending_break);
        let mut fx = Vec::new();
        match act {
            Act::Noop => {}
            Act::Insert(k) => {
                // The first Hangul after a confirmed Space conversion starts a new word.
                if brk && self.composer.is_empty() && self.conv.is_none() {
                    fx.push(Effect::Commit(" ".into()));
                }
                self.composer.input(k);
                fx.push(self.input_preedit());
            }
            Act::Literal(c) => {
                self.composer.push_literal(c);
                fx.push(self.input_preedit());
            }
            Act::Backspace => {
                self.composer.backspace();
                if self.composer.is_empty() {
                    fx.push(Effect::Cancel);
                } else {
                    fx.push(self.input_preedit());
                }
            }
            Act::Cancel => {
                self.composer.clear();
                fx.push(Effect::Cancel);
            }
            Act::Convert(spaced) => self.convert(conv, spaced, &mut fx),
            Act::CommitWith(c) => {
                let mut text = self.take_text();
                text.extend(c);
                if c.is_some() {
                    self.pending_break = false;
                }
                fx.push(Effect::Commit(text));
                fx.push(Effect::Popup(None));
            }
            Act::CommitThenInsert(k) => {
                let mut text = self.take_text();
                if std::mem::take(&mut self.pending_break) && !text.ends_with(char::is_whitespace) {
                    text.push(' ');
                }
                fx.push(Effect::Commit(text));
                fx.push(Effect::Popup(None));
                self.composer.input(k);
                fx.push(self.input_preedit());
            }
            Act::CommitThenLiteral(c) => {
                fx.push(Effect::Commit(self.take_text()));
                self.pending_break = false;
                fx.push(Effect::Popup(None));
                self.composer.push_literal(c);
                fx.push(self.input_preedit());
            }
            Act::Revert => {
                self.conv = None;
                fx.push(self.input_preedit());
                fx.push(Effect::Popup(None));
            }
            Act::CloseWindow => {
                if let Some(c) = &mut self.conv {
                    c.open = false;
                }
                self.push_view(&mut fx);
            }
            Act::Next | Act::Prev => {
                if let Some(c) = &mut self.conv {
                    let seg = &mut c.segs[c.focus];
                    let n = seg.cands.len();
                    seg.sel = if act == Act::Next { (seg.sel + 1) % n } else { (seg.sel + n - 1) % n };
                    c.open = true;
                }
                self.push_view(&mut fx);
            }
            Act::Page(d) => {
                if let Some(c) = &mut self.conv {
                    let seg = &mut c.segs[c.focus];
                    let n = seg.cands.len();
                    let pages = n.div_ceil(PAGE_SIZE) as isize;
                    let p = ((seg.sel / PAGE_SIZE) as isize + d).rem_euclid(pages.max(1)) as usize;
                    seg.sel = (p * PAGE_SIZE).min(n - 1);
                    c.open = true;
                }
                self.push_view(&mut fx);
            }
            Act::FocusLeft | Act::FocusRight => {
                if let Some(c) = &mut self.conv {
                    let step: isize = if act == Act::FocusLeft { -1 } else { 1 };
                    let mut i = c.focus as isize + step;
                    while i >= 0 && (i as usize) < c.segs.len() {
                        if focusable(&c.segs[i as usize]) {
                            c.focus = i as usize;
                            break;
                        }
                        i += step;
                    }
                    c.open = false;
                }
                self.push_view(&mut fx);
            }
            Act::Resize(d) => self.resize(conv, d, &mut fx),
            Act::Pick(n) => {
                if let Some(c) = &mut self.conv {
                    let seg = &mut c.segs[c.focus];
                    seg.sel = seg.sel / PAGE_SIZE * PAGE_SIZE + n;
                }
                self.commit_through(&mut fx);
            }
            Act::ToggleLatin => {
                let next = if self.mode == Mode::Latin { self.hangul_mode } else { Mode::Latin };
                fx.extend(self.set_mode(next));
            }
        }
        Output { eaten: true, effects: fx }
    }

    /// Confirm the segments up to the focused one; the rest stay converting.
    fn commit_through(&mut self, fx: &mut Vec<Effect>) {
        let Some(c) = self.conv.take() else { return };
        let (done, rest) = c.segs.split_at(c.focus + 1);
        let mut text: String = done.iter().map(Seg::text).collect();
        let left: String = rest.iter().map(|s| s.reading.as_str()).collect();
        fx.push(Effect::Popup(None));
        if rest.is_empty() || !left.chars().any(is_hangul) {
            text.push_str(&rest.iter().map(Seg::text).collect::<String>());
            self.composer.clear();
            self.pending_break = c.spaced;
            fx.push(Effect::Commit(text));
            return;
        }
        fx.push(Effect::Commit(text));
        self.composer.set_text(&left);
        let segs = rest.to_vec();
        let focus = segs.iter().position(focusable).unwrap_or(0);
        self.conv = Some(Conv { segs, focus, open: false, spaced: c.spaced });
        self.push_view(fx);
    }

    fn resize(&mut self, conv: &dyn Converter, d: isize, fx: &mut Vec<Effect>) {
        let Some(c) = &mut self.conv else { return };
        let i = c.focus;
        let cur: Vec<char> = c.segs[i].reading.chars().collect();
        let next: Vec<char> = c.segs.get(i + 1).map(|s| s.reading.chars().collect()).unwrap_or_default();
        if (d < 0 && cur.len() <= 1) || (d > 0 && next.is_empty()) {
            return;
        }
        let (a, b): (String, String) = if d < 0 {
            (cur[..cur.len() - 1].iter().collect(), std::iter::once(cur[cur.len() - 1]).chain(next.iter().copied()).collect())
        } else {
            (cur.iter().copied().chain(std::iter::once(next[0])).collect(), next[1..].iter().collect())
        };
        let span = |t: &str| {
            // Whole-phrase readings first; a single word also brings its
            // full candidate list, which the phrase list cuts short.
            let mut v = conv.whole(t);
            if let [seg] = conv.segments(t).as_slice() {
                for k in &seg.candidates {
                    if !v.iter().any(|x| x.text == k.text) {
                        v.push(k.clone());
                    }
                }
            }
            if !v.iter().any(|k| k.text == t) {
                v.push(Candidate { text: t.to_string(), note: String::new() });
            }
            Seg { reading: t.to_string(), cands: v, sel: 0 }
        };
        let mut fresh = vec![span(&a)];
        if !b.is_empty() {
            fresh.push(span(&b));
        }
        let end = (i + 2).min(c.segs.len());
        c.segs.splice(i..end, fresh);
        c.focus = i;
        c.open = false;
        self.push_view(fx);
    }

    /// Switch mode, committing any composition as it stands.
    pub fn set_mode(&mut self, mode: Mode) -> Vec<Effect> {
        let mode = self.allowed(mode);
        let mut fx = Vec::new();
        if self.is_composing() {
            fx.push(Effect::Commit(self.take_text()));
            fx.push(Effect::Popup(None));
        }
        if self.mode != Mode::Latin {
            self.hangul_mode = self.mode;
        }
        if mode != Mode::Latin {
            self.hangul_mode = mode;
        }
        self.mode = mode;
        fx
    }

    /// Commit and clear everything (focus loss, deactivation).
    pub fn flush(&mut self) -> Vec<Effect> {
        if self.is_composing() {
            vec![Effect::Commit(self.take_text()), Effect::Popup(None)]
        } else {
            Vec::new()
        }
    }

    /// Forget the composition without effects (the document ended it).
    pub fn reset(&mut self) {
        self.composer.clear();
        self.conv = None;
    }

    /// The text that committing now would insert.
    fn current_text(&self) -> String {
        match &self.conv {
            Some(c) => c.segs.iter().map(Seg::text).collect(),
            None => self.composer.text().to_string(),
        }
    }

    fn take_text(&mut self) -> String {
        let text = self.current_text();
        if let Some(c) = &self.conv {
            self.pending_break = c.spaced;
        }
        self.composer.clear();
        self.conv = None;
        text
    }

    fn input_preedit(&self) -> Effect {
        Effect::Preedit(vec![Run { text: self.composer.text().to_string(), attr: Attr::Input }])
    }

    fn convert(&mut self, conv: &dyn Converter, spaced: bool, fx: &mut Vec<Effect>) {
        let segs: Vec<Seg> = conv
            .segments(self.composer.text())
            .into_iter()
            .filter(|s| !s.candidates.is_empty())
            .map(|s| Seg { reading: s.reading, cands: s.candidates, sel: 0 })
            .collect();
        if segs.is_empty() {
            return;
        }
        let focus = segs.iter().position(focusable).unwrap_or(0);
        self.conv = Some(Conv { segs, focus, open: false, spaced });
        self.push_view(fx);
    }

    /// Emit the preedit and the candidate window for the converted state.
    fn push_view(&self, fx: &mut Vec<Effect>) {
        let Some(c) = &self.conv else { return };
        let runs = c
            .segs
            .iter()
            .enumerate()
            .map(|(i, s)| Run {
                text: s.text().to_string(),
                attr: if i == c.focus { Attr::Focus } else { Attr::Converted },
            })
            .collect();
        fx.push(Effect::Preedit(runs));
        fx.push(Effect::Popup(c.open.then(|| self.popup(c))));
    }

    fn popup(&self, c: &Conv) -> Popup {
        let seg = &c.segs[c.focus];
        let start = seg.sel / PAGE_SIZE * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(seg.cands.len());
        Popup {
            items: seg.cands[start..end].iter().map(|k| (k.text.clone(), k.note.clone())).collect(),
            selected: seg.sel - start,
            position: seg.sel + 1,
            total: seg.cands.len(),
            horizontal: self.horizontal,
        }
    }
}

fn is_hangul(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
}

/// Segments the focus can visit: a real choice, or Hangul to resize.
fn focusable(s: &Seg) -> bool {
    s.cands.len() > 1 || s.reading.chars().any(is_hangul)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Splits the input at spaces; every word has `n` candidates "<word>1"..;
    /// a word starting with '_' has one.
    struct Stub(usize);

    impl Converter for Stub {
        fn segments(&self, input: &str) -> Vec<Segment> {
            let mut out = Vec::new();
            for (i, w) in input.split(' ').enumerate() {
                if i > 0 {
                    out.push(Segment {
                        reading: " ".into(),
                        candidates: vec![Candidate { text: " ".into(), note: String::new() }],
                    });
                }
                if w.is_empty() {
                    continue;
                }
                let n = if w.starts_with('_') { 1 } else { self.0 };
                out.push(Segment {
                    reading: w.into(),
                    candidates: (1..=n)
                        .map(|k| Candidate { text: format!("{w}{k}"), note: format!("n{k}") })
                        .collect(),
                });
            }
            out
        }
    }

    fn letter(ch: char) -> Key {
        Key::Letter { ch, shift: false }
    }

    fn type_str(s: &mut Session, keys: &str) {
        for ch in keys.chars() {
            s.handle(letter(ch), &Stub(3));
        }
    }

    fn commit_of(o: &Output) -> Option<String> {
        o.effects.iter().find_map(|e| match e {
            Effect::Commit(t) => Some(t.clone()),
            _ => None,
        })
    }

    fn preedit_of(o: &Output) -> Option<Vec<Run>> {
        o.effects.iter().rev().find_map(|e| match e {
            Effect::Preedit(r) => Some(r.clone()),
            _ => None,
        })
    }

    fn popup_of(o: &Output) -> Option<Option<Popup>> {
        o.effects.iter().rev().find_map(|e| match e {
            Effect::Popup(p) => Some(p.clone()),
            _ => None,
        })
    }

    fn seokdok() -> Session {
        Session::new(Mode::Seokdok, true)
    }

    #[test]
    fn typing_builds_a_hangul_preedit() {
        let mut s = seokdok();
        let o = s.handle(letter('g'), &Stub(3));
        assert!(o.eaten);
        assert_eq!(preedit_of(&o).unwrap(), vec![Run { text: "ㅎ".into(), attr: Attr::Input }]);
        type_str(&mut s, "ks");
        assert_eq!(s.buffer(), "한");
        assert!(s.is_composing());
    }

    #[test]
    fn idle_passes_non_letters_through() {
        let s = seokdok();
        for k in [Key::Space { shift: false }, Key::Enter, Key::Esc, Key::Backspace, Key::Printable('1'), Key::Left { shift: false }, Key::Convert] {
            assert!(!s.eats(k), "{k:?}");
        }
        assert!(s.eats(letter('a')));
    }

    #[test]
    fn space_converts_and_underlines_per_segment() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        assert!(o.eaten);
        assert_eq!(preedit_of(&o).unwrap(), vec![Run { text: "한1".into(), attr: Attr::Focus }]);
        assert_eq!(popup_of(&o), Some(None));
    }

    #[test]
    fn second_space_cycles_and_opens_the_window() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한2");
        let p = popup_of(&o).unwrap().unwrap();
        assert_eq!(p.items.len(), 3);
        assert_eq!(p.selected, 1);
        assert_eq!((p.position, p.total), (2, 3));
        assert_eq!(p.items[1], ("한2".to_string(), "n2".to_string()));
        // down wraps around, up goes back
        s.handle(Key::Down, &Stub(3));
        let o = s.handle(Key::Down, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한1");
        let o = s.handle(Key::Up, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한3");
    }

    #[test]
    fn segments_get_focus_and_skip_unambiguous_ones() {
        let mut s = seokdok();
        // "a _b c" after Hangul typing is irrelevant; use the buffer directly.
        s.composer = Composer::from_text("a _b c");
        let o = s.handle(Key::Space { shift: false }, &Stub(2));
        let runs = preedit_of(&o).unwrap();
        let texts: Vec<_> = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, ["a1", " ", "_b1", " ", "c1"]);
        assert_eq!(runs[0].attr, Attr::Focus);
        assert_eq!(runs[2].attr, Attr::Converted);
        // Right skips the space and the one-candidate segment.
        let o = s.handle(Key::Right { shift: false }, &Stub(2));
        let runs = preedit_of(&o).unwrap();
        assert_eq!(runs[4].attr, Attr::Focus);
        assert_eq!(runs[0].attr, Attr::Converted);
        // Right at the end stays.
        let o = s.handle(Key::Right { shift: false }, &Stub(2));
        assert_eq!(preedit_of(&o).unwrap()[4].attr, Attr::Focus);
        // Cycling changes only the focused segment.
        let o = s.handle(Key::Space { shift: false }, &Stub(2));
        let all: String = preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect();
        assert_eq!(all, "a1 _b1 c2");
        s.handle(Key::Left { shift: false }, &Stub(2));
        let o = s.handle(Key::Enter, &Stub(2));
        assert_eq!(commit_of(&o).unwrap(), "a1 _b1 c2");
        assert!(!s.is_composing());
    }

    #[test]
    fn digits_pick_and_confirm_only_while_the_window_is_open() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(12));
        // window closed: a digit confirms the conversion and goes on as text
        let o = s.handle(Key::Printable('2'), &Stub(12));
        assert_eq!(commit_of(&o).unwrap(), "한1");
        assert_eq!(s.buffer(), "2");

        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(12));
        s.handle(Key::Space { shift: false }, &Stub(12));
        let o = s.handle(Key::Printable('4'), &Stub(12));
        assert_eq!(commit_of(&o).unwrap(), "한4");
        assert!(!s.is_composing());

        // Tab pages by nine; digit 2 then picks the 11th.
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(12));
        let o = s.handle(Key::Tab { shift: false }, &Stub(12));
        let p = popup_of(&o).unwrap().unwrap();
        assert_eq!((p.items.len(), p.position), (3, 10));
        let o = s.handle(Key::Printable('2'), &Stub(12));
        assert_eq!(commit_of(&o).unwrap(), "한11");
        let o = s.handle(Key::PageUp, &Stub(12));
        assert!(!o.eaten || o.effects.is_empty());
    }

    #[test]
    fn a_pick_confirms_through_the_focused_segment_only() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: true }, &Stub(3));
        type_str(&mut s, "rk");
        s.handle(Key::Space { shift: false }, &Stub(3));
        s.handle(Key::Down, &Stub(3));
        let o = s.handle(Key::Printable('2'), &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "한2");
        // " 가" stays converting with focus on 가
        let runs = preedit_of(&o).unwrap();
        assert_eq!(runs.iter().map(|r| r.text.as_str()).collect::<String>(), " 가1");
        let o = s.handle(Key::Enter, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), " 가1");
    }

    #[test]
    fn shift_arrows_resize_the_segment() {
        let mut s = seokdok();
        type_str(&mut s, "gksrk");
        s.handle(Key::Space { shift: false }, &Stub(3));
        let o = s.handle(Key::Left { shift: true }, &Stub(3));
        let texts: Vec<String> = preedit_of(&o).unwrap().into_iter().map(|r| r.text).collect();
        assert_eq!(texts, ["한1", "가1"]);
        let o = s.handle(Key::Right { shift: true }, &Stub(3));
        let texts: Vec<String> = preedit_of(&o).unwrap().into_iter().map(|r| r.text).collect();
        assert_eq!(texts, ["한가1"]);
    }

    #[test]
    fn a_resized_word_keeps_its_full_candidate_list() {
        // Whole-phrase conversion stops after one candidate, as the engine's
        // stops after max_candidates.
        struct Short(Stub);
        impl Converter for Short {
            fn segments(&self, input: &str) -> Vec<Segment> {
                self.0.segments(input)
            }
            fn whole(&self, input: &str) -> Vec<Candidate> {
                self.0.segments(input).into_iter().flat_map(|s| s.candidates).take(1).collect()
            }
        }
        let c = Short(Stub(5));
        let mut s = seokdok();
        for ch in "gksrk".chars() {
            s.handle(letter(ch), &c);
        }
        s.handle(Key::Space { shift: false }, &c);
        s.handle(Key::Left { shift: true }, &c);
        s.handle(Key::Space { shift: false }, &c);
        let o = s.handle(Key::Space { shift: false }, &c);
        let p = popup_of(&o).unwrap().unwrap();
        assert_eq!(p.total, 6, "{:?}", p.items);
    }

    #[test]
    fn horizontal_window_walks_with_arrows() {
        let mut s = seokdok();
        s.set_horizontal(true);
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        assert!(popup_of(&o).unwrap().unwrap().horizontal);
        let o = s.handle(Key::Right { shift: false }, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한3");
        let o = s.handle(Key::Left { shift: false }, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한2");
    }

    #[test]
    fn a_word_break_follows_enter_after_a_space_conversion() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        s.handle(Key::Enter, &Stub(3));
        let o = s.handle(letter('r'), &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), " ");
        assert_eq!(s.buffer(), "ㄱ");
    }
    #[test]
    fn enter_commits_raw_hangul_or_conversion() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        let o = s.handle(Key::Enter, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "한");
        assert!(!s.is_composing());
    }

    #[test]
    fn escape_closes_the_window_then_reverts_then_cancels() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        s.handle(Key::Space { shift: false }, &Stub(3));
        let o = s.handle(Key::Esc, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].attr, Attr::Focus);
        assert_eq!(popup_of(&o), Some(None));
        let o = s.handle(Key::Esc, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap(), vec![Run { text: "한".into(), attr: Attr::Input }]);
        let o = s.handle(Key::Esc, &Stub(3));
        assert!(o.effects.contains(&Effect::Cancel));
        assert!(!s.is_composing());
    }
    #[test]
    fn backspace_edits_and_reverts() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Backspace, &Stub(3));
        assert_eq!(s.buffer(), "하");
        s.handle(Key::Space { shift: false }, &Stub(3));
        let o = s.handle(Key::Backspace, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "하");
        s.handle(Key::Backspace, &Stub(3));
        let o = s.handle(Key::Backspace, &Stub(3));
        assert!(o.effects.contains(&Effect::Cancel));
        assert!(!s.is_composing());
    }

    #[test]
    fn a_new_key_commits_the_conversion_and_starts_over() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        // A Space conversion marks a word break: the commit ends in a space.
        let o = s.handle(letter('r'), &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "한1 ");
        assert_eq!(s.buffer(), "ㄱ");
        assert_eq!(preedit_of(&o).unwrap()[0].text, "ㄱ");
        // Converted with Tab: no space.
        let mut t = seokdok();
        type_str(&mut t, "gks");
        t.handle(Key::Tab { shift: false }, &Stub(3));
        let o = t.handle(letter('r'), &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "한1");
    }
    #[test]
    fn shift_space_inserts_a_space_then_steps_back() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        let o = s.handle(Key::Space { shift: true }, &Stub(3));
        assert_eq!(s.buffer(), "한 ");
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한 ");
        // converted: Shift+Space is the previous candidate
        let mut t = seokdok();
        type_str(&mut t, "gks");
        t.handle(Key::Space { shift: false }, &Stub(3));
        let o = t.handle(Key::Space { shift: true }, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한3");
    }
    #[test]
    fn space_converts_off() {
        let mut s = Session::new(Mode::Seokdok, false);
        type_str(&mut s, "gks");
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        assert_eq!(s.buffer(), "한 ");
        assert_eq!(preedit_of(&o).unwrap()[0].attr, Attr::Input);
        for k in [Key::Convert, Key::Tab { shift: false }, Key::Down] {
            let mut t = Session::new(Mode::Seokdok, false);
            type_str(&mut t, "gks");
            let o = t.handle(k, &Stub(3));
            assert_eq!(preedit_of(&o).unwrap()[0].attr, Attr::Focus, "{k:?}");
        }
        // converted: Space confirms and types a space
        let mut t = Session::new(Mode::Seokdok, false);
        type_str(&mut t, "gks");
        t.handle(Key::Convert, &Stub(3));
        let o = t.handle(Key::Space { shift: false }, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "한1 ");
    }
    #[test]
    fn hanja_and_f7_convert() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        let o = s.handle(Key::Convert, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].attr, Attr::Focus);
    }

    #[test]
    fn punctuation_joins_the_composition() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Printable(','), &Stub(3));
        assert_eq!(s.buffer(), "한,");
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        let all: String = preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect();
        assert_eq!(all, "한,1");
    }

    #[test]
    fn hangul_mode_never_converts() {
        let mut s = Session::new(Mode::Hangul, true);
        type_str(&mut s, "gks");
        for k in [Key::Convert, Key::Tab { shift: false }, Key::Down, Key::Left { shift: false }, Key::Other] {
            let o = s.handle(k, &Stub(3));
            assert!(o.eaten && o.effects.is_empty(), "{k:?}");
        }
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "한 ");
        type_str(&mut s, "rk");
        let o = s.handle(Key::Printable('.'), &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "가.");
        type_str(&mut s, "rk");
        let o = s.handle(Key::Enter, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "가");
        assert!(!s.eats(Key::Enter));
    }

    fn hangul_only() -> Session {
        let mut s = Session::new(Mode::Seokdok, true);
        s.set_hangul_only(true);
        s
    }

    #[test]
    fn hangul_only_has_no_converting_mode() {
        let mut s = hangul_only();
        assert_eq!(s.mode(), Mode::Hangul);
        assert!(s.set_mode(Mode::Seokdok).is_empty());
        assert_eq!(s.mode(), Mode::Hangul);
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Latin);
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Hangul);
        s.set_hangul_only(false);
        assert_eq!(s.set_mode(Mode::Seokdok).len(), 0);
        assert_eq!(s.mode(), Mode::Seokdok);
    }

    #[test]
    fn hangul_only_types_old_hangul_and_commits_on_space_and_enter() {
        let mut s = hangul_only();
        s.set_old_clusters(true);
        type_str(&mut s, "dkssud");
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "안녕 ");
        assert!(!matches!(popup_of(&o), Some(Some(_))));
        assert!(!s.is_composing());
        type_str(&mut s, "rk");
        for k in [Key::Convert, Key::Tab { shift: false }, Key::Down] {
            let o = s.handle(k, &Stub(3));
            assert!(o.eaten && o.effects.is_empty(), "{k:?}");
        }
        let o = s.handle(Key::Space { shift: true }, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "가 ");
        type_str(&mut s, "rk");
        let o = s.handle(Key::Enter, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "가");
        assert!(!s.eats(Key::Space { shift: false }));
        assert!(!s.eats(Key::Enter));
        // Old letters stay available: Shift+D is ㆁ.
        let o = s.handle(Key::Letter { ch: 'd', shift: true }, &Stub(3));
        assert_eq!(preedit_of(&o).unwrap()[0].text, "ㆁ");
    }

    #[test]
    fn hangul_only_ignores_space_converts() {
        for space_converts in [true, false] {
            let mut s = Session::new(Mode::Seokdok, space_converts);
            s.set_hangul_only(true);
            type_str(&mut s, "gks");
            let o = s.handle(Key::Space { shift: false }, &Stub(3));
            assert_eq!(commit_of(&o).unwrap(), "한 ");
        }
    }

    #[test]
    fn latin_mode_passes_everything_but_the_toggle() {
        let mut s = Session::new(Mode::Latin, true);
        for k in [letter('a'), Key::Space { shift: false }, Key::Enter, Key::Printable('1'), Key::Convert] {
            assert!(!s.eats(k));
        }
        assert!(s.eats(Key::HangulToggle));
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Seokdok);
    }

    #[test]
    fn toggle_commits_and_remembers_the_hangul_mode() {
        let mut s = Session::new(Mode::Hangul, true);
        type_str(&mut s, "gks");
        let o = s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(commit_of(&o).unwrap(), "한");
        assert_eq!(s.mode(), Mode::Latin);
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Hangul);
    }

    #[test]
    fn set_mode_commits_a_conversion() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        let fx = s.set_mode(Mode::Hangul);
        assert_eq!(fx[0], Effect::Commit("한1".into()));
        assert_eq!(s.mode(), Mode::Hangul);
        assert!(!s.is_composing());
    }

    #[test]
    fn eats_agrees_with_handle() {
        let keys = [
            letter('a'),
            Key::Letter { ch: 'q', shift: true },
            Key::Printable('1'),
            Key::Printable('!'),
            Key::Space { shift: false },
            Key::Space { shift: true },
            Key::Enter,
            Key::Esc,
            Key::Backspace,
            Key::Tab { shift: false },
            Key::Left { shift: false },
            Key::Right { shift: false },
            Key::Up,
            Key::Down,
            Key::Convert,
            Key::HangulToggle,
            Key::Other,
        ];
        for mode in [Mode::Seokdok, Mode::Hangul, Mode::Latin] {
            for space_converts in [true, false] {
                for prep in 0..3 {
                    for k in keys {
                        let mut s = Session::new(mode, space_converts);
                        if mode != Mode::Latin && prep >= 1 {
                            type_str(&mut s, "gks");
                        }
                        if mode == Mode::Seokdok && prep == 2 {
                            s.handle(Key::Convert, &Stub(3));
                            s.handle(Key::Down, &Stub(3));
                        }
                        let predicted = s.eats(k);
                        let o = s.handle(k, &Stub(3));
                        assert_eq!(o.eaten, predicted, "{mode:?} {space_converts} {prep} {k:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn unconvertible_input_stays_in_the_buffer() {
        struct Empty;
        impl Converter for Empty {
            fn segments(&self, _: &str) -> Vec<Segment> {
                Vec::new()
            }
        }
        let mut s = seokdok();
        type_str(&mut s, "gks");
        let o = s.handle(Key::Space { shift: false }, &Empty);
        assert!(o.eaten && o.effects.is_empty());
        assert_eq!(s.buffer(), "한");
    }

    #[test]
    fn real_engine_end_to_end() {
        use crate::engine_host::load_engine;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        let (engine, _) = load_engine(&dir, None, seokdok_core::Options::default());
        struct E(seokdok_core::Engine);
        impl Converter for E {
            fn segments(&self, input: &str) -> Vec<Segment> {
                self.0.segments(input)
            }
        }
        let c = E(engine);
        let mut s = seokdok();
        // 안녕하세요 on 두벌식: dkssudgktpdy
        for ch in "dkssudgktpdy".chars() {
            s.handle(letter(ch), &c);
        }
        assert_eq!(s.buffer(), "안녕하세요");
        let o = s.handle(Key::Space { shift: false }, &c);
        let converted: String = preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect();
        assert!(!converted.is_empty());
        let o = s.handle(Key::Enter, &c);
        assert_eq!(commit_of(&o).unwrap(), converted);
    }
}
