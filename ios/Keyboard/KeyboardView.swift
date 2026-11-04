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
    func focus(run: Int)
    func resize(_ step: Int)
    func toggleLearnMenu()
    func forgetPicks()
    func setLearn(_ on: Bool)
}

/// What the text field asks for (`textDocumentProxy.keyboardType` and secure
/// entry). Only `standard` composes and converts; the others type their
/// characters as they are.
enum Layout {
    case standard
    /// Latin letters: ASCII-only and secure fields.
    case ascii
    case email
    case url
    /// Digits only (number pad).
    case number
    /// Digits and the decimal separator (decimal pad).
    case decimal
    /// Digits, * # and + (phone pad).
    case phone

    var converts: Bool { self == .standard }
    var isPad: Bool { self == .number || self == .decimal || self == .phone }
}

/// What the keys and the bars show; built by the controller from the session.
struct KeyboardState {
    var layout = Layout.standard
    var shift: Shift = .off
    var mode: Session.Mode = .seokdok
    /// The strip shows the controls for what was learned.
    var learnMenu = false
    /// `input.learn`, when this keyboard rather than the app decides it.
    var learnOn: Bool?
    var script: Int32 = 0
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
    private(set) var globe: KeyButton?
    private var globeWidth: NSLayoutConstraint?

    private let strip = UIStackView()
    private let stripScroll = UIScrollView()
    private let resizeKeys = UIStackView()
    private let bar = UIStackView()
    private let barScroll = CandidateScroll()
    private let barEnd = UIStackView()
    private var shownCandidates: Session.Candidates?
    private let keysArea = UIStackView()
    private let letters = UIStackView()
    private let symbols = UIStackView()
    private let pad = UIStackView()
    private var stripRow: UIView!
    private var barRow: UIView!
    private var bottomLine: UIView!
    private var extraKeys: [KeyButton] = []
    private var extraChars: [Character] = []
    private var shownLayout = Layout.standard
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

        stripRow = scrolling(strip, in: stripScroll, trailing: resizeKeys, height: 26)
        barRow = scrolling(bar, in: barScroll, trailing: barEnd, height: 44)
        column.addArrangedSubview(stripRow)
        column.addArrangedSubview(barRow)
        column.setCustomSpacing(8, after: barRow)
        resizeKeys.addArrangedSubview(smallKey("−", label: "Shorter segment") { $0.resize(-1) })
        resizeKeys.addArrangedSubview(smallKey("+", label: "Longer segment") { $0.resize(1) })

