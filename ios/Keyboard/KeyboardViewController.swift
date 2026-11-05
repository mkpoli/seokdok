import UIKit

/// The 釋讀 keyboard: 두벌식 keys feed the Rust session; the text being
/// composed is the document's marked text (as azooKey does), confirmed text
/// is inserted, and the focused segment's candidates show in the bar above
/// the keys. The document's marked text is the only copy of the composition on
/// screen.
final class KeyboardViewController: UIInputViewController {
    private var session: Session!
    private var keys: KeyboardView!
    private var shift = KeyboardState.Shift.off
    private var lastShiftTap = Date.distantPast
    /// Shift is on because the field asked for a capital, and the next letter or a
    /// change of place may take it off again.
    private var shiftIsAuto = false
    /// Shift was tapped off where the field wanted a capital; it stays off for one letter.
    private var shiftDeclined = false
    /// The marked text this keyboard last wrote into the document, if any.
    private var marked: String?
    /// The document the marked text was written into
    /// (`textDocumentProxy.documentIdentifier`).
    private var composingDocument: UUID?
    private var loadTimer: Timer?
    /// Bumped whenever a composition ends or its field changes, so a delayed
    /// check for an earlier text change does nothing.
    private var checkGeneration = 0
    /// What the focused field asks for; only `.standard` converts.
    private var layout = Layout.standard
    /// A long press on the mode key opened the bar's controls for what was
    /// learned. Without Full Access the app cannot reach this keyboard's files.
    private var learnMenu = false

    /// The simulator shows the system's own globe, so the keyboard asks for
    /// none of its own. A Debug build can be made to show one for the UI
    /// tests: `defaults write li.mkpo.seokdok.keyboard forceGlobe -bool YES`.
    private static var forcedGlobe: Bool {
        #if DEBUG
        return UserDefaults.standard.bool(forKey: "forceGlobe")
        #else
        return false
        #endif
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        let dict = Bundle.main.bundleURL.appendingPathComponent("dict").path
        Session.setConfigDir(configDir())
        session = Session(dictDir: dict)

        keys = KeyboardView(globe: needsInputModeSwitchKey || Self.forcedGlobe)
        keys.delegate = self
        keys.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(keys)
        // As tall as the system keyboard with its suggestion bar.
        let height = keys.heightAnchor.constraint(equalToConstant: 261)
        // Below required, so it gives way to the system's own height while
        // the keyboard is being set up.
        height.priority = UILayoutPriority(999)
        NSLayoutConstraint.activate([
            keys.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            keys.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            keys.topAnchor.constraint(equalTo: view.topAnchor),
            keys.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            height,
        ])
        if let globe = keys.globe {
            globe.addTarget(self, action: #selector(handleInputModeList(from:with:)), for: .allTouchEvents)
        }
    }

    /// The App Group container holding the app's settings. Reading it needs
    /// Full Access and a signature that grants the group; without them the
    /// keyboard runs on the defaults.
    private func configDir() -> String? {
        guard hasFullAccess, let group = AppGroup.containerURL?.path else { return nil }
        adoptOwnPicks(into: group)
        return group
    }

    /// Picks kept while Full Access was off are in this keyboard's own folder,
    /// out of the app's reach. They move to the shared folder, where the app's
    /// button clears them; a table already there wins.
    private func adoptOwnPicks(into group: String) {
        let manager = FileManager.default
        let own = NSHomeDirectory() + "/.config/seokdok/learned.tsv"
        guard manager.fileExists(atPath: own) else { return }
        let shared = group + "/learned.tsv"
        if manager.fileExists(atPath: shared) {
            try? manager.removeItem(atPath: own)
        } else {
            try? manager.moveItem(atPath: own, toPath: shared)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        Session.setConfigDir(configDir())
        session.sync()
        keys.feedback.hapticsAllowed = hasFullAccess
        updateLayout()
        refresh()
        waitForDictionaries()
    }

    override func viewWillDisappear(_ animated: Bool) {
        session.flush()
        apply()
        loadTimer?.invalidate()
        super.viewWillDisappear(animated)
    }

    /// The text changed outside the keyboard (the cursor moved, the app
    /// cleared the field): the document now holds the marked text as plain
    /// text, so drop the composition instead of writing it again. The proxy's
    /// context can lag behind our own setMarkedText, by more than a moment
    /// while the extension is starting, so a mismatch counts only when it
    /// persists through several looks; judging too soon split the first jamo
    /// off a word (ㄱㅏ for 가). A change of field is told apart by its layout
    /// or its document identifier, ends the composition at once and cancels any
    /// check still waiting.
    override func textDidChange(_ textInput: UITextInput?) {
        super.textDidChange(textInput)
        if updateLayout() || dropForeignComposition() { return }
        guard marked != nil else { return }
        checkGeneration += 1
        checkComposition(generation: checkGeneration, looksLeft: 4)
    }

    private func checkComposition(generation: Int, looksLeft: Int) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
            guard let self, generation == self.checkGeneration, let m = self.marked else { return }
            let before = self.textDocumentProxy.documentContextBeforeInput ?? ""
            if before.hasSuffix(m) { return }
            if looksLeft > 1 {
                self.checkComposition(generation: generation, looksLeft: looksLeft - 1)
            } else {
                self.abandonComposition()
            }
        }
    }

