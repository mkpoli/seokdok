//! `ITfKeyEventSink`: decode the keystroke into a [`Key`] and ask the session
//! whether it is eaten. `OnTestKeyDown` only predicts; `OnKeyDown` edits.

use windows::core::{Ref, BOOL, GUID};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardState, ToUnicode, VK_CONTROL, VK_MENU, VK_SHIFT,
};
use windows::Win32::UI::TextServices::{ITfContext, ITfKeyEventSink_Impl};

use crate::engine_host::HOST;
use crate::keys::Key;
use crate::mode_key_events;
use crate::session::Effect;
use crate::text_service::TextService_Impl;

thread_local! {
    /// The key that just committed a candidate or switched modes, while it may
    /// still be held.
    static COMMITTED_BY: std::cell::Cell<Option<u16>> = const { std::cell::Cell::new(None) };
}

/// Whether this key-down is the auto-repeat of the key that committed: holding
/// Enter on a candidate must not go on to break the line. A fresh press clears
/// the mark, so the next Enter reaches the application.
fn repeat_of_commit(wparam: WPARAM, lparam: LPARAM) -> bool {
    let vk = (wparam.0 & 0xFFFF) as u16;
    // Bit 30: the key was already down.
    let held = (lparam.0 >> 30) & 1 == 1 && COMMITTED_BY.with(|c| c.get()) == Some(vk);
    if !held {
        COMMITTED_BY.with(|c| c.set(None));
    }
    held
}

/// Track modifiers for the taps in `[input.keys]`.
fn note_key_down(wparam: WPARAM) {
    mode_key_events::note_modifier_down(wparam);
}

fn key_down(vk: i32) -> bool {
    // SAFETY: GetKeyState is always safe to call.
    (unsafe { GetKeyState(vk) } as u16) & 0x8000 != 0
}

/// Translate a key-down message into a [`Key`]; `None` for keys the input
/// method never handles (modifier shortcuts, function keys, ...).
fn decode(wparam: WPARAM, lparam: LPARAM) -> Option<Key> {
    let vk = (wparam.0 & 0xFFFF) as u16;
    if key_down(VK_CONTROL.0 as i32) || key_down(VK_MENU.0 as i32) {
        return None;
    }
    let shift = key_down(VK_SHIFT.0 as i32);

    let scan = ((lparam.0 >> 16) & 0xFF) as u32;
    let mut state = [0u8; 256];
    let mut printable = None;
    // SAFETY: state is a valid 256-byte buffer.
    if unsafe { GetKeyboardState(&mut state) }.is_ok() {
        let mut buf = [0u16; 8];
        // Flag 4 leaves the keyboard's dead-key state untouched.
        // SAFETY: vk/scan are plain integers; state and buf are valid buffers.
        let n = unsafe { ToUnicode(vk as u32, scan, Some(&state), &mut buf, 4) };
        if n == 1 {
            printable = char::from_u32(buf[0] as u32);
        }
    }
    Key::from_vk(vk, shift, printable)
}

impl TextService_Impl {
    /// The key this key-down means: a bound one, else its plain meaning.
    fn key_of(&self, wparam: WPARAM, lparam: LPARAM) -> Option<Key> {
        let bound = mode_key_events::press(wparam, lparam, &self.inner().session);
        bound.or_else(|| decode(wparam, lparam))
    }

    /// Tell the language bar and the compartments the session's mode.
    fn show_mode(&self) {
        let state = self.inner();
        let mode = state.session.mode();
        if state.mode.shown() != mode {
            state.mode.show(mode);
            if let Some(tm) = state.thread_mgr.as_ref() {
                crate::compartment::publish(tm, state.client_id, mode);
            }
        }
    }

    /// With no document in focus there is nothing to edit, but a mode key
    /// still switches. True when the key was used.
    fn switch_without_document(&self, key: Key) -> bool {
        if !key.switches_mode() {
            return false;
        }
        self.inner_mut().session.handle(key, &HOST);
        self.show_mode();
        true
    }

