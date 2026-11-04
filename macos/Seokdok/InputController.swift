import AppKit
import Carbon
import InputMethodKit
import SeokdokCore

/// The input modes declared in Info.plist (ComponentInputModeDict). Each is
/// an input source of its own in the input menu.
enum InputModes {
    private static let prefix = Bundle.main.bundleIdentifier ?? "li.mkpo.inputmethod.seokdok"

    static func id(_ mode: Session.Mode) -> String {
        switch mode {
        case .seokdok: return prefix + ".seokdok"
        case .hangul: return prefix + ".hangul"
        case .latin: return prefix + ".latin"
        }
    }

    static func mode(_ id: String) -> Session.Mode? {
        Session.Mode.allCases.first { self.id($0) == id }
    }

    /// The mode of the input source selected now, when it is one of ours.
    static func selected() -> Session.Mode? {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let ptr = TISGetInputSourceProperty(source, kTISPropertyInputModeID) else { return nil }
        let id = Unmanaged<CFString>.fromOpaque(ptr).takeUnretainedValue() as String
        return mode(id)
    }
}

/// One controller per text-input client. Keys go to a session of the Rust
/// core; its preedit becomes the client's marked text and its candidate page
/// the candidate window.
@objc(SeokdokInputController)
final class SeokdokInputController: IMKInputController {
    private let session = Session()
    /// Option went down with nothing else since: a release switches modes.
    private var optionAlone = false
    private var hasMarkedText = false
    /// The input source the system has selected, as far as this controller
    /// knows. The script setting never changes it.
    private var shownMode: Session.Mode?
    /// The last source that was not Latin: where the Han/Eng toggle returns.
    private var lastSource: Session.Mode = .seokdok

    private static let notFound = NSRange(location: NSNotFound, length: 0)

