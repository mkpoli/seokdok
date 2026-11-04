//! Where the text being composed is drawn: in the document, or in a popup
//! because the document cannot hold a composition. The rules are kept apart
//! from the Windows calls that gather the facts, so they can be tested.

use std::time::Duration;

use crate::config::Surface;

/// A composition that ends this soon after its first write was ended by the
/// document's reaction to that write: nobody clicks or switches that fast.
pub const REFUSAL_WINDOW: Duration = Duration::from_millis(50);

/// Refusals seen after the edit session returned that make the choice stick.
const STRIKES: u8 = 2;

/// What is known about a document when a composition is about to start.
#[derive(Clone, Copy, Debug, Default)]
pub struct Facts {
    /// The context says `TS_SS_TRANSITORY`.
    pub transitory: bool,
    /// Keys reach the input method through IMM32, judged by `OnTestKeyDown`
    /// preceding `OnKeyDown`. An application calling `ITfKeystrokeMgr::TestKeyDown`
    /// itself looks the same and floats without need, which costs nothing but
    /// the inline display.
    pub through_imm32: bool,
    /// This process already refused a composition.
    pub remembered: bool,
    /// The setting for this application.
    pub forced: Option<Surface>,
}

/// The surface for a new composition and the reason, for the trace.
pub fn choose(facts: Facts) -> (Surface, &'static str) {
    if let Some(surface) = facts.forced {
        (surface, "set in config.toml")
    } else if !facts.transitory {
        (Surface::Inline, "the document keeps its text")
    } else if facts.remembered {
        (Surface::Floating, "this process refused a composition")
    } else if facts.through_imm32 {
        (Surface::Floating, "keys arrive through IMM32")
    } else {
        (Surface::Inline, "a transitory context with its own text store")
    }
}

/// How a composition came to an end without the input method asking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// Focus moved, the application reset itself: the text stays where it is.
    Ordinary,
    /// The document's reaction to what was just written. `in_edit` says it
    /// came while that edit session was still running.
    Refused { in_edit: bool },
}

/// Classify an end the input method did not ask for. `focus_moved`: the focused
/// context is no longer the one typed in, which explains the end whatever the
/// timing. `since_first_write` is the time since the composition's first write,
/// `None` once it has taken a second: a document that accepted a second write
/// did not refuse the composition.
pub fn classify_end(transitory: bool, focus_moved: bool, in_edit: bool, since_first_write: Option<Duration>) -> End {
    if !transitory || focus_moved {
        End::Ordinary
    } else if in_edit {
        End::Refused { in_edit: true }
    } else if since_first_write.is_some_and(|d| d < REFUSAL_WINDOW) {
        End::Refused { in_edit: false }
    } else {
        End::Ordinary
    }
}

/// What `text` adds to what the document already took: an empty composition
/// range cannot show that the application gave its text back, so what was
/// shown counts as delivered. Text that is not an extension of it, a conversion
/// say, cannot be put right and adds nothing.
pub fn undelivered<'a>(delivered: &str, text: &'a str) -> &'a str {
    text.strip_prefix(delivered).unwrap_or("")
}

/// What happens to the composition at a focus or context change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtBoundary {
    Commit,
    /// Do not write: the text is not in the document, and a text store that
    /// serves several fields may already address the next one. Chromium assigns
    /// the new field before it notifies of the focus change.
    Drop,
}

/// An open composition is a place in the old field, and the IMM32 path commits
/// through the bridge as it always has. Floating text in any other store has no
/// safe place to go.
pub fn at_boundary(composition_open: bool, through_imm32: bool) -> AtBoundary {
    if composition_open || through_imm32 {
        AtBoundary::Commit
    } else {
        AtBoundary::Drop
    }
}

/// Whether a commit with no composition open may insert plainly. Chromium fires
/// the key's keydown ahead of any composition started while handling it, so a
/// page can act on Enter before the text arrives; a plain insert avoids that.
/// Other stores may read their text only when a composition ends, as Windows
/// Terminal does, so they get a composition that starts and ends in the commit.
pub fn commits_plainly(through_imm32: bool, window_class: &str) -> bool {
    !through_imm32 && window_class.starts_with("Chrome_WidgetWin_")
}

