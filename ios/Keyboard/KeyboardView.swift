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
    var script: Int32 = 0
    var composing = false
    var spaceConverts = true
    var runs: [Session.Run] = []
    var candidates: Session.Candidates?
    var loading = false

    enum Shift { case off, once, locked }
}

/// The keyboard: one candidate bar and 두벌식 keys with the Shift layer in each
/// key's corner. A symbols layer replaces the letters. The bar's chevron opens
/// a drawer with every candidate and the composition's segments.
final class KeyboardView: UIView, UIInputViewAudioFeedback {
    weak var delegate: KeyboardViewDelegate?
    /// The input-mode switch key, when the system asks for one.
    private(set) var globe: KeyButton?
    private var globeWidth: NSLayoutConstraint?

    private let bar = UIStackView()
    private let barScroll = UIScrollView()
    private let barEnd = UIStackView()
    private var shownCandidates: Session.Candidates?
    private let more = BarButton(symbol: "chevron.down", label: "More candidates")
    private let drawer = CandidateDrawer()
    private var drawerOpen = false
    private var shownRuns: [Session.Run] = []
    private var barMessage: UIView?
    /// Everything the drawer covers.
    private let column = UIStackView()
    private let keysArea = UIStackView()
    private let letters = UIStackView()
    private let symbols = UIStackView()
    private let pad = UIStackView()
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

    private static let padKeyHeight: CGFloat = 46.8
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