    // MARK: Activation

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        // Digits and punctuation come from the ABC layout whatever the
        // user's other layouts are; letters go by key position anyway.
        (sender as? IMKTextInput)?.overrideKeyboard(withKeyboardNamed: "com.apple.keylayout.ABC")
        session.sync()
        if let mode = InputModes.selected() {
            session.mode = mode
            shownMode = mode
            if mode != .latin { lastSource = mode }
        }
        hasMarkedText = false
    }

    override func deactivateServer(_ sender: Any!) {
        // Focus moves away: keep what was typed.
        finish(sender)
        super.deactivateServer(sender)
    }

    /// The system asks for the composition to end (a click in the text, a
    /// change of input source).
    override func commitComposition(_ sender: Any!) {
        finish(sender)
    }

    /// Commit the composition and close the window, asking the client for
    /// nothing: it may be losing focus, and some clients block on queries
    /// then.
    private func finish(_ sender: Any?) {
        session.flush()
        CandidateWindow.shared.hide()
        guard let client = textInput(sender) else { return }
        if let text = session.takeCommit() {
            client.insertText(text, replacementRange: Self.notFound)
        } else if hasMarkedText {
            client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: Self.notFound)
        }
        hasMarkedText = false
    }

    /// A mode chosen in the input menu. The value is the mode's key in
    /// tsInputModeListKey, which here is also its TISInputSourceID.
    override func setValue(_ value: Any!, forTag tag: Int, client sender: Any!) {
        defer { super.setValue(value, forTag: tag, client: sender) }
        guard let id = value as? String, let mode = InputModes.mode(id) else { return }
        shownMode = mode
        if mode != .latin { lastSource = mode }
        if session.mode != mode {
            session.mode = mode
            update(sender)
        }
    }

    // MARK: Keys

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask([.keyDown, .flagsChanged]).rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event else { return false }
        switch event.type {
        case .flagsChanged:
            return flagsChanged(event, sender)
        case .keyDown:
            return keyDown(event, sender)
        default:
            return false
        }
    }

    private func flagsChanged(_ event: NSEvent, _ sender: Any?) -> Bool {
        let code = Int(event.keyCode)
        let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask).subtracting(.capsLock)
        let isOption = code == kVK_Option || code == kVK_RightOption
        if isOption && flags == .option {
            optionAlone = true
            return false
        }
        let tapped = isOption && optionAlone && !flags.contains(.option)
        optionAlone = false
        guard tapped, Session.altToggles else { return false }
        session.key(Int32(SK_TOGGLE))
        update(sender)
        return true
    }

    private func keyDown(_ event: NSEvent, _ sender: Any?) -> Bool {
        optionAlone = false
        switch KeyMap.action(for: event) {
        case .shortcut:
            // Shortcuts belong to the application; finish the composition first.
            session.flush()
            update(sender)
            return false
        case .latin:
            if session.mode != .latin {
                session.mode = .latin
                update(sender)
            }
            return true
        case .hangul:
            if session.mode == .latin {
                session.key(Int32(SK_TOGGLE))
                update(sender)
            }
            return true
        case let .key(kind, ch, shift):
            let eaten = session.key(kind, ch, shift: shift)
            update(sender)
            return eaten
        }
    }

    /// Choose candidate `index` on the page, as if its digit was typed.
    private func pick(_ index: Int) {
        session.key(Int32(SK_PRINTABLE), 0x31 + UInt32(index))
        update(nil)
    }

    // MARK: Drawing the session

    private func textInput(_ sender: Any?) -> IMKTextInput? {
        if let c = sender as? IMKTextInput {
            return c
        }
        return client()
    }

    /// Show the session's state: committed text, marked text, candidates and
    /// the mode in the input menu.
    private func update(_ sender: Any?) {
        guard let client = textInput(sender) else { return }
        if let text = session.takeCommit() {
            // Replaces the marked text, if any.
            client.insertText(text, replacementRange: Self.notFound)
            hasMarkedText = false
        }

        let runs = session.preedit()
        if runs.isEmpty {
            if hasMarkedText {
                client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: Self.notFound)
                hasMarkedText = false
            }
        } else {
            let marked = NSMutableAttributedString()
            var caret: Int?
            for run in runs {
                let start = marked.length
                let range = NSRange(location: start, length: (run.text as NSString).length)
                let style: Int
                switch run.attr {
                case 2: style = kTSMHiliteSelectedConvertedText
                case 1: style = kTSMHiliteConvertedText
                default: style = kTSMHiliteRawText
                }
                let attributes = mark(forStyle: style, at: range) as? [NSAttributedString.Key: Any]
                    ?? [.underlineStyle: NSUnderlineStyle.single.rawValue]
                marked.append(NSAttributedString(string: run.text, attributes: attributes))
                if run.attr == 2 && caret == nil {
                    caret = start
                }
            }
            client.setMarkedText(marked, selectionRange: NSRange(location: caret ?? marked.length, length: 0), replacementRange: Self.notFound)
            hasMarkedText = true
        }

        if let candidates = session.candidates() {
            var line = NSRect.zero
            _ = client.attributes(forCharacterIndex: 0, lineHeightRectangle: &line)
            CandidateWindow.shared.show(candidates, under: line) { [weak self] index in
                self?.pick(index)
            }
        } else {
            CandidateWindow.shared.hide()
        }

        let source = inputSource(for: session.mode)
        if source != shownMode {
            shownMode = source
            if source != .latin { lastSource = source }
            client.selectMode(InputModes.id(source))
        }
    }

    /// The input source that shows `mode`. With 韓㐎 專用 the session reports
    /// Hangul for 釋讀 as well, and the source the user picked stays.
    private func inputSource(for mode: Session.Mode) -> Session.Mode {
        mode == .hangul && Session.hangulOnly && lastSource == .seokdok ? .seokdok : mode
    }

    // MARK: Menu

    override func menu() -> NSMenu! {
        let menu = NSMenu()
        let current = shownMode ?? session.mode
        let items: [(String, Selector, Session.Mode)] = [
            ("釋讀 Seokdok", #selector(selectSeokdok(_:)), .seokdok),
            ("한글 Hangul", #selector(selectHangul(_:)), .hangul),
            ("A Latin", #selector(selectLatin(_:)), .latin),
        ]
        for (title, action, mode) in items {
            let item = NSMenuItem(title: title, action: action, keyEquivalent: "")
            item.state = mode == current ? .on : .off
            menu.addItem(item)
        }
        menu.addItem(.separator())
        menu.addItem(NSMenuItem(title: "設定 / Settings…", action: #selector(showPreferences(_:)), keyEquivalent: ""))
        return menu
    }

    // IMK calls menu actions on the controller with a dictionary as the
    // sender, so each mode has an action of its own.
    @objc private func selectSeokdok(_ sender: Any?) { choose(.seokdok) }
    @objc private func selectHangul(_ sender: Any?) { choose(.hangul) }
    @objc private func selectLatin(_ sender: Any?) { choose(.latin) }

    private func choose(_ mode: Session.Mode) {
        if mode != .latin { lastSource = mode }
        session.mode = mode
        update(nil)
    }

    override func showPreferences(_ sender: Any!) {
        SettingsWindow.shared.show()
    }
}
