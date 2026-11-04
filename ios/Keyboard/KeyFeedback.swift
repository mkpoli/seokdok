import UIKit

/// `UIDevice.playInputClick()` plays only for an input view that adopts this
/// protocol, and the keyboard's input view is the controller's own `UIInputView`
/// (azooKey, MIT, conforms the same way:
/// https://github.com/azooKey/azooKey/blob/af2d6b08733550449e100b1f2f1406bd4779d125/Keyboard/Display/KeyboardViewController.swift#L24-L28).
extension UIInputView: UIInputViewAudioFeedback {
    public var enableInputClicksWhenVisible: Bool { true }
}

/// What a press feels like. Both fire as the finger lands.
final class KeyFeedback {
    /// Haptics with Full Access only: other keyboards gate them on it too, and
    /// Apple does not say. The generators exist only then, to save memory.
    var hapticsAllowed = false {
        didSet {
            if hapticsAllowed { tap.prepare() }
        }
    }

    private lazy var tap = UIImpactFeedbackGenerator(style: .light)
    private lazy var bump = UIImpactFeedbackGenerator(style: .medium)

    /// The system's click, heard when the keyboard clicks setting is on and the
    /// keyboard has Full Access, and a light tap.
    func key() {
        UIDevice.current.playInputClick()
        guard hapticsAllowed else { return }
        tap.impactOccurred()
        tap.prepare()
    }

    /// Space has become the trackpad.
    func cursor() {
        guard hapticsAllowed else { return }
        bump.impactOccurred()
    }
}
