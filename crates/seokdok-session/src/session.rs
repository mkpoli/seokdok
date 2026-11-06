//! The input session: a platform-independent state machine from [`Key`] to
//! [`Effect`]s. The TSF layer applies the effects to the document and the
//! candidate window; nothing here touches Windows.
//!
//! Phases: idle (no composition), composing (Hangul in the buffer), and
//! converted (the buffer shown as 釋讀 segments, one of them focused).

use seokdok_core::dubeolsik::Composer;
use seokdok_core::jamo::{is_hangul, units};
use seokdok_core::{Candidate, Segment};
use std::sync::Arc;

use crate::keys::Key;
use crate::learned::Store;

/// Candidates shown per page of the candidate window; digits 1-9 pick one.
pub const PAGE_SIZE: usize = 9;

/// Every candidate of the focused segment, for a front end that scrolls the
/// list instead of paging it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateList {
    /// (text, short reading) of each candidate; see [`crate::brief`].
    pub items: Vec<(String, String)>,
    /// Index of the selected candidate within `items`.
    pub selected: usize,
    /// Whether the focused segment can give up a syllable, or take one from
    /// the next.
    pub can_shorten: bool,
    pub can_lengthen: bool,
}

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
    /// Shrink (-1) or extend (+1) the focused segment by one syllable unit.
    Resize(isize),
    /// Close the candidate window, keeping the conversion.
    CloseWindow,
    /// Switch to this mode; nothing happens when it is the current one.
    SwitchTo(Mode),
    /// Commit, then 釋讀 mode: the script changes, so this happens even in it.
    EnterScript,
    /// Commit the composition and leave the key to the application.
    CommitPass,
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
    /// What the person keeps picking; `None` when `input.learn` is off.
    learned: Option<Arc<Store>>,
    /// The field wants nothing remembered: a password, an address, a number.
    private_field: bool,
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
            learned: None,
            private_field: false,
        }
    }

    /// The mode in effect: the requested one, except that 韓㐎 專用 turns
    /// 釋讀 into plain Hangul.
    pub fn mode(&self) -> Mode {
        if self.hangul_only && self.mode == Mode::Seokdok {
            Mode::Hangul
        } else {
            self.mode
        }
    }

    pub fn set_space_converts(&mut self, v: bool) {
        self.space_converts = v;
    }

    /// 韓㐎 專用: nothing is converted. The requested mode is kept, so turning
    /// it off brings conversion back. A composition or conversion in progress
    /// is committed as it stands.
    pub fn set_hangul_only(&mut self, v: bool) -> Vec<Effect> {
        let mut fx = Vec::new();
        if v == self.hangul_only {
            return fx;
        }
        if self.is_composing() {
            fx.push(Effect::Commit(self.take_text()));
            fx.push(Effect::Popup(None));
        }
        self.hangul_only = v;
        fx
    }

    pub fn set_horizontal(&mut self, v: bool) {
        self.horizontal = v;
    }

    /// Let modern letters form old-Hangul clusters (ㅅ+ㄱ → ᄭ, ㅂ+ㅇ → ㅸ).
    pub fn set_old_clusters(&mut self, v: bool) {
        self.composer.set_old(v);
    }

    /// The table of picks to rank by and add to; `None` for neither.
    pub fn set_learned(&mut self, store: Option<Arc<Store>>) {
        self.learned = store;
    }

    /// The field in focus is a password, e-mail, URL or number field, or asks
    /// not to be learned from. In it the table is neither read nor added to.
    /// A front end that keeps such fields from the session needs no call.
    pub fn set_private_field(&mut self, private: bool) {
        self.private_field = private;
    }

    /// The table, unless the field in focus rules it out.
    fn learner(&self) -> Option<Arc<Store>> {
        self.learned.as_ref().filter(|_| !self.private_field).cloned()
    }

    /// True while a composition exists.
    pub fn is_composing(&self) -> bool {
        self.conv.is_some() || !self.composer.is_empty()
    }

    /// The Hangul typed so far.
    pub fn buffer(&self) -> &str {
        self.composer.text()
    }

    /// In plain Hangul, the syllables before the one being typed and that one,
    /// for a surface that shows one syllable at a time. Once the front end has
    /// put the finished ones in the document, it calls [`Session::keep_typing`].
    pub fn finished_syllables(&self) -> Option<(String, String)> {
        if self.mode() != Mode::Hangul || self.conv.is_some() {
            return None;
        }
        let text = self.composer.text();
        let last = *units(text).last()?;
        let finished = text.strip_suffix(last)?;
        (!finished.is_empty()).then(|| (finished.to_string(), last.to_string()))
    }

    /// The application has the text before `last`; compose only `last` from now on.
    pub fn keep_typing(&mut self, last: &str) {
        self.composer.set_text(last);
    }

    /// Would [`Session::handle`] consume this key?
    pub fn eats(&self, key: Key) -> bool {
        !matches!(self.plan(key), None | Some(Act::CommitPass))
    }

    /// Does this key commit the composition and then go to the application?
    /// The front end must apply the effects of [`Session::handle`] before it
    /// passes the key on.
    pub fn commits_then_passes(&self, key: Key) -> bool {
        self.plan(key) == Some(Act::CommitPass)
    }

    /// A key went to the application without reaching [`Session::handle`], as
    /// a front end that asks [`Session::eats`] first lets it. The application's
    /// text has moved on, so the next word adds no space of its own.
    pub fn key_passed(&mut self) {
        self.pending_break = false;
    }

    /// The mode a switching key leads to from here; `None` for other keys.
    /// 韓㐎 專用 has no 釋讀 to tell from 한글, so those keys only leave Latin.
    fn switch_target(&self, key: Key) -> Option<Mode> {
        let latin = self.mode == Mode::Latin;
        Some(match key {
            Key::HangulToggle if latin => self.hangul_mode,
            Key::HangulToggle | Key::ToLatin => Mode::Latin,
            Key::ToKorean if latin => self.hangul_mode,
            Key::ToKorean => self.mode,
            Key::ToSeokdok | Key::ToHangul if self.hangul_only && !latin => self.mode,
            Key::ToSeokdok => Mode::Seokdok,
            Key::ToHangul => Mode::Hangul,
            _ => return None,
        })
    }

    fn plan(&self, key: Key) -> Option<Act> {
        if key == Key::CycleScript {
            return Some(Act::EnterScript);
        }
        if let Some(target) = self.switch_target(key) {
            return Some(Act::SwitchTo(target));
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
                Key::HangulToggle | Key::ToLatin | Key::ToKorean | Key::CycleScript | Key::ToSeokdok | Key::ToHangul => unreachable!(),
            });
        }
        if self.composer.is_empty() {
            return match key {
                Key::Letter { .. } => Some(Act::Insert(jamo.unwrap())),
                _ => None,
            };
        }
        let converts = self.mode() == Mode::Seokdok;
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
            Key::Tab { .. } | Key::Down | Key::Convert => Act::Noop,
            // A plain keyboard: the composition is final, the key then acts
            // on the text as usual.
            Key::Up | Key::Left { .. } | Key::Right { .. } | Key::PageUp | Key::PageDown | Key::Other if !converts => Act::CommitPass,
            Key::Up | Key::Left { .. } | Key::Right { .. } | Key::PageUp | Key::PageDown | Key::Other => Act::Noop,
            Key::HangulToggle | Key::ToLatin | Key::ToKorean | Key::CycleScript | Key::ToSeokdok | Key::ToHangul => unreachable!(),
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
            Act::CommitPass => {
                fx.push(Effect::Commit(self.take_text()));
                fx.push(Effect::Popup(None));
            }
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
            Act::SwitchTo(next) => {
                if next != self.mode {
                    fx.extend(self.set_mode(next));
                }
            }
            Act::EnterScript => fx.extend(self.set_mode(Mode::Seokdok)),
        }
        Output { eaten: act != Act::CommitPass, effects: fx }
    }

    /// Confirm the segments up to the focused one; the rest stay converting.
    fn commit_through(&mut self, fx: &mut Vec<Effect>) {
        let Some(c) = self.conv.take() else { return };
        let (done, rest) = c.segs.split_at(c.focus + 1);
        let mut text: String = done.iter().map(Seg::text).collect();
        let left: String = rest.iter().map(|s| s.reading.as_str()).collect();
        fx.push(Effect::Popup(None));
        self.learn_from(done);
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
        let learner = self.learner();
        let Some(c) = &mut self.conv else { return };
        let i = c.focus;
        let cur = units(&c.segs[i].reading);
        let next = c.segs.get(i + 1).map(|s| units(&s.reading)).unwrap_or_default();
        if (d < 0 && cur.len() <= 1) || (d > 0 && next.is_empty()) {
            return;
        }
        let (a, b) = if d < 0 {
            let (last, kept) = cur.split_last().unwrap();
            (kept.concat(), [*last, &next.concat()].concat())
        } else {
            ([&cur.concat(), next[0]].concat(), next[1..].concat())
        };
        let mut fresh = vec![span(conv, &a)];
        if !b.is_empty() {
            fresh.push(span(conv, &b));
        }
        if let Some(l) = &learner {
            for seg in &mut fresh {
                l.rank(&seg.reading, &mut seg.cands);
            }
        }
        let end = (i + 2).min(c.segs.len());
        c.segs.splice(i..end, fresh);
        c.focus = i;
        c.open = false;
        self.push_view(fx);
    }

    /// The focused segment's whole list, or `None` while nothing is converted.
    pub fn candidate_list(&self) -> Option<CandidateList> {
        let c = self.conv.as_ref()?;
        let seg = &c.segs[c.focus];
        let brief = crate::brief::brief_notes(&seg.reading, seg.cands.iter().map(|k| k.note.as_str()));
        let items = seg.cands.iter().zip(brief).map(|(k, b)| (k.text.clone(), b)).collect();
        Some(CandidateList {
            items,
            selected: seg.sel,
            can_shorten: units(&seg.reading).len() > 1,
            can_lengthen: c.segs.get(c.focus + 1).is_some(),
        })
    }

    /// Choose candidate `index` of [`Session::candidate_list`] and confirm the
    /// segments through the focused one.
    pub fn pick_candidate(&mut self, index: usize) -> Vec<Effect> {
        let mut fx = Vec::new();
        let Some(c) = &mut self.conv else { return fx };
        let seg = &mut c.segs[c.focus];
        if index < seg.cands.len() {
            seg.sel = index;
            self.commit_through(&mut fx);
        }
        fx
    }

    /// Switch mode, committing any composition as it stands.
    pub fn set_mode(&mut self, mode: Mode) -> Vec<Effect> {
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

    /// Forget the composition without effects (the document ended it, or the
    /// focus moved); what comes next is unrelated, so no word break is owed.
    pub fn reset(&mut self) {
        self.composer.clear();
        self.conv = None;
        self.pending_break = false;
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
            self.learn_from(&c.segs);
        }
        self.composer.clear();
        self.conv = None;
        text
    }

    fn input_preedit(&self) -> Effect {
        Effect::Preedit(vec![Run { text: self.composer.text().to_string(), attr: Attr::Input }])
    }

    fn convert(&mut self, conv: &dyn Converter, spaced: bool, fx: &mut Vec<Effect>) {
        let mut segs: Vec<Seg> = Vec::new();
        for s in conv.segments(self.composer.text()) {
            let seg = match segs.last() {
                // A syllable unit never spans two segments.
                Some(last) if splits_unit(&last.reading, &s.reading) => {
                    let joined = segs.pop().unwrap().reading + &s.reading;
                    span(conv, &joined)
                }
                _ if s.candidates.is_empty() => span(conv, &s.reading),
                _ => Seg { reading: s.reading, cands: s.candidates, sel: 0 },
            };
            segs.push(seg);
        }
        if segs.is_empty() {
            return;
        }
        if let Some(l) = self.learner() {
            l.refresh();
            segs = with_learned(&l, conv, segs);
        }
        let focus = segs.iter().position(focusable).unwrap_or(0);
        self.conv = Some(Conv { segs, focus, open: false, spaced });
        self.push_view(fx);
    }

    /// These segments are being committed: tell the table what was chosen.
    fn learn_from(&self, segs: &[Seg]) {
        let Some(l) = self.learner() else { return };
        for s in segs.iter().filter(|s| s.cands.len() > 1) {
            l.commit(&s.reading, s.text(), s.sel == 0);
        }
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

/// Put what was learned into a fresh conversion. An empty table, or a reading
/// it does not hold, leaves the engine's segments exactly as they came.
fn with_learned(l: &Store, conv: &dyn Converter, segs: Vec<Seg>) -> Vec<Seg> {
    if l.is_empty() {
        return segs;
    }
    let readings: Vec<&str> = segs.iter().map(|s| s.reading.as_str()).collect();
    let joins = l.joins(&readings);
    let mut out = Vec::with_capacity(segs.len());
    let mut at = 0;
    for run in &joins {
        out.extend_from_slice(&segs[at..run.start]);
        let joined: String = readings[run.clone()].concat();
        let mut seg = span(conv, &joined);
        if l.rank(&joined, &mut seg.cands) {
            out.push(seg);
        } else {
            // The learned form is not on offer now, so the segments stay apart.
            out.extend_from_slice(&segs[run.clone()]);
        }
        at = run.end;
    }
    out.extend_from_slice(&segs[at..]);
    for i in 0..out.len() {
        if is_fragment(&out, i) {
            continue;
        }
        let seg = &mut out[i];
        if l.promotes(&seg.reading) && !l.rank(&seg.reading, &mut seg.cands) {
            // A segment's list can be cut short; the whole list may hold it.
            let mut full = span(conv, &seg.reading);
            if l.rank(&seg.reading, &mut full.cands) {
                *seg = full;
            }
        }
    }
    out
}

/// A piece of a longer run of Hangul that the engine left in Hangul: the 나 of
/// 나봐요 or 나올겁니다. It is the rest of a word the engine did not recognise,
/// and a form learned for the word 나 would cut it.
fn is_fragment(segs: &[Seg], i: usize) -> bool {
    let joins_before = i > 0 && segs[i - 1].reading.chars().last().is_some_and(is_hangul);
    let joins_after = segs.get(i + 1).is_some_and(|s| s.reading.chars().next().is_some_and(is_hangul));
    (joins_before || joins_after) && segs[i].cands.first().is_some_and(|c| c.text == segs[i].reading)
}

/// A segment for exactly this span of the input.
fn span(conv: &dyn Converter, t: &str) -> Seg {
    // Whole-phrase readings first; a single word also brings its full
    // candidate list, which the phrase list cuts short.
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
}

/// Would a boundary between these two readings fall inside a syllable unit?
fn splits_unit(a: &str, b: &str) -> bool {
    units(&[a, b].concat()).len() < units(a).len() + units(b).len()
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

    /// A session over a table in memory.
    fn learning(store: &Arc<Store>) -> Session {
        let mut s = seokdok();
        s.set_learned(Some(store.clone()));
        s
    }

    /// Type 한, convert, and return the first conversion's text with the session.
    fn converted_han(s: &mut Session) -> String {
        type_str(s, "gks");
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect()
    }

    /// Convert 한, step to candidate `n` (0-based) and commit it with Enter.
    fn commit_candidate(s: &mut Session, n: usize) -> String {
        converted_han(s);
        for _ in 0..n {
            s.handle(Key::Down, &Stub(3));
        }
        let o = s.handle(Key::Enter, &Stub(3));
        commit_of(&o).unwrap().trim_start().to_string()
    }

    #[test]
    fn a_form_picked_twice_comes_first_and_one_pick_changes_nothing() {
        let store = Store::open(None);
        let mut s = learning(&store);
        assert_eq!(commit_candidate(&mut s, 2), "한3");
        s.reset();
        assert_eq!(converted_han(&mut s), "한1", "one pick changed the order");
        s.reset();
        assert_eq!(commit_candidate(&mut s, 2), "한3");
        s.reset();
        assert_eq!(converted_han(&mut s), "한3");
        // The whole list is still there, with the pick in front.
        let o = s.handle(Key::Down, &Stub(3));
        let items: Vec<String> = popup_of(&o).unwrap().unwrap().items.into_iter().map(|i| i.0).collect();
        assert_eq!(items, ["한3", "한1", "한2"]);
    }

    #[test]
    fn digits_and_a_flush_count_as_picks_and_cancelling_does_not() {
        let store = Store::open(None);
        let mut s = learning(&store);
        converted_han(&mut s);
        s.handle(Key::Space { shift: false }, &Stub(3));
        s.handle(Key::Printable('2'), &Stub(3));
        assert!(!store.promotes("한"));
        s.reset();
        converted_han(&mut s);
        s.handle(Key::Down, &Stub(3));
        s.flush();
        assert!(store.promotes("한"));

        let store = Store::open(None);
        let mut s = learning(&store);
        for _ in 0..3 {
            converted_han(&mut s);
            s.handle(Key::Down, &Stub(3));
            s.handle(Key::Esc, &Stub(3));
            s.handle(Key::Esc, &Stub(3));
            s.reset();
        }
        assert!(store.is_empty());
    }

    #[test]
    fn taking_the_first_candidate_of_an_unpicked_reading_stores_nothing() {
        let store = Store::open(None);
        let mut s = learning(&store);
        assert_eq!(commit_candidate(&mut s, 0), "한1");
        assert!(store.is_empty());
    }

    #[test]
    fn a_pick_counts_for_its_own_segment_only() {
        let store = Store::open(None);
        let mut s = learning(&store);
        for _ in 0..2 {
            s.composer = Composer::from_text("가 나");
            s.handle(Key::Space { shift: false }, &Stub(3));
            s.handle(Key::Right { shift: false }, &Stub(3));
            s.handle(Key::Down, &Stub(3));
            let o = s.handle(Key::Enter, &Stub(3));
            assert_eq!(commit_of(&o).unwrap(), "가1 나2");
            s.reset();
        }
        s.composer = Composer::from_text("가 나");
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        let all: String = preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect();
        assert_eq!(all, "가1 나2");
    }

    #[test]
    fn a_span_learned_by_resizing_comes_back_as_one_and_leaves_longer_words() {
        // Segments of one syllable; a span of several has a word of its own.
        struct Syllables;
        impl Converter for Syllables {
            fn segments(&self, input: &str) -> Vec<Segment> {
                input
                    .chars()
                    .map(|c| Segment {
                        reading: c.to_string(),
                        candidates: vec![
                            Candidate { text: format!("{c}1"), note: String::new() },
                            Candidate { text: format!("{c}2"), note: String::new() },
                        ],
                    })
                    .collect()
            }
            fn whole(&self, input: &str) -> Vec<Candidate> {
                vec![
                    Candidate { text: format!("whole-{input}"), note: String::new() },
                    Candidate { text: format!("word-{input}"), note: String::new() },
                ]
            }
        }
        let c = Syllables;
        let store = Store::open(None);
        let mut s = learning(&store);
        for _ in 0..2 {
            s.composer = Composer::from_text("한가");
            s.handle(Key::Space { shift: false }, &c);
            s.handle(Key::Right { shift: true }, &c);
            s.handle(Key::Down, &c);
            let o = s.handle(Key::Enter, &c);
            assert_eq!(commit_of(&o).unwrap(), "word-한가");
            s.reset();
        }
        // The two segments now join, with the pick first.
        s.composer = Composer::from_text("한가");
        let o = s.handle(Key::Space { shift: false }, &c);
        assert_eq!(preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect::<Vec<_>>(), ["word-한가"]);
        s.reset();
        // Other readings do not.
        s.composer = Composer::from_text("한나");
        let o = s.handle(Key::Space { shift: false }, &c);
        assert_eq!(preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect::<Vec<_>>(), ["한1", "나1"]);
        s.reset();
        // A segment that merely contains the reading stays as it is.
        struct OneWord;
        impl Converter for OneWord {
            fn segments(&self, input: &str) -> Vec<Segment> {
                vec![Segment {
                    reading: input.into(),
                    candidates: vec![
                        Candidate { text: format!("{input}1"), note: String::new() },
                        Candidate { text: format!("word-한가"), note: String::new() },
                    ],
                }]
            }
        }
        s.composer = Composer::from_text("한가위");
        let o = s.handle(Key::Space { shift: false }, &OneWord);
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한가위1");
    }

    #[test]
    fn a_piece_the_engine_left_in_hangul_inside_a_run_stays_as_it_is() {
        // 나봐요 comes as 나 (in Hangul) and 봐요; 나 나 as two words.
        struct Run;
        impl Converter for Run {
            fn segments(&self, input: &str) -> Vec<Segment> {
                let cands = |t: &[&str]| t.iter().map(|x| Candidate { text: x.to_string(), note: String::new() }).collect();
                match input {
                    "나봐요" => vec![
                        Segment { reading: "나".into(), candidates: cands(&["나", "吾", "我"]) },
                        Segment { reading: "봐요".into(), candidates: cands(&["見요"]) },
                    ],
                    "봐요나" => vec![
                        Segment { reading: "봐요".into(), candidates: cands(&["見요"]) },
                        Segment { reading: "나".into(), candidates: cands(&["吾", "我"]) },
                    ],
                    "나 봐요" => vec![
                        Segment { reading: "나".into(), candidates: cands(&["나", "吾", "我"]) },
                        Segment { reading: " ".into(), candidates: cands(&[" "]) },
                        Segment { reading: "봐요".into(), candidates: cands(&["見요"]) },
                    ],
                    _ => vec![Segment { reading: input.into(), candidates: cands(&["나", "吾", "我"]) }],
                }
            }
        }
        let store = Store::open(None);
        store.commit("나", "我", false);
        store.commit("나", "我", false);
        let mut s = learning(&store);
        let show = |s: &mut Session, text: &str| -> String {
            s.composer = Composer::from_text(text);
            let o = s.handle(Key::Space { shift: false }, &Run);
            let shown = preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect();
            s.reset();
            shown
        };
        assert_eq!(show(&mut s, "나봐요"), "나見요");
        // Alone in its word, and where the engine converts it, the pick counts.
        assert_eq!(show(&mut s, "나"), "我");
        assert_eq!(show(&mut s, "나 봐요"), "我 見요");
        assert_eq!(show(&mut s, "봐요나"), "見요我");
    }

    #[test]
    fn a_form_cut_from_the_segments_list_is_fetched_from_the_whole_list() {
        // The segment offers one candidate; the span lists more.
        struct Short;
        impl Converter for Short {
            fn segments(&self, input: &str) -> Vec<Segment> {
                vec![Segment { reading: input.into(), candidates: vec![Candidate { text: "a".into(), note: String::new() }] }]
            }
            fn whole(&self, _: &str) -> Vec<Candidate> {
                vec![Candidate { text: "a".into(), note: String::new() }, Candidate { text: "b".into(), note: String::new() }]
            }
        }
        let store = Store::open(None);
        store.commit("한", "b", false);
        store.commit("한", "b", false);
        let mut s = learning(&store);
        s.composer = Composer::from_text("한");
        let o = s.handle(Key::Space { shift: false }, &Short);
        assert_eq!(preedit_of(&o).unwrap()[0].text, "b");
    }

    #[test]
    fn a_private_field_neither_learns_nor_ranks() {
        let store = Store::open(None);
        let mut s = learning(&store);
        s.set_private_field(true);
        for _ in 0..3 {
            assert_eq!(commit_candidate(&mut s, 1), "한2");
            s.reset();
        }
        assert!(store.is_empty(), "a private field learned");

        s.set_private_field(false);
        for _ in 0..2 {
            commit_candidate(&mut s, 1);
            s.reset();
        }
        assert_eq!(converted_han(&mut s), "한2");
        s.reset();
        s.set_private_field(true);
        assert_eq!(converted_han(&mut s), "한1");
        let before = store.len();
        s.handle(Key::Down, &Stub(3));
        s.handle(Key::Enter, &Stub(3));
        assert_eq!(store.len(), before);
    }

    #[test]
    fn a_field_the_front_end_bypasses_never_reaches_the_table() {
        // Latin mode is how the session itself passes keys on.
        let store = Store::open(None);
        let mut s = Session::new(Mode::Latin, true);
        s.set_learned(Some(store.clone()));
        for ch in "gks".chars() {
            assert!(!s.handle(letter(ch), &Stub(3)).eaten);
        }
        assert!(!s.handle(Key::Enter, &Stub(3)).eaten);
        assert!(store.is_empty());
        // Plain Hangul commits with no candidates to learn from.
        let mut s = Session::new(Mode::Hangul, true);
        s.set_learned(Some(store.clone()));
        type_str(&mut s, "gks");
        s.handle(Key::Enter, &Stub(3));
        assert!(store.is_empty());
    }

    #[test]
    fn off_means_no_table_at_all() {
        let store = Store::open(None);
        store.commit("한", "한2", false);
        store.commit("한", "한2", false);
        let mut s = learning(&store);
        s.set_learned(None);
        assert_eq!(converted_han(&mut s), "한1");
        s.handle(Key::Down, &Stub(3));
        s.handle(Key::Down, &Stub(3));
        s.handle(Key::Enter, &Stub(3));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn forgetting_restores_the_default_order_at_once() {
        let store = Store::open(None);
        let mut s = learning(&store);
        for _ in 0..2 {
            commit_candidate(&mut s, 1);
            s.reset();
        }
        assert_eq!(converted_han(&mut s), "한2");
        s.reset();
        store.forget().unwrap();
        assert_eq!(converted_han(&mut s), "한1");
    }

    #[test]
    fn an_empty_table_changes_no_view_of_any_session() {
        let store = Store::open(None);
        for text in ["한", "가 나", "_b c", "한글 가나"] {
            let run = |learner: Option<Arc<Store>>| {
                let mut s = seokdok();
                s.set_learned(learner);
                s.composer = Composer::from_text(text);
                let mut out = vec![s.handle(Key::Space { shift: false }, &Stub(3))];
                out.push(s.handle(Key::Down, &Stub(3)));
                out.push(s.handle(Key::Up, &Stub(3)));
                out
            };
            assert_eq!(run(Some(store.clone())), run(None), "{text}");
        }
    }

    #[test]
    fn the_real_engine_learns_a_homophone_and_keeps_longer_words() {
        let c = real_engine();
        let store = Store::open(None);
        let mut s = learning(&store);
        let convert = |s: &mut Session, text: &str| -> String {
            s.composer = Composer::from_text(text);
            let o = s.handle(Key::Space { shift: false }, &c);
            preedit_of(&o).unwrap().iter().map(|r| r.text.as_str()).collect()
        };
        let default = convert(&mut s, "하늘");
        s.reset();
        let longer = convert(&mut s, "하늘색");
        s.reset();
        assert_ne!(default, "乾", "pick another homophone for this test");
        for _ in 0..2 {
            convert(&mut s, "하늘");
            let mut steps = 0;
            loop {
                let o = s.handle(Key::Down, &c);
                steps += 1;
                if preedit_of(&o).unwrap()[0].text == "乾" {
                    break;
                }
                assert!(steps < 200, "乾 is not a candidate of 하늘");
            }
            s.handle(Key::Enter, &c);
            s.reset();
        }
        assert_eq!(convert(&mut s, "하늘"), "乾");
        s.reset();
        assert_eq!(convert(&mut s, "하늘색"), longer);
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
    fn a_space_the_application_typed_is_not_doubled() {
        let mut s = seokdok();
        s.composer = Composer::from_text("한");
        s.handle(Key::Space { shift: false }, &Stub(1));
        assert!(commit_of(&s.handle(Key::Enter, &Stub(1))).is_some());
        // The application types this Space: the front end only asked.
        assert!(!s.eats(Key::Space { shift: false }));
        s.key_passed();
        let o = s.handle(Key::Letter { ch: 'g', shift: false }, &Stub(1));
        assert_eq!(commit_of(&o), None);
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
        for k in [Key::Convert, Key::Tab { shift: false }, Key::Down] {
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
    fn hangul_only_reports_hangul_and_keeps_the_requested_mode() {
        let mut s = hangul_only();
        assert_eq!(s.mode(), Mode::Hangul);
        assert!(s.set_mode(Mode::Seokdok).is_empty());
        assert_eq!(s.mode(), Mode::Hangul);
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Latin);
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Hangul);
        s.set_hangul_only(false);
        assert_eq!(s.mode(), Mode::Seokdok);
        // Latin stays Latin through the setting and returns to 釋讀 afterwards.
        s.set_hangul_only(true);
        s.handle(Key::HangulToggle, &Stub(3));
        s.set_hangul_only(false);
        assert_eq!(s.mode(), Mode::Latin);
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Seokdok);
        // A session that asked for plain Hangul stays there.
        let mut h = Session::new(Mode::Hangul, true);
        h.set_hangul_only(true);
        h.set_hangul_only(false);
        assert_eq!(h.mode(), Mode::Hangul);
    }

    #[test]
    fn turning_hangul_only_off_converts_again() {
        // Windows: the session starts in 釋讀, the setting is on at activation,
        // then the user chooses 釋讀 in the settings.
        let mut s = Session::new(Mode::Seokdok, true);
        s.set_hangul_only(true);
        s.set_hangul_only(false);
        type_str(&mut s, "gks");
        let o = s.handle(Key::Space { shift: false }, &Stub(3));
        assert_eq!(commit_of(&o), None);
        assert_eq!(preedit_of(&o).unwrap()[0].text, "한1");
    }

    #[test]
    fn plain_hangul_delivers_all_but_the_syllable_being_typed() {
        let mut s = Session::new(Mode::Hangul, true);
        assert_eq!(s.finished_syllables(), None);
        for c in "eocn".chars() {
            s.handle(letter(c), &Stub(3));
        }
        assert_eq!(s.buffer(), "대추");
        assert_eq!(s.finished_syllables(), Some(("대".to_string(), "추".to_string())));
        assert_eq!(s.buffer(), "대추");
        s.keep_typing("추");
        assert_eq!(s.finished_syllables(), None);
        s.handle(letter('d'), &Stub(3));
        assert_eq!(s.buffer(), "충");
    }

    #[test]
    fn converting_modes_keep_every_syllable() {
        let mut s = Session::new(Mode::Seokdok, true);
        for c in "eocn".chars() {
            s.handle(letter(c), &Stub(3));
        }
        assert_eq!(s.finished_syllables(), None);
    }

    #[test]
    fn hangul_only_flushes_a_composition_or_conversion() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        s.handle(Key::Down, &Stub(3));
        let fx = s.set_hangul_only(true);
        assert_eq!(fx[0], Effect::Commit("한2".into()));
        assert_eq!(fx[1], Effect::Popup(None));
        assert!(!s.is_composing());
        assert!(s.set_hangul_only(true).is_empty());
        type_str(&mut s, "rk");
        let fx = s.set_hangul_only(false);
        assert_eq!(fx[0], Effect::Commit("가".into()));
    }

    #[test]
    fn a_plain_keyboard_commits_before_navigation_keys() {
        for mut s in [Session::new(Mode::Hangul, true), hangul_only()] {
            for k in [
                Key::Left { shift: false },
                Key::Right { shift: true },
                Key::Up,
                Key::PageUp,
                Key::PageDown,
                Key::Other,
            ] {
                type_str(&mut s, "gks");
                assert!(!s.eats(k) && s.commits_then_passes(k), "{k:?}");
                let o = s.handle(k, &Stub(3));
                assert!(!o.eaten, "{k:?}");
                assert_eq!(commit_of(&o).unwrap(), "한", "{k:?}");
                assert!(!s.is_composing());
                // Nothing composing: the key is not touched.
                assert!(!s.commits_then_passes(k));
            }
        }
        // 釋讀 keeps its own navigation.
        let mut s = seokdok();
        type_str(&mut s, "gks");
        assert!(s.eats(Key::Left { shift: false }) && !s.commits_then_passes(Key::Left { shift: false }));
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
    fn after_a_reset_punctuation_goes_to_the_application() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.reset();
        let o = s.handle(Key::Printable(','), &Stub(3));
        assert!(!o.eaten);
        assert!(o.effects.is_empty());
    }

    #[test]
    fn a_reset_owes_no_word_break() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Space { shift: false }, &Stub(3));
        assert_eq!(s.flush()[0], Effect::Commit("한1".into()));
        s.reset();
        let o = s.handle(letter('r'), &Stub(3));
        assert!(!o.effects.iter().any(|e| matches!(e, Effect::Commit(_))));
    }

    #[test]
    fn switching_keys_commit_and_reach_every_mode() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        // Mid-composition the switch commits, as set_mode does.
        let o = s.handle(Key::ToHangul, &Stub(3));
        assert!(o.eaten);
        assert_eq!(commit_of(&o).as_deref(), Some("한"));
        assert_eq!(s.mode(), Mode::Hangul);
        s.handle(Key::ToLatin, &Stub(3));
        assert_eq!(s.mode(), Mode::Latin);
        // Back to the Korean mode last in use, by any of the keys that lead there.
        s.handle(Key::ToKorean, &Stub(3));
        assert_eq!(s.mode(), Mode::Hangul);
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Latin);
        s.handle(Key::ToKorean, &Stub(3));
        assert_eq!(s.mode(), Mode::Hangul);
        s.handle(Key::ToSeokdok, &Stub(3));
        assert_eq!(s.mode(), Mode::Seokdok);
    }

    #[test]
    fn the_script_key_commits_and_enters_seokdok_from_any_mode() {
        for start in [Mode::Seokdok, Mode::Hangul] {
            let mut s = Session::new(start, true);
            type_str(&mut s, "gks");
            let o = s.handle(Key::CycleScript, &Stub(3));
            assert!(o.eaten);
            assert_eq!(commit_of(&o).as_deref(), Some("한"));
            assert_eq!(s.mode(), Mode::Seokdok);
            assert!(!s.is_composing());
        }
        let mut s = Session::new(Mode::Latin, true);
        assert!(s.eats(Key::CycleScript));
        assert_eq!(s.handle(Key::CycleScript, &Stub(3)).effects, vec![]);
        assert_eq!(s.mode(), Mode::Seokdok);
    }

    #[test]
    fn a_switch_to_the_current_mode_leaves_the_composition() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        for k in [Key::ToKorean, Key::ToSeokdok] {
            let o = s.handle(k, &Stub(3));
            assert!(o.eaten);
            assert_eq!(o.effects, vec![]);
            assert_eq!(s.buffer(), "한");
        }
        let mut latin = Session::new(Mode::Latin, true);
        let o = latin.handle(Key::ToLatin, &Stub(3));
        assert!(o.eaten && o.effects.is_empty());
        assert_eq!(latin.mode(), Mode::Latin);
    }

    #[test]
    fn switching_a_conversion_commits_it() {
        let mut s = seokdok();
        type_str(&mut s, "gks");
        s.handle(Key::Convert, &Stub(3));
        let o = s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(commit_of(&o).as_deref(), Some("한1"));
        assert_eq!(popup_of(&o), Some(None));
        assert_eq!(s.mode(), Mode::Latin);
    }

    #[test]
    fn hangul_only_has_no_seokdok_to_switch_to() {
        let mut s = seokdok();
        s.set_hangul_only(true);
        assert_eq!(s.mode(), Mode::Hangul);
        type_str(&mut s, "gks");
        // Nothing to switch between: the composition is not disturbed.
        for k in [Key::ToSeokdok, Key::ToHangul] {
            let o = s.handle(k, &Stub(3));
            assert!(o.eaten && o.effects.is_empty());
            assert_eq!(s.buffer(), "한");
        }
        // Latin stays reachable, and the toggle comes back to Hangul.
        s.handle(Key::HangulToggle, &Stub(3));
        assert_eq!(s.mode(), Mode::Latin);
        s.handle(Key::ToKorean, &Stub(3));
        assert_eq!(s.mode(), Mode::Hangul);
        // Turning the script off brings the requested 釋讀 back.
        s.set_hangul_only(false);
        assert_eq!(s.mode(), Mode::Seokdok);
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
            Key::CycleScript,
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

    // Old Hangul in escapes, so that no editor can normalise it.
    const HOLL: &str = "\u{1112}\u{119E}\u{11AF}"; // ᄒᆞᆯ
    const SA: &str = "\u{1109}\u{119E}"; // ᄉᆞ
    const SSAM: &str = "\u{110A}\u{119E}\u{11B7}"; // ᄊᆞᆷ
    const STTO: &str = "\u{112F}\u{1169}"; // ᄯᅩ

    /// Words split at spaces; the reading comes back as the text of two
    /// candidates (a space has one), so that the window opens and committing
    /// returns the input.
    struct Echo;

    impl Converter for Echo {
        fn segments(&self, input: &str) -> Vec<Segment> {
            let candidates = |w: &str| vec![Candidate { text: w.into(), note: String::new() }; if w == " " { 1 } else { 2 }];
            input
                .split_inclusive(' ')
                .flat_map(|w| match w.strip_suffix(' ') {
                    Some(word) => vec![word, " "],
                    None => vec![w],
                })
                .filter(|w| !w.is_empty())
                .map(|w| Segment { reading: w.into(), candidates: candidates(w) })
                .collect()
        }
        fn whole(&self, input: &str) -> Vec<Candidate> {
            vec![Candidate { text: input.into(), note: String::new() }]
        }
    }

    #[test]
    fn the_whole_list_reaches_past_the_first_page_and_picks_by_index() {
        struct Many;
        impl Converter for Many {
            fn segments(&self, input: &str) -> Vec<Segment> {
                let candidates = (0..PAGE_SIZE + 3)
                    .map(|i| Candidate { text: format!("{input}{i}"), note: format!("{input} · 音讀") })
                    .collect();
                vec![Segment { reading: input.into(), candidates }]
            }
        }
        let mut s = Session::new(Mode::Seokdok, true);
        assert!(s.candidate_list().is_none());
        s.handle(Key::Letter { ch: 'r', shift: false }, &Many);
        s.handle(Key::Letter { ch: 'k', shift: false }, &Many);
        assert!(s.candidate_list().is_none(), "nothing is converted yet");
        s.handle(Key::Convert, &Many);
        let list = s.candidate_list().unwrap();
        assert_eq!(list.items.len(), PAGE_SIZE + 3);
        assert_eq!(list.items[0].1, "", "a reading the input says is not repeated");
        assert!(!list.can_shorten, "one syllable");
        assert!(!list.can_lengthen, "no next segment");
        assert_eq!(s.pick_candidate(99), vec![], "an index past the end does nothing");
        let fx = s.pick_candidate(PAGE_SIZE + 2);
        assert!(fx.contains(&Effect::Commit(format!("가{}", PAGE_SIZE + 2))), "{fx:?}");
        assert!(!s.is_composing());
    }

    /// A careless converter: one segment for every char, conjoining jamo too.
    struct Chop;

    impl Converter for Chop {
        fn segments(&self, input: &str) -> Vec<Segment> {
            input
                .chars()
                .map(|c| Segment { reading: c.into(), candidates: vec![Candidate { text: c.into(), note: String::new() }] })
                .collect()
        }
        fn whole(&self, input: &str) -> Vec<Candidate> {
            vec![Candidate { text: input.into(), note: String::new() }]
        }
    }

    fn real_engine() -> impl Converter {
        struct E(seokdok_core::Engine);
        impl Converter for E {
            fn segments(&self, input: &str) -> Vec<Segment> {
                self.0.segments(input)
            }
            fn whole(&self, input: &str) -> Vec<Candidate> {
                self.0.convert(input)
            }
        }
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dict");
        E(crate::engine_host::load_engine(&dir, None, seokdok_core::Options::default()).0)
    }

    fn readings(s: &Session) -> Vec<String> {
        s.conv.as_ref().unwrap().segs.iter().map(|g| g.reading.clone()).collect()
    }

    /// The segments add up to `text` and every boundary is a unit boundary.
    fn assert_whole_units(s: &Session, text: &str) {
        let segs = readings(s);
        assert_eq!(segs.concat(), text, "{segs:?}");
        let mut starts = vec![0];
        for u in units(text) {
            starts.push(starts.last().unwrap() + u.len());
        }
        let mut at = 0;
        for g in &segs {
            at += g.len();
            assert!(starts.contains(&at), "cut at byte {at} of {text:?}: {segs:?}");
        }
        for k in &s.conv.as_ref().unwrap().segs {
            assert!(!k.cands.is_empty());
        }
    }

    /// Convert `text`, then step the focused segment through every width: out
    /// to the end, in to one unit, and out again.
    fn walk_widths(text: &str, c: &dyn Converter) -> Session {
        let mut s = seokdok();
        s.composer.set_text(text);
        s.handle(Key::Space { shift: false }, c);
        assert_whole_units(&s, text);
        let focus = s.conv.as_ref().unwrap().focus;
        let head = readings(&s)[..focus].concat();
        let span = units(&text[head.len()..]);
        let n = span.len();
        let width = |s: &Session| readings(s)[focus].clone();
        let mut guard = n + 1;
        while readings(&s).len() > focus + 1 {
            s.handle(Key::Right { shift: true }, c);
            assert_whole_units(&s, text);
            guard -= 1;
            assert!(guard > 0, "extending does not end: {:?}", readings(&s));
        }
        assert_eq!(width(&s), span.concat());
        let no_move = s.handle(Key::Right { shift: true }, c);
        assert!(no_move.effects.is_empty());
        for w in (1..n).rev() {
            s.handle(Key::Left { shift: true }, c);
            assert_whole_units(&s, text);
            assert_eq!(width(&s), span[..w].concat());
        }
        let no_move = s.handle(Key::Left { shift: true }, c);
        assert!(no_move.effects.is_empty());
        for w in 2..=n {
            s.handle(Key::Right { shift: true }, c);
            assert_whole_units(&s, text);
            assert_eq!(width(&s), span[..w].concat());
        }
        s
    }

    fn mixed_texts() -> Vec<String> {
        vec![
            format!("追加{HOLL}까요"),
            "나랏말\u{110A}\u{119E}미".to_string(),
            HOLL.to_string(),
            format!("{HOLL}{SA}{SSAM}{STTO}"),
            format!("{STTO} {HOLL}까요, {SSAM}"),
            format!("\u{A960}\u{D7B0}\u{D7CB}\u{115F}\u{119E}\u{1100}\u{1160}{SA}"),
            // One syllable of two medials (the engine reads ᄒᆞᅵ as 해), of two
            // initials, and of a medial and final each in two jamo.
            "\u{1112}\u{119E}\u{1175}요 \u{1109}\u{1100}\u{1169}\u{1161}\u{11AF}\u{11A8}까요".to_string(),
        ]
    }

    #[test]
    fn units_survive_every_width_and_commit_back() {
        for text in mixed_texts() {
            for c in [&Echo as &dyn Converter, &Chop] {
                let mut s = walk_widths(&text, c);
                let o = s.handle(Key::Enter, c);
                assert_eq!(commit_of(&o).unwrap(), text);
                assert!(!s.is_composing());
            }
        }
    }

    #[test]
    fn units_survive_every_width_with_the_real_engine() {
        let c = real_engine();
        for text in mixed_texts() {
            let mut s = walk_widths(&text, &c);
            // Back to Hangul: the text as typed comes back whole.
            s.handle(Key::Backspace, &c);
            assert_eq!(s.buffer(), text);
            let o = s.handle(Key::Enter, &c);
            assert_eq!(commit_of(&o).unwrap(), text);
        }
    }

    #[test]
    fn a_shrunk_old_syllable_moves_as_one() {
        let text = format!("추{HOLL}");
        let mut s = seokdok();
        s.composer.set_text(&text);
        s.handle(Key::Convert, &Echo);
        let o = s.handle(Key::Left { shift: true }, &Echo);
        let got: Vec<String> = preedit_of(&o).unwrap().into_iter().map(|r| r.text).collect();
        assert_eq!(got, ["추".to_string(), HOLL.to_string()]);
    }

    #[test]
    fn an_old_only_reading_can_be_focused_and_resized() {
        // One candidate each: only the Hangul in the reading makes it a stop.
        struct One;
        impl Converter for One {
            fn segments(&self, input: &str) -> Vec<Segment> {
                Echo.segments(input)
                    .into_iter()
                    .map(|mut g| {
                        g.candidates.truncate(1);
                        g
                    })
                    .collect()
            }
        }
        let text = format!("1 {HOLL}{SA} 2 \u{119E}");
        let mut s = seokdok();
        s.composer.set_text(&text);
        s.handle(Key::Convert, &One);
        assert_eq!(readings(&s)[s.conv.as_ref().unwrap().focus], format!("{HOLL}{SA}"));
        s.handle(Key::Left { shift: true }, &One);
        assert_whole_units(&s, &text);
        assert_eq!(readings(&s)[2], HOLL);
        assert_eq!(readings(&s)[3], format!("{SA} "));
        // Right walks on to the stray old vowel, over the digit and the space.
        s.handle(Key::Right { shift: false }, &One);
        assert_eq!(readings(&s)[s.conv.as_ref().unwrap().focus], format!("{SA} "));
        s.handle(Key::Right { shift: false }, &One);
        assert_eq!(readings(&s)[s.conv.as_ref().unwrap().focus], "\u{119E}");
        // Non-Hangul with one candidate stays out.
        let mut s = seokdok();
        s.composer.set_text("1 2");
        s.handle(Key::Convert, &One);
        assert_eq!(s.conv.as_ref().unwrap().focus, 0);
        let c = s.conv.as_ref().unwrap();
        assert!(!focusable(&c.segs[0]) && !focusable(&c.segs[2]));
    }

    #[test]
    fn a_pick_keeps_old_hangul_converting() {
        let mut s = seokdok();
        s.composer.set_text(&format!("가 {HOLL}"));
        s.handle(Key::Convert, &Echo);
        s.handle(Key::Down, &Echo);
        let o = s.handle(Key::Printable('1'), &Echo);
        assert_eq!(commit_of(&o).unwrap(), "가");
        assert_eq!(s.buffer(), format!(" {HOLL}"));
        assert_eq!(readings(&s)[s.conv.as_ref().unwrap().focus], HOLL);
        // Only spaces and digits left: all of it goes.
        let mut s = seokdok();
        s.composer.set_text("가 1");
        s.handle(Key::Convert, &Echo);
        s.handle(Key::Down, &Echo);
        let o = s.handle(Key::Printable('1'), &Echo);
        assert_eq!(commit_of(&o).unwrap(), "가 1");
        assert!(!s.is_composing());
    }

    #[test]
    fn backspace_never_leaves_part_of_an_earlier_syllable() {
        // Typed: one letter at a time, inside the last syllable only.
        let mut s = seokdok();
        type_str(&mut s, "gkr");
        s.handle(Key::Printable(','), &Echo);
        s.handle(Key::Letter { ch: 'g', shift: false }, &Echo);
        s.handle(Key::Letter { ch: 'k', shift: true }, &Echo);
        s.handle(Key::Letter { ch: 'f', shift: false }, &Echo);
        assert_eq!(s.buffer(), format!("학,{HOLL}"));
        s.handle(Key::Backspace, &Echo);
        assert_eq!(s.buffer(), "학,\u{1112}\u{119E}");
        s.handle(Key::Backspace, &Echo);
        assert_eq!(s.buffer(), "학,ㅎ");
        s.handle(Key::Backspace, &Echo);
        assert_eq!(s.buffer(), "학,");
        // After a conversion it goes back to the same text.
        let text = format!("追加{HOLL}까요");
        let mut s = seokdok();
        s.composer.set_text(&text);
        s.handle(Key::Convert, &Echo);
        s.handle(Key::Left { shift: true }, &Echo);
        let o = s.handle(Key::Backspace, &Echo);
        assert_eq!(preedit_of(&o).unwrap()[0].text, text);
        assert_eq!(s.buffer(), text);
    }

    #[test]
    fn a_conversion_without_candidates_keeps_its_text() {
        struct Bare;
        impl Converter for Bare {
            fn segments(&self, input: &str) -> Vec<Segment> {
                Echo.segments(input)
                    .into_iter()
                    .map(|g| Segment { reading: g.reading, candidates: Vec::new() })
                    .collect()
            }
        }
        let mut s = seokdok();
        s.composer.set_text(&format!("{HOLL} 까요"));
        s.handle(Key::Convert, &Bare);
        let o = s.handle(Key::Enter, &Bare);
        assert_eq!(commit_of(&o).unwrap(), format!("{HOLL} 까요"));
    }

    struct XorShift(u64);

    impl XorShift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
        fn pick<T: Copy>(&mut self, items: &[T]) -> T {
            items[self.below(items.len())]
        }
    }

    fn random_text(r: &mut XorShift) -> String {
        const MODERN: &[&str] = &["가", "한", "까", "요", "말", "미", "나", "랏", "하", "늘", "았", "봤"];
        const OLD: &[&str] = &[
            HOLL,
            SA,
            SSAM,
            STTO,
            "\u{1140}\u{1161}\u{11AB}",
            "\u{A960}\u{D7B0}\u{D7CB}",
            "\u{115F}\u{119E}",
            "\u{1100}\u{1160}",
            "\u{1100}\u{1161}",
            "\u{1112}\u{1161}\u{11BC}\u{11AB}",
            "\u{114C}\u{119E}\u{11F0}",
        ];
        const STRAY: &[&str] = &[
            "\u{1112}", "\u{1100}", "\u{A960}", "\u{115F}", "\u{1160}", "\u{119E}", "\u{1161}", "\u{D7B0}", "\u{11AF}", "\u{11A8}", "\u{D7CB}",
        ];
        const OTHER: &[&str] = &["追", "加", "天", " ", " ", ",", "1", "a", "ㅎ", "ㆍ"];
        let mut out = String::new();
        for _ in 0..r.below(12) + 1 {
            let set = r.pick(&[MODERN, OLD, OLD, STRAY, OTHER]);
            out.push_str(r.pick(set));
        }
        out
    }

    /// Every unit of `text` but the last.
    fn all_but_last_unit(text: &str) -> String {
        let u = units(text);
        u[..u.len().saturating_sub(1)].concat()
    }

    /// One random session, all keys through `handle`. The converter must hand
    /// back the reading as the text, so that the text can be followed through.
    fn fuzz_echoing_session(r: &mut XorShift, c: &dyn Converter) {
        let text = random_text(r);
        let mut s = Session::new(Mode::Seokdok, r.below(2) == 0);
        s.set_old_clusters(r.below(2) == 0);
        s.set_horizontal(r.below(2) == 0);
        s.composer.set_text(&text);
        let current = |s: &Session| match &s.conv {
            Some(_) => readings(s).concat(),
            None => s.buffer().to_string(),
        };
        let mut committed = String::new();
        let shift = |r: &mut XorShift| r.below(2) == 0;
        for _ in 0..r.below(40) {
            let key = match r.below(16) {
                0 | 1 => Key::Left { shift: true },
                2 | 3 => Key::Right { shift: true },
                4 => Key::Left { shift: false },
                5 => Key::Right { shift: false },
                6 => Key::Space { shift: shift(r) },
                7 => Key::Convert,
                8 => Key::Tab { shift: shift(r) },
                9 => r.pick(&[Key::Down, Key::Up, Key::PageUp, Key::PageDown]),
                10 | 11 => Key::Backspace,
                12 => Key::Printable(r.pick(&['1', '2', ',', '!'])),
                13 => Key::Letter { ch: r.pick(&['g', 'k', 'f', 'r', 'd', 'l', 't', 'h']), shift: shift(r) },
                14 => Key::Esc,
                _ => Key::Enter,
            };
            let was_conv = s.conv.is_some();
            let before = format!("{committed}{}", current(&s));
            let buffered = s.buffer().len();
            let out = s.handle(key, c);
            for e in &out.effects {
                if let Effect::Commit(t) = e {
                    if s.conv.is_some() {
                        assert!(!splits_unit(t, &current(&s)), "{text:?}: commit {t:?} cuts into {:?}", current(&s));
                    }
                    committed.push_str(t);
                }
            }
            let now = current(&s);
            let after = format!("{committed}{now}");
            match key {
                Key::Letter { .. } if was_conv => assert!(after.starts_with(&before), "{text:?} {key:?}: {before:?} -> {after:?}"),
                Key::Letter { .. } => {
                    let kept = all_but_last_unit(&before);
                    assert!(after.starts_with(&kept), "{text:?} {key:?}: {before:?} -> {after:?}");
                }
                Key::Backspace if !was_conv && buffered > 0 => {
                    let kept = all_but_last_unit(&before);
                    assert!(after.starts_with(&kept) && after != before, "{text:?} {key:?}: {before:?} -> {after:?}");
                }
                Key::Esc if !was_conv && buffered > 0 => assert_eq!(now, ""),
                Key::Printable(k) => assert!(after == before || after == format!("{before}{k}"), "{text:?} {key:?}: {before:?} -> {after:?}"),
                Key::Space { .. } => assert!(after == before || after == format!("{before} "), "{text:?} {key:?}: {before:?} -> {after:?}"),
                _ => assert_eq!(after, before, "{text:?} {key:?}"),
            }
            match &s.conv {
                Some(_) => {
                    assert_whole_units(&s, s.buffer());
                    if let Some(runs) = preedit_of(&out) {
                        assert_eq!(runs.iter().map(|x| x.text.as_str()).collect::<String>(), now);
                    }
                }
                None => {
                    if let Some(runs) = preedit_of(&out) {
                        assert_eq!(runs[0].text, s.buffer());
                    }
                }
            }
        }
        let end = format!("{committed}{}", current(&s));
        for e in s.flush() {
            if let Effect::Commit(t) = e {
                committed.push_str(&t);
            }
        }
        assert_eq!(committed, end, "{text:?}");
        assert!(!s.is_composing());
    }

    #[test]
    fn fuzz_sessions_keep_the_text_and_its_units() {
        let mut r = XorShift(0x5EED_0B7D_1234_ABCD);
        for _ in 0..3000 {
            fuzz_echoing_session(&mut r, &Echo);
        }
        for _ in 0..1000 {
            fuzz_echoing_session(&mut r, &Chop);
        }
    }

    #[test]
    fn fuzz_composer_backspace_empties_without_cutting() {
        let mut r = XorShift(0x0DD1_EE7A_90F0_0001);
        for _ in 0..3000 {
            let text = random_text(&mut r);
            let mut s = seokdok();
            s.composer.set_text(&text);
            // A letter typed onto the text must keep what lies before it.
            if r.below(2) == 0 {
                let kept = all_but_last_unit(&text);
                let ch = r.pick(&['g', 'k', 'f', 'd', 'K']);
                s.handle(Key::Letter { ch: ch.to_ascii_lowercase(), shift: ch.is_uppercase() }, &Echo);
                assert!(s.buffer().starts_with(&kept), "{text:?} + {ch}: {:?}", s.buffer());
            }
            let mut left = s.buffer().to_string();
            let mut steps = 0;
            while !left.is_empty() {
                let kept = all_but_last_unit(&left);
                s.handle(Key::Backspace, &Echo);
                assert!(s.buffer().starts_with(&kept) && s.buffer() != left, "{left:?} -> {:?}", s.buffer());
                left = s.buffer().to_string();
                steps += 1;
                assert!(steps < 200);
            }
            assert!(!s.is_composing());
        }
    }

    #[test]
    fn fuzz_real_engine_keeps_readings_whole() {
        let c = real_engine();
        let mut r = XorShift(0xA11C_E5ED_0000_7777);
        for _ in 0..600 {
            let text = random_text(&mut r);
            let mut s = seokdok();
            s.composer.set_text(&text);
            s.handle(Key::Convert, &c);
            if s.conv.is_none() {
                assert_eq!(s.buffer(), text);
                continue;
            }
            assert_whole_units(&s, &text);
            for g in &s.conv.as_ref().unwrap().segs {
                let old = g.reading.chars().any(|c| is_hangul(c) && !('\u{AC00}'..='\u{D7A3}').contains(&c) && !('\u{3131}'..='\u{318E}').contains(&c));
                assert!(!old || g.cands.iter().any(|k| k.text == g.reading), "{:?} cannot be kept as typed", g.reading);
            }
            for _ in 0..r.below(10) {
                let key = r.pick(&[
                    Key::Left { shift: true },
                    Key::Right { shift: true },
                    Key::Left { shift: false },
                    Key::Right { shift: false },
                    Key::Down,
                ]);
                s.handle(key, &c);
                assert_whole_units(&s, &text);
            }
            if r.below(2) == 0 {
                // Back to Hangul: nothing was lost.
                s.handle(Key::Backspace, &c);
                let o = s.handle(Key::Enter, &c);
                assert_eq!(commit_of(&o).unwrap(), text);
            } else {
                let o = s.handle(Key::Enter, &c);
                assert!(!commit_of(&o).unwrap().is_empty(), "{text:?}");
            }
            assert!(!s.is_composing());
        }
    }
}