        column.axis = .vertical
        column.spacing = 12
        column.translatesAutoresizingMaskIntoConstraints = false
        addSubview(column)
        NSLayoutConstraint.activate([
            column.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 3),
            column.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -3),
            column.topAnchor.constraint(equalTo: topAnchor),
            column.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -3),
        ])

        barRow = scrolling(bar, in: barScroll, trailing: barEnd, height: 45)
        barScroll.delaysContentTouches = false
        column.addArrangedSubview(barRow)
        column.setCustomSpacing(9, after: barRow)

        letters.axis = .vertical
        letters.spacing = 12
        symbols.axis = .vertical
        symbols.spacing = 12
        keysArea.axis = .vertical
        keysArea.addArrangedSubview(letters)
        keysArea.addArrangedSubview(symbols)
        column.addArrangedSubview(keysArea)
        buildLetters()
        buildSymbols()
        symbols.isHidden = true
        bottomLine = bottomRow(needsGlobe)
        column.addArrangedSubview(bottomLine)
        // The pad layouts replace the bar and the keys; five rows with 6
        // between fill the 258 the column has.
        pad.axis = .vertical
        pad.spacing = 6
        pad.isHidden = true
        column.addArrangedSubview(pad)
        barEnd.addArrangedSubview(more)
        more.isHidden = true
        let divider = UIView()
        divider.backgroundColor = .separator
        divider.isUserInteractionEnabled = false
        divider.translatesAutoresizingMaskIntoConstraints = false
        more.addSubview(divider)
        NSLayoutConstraint.activate([
            divider.leadingAnchor.constraint(equalTo: more.leadingAnchor),
            divider.centerYAnchor.constraint(equalTo: more.centerYAnchor),
            divider.widthAnchor.constraint(equalToConstant: 1 / UIScreen.main.scale),
            divider.heightAnchor.constraint(equalToConstant: 28),
        ])
        more.addAction(UIAction { [weak self] _ in
            UIDevice.current.playInputClick()
            self?.setDrawer(open: true)
        }, for: .touchUpInside)
        drawer.onPick = { [weak self] i in
            self?.setDrawer(open: false)
            self?.tap { $0.pick(i) }
        }
        drawer.onFocus = { [weak self] run in self?.tap { $0.focus(run: run) } }
        drawer.onResize = { [weak self] step in self?.tap { $0.resize(step) } }
        drawer.close.addAction(UIAction { [weak self] _ in
            UIDevice.current.playInputClick()
            self?.setDrawer(open: false)
        }, for: .touchUpInside)
        drawer.isHidden = true
        drawer.translatesAutoresizingMaskIntoConstraints = false
        addSubview(drawer)
        NSLayoutConstraint.activate([
            drawer.leadingAnchor.constraint(equalTo: leadingAnchor),
            drawer.trailingAnchor.constraint(equalTo: trailingAnchor),
            drawer.topAnchor.constraint(equalTo: topAnchor),
            drawer.bottomAnchor.constraint(equalTo: bottomAnchor),
        ])
        render(KeyboardState())
    }

    /// Open or close the drawer over the keys, on the curve the system uses
    /// for its keyboard's own drawer.
    private func setDrawer(open: Bool) {
        guard open != drawerOpen else { return }
        drawerOpen = open
        if open, let c = shownCandidates {
            drawer.isHidden = false
            drawer.alpha = 0
            drawer.show(c, runs: shownRuns)
        }
        UIView.animate(withDuration: Self.drawerDuration, delay: 0, options: [.curveEaseInOut, .beginFromCurrentState]) {
            self.drawer.alpha = open ? 1 : 0
            self.column.alpha = open ? 0 : 1
        } completion: { _ in
            guard !self.drawerOpen else { return }
            self.drawer.isHidden = true
            self.drawer.clear()
        }
    }

    private static let drawerDuration: TimeInterval = 0.25

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
            letters.addArrangedSubview(line(keys, inset: r == 1 ? 21 : 0, edgeKeys: r == 2))
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
        row.spacing = 6
        if edgeKeys, let first = keys.first, let last = keys.last {
            let middle = UIStackView(arrangedSubviews: Array(keys.dropFirst().dropLast()))
            middle.axis = .horizontal
            middle.spacing = 6
            middle.distribution = .fillEqually
            [first, middle, last].forEach(row.addArrangedSubview)
            first.widthAnchor.constraint(equalToConstant: 45).isActive = true
            last.widthAnchor.constraint(equalToConstant: 45).isActive = true
            row.spacing = 15
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
        bottom.spacing = 6
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
        spaceKey = KeyButton(title: "띄어쓰기")
        spaceKey.addAction(UIAction { [weak self] _ in self?.tap { $0.space() } }, for: .touchUpInside)
        enterKey = KeyButton(title: "改行", accent: true)
        enterKey.addAction(UIAction { [weak self] _ in self?.tap { $0.enter() } }, for: .touchUpInside)
        enterKey.widthAnchor.constraint(equalToConstant: 94).isActive = true
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
        shownRuns = s.runs
        showBar(s)
    }

    /// Switch between the Hangul keyboard, the Latin one for ASCII fields and
    /// the number pads. A field that cannot take Hangul has no mode key or
    /// candidates.
    private func showLayout(_ layout: Layout) {
        guard layout != shownLayout else { return }
        shownLayout = layout
        let pads = layout.isPad
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
            let k = KeyButton(title: String(ch), height: Self.padKeyHeight)
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
            pad.addArrangedSubview(row([key(Character(Locale.current.decimalSeparator ?? ".")), key("0"), backspaceKey(height: Self.padKeyHeight)]))
        default:
            pad.addArrangedSubview(row([UIView(), key("0"), backspaceKey(height: Self.padKeyHeight)]))
        }
        var bottom: [UIView] = []
        if let g = globe { bottom.append(g) }
        if layout == .phone {
            bottom.append(key("+"))
            bottom.append(backspaceKey(height: Self.padKeyHeight))
        }
        let enter = KeyButton(title: "改行", accent: true, height: Self.padKeyHeight)
        enter.addAction(UIAction { [weak self] _ in self?.tap { $0.enter() } }, for: .touchUpInside)
        bottom.append(enter)
        let r = row(bottom)
        r.distribution = .fill
        for v in bottom where v !== enter && v !== globe {
            v.widthAnchor.constraint(equalToConstant: 70).isActive = true
        }
        sizeGlobe(width: 50, height: Self.padKeyHeight)
        pad.addArrangedSubview(r)
    }

    /// The globe is one button that moves between the letter row and the pad
    /// row, so its two size constraints change with the row instead of new
    /// ones piling up against them.
    private func sizeGlobe(width: CGFloat, height: CGFloat) {
        globeWidth?.constant = width
        globe?.heightConstraint.constant = height
    }

    /// The bar: the focused segment's candidates, or while nothing is
    /// converted the Hangul being typed, which a tap converts, or the note that
    /// the dictionaries are loading.
    private func showBar(_ s: KeyboardState) {
        barMessage?.removeFromSuperview()
        barMessage = nil
        guard let c = s.candidates else {
            shownCandidates = nil
            bar.arrangedSubviews.forEach { $0.removeFromSuperview() }
            more.isHidden = true
            setDrawer(open: false)
            if s.mode == .seokdok, let input = s.runs.first(where: { $0.attr == 0 }) {
                let cell = CandidateCell(.init(text: input.text, reading: ""))
                cell.heightAnchor.constraint(equalToConstant: 36).isActive = true
                cell.accessibilityHint = "Convert"
                cell.addAction(UIAction { [weak self] _ in self?.tap { $0.convert() } }, for: .touchUpInside)
                bar.addArrangedSubview(cell)
                barMessage = cell
            } else if s.loading {
                let label = UILabel()
                label.font = .systemFont(ofSize: 14)
                label.textColor = .secondaryLabel
                label.text = "辭典 讀는 中…"
                bar.addArrangedSubview(label)
                barMessage = label
            }
            return
        }
        more.isHidden = false
        if drawerOpen {
            drawer.show(c, runs: s.runs)
        }
        guard c != shownCandidates else { return }
        let old = shownCandidates
        shownCandidates = c
        if old?.items != c.items {
            bar.arrangedSubviews.forEach { $0.removeFromSuperview() }
            for (i, item) in c.items.enumerated() {
                let cell = CandidateCell(item)
                cell.heightAnchor.constraint(equalToConstant: 36).isActive = true
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