/// Decides when refusals are enough to float this process from now on. A web
/// page can cancel a composition right after a key too, so one refusal after
/// the edit session only floats that composition.
#[derive(Clone, Copy, Debug, Default)]
pub struct Refusals {
    strikes: u8,
}

impl Refusals {
    /// Record a refusal; true when this process should float from now on.
    pub fn refused(&mut self, in_edit: bool) -> bool {
        self.strikes = self.strikes.saturating_add(1);
        in_edit || self.strikes >= STRIKES
    }

    /// A composition took a second write, so the document accepts them.
    pub fn accepted(&mut self) {
        self.strikes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts { transitory: true, ..Facts::default() }
    }

    #[test]
    fn a_document_that_keeps_text_is_inline() {
        let f = Facts { transitory: false, through_imm32: true, remembered: true, forced: None };
        assert_eq!(choose(f).0, Surface::Inline);
    }

    #[test]
    fn a_transitory_context_is_inline_unless_something_speaks_against_it() {
        assert_eq!(choose(facts()).0, Surface::Inline);
        assert_eq!(choose(Facts { through_imm32: true, ..facts() }).0, Surface::Floating);
        assert_eq!(choose(Facts { remembered: true, ..facts() }).0, Surface::Floating);
    }

    #[test]
    fn the_setting_wins() {
        let f = Facts { through_imm32: true, remembered: true, forced: Some(Surface::Inline), ..facts() };
        assert_eq!(choose(f).0, Surface::Inline);
        let f = Facts { transitory: false, forced: Some(Surface::Floating), ..facts() };
        assert_eq!(choose(f).0, Surface::Floating);
    }

    #[test]
    fn an_end_counts_as_a_refusal_only_in_a_transitory_context_right_after_the_first_write() {
        let soon = Some(Duration::from_millis(1));
        let later = Some(Duration::from_millis(400));
        assert_eq!(classify_end(true, false, true, soon), End::Refused { in_edit: true });
        assert_eq!(classify_end(true, false, false, soon), End::Refused { in_edit: false });
        assert_eq!(classify_end(true, false, false, later), End::Ordinary);
        assert_eq!(classify_end(true, false, false, None), End::Ordinary);
        assert_eq!(classify_end(false, false, true, soon), End::Ordinary);
    }

    #[test]
    fn a_moved_focus_is_never_a_refusal() {
        let soon = Some(Duration::from_millis(1));
        assert_eq!(classify_end(true, true, true, soon), End::Ordinary);
        assert_eq!(classify_end(true, true, false, soon), End::Ordinary);
    }

    #[test]
    fn only_what_extends_the_delivered_text_is_still_owed() {
        assert_eq!(undelivered("대충", "대충"), "");
        assert_eq!(undelivered("대충", "대충 "), " ");
        assert_eq!(undelivered("대충", "大忠"), "");
    }

    #[test]
    fn floating_text_outside_imm32_is_not_written_at_a_boundary() {
        assert_eq!(at_boundary(false, false), AtBoundary::Drop);
        assert_eq!(at_boundary(false, true), AtBoundary::Commit);
        assert_eq!(at_boundary(true, false), AtBoundary::Commit);
    }

    #[test]
    fn only_a_chromium_window_commits_plainly() {
        assert!(commits_plainly(false, "Chrome_WidgetWin_1"));
        assert!(!commits_plainly(true, "Chrome_WidgetWin_1"));
        assert!(!commits_plainly(false, "CASCADIA_HOSTING_WINDOW_CLASS"));
        assert!(!commits_plainly(false, ""));
    }

    #[test]
    fn a_refusal_inside_the_edit_session_sticks_at_once() {
        assert!(Refusals::default().refused(true));
    }

    #[test]
    fn a_late_refusal_sticks_only_when_repeated() {
        let mut r = Refusals::default();
        assert!(!r.refused(false));
        assert!(r.refused(false));
    }

    #[test]
    fn an_accepted_write_forgives_a_late_refusal() {
        let mut r = Refusals::default();
        assert!(!r.refused(false));
        r.accepted();
        assert!(!r.refused(false));
    }
}
