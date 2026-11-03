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
use crate::text_service::TextService_Impl;

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
        let Some(key) = decode(wparam, lparam) else {
            return Ok(false.into());
        };
        if let Some(context) = pic.as_ref() {
            self.sync(context);
        }
        Ok(self.inner().session.eats(key).into())
    }

    fn OnTestKeyUp(
        &self,
        _pic: Ref<'_, ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        Ok(false.into())
    }

    fn OnKeyDown(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
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
        if !output.eaten {
            return Ok(false.into());
        }
        {
            let state = self.inner();
            let mode = state.session.mode();
            if state.mode.shown() != mode {
                state.mode.show(mode);
            }
        }
        // An edit failure still consumed the key; a failed session leaves the
        // document unchanged.
        let _ = self.apply(context, output.effects);
        Ok(true.into())
    }

    fn OnKeyUp(
        &self,
        _pic: Ref<'_, ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        Ok(false.into())
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _rguid: *const GUID) -> windows::core::Result<BOOL> {
        Ok(false.into())
    }
}
