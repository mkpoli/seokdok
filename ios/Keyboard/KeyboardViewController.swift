import UIKit

/// The 釋讀 keyboard: 두벌식 keys feed the Rust session; the text being
/// composed is the document's marked text (as azooKey does), confirmed text
/// is inserted, and the candidates show in the bar above the keys.
final class KeyboardViewController: UIInputViewController {
    private var session: Session!
    private var keys: KeyboardView!
    private var shift = false
    /// Marked text is in the document.
    private var marked = false

    static let appGroup = "group.li.mkpo.seokdok"

    override func viewDidLoad() {
        super.viewDidLoad()
        let dict = Bundle.main.resourceURL!.appendingPathComponent("dict").path
        let shared = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: Self.appGroup)?.path
        session = Session(dictDir: dict, configDir: shared)

        keys = KeyboardView(globe: needsInputModeSwitchKey)
        keys.delegate = self
        keys.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(keys)
        NSLayoutConstraint.activate([
            keys.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            keys.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            keys.topAnchor.constraint(equalTo: view.topAnchor),
            keys.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            keys.heightAnchor.constraint(equalToConstant: 268),
        ])
        if let globe = keys.globe {
            globe.addTarget(self, action: #selector(handleInputModeList(from:with:)), for: .allTouchEvents)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        session.sync()
        keys.update(shift: shift, mode: session.mode)
        refresh()
    }

    override func viewWillDisappear(_ animated: Bool) {
        session.flush()
        apply()
        super.viewWillDisappear(animated)
    }

    /// Write the session's view into the document and the candidate bar.
    private func apply() {
        let proxy = textDocumentProxy
        if let commit = session.takeCommit() {
            // Replaces the marked text, if any.
            proxy.insertText(commit)
            marked = false
        }
        let runs = session.preedit()
        let text = runs.map(\.text).joined()
        if !text.isEmpty {
            proxy.setMarkedText(text, selectedRange: NSRange(location: (text as NSString).length, length: 0))
            marked = true
        } else if marked {
            proxy.setMarkedText("", selectedRange: NSRange(location: 0, length: 0))
            proxy.unmarkText()
            marked = false
        }
        refresh(runs)
    }

    private func refresh(_ runs: [Session.Run]? = nil) {
        keys.show(candidates: session.candidates(), preedit: runs ?? session.preedit(), loading: !Session.ready)
    }
}

extension KeyboardViewController: KeyboardViewDelegate {
    func letter(_ key: Character) {
        if !session.key(Session.Key.letter, key, shift: shift) {
            textDocumentProxy.insertText(String(shift ? Character(key.uppercased()) : key))
        }
        if shift {
            shift = false
            keys.update(shift: false, mode: session.mode)
        }
        apply()
    }

    func space() {
        if !session.key(Session.Key.space) {
            textDocumentProxy.insertText(" ")
        }
        apply()
    }

    func enter() {
        if !session.key(Session.Key.enter) {
            textDocumentProxy.insertText("\n")
        }
        apply()
    }

    func backspace() {
        if !session.key(Session.Key.backspace) {
            textDocumentProxy.deleteBackward()
        }
        apply()
    }

    func toggleShift() {
        shift.toggle()
        keys.update(shift: shift, mode: session.mode)
    }

    func toggleMode() {
        session.key(Session.Key.toggle)
        apply()
        keys.update(shift: shift, mode: session.mode)
    }

    /// Candidate `index` on the shown page, picked as its digit key.
    func pick(_ index: Int) {
        guard (0..<9).contains(index) else { return }
        session.key(Session.Key.printable, Character(String(index + 1)))
        apply()
    }
}