    /// `textDocumentProxy.documentIdentifier`, or nil while the system has
    /// none to give. The property is declared non-optional, and reading it
    /// as a UUID traps when the system returns nil (seen in the simulator
    /// during a field change), so it is read through key-value coding.
    private var documentIdentifier: UUID? {
        (textDocumentProxy as? NSObject)?.value(forKey: "documentIdentifier") as? UUID
    }

    /// A composition typed into another document is left there. Called when
    /// the text changes and before each key, so a key tapped right after a
    /// field switch starts a new composition in the new field.
    @discardableResult
    private func dropForeignComposition() -> Bool {
        guard marked != nil, let doc = composingDocument, let now = documentIdentifier,
              doc != now else { return false }
        abandonComposition()
        return true
    }

    /// Forget the composition without writing it: the document it was typed
    /// into already holds it as plain text, and the document now in focus
    /// does not own it.
    private func abandonComposition() {
        checkGeneration += 1
        session.reset()
        marked = nil
        composingDocument = nil
        refresh()
    }

    /// The layout the field in focus asks for. Number pads and the like
    /// type digits as they are; email, URL, ASCII and secure fields type
    /// Latin letters. Neither converts. (The system swaps in its own keyboard
    /// for secure and phone fields, but a field can still reach this one.)
    private func fieldLayout() -> Layout {
        let proxy = textDocumentProxy
        if proxy.isSecureTextEntry == true { return .ascii }
        switch proxy.keyboardType ?? .default {
        case .numberPad, .asciiCapableNumberPad: return .number
        case .decimalPad: return .decimal
        case .phonePad: return .phone
        case .emailAddress: return .email
        case .URL: return .url
        case .asciiCapable, .namePhonePad: return .ascii
        default: return .standard
        }
    }

    /// A field whose content the system describes as private, on a keyboard
    /// type that would otherwise convert. Asked as a composition starts, since
    /// a composition can still be open when the focus has moved on.
    private func fieldIsPrivate() -> Bool {
        guard let type = textDocumentProxy.textContentType ?? nil else { return false }
        let hidden: [UITextContentType] = [.password, .newPassword, .oneTimeCode, .username, .emailAddress, .URL, .telephoneNumber, .creditCardNumber]
        return hidden.contains(type)
    }

    /// Follow the field in focus. A field that wants another layout is
    /// another field, so what was being composed stays behind in the old one
    /// (the system keeps its marked text as plain text) instead of being
    /// inserted into the new one. Returns whether the layout changed.
    @discardableResult
    private func updateLayout() -> Bool {
        let next = fieldLayout()
        guard next != layout else { return false }
        if marked != nil || !session.preedit().isEmpty {
            abandonComposition()
        }
        layout = next
        refresh()
        return true
    }

