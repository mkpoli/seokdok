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
    pub const F7: u16 = 0x76;
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
    Tab,
    Left,
    Right,
    Up,
    Down,
    /// F7 or the Hanja key.
    Convert,
    /// The Han/Eng key.
    HangulToggle,
    /// Another editing or navigation key (Home, End, Delete, ...).
    Other,
}

impl Key {
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
            vk::TAB => Key::Tab,
            vk::LEFT => Key::Left,
            vk::RIGHT => Key::Right,
            vk::UP => Key::Up,
            vk::DOWN => Key::Down,
            vk::F7 | vk::HANJA => Key::Convert,
            vk::HANGUL => Key::HangulToggle,
            vk::PRIOR | vk::NEXT | vk::END | vk::HOME | vk::DELETE => Key::Other,
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
        assert_eq!(Key::from_vk(vk::F7, false, None), Some(Key::Convert));
        assert_eq!(Key::from_vk(vk::HANJA, false, None), Some(Key::Convert));
        assert_eq!(Key::from_vk(vk::SPACE, true, Some(' ')), Some(Key::Space { shift: true }));
        assert_eq!(Key::from_vk(vk::HANGUL, false, None), Some(Key::HangulToggle));
    }

    #[test]
    fn printable_and_unknown() {
        assert_eq!(Key::from_vk(0x31, false, Some('1')), Some(Key::Printable('1')));
        assert_eq!(Key::from_vk(0xBC, false, Some(',')), Some(Key::Printable(',')));
        assert_eq!(Key::from_vk(0x70, false, None), None);
    }
}
