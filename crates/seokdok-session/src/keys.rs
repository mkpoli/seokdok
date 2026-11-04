//! Platform-independent key vocabulary of the input method. The Windows key
//! event sink translates virtual-key codes into [`Key`]; the session state
//! machine only sees this type.

/// Virtual-key codes the key event sink cares about (winuser.h values).
pub mod vk {
    pub const BACK: u16 = 0x08;
    pub const TAB: u16 = 0x09;
    pub const RETURN: u16 = 0x0D;
    pub const HANGUL: u16 = 0x15;
    pub const HANJA: u16 = 0x19;
    pub const ESCAPE: u16 = 0x1B;
    pub const SPACE: u16 = 0x20;
    pub const PRIOR: u16 = 0x21;
    pub const NEXT: u16 = 0x22;
    pub const END: u16 = 0x23;
    pub const HOME: u16 = 0x24;
    pub const LEFT: u16 = 0x25;
    pub const UP: u16 = 0x26;
    pub const RIGHT: u16 = 0x27;
    pub const DOWN: u16 = 0x28;
    pub const DELETE: u16 = 0x2E;
    pub const F1: u16 = 0x70;
    pub const F7: u16 = 0x76;
    pub const F24: u16 = 0x87;
    pub const LWIN: u16 = 0x5B;
    pub const RWIN: u16 = 0x5C;
    pub const SHIFT: u16 = 0x10;
    pub const CONTROL: u16 = 0x11;
    pub const MENU: u16 = 0x12;
    pub const IME_ON: u16 = 0x16;
    pub const IME_OFF: u16 = 0x1A;
    pub const CONVERT: u16 = 0x1C;
    pub const NONCONVERT: u16 = 0x1D;
    pub const LSHIFT: u16 = 0xA0;
    pub const RSHIFT: u16 = 0xA1;
    pub const LCONTROL: u16 = 0xA2;
    pub const RCONTROL: u16 = 0xA3;
    pub const LMENU: u16 = 0xA4;
    pub const RMENU: u16 = 0xA5;
    /// VK_DBE_HIRAGANA: カタカナ/ひらがな on a Japanese layout.
    pub const DBE_HIRAGANA: u16 = 0xF2;
    /// VK_DBE_SBCSCHAR and VK_DBE_DBCSCHAR: 半角/全角 on a Japanese layout.
    pub const DBE_SBCSCHAR: u16 = 0xF3;
    pub const DBE_DBCSCHAR: u16 = 0xF4;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A letter key. `ch` is the lowercase ASCII letter; Caps Lock is ignored.
    Letter { ch: char, shift: bool },
    /// Any other printable ASCII character except space (digits, punctuation).
    Printable(char),
    Space { shift: bool },
    Enter,
    Esc,
    Backspace,
    Tab { shift: bool },
    Left { shift: bool },
    Right { shift: bool },
    Up,
    Down,
    PageUp,
    PageDown,
    /// Convert the composition: a key from `convert` in `[input.keys]`.
    Convert,
    /// Korean ↔ Latin.
    HangulToggle,
    /// To Latin; quiet when already there.
    ToLatin,
    /// From Latin back to Korean; quiet when already Korean.
    ToKorean,
    /// 釋讀 ↔ 한글. From Latin, back to whichever was last in use.
    FlipHangul,
    /// To 釋讀.
    ToSeokdok,
    /// To 한글.
    ToHangul,
    /// Another editing or navigation key (Home, End, Delete, ...).
    Other,
}

impl Key {
    /// Does this key switch modes?
    pub fn switches_mode(self) -> bool {
        matches!(self, Key::HangulToggle | Key::ToLatin | Key::ToKorean | Key::FlipHangul | Key::ToSeokdok | Key::ToHangul)
    }

    /// The character that selects a jamo in the 두벌식 layout.
    pub fn jamo_key(self) -> Option<char> {
        match self {
            Key::Letter { ch, shift } => Some(if shift { ch.to_ascii_uppercase() } else { ch }),
            _ => None,
        }
    }

    /// Classify a virtual key that is not a letter. `printable` is the
    /// character the keyboard layout produces for it, if any.
    pub fn from_vk(vk: u16, shift: bool, printable: Option<char>) -> Option<Key> {
        Some(match vk {
            0x41..=0x5A => Key::Letter { ch: (b'a' + (vk - 0x41) as u8) as char, shift },
            vk::SPACE => Key::Space { shift },
            vk::RETURN => Key::Enter,
            vk::ESCAPE => Key::Esc,
            vk::BACK => Key::Backspace,
            vk::TAB => Key::Tab { shift },
            vk::LEFT => Key::Left { shift },
            vk::RIGHT => Key::Right { shift },
            vk::UP => Key::Up,
            vk::DOWN => Key::Down,
            vk::PRIOR => Key::PageUp,
            vk::NEXT => Key::PageDown,
            vk::END | vk::HOME | vk::DELETE => Key::Other,
            _ => match printable {
                Some(c) if c.is_ascii_graphic() => Key::Printable(c),
                _ => return None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_and_shift() {
        let k = Key::from_vk(0x52, true, Some('R')).unwrap();
        assert_eq!(k, Key::Letter { ch: 'r', shift: true });
        assert_eq!(k.jamo_key(), Some('R'));
        assert_eq!(Key::from_vk(0x52, false, Some('r')).unwrap().jamo_key(), Some('r'));
    }

    #[test]
    fn named_keys() {
        assert_eq!(Key::from_vk(vk::RETURN, false, None), Some(Key::Enter));
        assert_eq!(Key::from_vk(vk::SPACE, true, Some(' ')), Some(Key::Space { shift: true }));
        // Mode keys go by the lists in `mode_keys`.
        assert_eq!(Key::from_vk(vk::F7, false, None), None);
        assert_eq!(Key::from_vk(vk::HANJA, false, None), None);
        assert_eq!(Key::from_vk(vk::HANGUL, false, None), None);
    }

    #[test]
    fn printable_and_unknown() {
        assert_eq!(Key::from_vk(0x31, false, Some('1')), Some(Key::Printable('1')));
        assert_eq!(Key::from_vk(0xBC, false, Some(',')), Some(Key::Printable(',')));
        assert_eq!(Key::from_vk(0x70, false, None), None);
    }
}
