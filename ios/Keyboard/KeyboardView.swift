import UIKit

protocol KeyboardViewDelegate: AnyObject {
    func letter(_ key: Character)
    func symbol(_ ch: Character)
    func space()
    func enter()
    func backspace()
    func toggleShift()
    func cycleMode()
    func pick(_ index: Int)
    func convert()
    func page(_ step: Int)
    func focus(run: Int)
    func resize(_ step: Int)
}

/// What the keys and the bars show; built by the controller from the session.
struct KeyboardState {
    var shift: Shift = .off
    var mode: Session.Mode = .seokdok
    var composing = false
    var spaceConverts = true
    var runs: [Session.Run] = []
    var candidates: Session.Candidates?
    var loading = false

    enum Shift { case off, once, locked }
}

/// The keyboard: a strip with the text being composed (tap the Hangul to
/// convert it, a converted segment to focus it),
/// the candidate bar, and 두벌식 keys with the Shift layer in each key's
/// corner. A symbols layer replaces the letters.
final class KeyboardView: UIView, UIInputViewAudioFeedback {
    weak var delegate: KeyboardViewDelegate?
    /// The input-mode switch key, when the system asks for one.
    private(set) var globe: UIButton?

    private let strip = UIStackView()
    private let stripScroll = UIScrollView()
    private let resizeKeys = UIStackView()
    private let bar = UIStackView()
    private let barScroll = UIScrollView()
    private let pageKeys = UIStackView()
    private let letters = UIStackView()
    private let symbols = UIStackView()
    private var letterKeys: [(Character, KeyButton)] = []
    private var symbolKeys: [KeyButton] = []
    private var symbolPage = 0
    private var shiftKey: KeyButton!
    private var symbolPageKey: KeyButton!
    private var modeKey: KeyButton!
    private var layerKey: KeyButton!
    private var spaceKey: KeyButton!
    private var enterKey: KeyButton!
    private var repeatTimer: Timer?

    private static let rows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"]
    /// Two pages of symbols, three rows each, as on the system keyboard.
    private static let symbolRows: [[String]] = [
        ["1234567890", "-/:;()₩&@\"", ".,?!'"],
        ["[]{}#%^*+=", "_\\|~<>$€£·", ".,?!'"],
    ]

    var enableInputClicksWhenVisible: Bool { true }

