import AppKit
import Carbon.HIToolbox
import SeokdokCore

/// What a key press means to the session.
enum KeyAction: Equatable {
    /// Feed `kind` (an SK_* value) with its character and Shift.
    case key(kind: Int32, ch: UInt32, shift: Bool)
    /// A shortcut for the application: finish the composition, pass it on.
    case shortcut
}

enum KeyMap {
    /// 두벌식 is positional: letters come from the key's place on the
    /// keyboard, whatever the ASCII layout prints.
    private static let letters: [Int: Character] = [
        kVK_ANSI_A: "a", kVK_ANSI_B: "b", kVK_ANSI_C: "c", kVK_ANSI_D: "d", kVK_ANSI_E: "e",
        kVK_ANSI_F: "f", kVK_ANSI_G: "g", kVK_ANSI_H: "h", kVK_ANSI_I: "i", kVK_ANSI_J: "j",
        kVK_ANSI_K: "k", kVK_ANSI_L: "l", kVK_ANSI_M: "m", kVK_ANSI_N: "n", kVK_ANSI_O: "o",
        kVK_ANSI_P: "p", kVK_ANSI_Q: "q", kVK_ANSI_R: "r", kVK_ANSI_S: "s", kVK_ANSI_T: "t",
        kVK_ANSI_U: "u", kVK_ANSI_V: "v", kVK_ANSI_W: "w", kVK_ANSI_X: "x", kVK_ANSI_Y: "y",
        kVK_ANSI_Z: "z",
    ]

    private static let named: [Int: Int32] = [
        kVK_Space: Int32(SK_SPACE),
        kVK_Return: Int32(SK_ENTER),
        kVK_ANSI_KeypadEnter: Int32(SK_ENTER),
        kVK_Escape: Int32(SK_ESC),
        kVK_Delete: Int32(SK_BACKSPACE),
        kVK_Tab: Int32(SK_TAB),
        kVK_LeftArrow: Int32(SK_LEFT),
        kVK_RightArrow: Int32(SK_RIGHT),
        kVK_UpArrow: Int32(SK_UP),
        kVK_DownArrow: Int32(SK_DOWN),
        kVK_PageUp: Int32(SK_PAGEUP),
        kVK_PageDown: Int32(SK_PAGEDOWN),
        kVK_Home: Int32(SK_OTHER),
        kVK_End: Int32(SK_OTHER),
        kVK_ForwardDelete: Int32(SK_OTHER),
    ]

    private static let digits: [Int: Character] = [
        kVK_ANSI_0: "0", kVK_ANSI_1: "1", kVK_ANSI_2: "2", kVK_ANSI_3: "3", kVK_ANSI_4: "4",
        kVK_ANSI_5: "5", kVK_ANSI_6: "6", kVK_ANSI_7: "7", kVK_ANSI_8: "8", kVK_ANSI_9: "9",
    ]

    /// Key names of the `[input.keys]` lists. A Korean PC keyboard reports
    /// 한/영 and 한자 with the codes of かな and 英数.
    private static let keyNames: [Int: String] = [
        kVK_JIS_Eisu: "Eisu", kVK_JIS_Kana: "Kana",
        kVK_Space: "Space", kVK_Return: "Enter", kVK_ANSI_KeypadEnter: "Enter",
        kVK_Tab: "Tab", kVK_Escape: "Esc",
        kVK_F1: "F1", kVK_F2: "F2", kVK_F3: "F3", kVK_F4: "F4", kVK_F5: "F5",
        kVK_F6: "F6", kVK_F7: "F7", kVK_F8: "F8", kVK_F9: "F9", kVK_F10: "F10",
        kVK_F11: "F11", kVK_F12: "F12", kVK_F13: "F13", kVK_F14: "F14", kVK_F15: "F15",
        kVK_F16: "F16", kVK_F17: "F17", kVK_F18: "F18", kVK_F19: "F19", kVK_F20: "F20",
    ]

    /// Each modifier with its device-dependent flag (IOKit/hidsystem/IOLLEvent.h):
    /// the only way to tell which side went up while the other stays down.
    private static let modifiers: [Int: (name: String, mask: UInt)] = [
        kVK_Control: ("LCtrl", 0x0001), kVK_RightControl: ("RCtrl", 0x2000),
        kVK_Shift: ("LShift", 0x0002), kVK_RightShift: ("RShift", 0x0004),
        kVK_Command: ("LCmd", 0x0008), kVK_RightCommand: ("RCmd", 0x0010),
        kVK_Option: ("LAlt", 0x0020), kVK_RightOption: ("RAlt", 0x0040),
    ]

    /// The name of the key in the `[input.keys]` lists, if it has one.
    static func name(for event: NSEvent) -> String? {
        let code = Int(event.keyCode)
        if let letter = letters[code] ?? digits[code] {
            return String(letter)
        }
        return keyNames[code]
    }

    /// The SK_MOD_* flags held with this event.
    static func mods(of event: NSEvent) -> UInt32 {
        let flags = event.modifierFlags
        var mods: UInt32 = 0
        if flags.contains(.control) { mods |= UInt32(SK_MOD_CTRL) }
        if flags.contains(.shift) { mods |= UInt32(SK_MOD_SHIFT) }
        if flags.contains(.option) { mods |= UInt32(SK_MOD_ALT) }
        if flags.contains(.command) { mods |= UInt32(SK_MOD_SUPER) }
        return mods
    }

    /// The modifier a flagsChanged event is about, and whether it is now down.
    static func modifierChange(of event: NSEvent) -> (name: String, down: Bool)? {
        guard let modifier = modifiers[Int(event.keyCode)] else { return nil }
        return (modifier.name, event.modifierFlags.rawValue & modifier.mask != 0)
    }

    static func action(for event: NSEvent) -> KeyAction {
        let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let shift = flags.contains(.shift)
        let code = Int(event.keyCode)

        if !flags.isDisjoint(with: [.command, .control, .option]) {
            return .shortcut
        }
        if let letter = letters[code] {
            // Caps Lock is ignored: Shift alone picks the doubled consonants.
            return .key(kind: Int32(SK_LETTER), ch: letter.unicodeScalars.first!.value, shift: shift)
        }
        if let kind = named[code] {
            return .key(kind: kind, ch: 0, shift: shift)
        }
        // Digits and punctuation as the ASCII layout types them.
        if let scalar = event.characters?.unicodeScalars.first, event.characters?.unicodeScalars.count == 1,
           scalar.value >= 0x20, scalar.value != 0x7F, !(0xF700...0xF8FF).contains(scalar.value) {
            return .key(kind: Int32(SK_PRINTABLE), ch: scalar.value, shift: shift)
        }
        return .key(kind: Int32(SK_OTHER), ch: 0, shift: shift)
    }
}