    /// Until the dictionaries load, the bar says so; refresh once they have.
    private func waitForDictionaries() {
        guard !Session.ready, loadTimer == nil else { return }
        loadTimer = Timer.scheduledTimer(withTimeInterval: 0.25, repeats: true) { [weak self] t in
            guard Session.ready else { return }
            t.invalidate()
            self?.loadTimer = nil
            self?.refresh()
        }
    }

    /// Write the session's view into the document and the keyboard.
    private func apply() {
        learnMenu = false
        let proxy = textDocumentProxy
        if let commit = session.takeCommit() {
            // Replaces the marked text, if any.
            proxy.insertText(commit)
            marked = nil
        }
        let text = session.preedit().map(\.text).joined()
        if !text.isEmpty {
            if marked == nil { composingDocument = documentIdentifier }
            proxy.setMarkedText(text, selectedRange: NSRange(location: (text as NSString).length, length: 0))
            marked = text
        } else if marked != nil {
            proxy.setMarkedText("", selectedRange: NSRange(location: 0, length: 0))
            proxy.unmarkText()
            marked = nil
        }
        refresh()
    }

    /// Latin letters capitalize as the field asks, as on the system's English
    /// keyboard; its Korean keyboard does nothing of the kind on Hangul, nor do its
    /// email and web fields here. `autocapitalizationType` is the field's own trait,
    /// though the Settings switch is out of a keyboard's reach (see ios/README.md).
    private func updateAutoShift() {
        guard layout == .ascii || (layout.converts && session.mode == .latin) else {
            if shiftIsAuto { shift = .off }
            shiftIsAuto = false
            return
        }
        guard shift != .locked else { return }
        let before = textDocumentProxy.documentContextBeforeInput ?? ""
        let wanted: Bool
        switch textDocumentProxy.autocapitalizationType ?? .sentences {
        case .none: wanted = false
        case .allCharacters: wanted = true
        case .words: wanted = before.isEmpty || before.last?.isWhitespace == true
        default:
            let trimmed = before.reversed().drop { $0 == " " }
            wanted = before.isEmpty || trimmed.isEmpty || trimmed.first?.isNewline == true
                || (before.last == " " && [".", "!", "?"].contains(trimmed.first))
        }
        if wanted, shift == .off, !shiftDeclined {
            shift = .once
            shiftIsAuto = true
        } else if !wanted, shiftIsAuto {
            shift = .off
            shiftIsAuto = false
        }
    }

    private func refresh() {
        updateAutoShift()
        let runs = session.preedit()
        var s = KeyboardState()
        s.layout = layout
        s.shift = shift
        s.mode = session.mode
        s.script = Session.script
        s.composing = !runs.isEmpty
        s.spaceConverts = Session.spaceConverts
        s.runs = runs
        s.candidates = session.candidates()
        s.loading = !Session.ready
        switch textDocumentProxy.returnKeyType ?? .default {
        case .go, .google, .join, .route, .search, .send, .yahoo, .done, .emergencyCall, .continue:
            s.returnIsAction = true
        default:
            s.returnIsAction = false
        }
        s.learnMenu = learnMenu
        s.learnOn = hasFullAccess && AppGroup.containerURL != nil ? nil : Session.learn
        keys.render(s)
    }

    /// Insert text the session did not take.
    private func insert(_ text: String) {
        textDocumentProxy.insertText(text)
    }
}

extension KeyboardViewController: KeyboardViewDelegate {
    func letter(_ key: Character) {
        dropForeignComposition()
        let shifted = shift != .off
        guard layout.converts else {
            insert(String(shifted ? Character(key.uppercased()) : key))
            if shift == .once { shift = .off }
            shiftIsAuto = false
            shiftDeclined = false
            refresh()
            return
        }
        if session.preedit().isEmpty {
            session.setPrivateField(fieldIsPrivate())
        }
        if !session.key(Session.Key.letter, key, shift: shifted) {
            insert(String(shifted ? Character(key.uppercased()) : key))
        }
        if shift == .once {
            shift = .off
        }
        shiftIsAuto = false
        shiftDeclined = false
        apply()
    }