    init(globe needsGlobe: Bool) {
        super.init(frame: .zero)
        backgroundColor = .clear

        let column = UIStackView()
        column.axis = .vertical
        column.spacing = 6
        column.translatesAutoresizingMaskIntoConstraints = false
        addSubview(column)
        NSLayoutConstraint.activate([
            column.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 3),
            column.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -3),
            column.topAnchor.constraint(equalTo: topAnchor, constant: 2),
            column.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -4),
        ])

        column.addArrangedSubview(scrolling(strip, in: stripScroll, trailing: resizeKeys, height: 26))
        column.addArrangedSubview(scrolling(bar, in: barScroll, trailing: pageKeys, height: 44))
        column.setCustomSpacing(8, after: column.arrangedSubviews[1])
        resizeKeys.addArrangedSubview(smallKey("−", label: "Shorter segment") { $0.resize(-1) })
        resizeKeys.addArrangedSubview(smallKey("+", label: "Longer segment") { $0.resize(1) })
        pageKeys.addArrangedSubview(smallKey("‹", label: "Previous page") { $0.page(-1) })
        pageKeys.addArrangedSubview(smallKey("›", label: "Next page") { $0.page(1) })

        letters.axis = .vertical
        letters.spacing = 10
        symbols.axis = .vertical
        symbols.spacing = 10
        column.addArrangedSubview(letters)
        column.addArrangedSubview(symbols)
        buildLetters()
        buildSymbols()
        symbols.isHidden = true
        column.addArrangedSubview(bottomRow(needsGlobe))
        render(KeyboardState())
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    private func scrolling(_ stack: UIStackView, in scroll: UIScrollView, trailing: UIStackView, height: CGFloat) -> UIView {
        scroll.showsHorizontalScrollIndicator = false
        stack.axis = .horizontal
        stack.spacing = 4
        stack.alignment = .center
        stack.translatesAutoresizingMaskIntoConstraints = false
        scroll.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: scroll.contentLayoutGuide.leadingAnchor, constant: 6),
            stack.trailingAnchor.constraint(equalTo: scroll.contentLayoutGuide.trailingAnchor, constant: -6),
            stack.topAnchor.constraint(equalTo: scroll.contentLayoutGuide.topAnchor),
            stack.bottomAnchor.constraint(equalTo: scroll.contentLayoutGuide.bottomAnchor),
            stack.heightAnchor.constraint(equalTo: scroll.frameLayoutGuide.heightAnchor),
        ])
        trailing.axis = .horizontal
        trailing.spacing = 4
        trailing.setContentHuggingPriority(.required, for: .horizontal)
        trailing.setContentCompressionResistancePriority(.required, for: .horizontal)
        let row = UIStackView(arrangedSubviews: [scroll, trailing])
        row.axis = .horizontal
        row.spacing = 4
        row.heightAnchor.constraint(equalToConstant: height).isActive = true
        return row
    }

    private func smallKey(_ title: String, label: String, _ act: @escaping (KeyboardViewDelegate) -> Void) -> UIButton {
        let b = UIButton(type: .system)
        b.setTitle(title, for: .normal)
        b.titleLabel?.font = .systemFont(ofSize: 20, weight: .medium)
        b.tintColor = .label
        b.accessibilityLabel = label
        b.widthAnchor.constraint(equalToConstant: 34).isActive = true
        b.addAction(UIAction { [weak self] _ in
            guard let self, let d = self.delegate else { return }
            UIDevice.current.playInputClick()
            act(d)
        }, for: .touchUpInside)
        return b
    }

    private func buildLetters() {
        for (r, row) in Self.rows.enumerated() {
            var keys: [UIView] = []
            for ch in row {
                let k = KeyButton()
                k.addAction(UIAction { [weak self] _ in self?.tap { $0.letter(ch) } }, for: .touchUpInside)
                letterKeys.append((ch, k))
                keys.append(k)
            }
            if r == 2 {
                shiftKey = KeyButton(symbol: "shift")
                shiftKey.accessibilityLabel = "Shift"
                shiftKey.addAction(UIAction { [weak self] _ in self?.tap { $0.toggleShift() } }, for: .touchDown)
                keys = [shiftKey!] + keys + [backspaceKey()]
            }
            letters.addArrangedSubview(line(keys, inset: r == 1 ? 18 : 0, edgeKeys: r == 2))
        }
    }

    private func buildSymbols() {
        for r in 0..<3 {
            var keys: [UIView] = []
            for i in 0..<Self.symbolRows[0][r].count {
                let k = KeyButton()
                k.addAction(UIAction { [weak self] _ in
                    guard let self else { return }
                    let row = Array(Self.symbolRows[self.symbolPage][r])
                    if i < row.count { self.tap { $0.symbol(row[i]) } }
                }, for: .touchUpInside)
                symbolKeys.append(k)
                keys.append(k)
            }
            if r == 2 {
                symbolPageKey = KeyButton(title: "#+=")
                symbolPageKey.addAction(UIAction { [weak self] _ in
                    guard let self else { return }
                    UIDevice.current.playInputClick()
                    self.symbolPage = 1 - self.symbolPage
                    self.labelSymbols()
                }, for: .touchUpInside)
                keys = [symbolPageKey!] + keys + [backspaceKey()]
            }
            symbols.addArrangedSubview(line(keys, inset: 0, edgeKeys: r == 2))
        }
        labelSymbols()
    }

    private func labelSymbols() {
        let chars = Self.symbolRows[symbolPage].flatMap { Array($0) }
        for (k, ch) in zip(symbolKeys, chars) {
            k.set(main: String(ch), hint: nil)
        }
        symbolPageKey.setTitle(symbolPage == 0 ? "#+=" : "123", for: .normal)
    }

    /// A row of keys; `edgeKeys` makes the first and last (Shift, Backspace)
    /// wider and keeps the letters between them equal.
    private func line(_ keys: [UIView], inset: CGFloat, edgeKeys: Bool) -> UIView {
        let row = UIStackView()
        row.axis = .horizontal
        row.spacing = 5
        if edgeKeys, let first = keys.first, let last = keys.last {
            let middle = UIStackView(arrangedSubviews: Array(keys.dropFirst().dropLast()))
            middle.axis = .horizontal
            middle.spacing = 5
            middle.distribution = .fillEqually
            [first, middle, last].forEach(row.addArrangedSubview)
            first.widthAnchor.constraint(equalToConstant: 46).isActive = true
            last.widthAnchor.constraint(equalToConstant: 46).isActive = true
            row.spacing = 10
        } else {
            row.distribution = .fillEqually
            keys.forEach(row.addArrangedSubview)
        }
        guard inset > 0 else { return row }
        // The middle letter row is set in by half a key, as on a real keyboard.
        let wrap = UIStackView(arrangedSubviews: [row])
        wrap.isLayoutMarginsRelativeArrangement = true
        wrap.directionalLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: inset, bottom: 0, trailing: inset)
        return wrap
    }

    /// Backspace repeats while held.
    private func backspaceKey() -> KeyButton {
        let k = KeyButton(symbol: "delete.left")
        k.accessibilityLabel = "Delete"
        k.addAction(UIAction { [weak self] _ in
            guard let self else { return }
            self.tap { $0.backspace() }
            self.repeatTimer?.invalidate()
            self.repeatTimer = Timer.scheduledTimer(withTimeInterval: 0.45, repeats: false) { [weak self] _ in
                self?.repeatTimer = Timer.scheduledTimer(withTimeInterval: 0.08, repeats: true) { [weak self] _ in
                    self?.delegate?.backspace()
                }
            }
        }, for: .touchDown)
        k.addAction(UIAction { [weak self] _ in
            self?.repeatTimer?.invalidate()
            self?.repeatTimer = nil
        }, for: [.touchUpInside, .touchUpOutside, .touchCancel])
        return k
    }

    private func bottomRow(_ needsGlobe: Bool) -> UIView {
        let bottom = UIStackView()
        bottom.axis = .horizontal
        bottom.spacing = 5
        layerKey = KeyButton(title: "123")
        layerKey.addAction(UIAction { [weak self] _ in
            guard let self else { return }
            UIDevice.current.playInputClick()
            self.symbolPage = 0
            self.labelSymbols()
            self.letters.isHidden.toggle()
            self.symbols.isHidden = !self.letters.isHidden
            self.relabelLayerKey()
        }, for: .touchUpInside)
        layerKey.widthAnchor.constraint(equalToConstant: 46).isActive = true
        bottom.addArrangedSubview(layerKey)
        if needsGlobe {
            let g = KeyButton(symbol: "globe")
            g.accessibilityLabel = "Next keyboard"
            globe = g
            bottom.addArrangedSubview(g)
            g.widthAnchor.constraint(equalToConstant: 40).isActive = true
        }
        modeKey = KeyButton(title: "釋")
        modeKey.accessibilityLabel = "Input mode"
        modeKey.addAction(UIAction { [weak self] _ in self?.tap { $0.cycleMode() } }, for: .touchUpInside)
        modeKey.widthAnchor.constraint(equalToConstant: 46).isActive = true
        spaceKey = KeyButton(title: "띄어쓰기")
        spaceKey.addAction(UIAction { [weak self] _ in self?.tap { $0.space() } }, for: .touchUpInside)
        enterKey = KeyButton(title: "改行", accent: true)
        enterKey.addAction(UIAction { [weak self] _ in self?.tap { $0.enter() } }, for: .touchUpInside)
        enterKey.widthAnchor.constraint(equalToConstant: 84).isActive = true
        for k in [modeKey!, spaceKey!, enterKey!] {
            bottom.addArrangedSubview(k)
        }
        return bottom
    }

    private func tap(_ act: (KeyboardViewDelegate) -> Void) {
        UIDevice.current.playInputClick()
        if let d = delegate { act(d) }
    }

    private var lettersShown: Bool { !letters.isHidden }
    private var lastMode: Session.Mode = .seokdok

    private func relabelLayerKey() {
        layerKey.setTitle(lettersShown ? "123" : (lastMode == .latin ? "ABC" : "가"), for: .normal)
    }

    /// Show the keyboard state: key labels, the strip and the candidate bar.
    func render(_ s: KeyboardState) {
        lastMode = s.mode
        relabelLayerKey()
        let shifted = s.shift != .off
        let latin = s.mode == .latin
        for (ch, key) in letterKeys {
            let up = Character(ch.uppercased())
            if latin {
                key.set(main: String(shifted ? up : ch), hint: nil)
                continue
            }
            let main = Session.jamo(shifted ? up : ch) ?? String(ch)
            let other = Session.jamo(shifted ? ch : up)
            let hint = other == main ? nil : other
            key.set(main: shown(main), hint: hint.map(shown))
        }
        shiftKey.isSelected = shifted
        shiftKey.setImage(UIImage(systemName: s.shift == .locked ? "capslock.fill" : "shift.fill"), for: .selected)
        modeKey.setTitle(["釋", "한", "A"][Int(s.mode.rawValue)], for: .normal)
        let converts = s.composing && s.mode == .seokdok && (s.spaceConverts || s.candidates != nil)
        spaceKey.setTitle(converts ? "變換" : "띄어쓰기", for: .normal)
        enterKey.setTitle(s.composing ? "確定" : "改行", for: .normal)
        showStrip(s)
        showBar(s)
    }

    /// The text being composed. Tapping the Hangul converts it (in 釋讀
    /// mode); tapping a converted segment focuses it.
    private func showStrip(_ s: KeyboardState) {
        strip.arrangedSubviews.forEach { $0.removeFromSuperview() }
        let converted = s.runs.contains { $0.attr != 0 }
        resizeKeys.isHidden = !converted
        if s.runs.isEmpty {
            let label = UILabel()
            label.font = .systemFont(ofSize: 14)
            label.textColor = .secondaryLabel
            label.text = s.loading ? "辭典 讀는 中…" : nil
            strip.addArrangedSubview(label)
            return
        }
        for (i, run) in s.runs.enumerated() {
            let b = UIButton(type: .system)
            b.setTitle(run.text, for: .normal)
            b.titleLabel?.font = OldHangul.font(ofSize: 16)
            b.tintColor = run.attr == 0 ? .secondaryLabel : .label
            b.backgroundColor = run.attr == 2 ? UIColor.systemBlue.withAlphaComponent(0.22) : .clear
            b.layer.cornerRadius = 4
            b.isUserInteractionEnabled = run.attr != 0 || s.mode == .seokdok
            b.addAction(UIAction { [weak self] _ in
                self?.tap { run.attr == 0 ? $0.convert() : $0.focus(run: i) }
            }, for: .touchUpInside)
            strip.addArrangedSubview(b)
        }
    }

    /// The candidates of the focused segment with their notes, and the page
    /// position.
    private func showBar(_ s: KeyboardState) {
        bar.arrangedSubviews.forEach { $0.removeFromSuperview() }
        guard let c = s.candidates else {
            pageKeys.isHidden = true
            return
        }
        pageKeys.isHidden = c.total <= c.items.count
        for (i, item) in c.items.enumerated() {
            var conf = UIButton.Configuration.plain()
            conf.title = item.text
            conf.subtitle = item.note.isEmpty ? nil : item.note
            conf.titleAlignment = .center
            conf.titleTextAttributesTransformer = UIConfigurationTextAttributesTransformer { incoming in
                var out = incoming
                out.font = OldHangul.font(ofSize: 19)
                return out
            }
            conf.subtitleTextAttributesTransformer = UIConfigurationTextAttributesTransformer { incoming in
                var out = incoming
                out.font = OldHangul.font(ofSize: 10)
                out.foregroundColor = UIColor.secondaryLabel
                return out
            }
            conf.baseForegroundColor = .label
            conf.contentInsets = NSDirectionalEdgeInsets(top: 2, leading: 8, bottom: 2, trailing: 8)
            conf.background.cornerRadius = 6
            conf.background.backgroundColor = i == c.selected ? UIColor.systemBlue.withAlphaComponent(0.18) : .clear
            let b = UIButton(configuration: conf)
            b.addAction(UIAction { [weak self] _ in self?.tap { $0.pick(i) } }, for: .touchUpInside)
            bar.addArrangedSubview(b)
        }
        let count = UILabel()
        count.text = "\(c.position)/\(c.total)"
        count.font = .systemFont(ofSize: 12)
        count.textColor = .secondaryLabel
        bar.addArrangedSubview(count)
        // Keep the selected candidate in view.
        layoutIfNeeded()
        if c.items.indices.contains(c.selected) {
            let v = bar.arrangedSubviews[c.selected]
            barScroll.scrollRectToVisible(v.convert(v.bounds, to: barScroll), animated: false)
        }
    }

    /// An initial-only jamo (ᄼ) needs a vowel filler to show on its own.
    private func shown(_ j: String) -> String {
        guard let v = j.unicodeScalars.first?.value, (0x1100...0x115F).contains(v) else { return j }
        return j + "\u{1160}"
    }
}

