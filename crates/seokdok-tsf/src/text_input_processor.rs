//! `ITfTextInputProcessor` + `ITfTextInputProcessorEx`: the activation
//! lifecycle. `ActivateEx` advises the sinks, registers the display
//! attributes, adds the language-bar button and starts loading the engine;
//! `Deactivate` undoes it in reverse, idempotently.

use windows::core::{IUnknownImpl, Interface, Ref};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, ITfCategoryMgr, ITfKeyEventSink, ITfKeystrokeMgr, ITfLangBarItem,
    ITfLangBarItemMgr, ITfSource, ITfTextInputProcessorEx_Impl, ITfTextInputProcessor_Impl,
    ITfThreadMgr, ITfThreadMgrEventSink, TF_INVALID_COOKIE,
};

use crate::config::InputMode;
use crate::guids::{GUID_DA_CONVERTED, GUID_DA_FOCUS, GUID_DA_INPUT};
use crate::lang_bar::ModeButton;
use crate::session::{Mode, Session};
use crate::text_service::TextService_Impl;

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> windows::core::Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> windows::core::Result<()> {
        let thread_mgr = self.inner().thread_mgr.clone();
        if let Some(tm) = thread_mgr.as_ref() {
            if let Some(item) = self.inner_mut().langbar_item.take() {
                if let Ok(lbim) = tm.cast::<ITfLangBarItemMgr>() {
                    // SAFETY: lbim and item are valid COM interfaces.
                    unsafe {
                        let _ = lbim.RemoveItem(&item);
                    }
                }
            }

            if let Ok(keystroke) = tm.cast::<ITfKeystrokeMgr>() {
                let cid = self.inner().client_id;
                // SAFETY: keystroke is a valid ITfKeystrokeMgr from the TM.
                unsafe {
                    let _ = keystroke.UnadviseKeyEventSink(cid);
                }
            }

            let cookie = self.inner().thread_mgr_cookie;
            if cookie != TF_INVALID_COOKIE {
                if let Ok(source) = tm.cast::<ITfSource>() {
                    // SAFETY: source is a valid ITfSource from the TM.
                    unsafe {
                        let _ = source.UnadviseSink(cookie);
                    }
                }
                self.inner_mut().thread_mgr_cookie = TF_INVALID_COOKIE;
            }
        }

        let mut state = self.inner_mut();
        state.session.reset();
        state.composition = None;
        state.candidate_window = None;
        state.category_mgr = None;
        state.display_atoms = [0; 3];
        state.client_id = 0;
        state.anchor = None;
        // Release the thread manager last.
        state.thread_mgr = None;
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(
        &self,
        ptim: Ref<'_, ITfThreadMgr>,
        tid: u32,
        _dwflags: u32,
    ) -> windows::core::Result<()> {
        if self.inner().thread_mgr.is_some() {
            return Ok(());
        }
        let thread_mgr = match ptim.as_ref() {
            Some(tm) => tm.clone(),
            None => return Ok(()),
        };
        {
            let mut state = self.inner_mut();
            state.thread_mgr = Some(thread_mgr.clone());
            state.client_id = tid;
        }
        if let Err(err) = self.setup(&thread_mgr, tid) {
            let _ = self.Deactivate();
            return Err(err);
        }
        Ok(())
    }
}

impl TextService_Impl {
    fn setup(&self, thread_mgr: &ITfThreadMgr, tid: u32) -> windows::core::Result<()> {
        // Make sure config.toml exists, pick up edits, and start in the
        // configured mode.
        crate::config::ensure_file();
        let cfg = crate::config::reload();
        let mode = match cfg.input.default_mode {
            InputMode::Seokdok => Mode::Seokdok,
            InputMode::Hangul => Mode::Hangul,
            InputMode::Latin => Mode::Latin,
        };
        {
            let mut state = self.inner_mut();
            state.session = Session::new(mode, cfg.input.space_converts);
            state.mode.show(mode);
        }
        crate::compartment::publish(thread_mgr, tid, mode);

        // Load the dictionaries in the background; typing works meanwhile and
        // conversion starts to produce results once the load finishes.
        if !crate::engine_host::HOST.started() {
            crate::dll::load_engine();
        }

        let keystroke: ITfKeystrokeMgr = thread_mgr.cast()?;
        let this_kes: ITfKeyEventSink = self.to_interface();
        // SAFETY: keystroke / this_kes are valid; fforeground = true.
        unsafe {
            keystroke.AdviseKeyEventSink(tid, &this_kes, true)?;
        }

        let source: ITfSource = thread_mgr.cast()?;
        let this_tmes: ITfThreadMgrEventSink = self.to_interface();
        // SAFETY: source / this_tmes valid; returns a cookie.
        let cookie = unsafe { source.AdviseSink(&ITfThreadMgrEventSink::IID, &this_tmes)? };
        self.inner_mut().thread_mgr_cookie = cookie;

        // SAFETY: standard in-proc COM creation.
        let catmgr: ITfCategoryMgr =
            unsafe { CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)? };
        // SAFETY: catmgr valid; GUID pointers valid.
        let atoms = unsafe {
            [
                catmgr.RegisterGUID(&GUID_DA_INPUT)?,
                catmgr.RegisterGUID(&GUID_DA_CONVERTED)?,
                catmgr.RegisterGUID(&GUID_DA_FOCUS)?,
            ]
        };
        {
            let mut state = self.inner_mut();
            state.display_atoms = atoms;
            state.category_mgr = Some(catmgr);
            state.composition = None;
        }

        // Language-bar button; best effort, a missing language bar must not
        // fail activation.
        let shared = self.inner().mode.clone();
        let item: ITfLangBarItem = ModeButton::new(shared).into();
        if let Ok(lbim) = thread_mgr.cast::<ITfLangBarItemMgr>() {
            // SAFETY: lbim and item are valid COM interfaces.
            unsafe {
                if lbim.AddItem(&item).is_ok() {
                    self.inner_mut().langbar_item = Some(item);
                }
            }
        }

        self.inner_mut().candidate_window = crate::candidate_window::CandidateWindow::new();
        Ok(())
    }
}