    func symbol(_ ch: Character) {
        dropForeignComposition()
        guard layout.converts else {
            insert(String(ch))
            refresh()
            return
        }
        // Digits pick candidates after Space has walked the list; close it
        // first so a digit confirms the conversion and is typed after it.
        if session.digitsPick, ch.isNumber {
            session.key(Session.Key.esc)
        }
        if !session.key(Session.Key.printable, ch) {
            insert(String(ch))
        }
        apply()
    }

    func space() {
        dropForeignComposition()
        guard layout.converts else {
            insert(" ")
            refresh()
            return
        }
        if !session.key(Session.Key.space) {
            insert(" ")
        }
        apply()
    }

    func enter() {
        dropForeignComposition()
        guard layout.converts else {
            insert("\n")
            refresh()
            return
        }
        if !session.key(Session.Key.enter) {
            insert("\n")
        }
        apply()
    }

    func backspace() {
        dropForeignComposition()
        guard layout.converts else {
            textDocumentProxy.deleteBackward()
            refresh()
            return
        }
        if !session.key(Session.Key.backspace) {
            textDocumentProxy.deleteBackward()
        }
        apply()
    }

    /// What is being composed goes into the document first, with no word break
    /// owed afterwards: the caret is leaving.
    func beginCursor() {
        session.flush()
        apply()
        session.reset()
        refresh()
    }

    /// Step over whole characters, so the caret never lands inside an old
    /// Hangul cluster or a surrogate pair. `adjustTextPosition` counts UTF-16
    /// units. The context can be empty where the proxy cannot see the text;
    /// the steps are then single units.
    func moveCursor(by steps: Int) {
        let proxy = textDocumentProxy
        let context = (steps < 0 ? proxy.documentContextBeforeInput : proxy.documentContextAfterInput) ?? ""
        guard !context.isEmpty else {
            proxy.adjustTextPosition(byCharacterOffset: steps)
            return
        }
        let span = steps < 0 ? context.suffix(-steps) : context.prefix(steps)
        let units = span.utf16.count
        proxy.adjustTextPosition(byCharacterOffset: steps < 0 ? -units : units)
    }

    /// Tap: Shift for one letter; double tap: locked until tapped again.
    func toggleShift() {
        let now = Date()
        switch shift {
        case .off:
            shift = .once
        case .once:
            shift = now.timeIntervalSince(lastShiftTap) < 0.35 && !shiftIsAuto ? .locked : .off
            shiftDeclined = shift == .off && shiftIsAuto
        case .locked:
            shift = .off
        }
        lastShiftTap = now
        shiftIsAuto = false
        refresh()
    }

    /// 釋 → 混 → 한 → A → 釋. The app's settings screen follows the script,
    /// so it does not write the old one back.
    func cycleMode() {
        session.modeControl()
        if hasFullAccess {
            AppGroup.defaults?.set(["seokdok", "sino_only", "hangul_only"][Int(Session.script)], forKey: "script")
        }
        apply()
    }

    /// A long press on the mode key: the controls for what was learned, here
    /// because a keyboard without Full Access is out of the app's reach.
    func toggleLearnMenu() {
        learnMenu.toggle()
        refresh()
    }

    func forgetPicks() {
        Session.forgetPicks()
        learnMenu = false
        session.sync()
        refresh()
    }

    func setLearn(_ on: Bool) {
        Session.learn = on
        learnMenu = false
        session.sync()
        refresh()
    }

    func pick(_ index: Int) {
        session.pick(index)
        apply()
    }

    func convert() {
        session.key(Session.Key.convert)
        apply()
    }

    /// Move the focus to segment `run`, one segment at a time (the session
    /// skips segments with nothing to choose).
    func focus(run target: Int) {
        for _ in 0..<session.preedit().count {
            guard let at = session.preedit().firstIndex(where: { $0.attr == 2 }), at != target else { break }
            session.key(target < at ? Session.Key.left : Session.Key.right)
            if session.preedit().firstIndex(where: { $0.attr == 2 }) == at { break }
        }
        apply()
    }

    /// Shorten (-1) or lengthen (+1) the focused segment by a syllable.
    func resize(_ step: Int) {
        session.key(step < 0 ? Session.Key.left : Session.Key.right, shift: true)
        apply()
    }
}