/// A key: the main label and an optional corner hint.
final class KeyButton: UIButton {
    private let hint = UILabel()
    private let accent: Bool

    init(title: String? = nil, symbol: String? = nil, accent: Bool = false) {
        self.accent = accent
        super.init(frame: .zero)
        layer.cornerRadius = 6
        layer.shadowColor = UIColor.black.cgColor
        layer.shadowOpacity = 0.25
        layer.shadowOffset = CGSize(width: 0, height: 1)
        layer.shadowRadius = 0
        backgroundColor = accent ? .systemBlue : .secondarySystemBackground
        setTitleColor(accent ? .white : .label, for: .normal)
        titleLabel?.font = OldHangul.font(ofSize: title == nil ? 22 : 15, weight: .medium)
        titleLabel?.adjustsFontSizeToFitWidth = true
        if let title { setTitle(title, for: .normal) }
        if let symbol {
            setImage(UIImage(systemName: symbol), for: .normal)
            setImage(UIImage(systemName: symbol + ".fill"), for: .selected)
            tintColor = .label
        }
        heightAnchor.constraint(equalToConstant: 42).isActive = true
        hint.font = OldHangul.font(ofSize: 11, weight: .medium)
        hint.textColor = .secondaryLabel
        hint.translatesAutoresizingMaskIntoConstraints = false
        addSubview(hint)
        NSLayoutConstraint.activate([
            hint.topAnchor.constraint(equalTo: topAnchor, constant: 2),
            hint.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -4),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    func set(main: String, hint text: String?) {
        setTitle(main, for: .normal)
        hint.text = text
    }
}
