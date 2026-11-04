//! Windows key events for the `[input.keys]` lists: which physical key a
//! virtual-key code and scan code mean, and the tracker for modifier taps.
//!
//! The Korean keyboard drivers already turn the keys that stand in for 한/영
//! and 한자 into VK_HANGUL and VK_HANJA (Right Alt and Right Ctrl on the 101-key
//! type 1, Shift+Space and Ctrl+Space on type 3). AltGr arrives as left Ctrl
//! plus right Alt, which no default entry matches.

use std::cell::RefCell;

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, GetKeyboardType};

use crate::keys::{vk, Key};
use crate::mode_keys::{KeyName, ModeKeyboard, ModeKeys, Modifier, MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_SUPER};
use crate::session::Session;

thread_local! {
    static KEYBOARD: RefCell<ModeKeyboard> = RefCell::new(ModeKeyboard::new(&ModeKeys::default()));
}

/// Each side of each modifier, with its virtual key.
const SIDES: [(u16, Modifier); 8] = [
    (vk::LCONTROL, Modifier::LCtrl),
    (vk::RCONTROL, Modifier::RCtrl),
    (vk::LSHIFT, Modifier::LShift),
    (vk::RSHIFT, Modifier::RShift),
    (vk::LMENU, Modifier::LAlt),
    (vk::RMENU, Modifier::RAlt),
    (vk::LWIN, Modifier::LSuper),
    (vk::RWIN, Modifier::RSuper),
];

fn down(vk: u16) -> bool {
    // SAFETY: GetKeyState is always safe to call.
    (unsafe { GetKeyState(vk as i32) } as u16) & 0x8000 != 0
}

fn is_modifier(vk: u16) -> bool {
    matches!(vk, vk::SHIFT | vk::CONTROL | vk::MENU) || SIDES.iter().any(|(v, _)| *v == vk)
}

fn mods_now() -> u8 {
    let mut m = 0;
    if down(vk::CONTROL) {
        m |= MOD_CTRL;
    }
    if down(vk::SHIFT) {
        m |= MOD_SHIFT;
    }
    if down(vk::MENU) {
        m |= MOD_ALT;
    }
    if down(vk::LWIN) || down(vk::RWIN) {
        m |= MOD_SUPER;
    }
    m
}

/// The key a non-modifier event is. Japanese keys are also told by scan code,
/// because a Korean layout need not give them a virtual key of their own.
fn name_of(vk: u16, lparam: LPARAM) -> Option<KeyName> {
    let scan = (lparam.0 >> 16) & 0xFF;
    let extended = (lparam.0 >> 24) & 1 == 1;
    Some(match vk {
        vk::HANGUL => KeyName::Hangul,
        vk::HANJA => KeyName::Hanja,
        vk::IME_ON => KeyName::ImeOn,
        vk::IME_OFF => KeyName::ImeOff,
        vk::CONVERT => KeyName::Henkan,
        vk::NONCONVERT => KeyName::Muhenkan,
        vk::DBE_HIRAGANA => KeyName::Kana,
        vk::DBE_SBCSCHAR | vk::DBE_DBCSCHAR => KeyName::Zenkaku,
        vk::SPACE => KeyName::Space,
        vk::RETURN => KeyName::Enter,
        vk::TAB => KeyName::Tab,
        vk::ESCAPE => KeyName::Esc,
        vk::F1..=vk::F24 => KeyName::Function((vk - vk::F1 + 1) as u8),
        0x30..=0x39 | 0x41..=0x5A => KeyName::Char(vk as u8 as char),
        _ if extended => return None,
        _ => match scan {
            0x79 => KeyName::Henkan,
            0x7B => KeyName::Muhenkan,
            0x70 => KeyName::Kana,
            // 半角/全角 sits where ` does; only a Japanese keyboard has it.
            // SAFETY: GetKeyboardType is always safe to call.
            0x29 if unsafe { GetKeyboardType(0) } == 7 => KeyName::Zenkaku,
            _ => return None,
        },
    })
}

/// The side of the modifier a key-up reports.
fn released_modifier(vk: u16, lparam: LPARAM) -> Option<KeyName> {
    let scan = (lparam.0 >> 16) & 0xFF;
    let extended = (lparam.0 >> 24) & 1 == 1;
    let m = match vk {
        vk::CONTROL if extended => Modifier::RCtrl,
        vk::CONTROL => Modifier::LCtrl,
        vk::MENU if extended => Modifier::RAlt,
        vk::MENU => Modifier::LAlt,
        // Right Shift is not extended; only its scan code tells it apart.
        vk::SHIFT if scan == 0x36 => Modifier::RShift,
        vk::SHIFT => Modifier::LShift,
        _ => SIDES.iter().find(|(v, _)| *v == vk)?.1,
    };
    Some(KeyName::Modifier(m))
}

fn with_keyboard<T>(f: impl FnOnce(&mut ModeKeyboard) -> T) -> T {
    KEYBOARD.with(|k| {
        let mut k = k.borrow_mut();
        k.set_keys(&crate::config::current().input.keys);
        f(&mut k)
    })
}

fn vk_of(wparam: WPARAM) -> u16 {
    (wparam.0 & 0xFFFF) as u16
}

/// A modifier went down: note every side held now. This sees the left Ctrl
/// that comes with AltGr even if its own event never reached the text service.
pub fn note_modifier_down(wparam: WPARAM) {
    if !is_modifier(vk_of(wparam)) {
        return;
    }
    with_keyboard(|k| {
        for (v, m) in SIDES {
            if down(v) {
                k.modifier_down(m);
            }
        }
    });
}

/// The session key a non-modifier key-down is bound to.
pub fn press(wparam: WPARAM, lparam: LPARAM, session: &Session) -> Option<Key> {
    let vk = vk_of(wparam);
    if is_modifier(vk) {
        return None;
    }
    let name = name_of(vk, lparam);
    with_keyboard(|k| k.press(name, mods_now(), session))
}

/// What releasing this key would do, for `OnTestKeyUp`.
pub fn peek_release(wparam: WPARAM, lparam: LPARAM, session: &Session) -> Option<Key> {
    let name = released_modifier(vk_of(wparam), lparam)?;
    with_keyboard(|k| k.peek_release(name, session))
}

/// The key came up: the session key when this completes a tap.
pub fn release(wparam: WPARAM, lparam: LPARAM, session: &Session) -> Option<Key> {
    let name = released_modifier(vk_of(wparam), lparam)?;
    with_keyboard(|k| k.release(name, session))
}

/// Focus moved: a key held across it will not report its release here.
pub fn reset() {
    KEYBOARD.with(|k| k.borrow_mut().reset());
}
