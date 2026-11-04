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
use crate::session::Effect;
use crate::text_service::TextService_Impl;

thread_local! {
    /// Alt went down with no other key since: releasing it switches modes.
    static ALT_ALONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The key that just committed a candidate, while it may still be held.
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

fn is_alt(wparam: WPARAM) -> bool {
    let vk = (wparam.0 & 0xFFFF) as u16;
    vk == VK_MENU.0 || vk == 0xA4 || vk == 0xA5
}

/// Track Alt on its own: down marks it, any other key clears it.
fn note_key_down(wparam: WPARAM) {
    ALT_ALONE.with(|a| a.set(is_alt(wparam) && !key_down(VK_CONTROL.0 as i32) && !key_down(VK_SHIFT.0 as i32)));
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
    /// Apply the settings and a mode requested from the language bar. Called
    /// at the start of every key event, so the session is up to date.
    fn sync(&self, context: &ITfContext) {
        let cfg = crate::config::current();
        let (requested, effects) = {
            let mut state = self.inner_mut();
            state.session.set_space_converts(cfg.input.space_converts);
            state.session.set_horizontal(cfg.input.candidate_layout == crate::config::Layout::Horizontal);
            state.session.set_old_clusters(cfg.input.old_clusters);
            state.session.set_hangul_only(cfg.conversion.script == crate::config::Script::HangulOnly);
            let requested = state.mode.take_request();
            let effects = requested.map(|m| state.session.set_mode(m)).unwrap_or_default();
            (requested, effects)
        };
        if requested.is_some() {
            let _ = self.apply(context, effects);
        }
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> windows::core::Result<()> {
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
        let Some(key) = decode(wparam, lparam) else {
            return Ok(false.into());
        };
        if let Some(context) = pic.as_ref() {
            self.sync(context);
        }
        let eats = self.inner().session.eats(key);
        crate::debug_log::log(|| format!("OnTestKeyDown vk={:#x} eats={eats} ctx={:?}", wparam.0, pic.as_ref().map(|c| windows::core::Interface::as_raw(c))));
        Ok(eats.into())
    }

    fn OnTestKeyUp(
        &self,
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        let tap = is_alt(wparam) && ALT_ALONE.with(|a| a.get()) && crate::config::current().input.alt_toggles;
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
        let Some(key) = decode(wparam, lparam) else {
            return Ok(false.into());
        };
        let Some(context) = pic.as_ref() else {
            return Ok(false.into());
        };
        self.sync(context);
        // Release the borrow before touching the document: TSF calls back
        // into this object during edit sessions.
        let output = self.inner_mut().session.handle(key, &HOST);
        crate::debug_log::log(|| format!("OnKeyDown vk={:#x} lparam={:#x} eaten={} effects={:?} ctx={:?}", wparam.0, lparam.0, output.eaten, output.effects, windows::core::Interface::as_raw(context)));
        if matches!(key, Key::Enter | Key::Printable('1'..='9')) && output.effects.iter().any(|e| matches!(e, Effect::Commit(_))) {
            COMMITTED_BY.with(|c| c.set(Some((wparam.0 & 0xFFFF) as u16)));
        }
        if !output.eaten {
            return Ok(false.into());
        }
        {
            let state = self.inner();
            let mode = state.session.mode();
            if state.mode.shown() != mode {
                state.mode.show(mode);
                if let Some(tm) = state.thread_mgr.as_ref() {
                    crate::compartment::publish(tm, state.client_id, mode);
                }
            }
        }
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
        _lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        let tap = is_alt(wparam) && ALT_ALONE.with(|a| a.replace(false)) && crate::config::current().input.alt_toggles;
        if !tap {
            return Ok(false.into());
        }
        let Some(context) = pic.as_ref() else {
            return Ok(false.into());
        };
        self.sync(context);
        let output = self.inner_mut().session.handle(Key::HangulToggle, &HOST);
        {
            let state = self.inner();
            let mode = state.session.mode();
            if state.mode.shown() != mode {
                state.mode.show(mode);
                if let Some(tm) = state.thread_mgr.as_ref() {
                    crate::compartment::publish(tm, state.client_id, mode);
                }
            }
        }
        let _ = self.apply(context, output.effects);
        // Eating the key-up keeps Alt from opening the window menu.
        Ok(true.into())
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _rguid: *const GUID) -> windows::core::Result<BOOL> {
        Ok(false.into())
    }
}
