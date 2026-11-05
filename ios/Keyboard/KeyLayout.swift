import CoreGraphics
import Foundation

/// Where the keys go, as on the system's Korean keyboard. Measured from its
/// screenshots in simulators: iOS 26.5 on iPhone SE (3rd generation), iPhone 16
/// Pro and iPhone 16 Pro Max, and iOS 18.5 on iPhone 16 Pro. The UI tests compile
/// this file to find the keys.
struct KeyLayout: Equatable {
    /// From the keyboard's edge to the first key.
    var margin: CGFloat
    /// Between two keys of a row.
    var gap: CGFloat
    var keyHeight: CGFloat
    /// From a row's top to the next row's top.
    var rowPitch: CGFloat
    /// From the keyboard's top to the first row, the candidate bar included.
    var firstRow: CGFloat
    var barHeight: CGFloat
    /// Under the last row.
    var bottom: CGFloat
    /// Between Shift or Delete and the letters.
    var edgeGap: CGFloat = 14.33

    /// The height the keyboard asks for.
    var height: CGFloat { firstRow + 3 * rowPitch + keyHeight + bottom }

    /// System version with flat, rounder keys.
    static var flat: Bool {
        if #available(iOS 26, *) { return true }
        return false
    }

    static func current(width: CGFloat, landscape: Bool) -> KeyLayout {
        if landscape {
            return flat
                ? KeyLayout(margin: 4, gap: 5.8, keyHeight: 27, rowPitch: 35.33, firstRow: 50, barHeight: 42, bottom: 4)
                : KeyLayout(margin: 3, gap: 5.8, keyHeight: 32, rowPitch: 40, firstRow: 43, barHeight: 36, bottom: 3)
        }
        // Before iOS 26 a key's shadow is the bottom point of the system's height.
        let shadow: CGFloat = flat ? 0 : 1
        let margin: CGFloat = flat ? 7 : 3
        let gap: CGFloat = flat ? 6.67 : 6.33
        let (key, pitch, bottom): (CGFloat, CGFloat, CGFloat) =
            width >= 428 ? (45, 56, 5) : width < 390 ? (42, 54, 3) : (43, 54, 3)
        return KeyLayout(margin: margin, gap: gap, keyHeight: key - shadow, rowPitch: pitch,
                         firstRow: 53, barHeight: 44, bottom: bottom + shadow)
    }

    func keyWidth(in width: CGFloat) -> CGFloat {
        (width - 2 * margin - 9 * gap) / 10
    }

    /// From a key's left edge to the next key's.
    func columnPitch(in width: CGFloat) -> CGFloat {
        keyWidth(in: width) + gap
    }

    /// Shift and Delete: the letters of their row start a column and a half in.
    func edgeKeyWidth(in width: CGFloat) -> CGFloat {
        1.5 * columnPitch(in: width) - edgeGap
    }

    /// The slot at each end of the bottom row (123 and the mode key; Return),
    /// two and a half columns wide less a gap.
    func bottomSlotWidth(in width: CGFloat) -> CGFloat {
        2.5 * columnPitch(in: width) - gap
    }
}