        letters.axis = .vertical
        letters.spacing = 10
        symbols.axis = .vertical
        symbols.spacing = 10
        keysArea.axis = .vertical
        keysArea.addArrangedSubview(letters)
        keysArea.addArrangedSubview(symbols)
        column.addArrangedSubview(keysArea)
        buildLetters()
        buildSymbols()
        symbols.isHidden = true
        bottomLine = bottomRow(needsGlobe)
        column.addArrangedSubview(bottomLine)
        // The pad layouts replace the strip, the bars and the keys; five rows
        // of 50 with 7 between fill the 278 the column has.
        pad.axis = .vertical
        pad.spacing = 7
        pad.isHidden = true
        column.addArrangedSubview(pad)
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
    private func backspaceKey(height: CGFloat = 42) -> KeyButton {
        let k = KeyButton(symbol: "delete.left", height: height)
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
            globeWidth = g.widthAnchor.constraint(equalToConstant: 40)
            globeWidth?.isActive = true
        }
        for i in 0..<2 {
            let k = KeyButton(title: "")
            k.isHidden = true
            k.widthAnchor.constraint(equalToConstant: 40).isActive = true
            k.addAction(UIAction { [weak self] _ in
                guard let self, i < self.extraChars.count else { return }
                let ch = self.extraChars[i]
                self.tap { $0.symbol(ch) }
            }, for: .touchUpInside)
            extraKeys.append(k)
            bottom.addArrangedSubview(k)
        }
        modeKey = KeyButton(title: "釋")
        modeKey.accessibilityLabel = "Input mode"
        modeKey.addAction(UIAction { [weak self] _ in self?.tap { $0.cycleMode() } }, for: .touchUpInside)
        modeKey.widthAnchor.constraint(equalToConstant: 46).isActive = true
        modeKey.addGestureRecognizer(UILongPressGestureRecognizer(target: self, action: #selector(modeKeyHeld(_:))))
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

    @objc private func modeKeyHeld(_ press: UILongPressGestureRecognizer) {
        if press.state == .began {
            tap { $0.toggleLearnMenu() }
        }
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
        showLayout(s.layout)
        relabelLayerKey()
        let shifted = s.shift != .off
        let latin = s.mode == .latin || !s.layout.converts
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
        modeKey.setTitle(s.mode == .seokdok && s.script == 1 ? "混" : ["釋", "한", "A"][Int(s.mode.rawValue)], for: .normal)
        let converts = s.composing && s.mode == .seokdok && (s.spaceConverts || s.candidates != nil)
        spaceKey.setTitle(converts ? "變換" : "띄어쓰기", for: .normal)
        enterKey.setTitle(s.composing ? "確定" : "改行", for: .normal)
        showStrip(s)
        showBar(s)
    }

    /// Switch between the Hangul keyboard, the Latin one for ASCII fields and
    /// the number pads. A field that cannot take Hangul has no strip, mode key
    /// or candidates.
    private func showLayout(_ layout: Layout) {
        guard layout != shownLayout else { return }
        shownLayout = layout
        let pads = layout.isPad
        stripRow.isHidden = pads
        barRow.isHidden = pads
        keysArea.isHidden = pads
        bottomLine.isHidden = pads
        pad.isHidden = !pads
        modeKey.isHidden = !layout.converts
        extraChars = layout == .email ? ["@", "."] : layout == .url ? [".", "/"] : []
        for (i, k) in extraKeys.enumerated() {
            k.isHidden = i >= extraChars.count
            if i < extraChars.count { k.set(main: String(extraChars[i]), hint: nil) }
        }
        if pads {
            buildPad(layout)
        } else if let g = globe, let bottom = bottomLine as? UIStackView {
            bottom.insertArrangedSubview(g, at: 1)
            sizeGlobe(width: 40, height: 42)
        }
    }

    /// Three columns of digits; the fourth row and the bottom row differ by
    /// pad. The globe moves into the bottom row while a pad is shown.
    private func buildPad(_ layout: Layout) {
        pad.arrangedSubviews.forEach { $0.removeFromSuperview() }
        func row(_ views: [UIView]) -> UIStackView {
            let r = UIStackView(arrangedSubviews: views)
            r.axis = .horizontal
            r.spacing = 6
            r.distribution = .fillEqually
            return r
        }
        func key(_ ch: Character) -> KeyButton {
            let k = KeyButton(title: String(ch), height: 50)
            k.titleLabel?.font = .systemFont(ofSize: 24, weight: .regular)
            k.addAction(UIAction { [weak self] _ in self?.tap { $0.symbol(ch) } }, for: .touchUpInside)
            return k
        }
        for digits in ["123", "456", "789"] {
            pad.addArrangedSubview(row(digits.map(key)))
        }
        switch layout {
        case .phone:
            pad.addArrangedSubview(row(["*", "0", "#"].map { key($0) }))
        case .decimal:
            pad.addArrangedSubview(row([key(Character(Locale.current.decimalSeparator ?? ".")), key("0"), backspaceKey(height: 50)]))
        default:
            pad.addArrangedSubview(row([UIView(), key("0"), backspaceKey(height: 50)]))
        }
        var bottom: [UIView] = []
        if let g = globe { bottom.append(g) }
        if layout == .phone {
            bottom.append(key("+"))
            bottom.append(backspaceKey(height: 50))
        }
        let enter = KeyButton(title: "改行", accent: true, height: 50)
        enter.addAction(UIAction { [weak self] _ in self?.tap { $0.enter() } }, for: .touchUpInside)
        bottom.append(enter)
        let r = row(bottom)
        r.distribution = .fill
        for v in bottom where v !== enter && v !== globe {
            v.widthAnchor.constraint(equalToConstant: 70).isActive = true
        }
        sizeGlobe(width: 50, height: 50)
        pad.addArrangedSubview(r)
    }

    /// The globe is one button that moves between the letter row and the pad
    /// row, so its two size constraints change with the row instead of new
    /// ones piling up against them.
    private func sizeGlobe(width: CGFloat, height: CGFloat) {
        globeWidth?.constant = width
        globe?.heightConstraint.constant = height
    }

    /// The text being composed. Tapping the Hangul converts it (in 釋讀
    /// mode); tapping a converted segment focuses it.
    private func showStrip(_ s: KeyboardState) {
        strip.arrangedSubviews.forEach { $0.removeFromSuperview() }
        let converted = s.runs.contains { $0.attr != 0 }
        resizeKeys.isHidden = !converted
        if s.learnMenu && s.runs.isEmpty {
            func control(_ title: String, _ act: @escaping (KeyboardViewDelegate) -> Void) {
                let b = UIButton(type: .system)
                b.setTitle(title, for: .normal)
                b.titleLabel?.font = .systemFont(ofSize: 14)
                b.addAction(UIAction { [weak self] _ in self?.tap(act) }, for: .touchUpInside)
                strip.addArrangedSubview(b)
            }
            if let on = s.learnOn {
                control(on ? "學習 끄기 · Stop learning" : "學習 켜기 · Learn picks") { $0.setLearn(!on) }
            }
            control("고른 것 잊기 · Forget picks") { $0.forgetPicks() }
            return
        }
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

    /// The focused segment's candidates, each with its short reading beside
    /// it. The cells stay while only the selection moves.
    private func showBar(_ s: KeyboardState) {
        guard s.candidates != shownCandidates else { return }
        let old = shownCandidates
        shownCandidates = s.candidates
        guard let c = s.candidates else {
            bar.arrangedSubviews.forEach { $0.removeFromSuperview() }
            return
        }
        if old?.items != c.items {
            bar.arrangedSubviews.forEach { $0.removeFromSuperview() }
            for (i, item) in c.items.enumerated() {
                let cell = CandidateCell(item)
                cell.addAction(UIAction { [weak self] _ in self?.tap { $0.pick(i) } }, for: .touchUpInside)
                bar.addArrangedSubview(cell)
            }
            barScroll.contentOffset = .zero
        }
        for (i, v) in bar.arrangedSubviews.enumerated() {
            (v as? CandidateCell)?.isSelected = i == c.selected
        }
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

/// A scroll view of candidates. Touches reach the cells at once so that they
/// highlight, and a drag starting on a cell still scrolls.
final class CandidateScroll: UIScrollView {
    override init(frame: CGRect) {
        super.init(frame: frame)
        delaysContentTouches = false
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    override func touchesShouldCancel(in view: UIView) -> Bool {
        true
    }
}

/// One candidate: its text with the reading beside it, smaller, on the same
/// baseline, so that candidates line up whether they have a reading or not.
final class CandidateCell: UIControl {
    private let label = UILabel()

    init(_ item: Session.Candidates.Item) {
        super.init(frame: .zero)
        layer.cornerRadius = 6
        let text = NSMutableAttributedString(string: item.text, attributes: [
            .font: OldHangul.font(ofSize: 19),
            .foregroundColor: UIColor.label,
        ])
        if !item.reading.isEmpty {
            text.append(NSAttributedString(string: "\u{2009}" + item.reading, attributes: [
                .font: OldHangul.font(ofSize: 12),
                .foregroundColor: UIColor.secondaryLabel,
            ]))
        }
        label.attributedText = text
        label.isUserInteractionEnabled = false
        label.translatesAutoresizingMaskIntoConstraints = false
        addSubview(label)
        NSLayoutConstraint.activate([
            label.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 9),
            label.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -9),
            label.centerYAnchor.constraint(equalTo: centerYAnchor),
            heightAnchor.constraint(equalToConstant: 36),
        ])
        accessibilityTraits = .keyboardKey
        accessibilityLabel = item.reading.isEmpty ? item.text : item.text + ", " + item.reading
        isAccessibilityElement = true
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    override var isSelected: Bool {
        didSet { refill() }
    }

    override var isHighlighted: Bool {
        didSet { refill() }
    }

    private func refill() {
        backgroundColor = isHighlighted
            ? UIColor.label.withAlphaComponent(0.14)
            : isSelected ? UIColor.systemBlue.withAlphaComponent(0.18) : .clear
    }
}

/// A key: the main label and an optional corner hint, larger and darker when
/// the hint is an old letter.
final class KeyButton: UIButton {
    private let hint = UILabel()
    private let accent: Bool
    /// The key's height, changed in place when a row needs another.
    private(set) var heightConstraint: NSLayoutConstraint!

    init(title: String? = nil, symbol: String? = nil, accent: Bool = false, height: CGFloat = 42) {
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
        heightConstraint = heightAnchor.constraint(equalToConstant: height)
        heightConstraint.isActive = true
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
        let old = text.map(Self.isOld) ?? false
        hint.font = OldHangul.font(ofSize: old ? 14 : 11, weight: old ? .bold : .medium)
        hint.textColor = old ? .label : .secondaryLabel
    }

    private static func isOld(_ letter: String) -> Bool {
        guard let v = letter.unicodeScalars.first?.value else { return false }
        return "ㆍㅿㆁㆆ".unicodeScalars.contains { $0.value == v } || (0x1100...0x115F).contains(v)
    }
}