    /// Apply the settings and a mode requested from the language bar. Called
    /// at the start of every key event, so the session is up to date.
    fn sync(&self, context: &ITfContext) {
        let cfg = crate::config::current();
        let (requested, effects) = {
            let mut state = self.inner_mut();
            let hangul_only = cfg.conversion.script == crate::config::Script::HangulOnly;
            state.mode.set_hangul_only(hangul_only);
            let mut effects = state.session.set_hangul_only(hangul_only);
            state.session.set_space_converts(cfg.input.space_converts);
            state.session.set_horizontal(cfg.input.candidate_layout == crate::config::Layout::Horizontal);
            state.session.set_old_clusters(cfg.input.old_clusters);
            let requested = state.mode.take_request();
            effects.extend(requested.map(|m| state.session.set_mode(m)).unwrap_or_default());
            (requested, effects)
        };
        if requested.is_some() || !effects.is_empty() {
            let _ = self.apply(context, effects);
        }
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> windows::core::Result<()> {
        mode_key_events::reset();
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        note_key_down(wparam);
        if repeat_of_commit(wparam, lparam) {
            return Ok(true.into());
        }
        let Some(key) = self.key_of(wparam, lparam) else {
            return Ok(false.into());
        };
        if let Some(context) = pic.as_ref() {
            self.sync(context);
        }
        let eats = {
            let state = self.inner();
            state.session.eats(key) || state.session.commits_then_passes(key)
        };
        crate::debug_log::log(|| format!("OnTestKeyDown vk={:#x} eats={eats} ctx={:?}", wparam.0, pic.as_ref().map(|c| windows::core::Interface::as_raw(c))));
        Ok(eats.into())
    }

    fn OnTestKeyUp(
        &self,
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        let state = self.inner();
        let tap = mode_key_events::peek_release(wparam, lparam, &state.session).is_some_and(|k| state.session.eats(k));
        Ok(tap.into())
    }

    fn OnKeyDown(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        note_key_down(wparam);
        if repeat_of_commit(wparam, lparam) {
            crate::debug_log::log(|| format!("OnKeyDown vk={:#x} repeat of commit, eaten", wparam.0));
            return Ok(true.into());
        }
        let Some(key) = self.key_of(wparam, lparam) else {
            return Ok(false.into());
        };
        let Some(context) = pic.as_ref() else {
            return Ok(self.switch_without_document(key).into());
        };
        self.sync(context);
        // Release the borrow before touching the document: TSF calls back
        // into this object during edit sessions.
        let output = self.inner_mut().session.handle(key, &HOST);
        crate::debug_log::log(|| format!("OnKeyDown vk={:#x} lparam={:#x} eaten={} effects={:?} ctx={:?}", wparam.0, lparam.0, output.eaten, output.effects, windows::core::Interface::as_raw(context)));
        if key.switches_mode() || matches!(key, Key::Enter | Key::Printable('1'..='9')) && output.effects.iter().any(|e| matches!(e, Effect::Commit(_))) {
            COMMITTED_BY.with(|c| c.set(Some((wparam.0 & 0xFFFF) as u16)));
        }
        if !output.eaten {
            // A plain keyboard commits its composition before the key goes on.
            if !output.effects.is_empty() {
                let _ = self.apply(context, output.effects);
            }
            return Ok(false.into());
        }
        self.show_mode();
        // An edit failure still consumed the key; a failed session leaves the
        // document unchanged.
        if let Err(e) = self.apply(context, output.effects) {
            crate::debug_log::log(|| format!("apply failed: {e:?}"));
        }
        Ok(true.into())
    }

    fn OnKeyUp(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        let tap = mode_key_events::release(wparam, lparam, &self.inner().session);
        let Some(key) = tap.filter(|k| self.inner().session.eats(*k)) else {
            return Ok(false.into());
        };
        let Some(context) = pic.as_ref() else {
            return Ok(self.switch_without_document(key).into());
        };
        self.sync(context);
        let output = self.inner_mut().session.handle(key, &HOST);
        self.show_mode();
        let _ = self.apply(context, output.effects);
        // Eating the key-up keeps Alt from opening the window menu.
        Ok(true.into())
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _rguid: *const GUID) -> windows::core::Result<BOOL> {
        Ok(false.into())
    }
}
