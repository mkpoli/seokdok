//! The master TSF COM object. A single `TextService` implements every
//! interface the running TIP needs; its mutable state sits in a `RefCell`.
//! No borrow is ever held across a call into TSF, which may call back into
//! this object.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use std::time::Instant;

use windows::core::implement;
use windows::Win32::UI::TextServices::{
    ITfCategoryMgr, ITfComposition, ITfCompositionSink, ITfContext, ITfDisplayAttributeProvider,
    ITfFnConfigure, ITfFunction, ITfFunctionProvider, ITfKeyEventSink, ITfLangBarItem,
    ITfTextInputProcessor, ITfTextInputProcessorEx, ITfThreadMgr, ITfThreadMgrEventSink,
    TF_INVALID_COOKIE,
};

use crate::candidate_window::CandidateWindow;
use crate::lang_bar::ModeShared;
use crate::keys::Key;
use crate::placement::Caret;
use crate::session::{Mode, Popup, Run, Session};
use crate::surface::Refusals;

pub struct TextServiceState {
    pub thread_mgr: Option<ITfThreadMgr>,
    pub client_id: u32,
    pub thread_mgr_cookie: u32,
    pub category_mgr: Option<ITfCategoryMgr>,
    /// TfGuidAtoms of the display attributes: input, converted, focus.
    pub display_atoms: [u32; 3],
    pub composition: Option<ITfComposition>,
    pub session: Session,
    /// Mode shared with the input-indicator button.
    pub mode: Rc<ModeShared>,
    pub langbar_item: Option<ITfLangBarItem>,
    pub candidate_window: Option<CandidateWindow>,
    /// Where the focused segment is on the screen, for the popup.
    pub anchor: Option<Caret>,
    /// Text being composed, drawn in the popup instead of the document when a
    /// converting composition goes through IMM32 (see `surface.rs`).
    pub floating: Option<Vec<Run>>,
    /// The candidate page last shown.
    pub popup: Option<Popup>,
    /// The context whose keys come through IMM32 (`OnTestKeyDown` ran for it),
    /// compared by address only.
    pub imm32_context: Option<usize>,
    /// What the open composition shows, to carry on floating if it is refused.
    pub drawn: Vec<Run>,
    /// The open composition's context says `TS_SS_TRANSITORY`.
    pub composition_transitory: bool,
    /// The context the session's text was typed in.
    pub typing_context: Option<ITfContext>,
    /// The key being handled, to replay on what is left if the text so far
    /// turns out to have reached the application.
    pub current_key: Option<Key>,
    /// The replay did not eat the key, so it goes on to the application.
    pub replay_passed: bool,
    /// An edit session is writing the preedit.
    pub writing: bool,
    /// An edit session is committing or cancelling the composition.
    pub ending: bool,
    /// When the open composition was first written, until a second write.
    pub first_write_at: Option<Instant>,
    /// How many compositions the document has ended.
    pub compositions_ended: u32,
    pub refusals: Refusals,
}

impl Default for TextServiceState {
    fn default() -> Self {
        Self {
            thread_mgr: None,
            client_id: 0,
            thread_mgr_cookie: TF_INVALID_COOKIE,
            category_mgr: None,
            display_atoms: [0; 3],
            composition: None,
            session: Session::new(Mode::Seokdok, true),
            mode: Rc::new(ModeShared::new(Mode::Seokdok)),
            langbar_item: None,
            candidate_window: None,
            anchor: None,
            floating: None,
            popup: None,
            imm32_context: None,
            drawn: Vec::new(),
            composition_transitory: false,
            typing_context: None,
            current_key: None,
            replay_passed: false,
            writing: false,
            ending: false,
            first_write_at: None,
            compositions_ended: 0,
            refusals: Refusals::default(),
        }
    }
}

#[implement(
    ITfTextInputProcessor,
    ITfTextInputProcessorEx,
    ITfKeyEventSink,
    ITfThreadMgrEventSink,
    ITfCompositionSink,
    ITfDisplayAttributeProvider,
    ITfFunctionProvider,
    ITfFnConfigure,
    ITfFunction
)]
pub struct TextService {
    inner: RefCell<TextServiceState>,
}

impl TextService {
    pub fn new() -> Self {
        // The object itself pins the DLL: COM may release the class factory
        // while clients still hold a TextService.
        crate::lock_module();
        Self { inner: RefCell::new(TextServiceState::default()) }
    }
}

impl Drop for TextService {
    fn drop(&mut self) {
        crate::unlock_module();
    }
}

impl TextService_Impl {
    pub(crate) fn inner(&self) -> Ref<'_, TextServiceState> {
        self.inner.borrow()
    }

    pub(crate) fn inner_mut(&self) -> RefMut<'_, TextServiceState> {
        self.inner.borrow_mut()
    }
}
