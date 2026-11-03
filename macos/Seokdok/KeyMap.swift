import AppKit
import Carbon.HIToolbox
import SeokdokCore

/// What a key press means to the session.
enum KeyAction: Equatable {
    /// Feed `kind` (an SK_* value) with its character and Shift.
    case key(kind: Int32, ch: UInt32, shift: Bool)
    /// 英數 (JIS): switch to Latin.
    case latin
    /// かな (JIS): switch back to Hangul.
    case hangul
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
        kVK_F7: Int32(SK_CONVERT),
        kVK_Home: Int32(SK_OTHER),
        kVK_End: Int32(SK_OTHER),
        kVK_ForwardDelete: Int32(SK_OTHER),
    ]

    static func action(for event: NSEvent) -> KeyAction {
        let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let shift = flags.contains(.shift)
        let code = Int(event.keyCode)

        switch code {
        case kVK_JIS_Eisu: return .latin
        case kVK_JIS_Kana: return .hangul
        default: break
        }
        // Option-Return converts, as in the Korean input sources of macOS.
        if code == kVK_Return && flags.subtracting([.shift, .capsLock]) == .option {
            return .key(kind: Int32(SK_CONVERT), ch: 0, shift: false)
        }
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
