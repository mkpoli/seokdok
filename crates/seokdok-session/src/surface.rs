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

/// Classify an end the input method did not ask for. `since_first_write` is the
/// time since the composition's first write, `None` once it has taken a second:
/// a document that accepted a second write did not refuse the composition.
pub fn classify_end(transitory: bool, in_edit: bool, since_first_write: Option<Duration>) -> End {
    if !transitory {
        End::Ordinary
    } else if in_edit {
        End::Refused { in_edit: true }
    } else if since_first_write.is_some_and(|d| d < REFUSAL_WINDOW) {
        End::Refused { in_edit: false }
    } else {
        End::Ordinary
    }
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
        assert_eq!(classify_end(true, true, soon), End::Refused { in_edit: true });
        assert_eq!(classify_end(true, false, soon), End::Refused { in_edit: false });
        assert_eq!(classify_end(true, false, later), End::Ordinary);
        assert_eq!(classify_end(true, false, None), End::Ordinary);
        assert_eq!(classify_end(false, true, soon), End::Ordinary);
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
