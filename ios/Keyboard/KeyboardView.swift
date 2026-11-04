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
    /// The bar shows the controls for what was learned.
    var learnMenu = false
    /// `input.learn`, when this keyboard rather than the app decides it.
    var learnOn: Bool?
    var script: Int32 = 0
    var composing = false
    var spaceConverts = true
    var runs: [Session.Run] = []
    var candidates: Session.Candidates?
    var loading = false
    /// The field's Return does something (Go, Search, Send …) and is blue.
    var returnIsAction = false

    enum Shift { case off, once, locked }
}

/// The keyboard: one candidate bar and 두벌식 keys with the Shift layer in each
/// key's corner. A symbols layer replaces the letters. The bar's chevron opens
/// a drawer with every candidate and the composition's segments.
final class KeyboardView: UIView, UIInputViewAudioFeedback {
    weak var delegate: KeyboardViewDelegate?
    /// The input-mode switch key, when the system asks for one.
    private(set) var globe: GlobeKey?
    private var globeWidth: NSLayoutConstraint?

    private let bar = UIStackView()
    private let barScroll = CandidateScroll()
    private let barEnd = UIStackView()
    private var shownCandidates: Session.Candidates?
    private let more = BarButton(symbol: "chevron.down", label: "More candidates")
    private let drawer = CandidateDrawer()
    private var drawerOpen = false
    private var shownRuns: [Session.Run] = []
    private var barMessage: UIView?
    /// Everything the drawer covers.
    private let column = UIStackView()
    private let keyPad = KeyPad()
    private let popup = KeyPopup()
    private var keysTop: NSLayoutConstraint!
    private let keysArea = UIStackView()
    private let letters = UIStackView()
    private let symbols = UIStackView()
    private let pad = UIStackView()
    private var barRow: UIView!
    private var bottomLine: UIView!
    private var extraKeys: [Key] = []
    private var extraChars: [Character] = []
    private var shownLayout = Layout.standard
    private var letterKeys: [(Character, Key)] = []
    private var symbolKeys: [Key] = []
    private var symbolPage = 0
    private var shiftKey: Key!
    private var symbolPageKey: Key!
    private var modeKey: Key!
    private var layerKey: Key!
    private var spaceKey: Key!
    private var enterKey: Key!

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
        column.translatesAutoresizingMaskIntoConstraints = false
        addSubview(column)
        NSLayoutConstraint.activate([
            column.leadingAnchor.constraint(equalTo: leadingAnchor),
            column.trailingAnchor.constraint(equalTo: trailingAnchor),
            column.topAnchor.constraint(equalTo: topAnchor),
            column.bottomAnchor.constraint(equalTo: bottomAnchor),
        ])

        barRow = scrolling(bar, in: barScroll, trailing: barEnd, height: 45)
        column.addArrangedSubview(barRow)
        column.addArrangedSubview(keyPad)

        // The pad runs to the keyboard's edges so that its margins and the gap
        // under the bar type too. A nearly clear fill, because a keyboard gets
        // no touches where it draws nothing.
        keyPad.backgroundColor = UIColor(white: 1, alpha: 0.01)
        let keysColumn = UIStackView()
        keysColumn.axis = .vertical
        keysColumn.spacing = 12
        keysColumn.translatesAutoresizingMaskIntoConstraints = false
        keyPad.addSubview(keysColumn)
        keysTop = keysColumn.topAnchor.constraint(equalTo: keyPad.topAnchor, constant: 9)
        NSLayoutConstraint.activate([
            keysTop,
            keysColumn.leadingAnchor.constraint(equalTo: keyPad.leadingAnchor, constant: 3),
            keysColumn.trailingAnchor.constraint(equalTo: keyPad.trailingAnchor, constant: -3),
            keysColumn.bottomAnchor.constraint(equalTo: keyPad.bottomAnchor, constant: -3),
        ])
        keyPad.onPreview = { [weak self] key in self?.showPopup(key) }

        letters.axis = .vertical
        letters.spacing = 12
        symbols.axis = .vertical
        symbols.spacing = 12
        keysArea.axis = .vertical
        keysArea.addArrangedSubview(letters)
        keysArea.addArrangedSubview(symbols)
        keysColumn.addArrangedSubview(keysArea)
        buildLetters()
        buildSymbols()
        symbols.isHidden = true
        bottomLine = bottomRow(needsGlobe)
        keysColumn.addArrangedSubview(bottomLine)
        // The pad layouts replace the bar and the keys; five rows with 6
        // between fill the 258 there are.
        pad.axis = .vertical
        pad.spacing = 6
        pad.isHidden = true
        keysColumn.addArrangedSubview(pad)
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
        addSubview(popup)
        popup.frame = bounds
        NSLayoutConstraint.activate([
            drawer.leadingAnchor.constraint(equalTo: leadingAnchor),
            drawer.trailingAnchor.constraint(equalTo: trailingAnchor),
            drawer.topAnchor.constraint(equalTo: topAnchor),
            drawer.bottomAnchor.constraint(equalTo: bottomAnchor),
        ])
        render(KeyboardState())
    }

    /// Open or close the drawer over the keys. In recordings of the system's
    /// Chinese keyboard (iOS 18.5 simulator) its drawer changes within about
    /// 50 ms with no slide, so this is a very short cross-fade.
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

    private static let drawerDuration: TimeInterval = 0.06

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
        row.isLayoutMarginsRelativeArrangement = true
        row.directionalLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: 3, bottom: 0, trailing: 3)
        row.heightAnchor.constraint(equalToConstant: height).isActive = true
        return row
    }

    /// The pop-up over a held key, drawn in the keyboard's own bounds: a custom
    /// keyboard can draw nowhere else, and the bar leaves room over the top row.
    private func showPopup(_ key: Key?) {
        guard let key else {
            popup.hide()
            return
        }
        popup.show(key, over: key.convert(key.bounds, to: self), within: bounds)
    }

    private func letterKey(_ action: @escaping () -> Void) -> Key {
        let k = Key(look: .letter)
        k.previews = true
        k.action = action
        return k
    }

    private func buildLetters() {
        for (r, row) in Self.rows.enumerated() {
            var keys: [UIView] = []
            for ch in row {
                let k = letterKey { [weak self] in self?.tap { $0.letter(ch) } }
                letterKeys.append((ch, k))
                keys.append(k)
            }
            if r == 2 {
                shiftKey = Key(symbol: "shift")
                shiftKey.spoken = "Shift"
                shiftKey.fires = .onPress
                shiftKey.action = { [weak self] in self?.tap { $0.toggleShift() } }
                keys = [shiftKey!] + keys + [backspaceKey()]
            }
            letters.addArrangedSubview(line(keys, inset: r == 1 ? 21 : 0, edgeKeys: r == 2))
        }
    }

    private func buildSymbols() {
        for r in 0..<3 {
            var keys: [UIView] = []
            for i in 0..<Self.symbolRows[0][r].count {
                let k = letterKey { [weak self] in
                    guard let self else { return }
                    let row = Array(Self.symbolRows[self.symbolPage][r])
                    if i < row.count { self.tap { $0.symbol(row[i]) } }
                }
                symbolKeys.append(k)
                keys.append(k)
            }
            if r == 2 {
                symbolPageKey = Key(title: "#+=")
                symbolPageKey.action = { [weak self] in
                    guard let self else { return }
                    UIDevice.current.playInputClick()
                    self.symbolPage = 1 - self.symbolPage
                    self.labelSymbols()
                }
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
        symbolPageKey.title = symbolPage == 0 ? "#+=" : "123"
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

    /// Backspace fires as the finger lands and repeats while held.
    private func backspaceKey(height: CGFloat = KeyMetrics.keyHeight) -> Key {
        let k = Key(symbol: "delete.left", height: height)
        k.spoken = "Delete"
        k.fires = .onPress
        k.repeats = true
        k.action = { [weak self] in self?.tap { $0.backspace() } }
        return k
    }

    private func bottomRow(_ needsGlobe: Bool) -> UIView {
        let bottom = UIStackView()
        bottom.axis = .horizontal
        bottom.spacing = 6
        layerKey = Key(title: "123")
        layerKey.action = { [weak self] in
            guard let self else { return }
            UIDevice.current.playInputClick()
            self.symbolPage = 0
            self.labelSymbols()
            self.letters.isHidden.toggle()
            self.symbols.isHidden = !self.letters.isHidden
            self.relabelLayerKey()
        }
        layerKey.widthAnchor.constraint(equalToConstant: 46).isActive = true
        bottom.addArrangedSubview(layerKey)
        if needsGlobe {
            let g = GlobeKey()
            globe = g
            keyPad.passThrough = g
            bottom.addArrangedSubview(g)
            globeWidth = g.widthAnchor.constraint(equalToConstant: 40)
            globeWidth?.isActive = true
        }
        for i in 0..<2 {
            let k = letterKey { [weak self] in
                guard let self, i < self.extraChars.count else { return }
                let ch = self.extraChars[i]
                self.tap { $0.symbol(ch) }
            }
            k.isHidden = true
            k.widthAnchor.constraint(equalToConstant: 40).isActive = true
            extraKeys.append(k)
            bottom.addArrangedSubview(k)
        }
        modeKey = Key(title: "釋")
        modeKey.spoken = "Input mode"
        modeKey.action = { [weak self] in self?.tap { $0.cycleMode() } }
        modeKey.widthAnchor.constraint(equalToConstant: 46).isActive = true
        modeKey.onHold = { [weak self] in self?.tap { $0.toggleLearnMenu() } }
        spaceKey = Key(title: "띄어쓰기", look: .letter)
        spaceKey.action = { [weak self] in self?.tap { $0.space() } }
        enterKey = Key(title: "改行")
        enterKey.action = { [weak self] in self?.tap { $0.enter() } }
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
        layerKey.title = lettersShown ? "123" : (lastMode == .latin ? "ABC" : "가")
    }

    /// Show the keyboard state: key labels and the candidate bar.
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
        shiftKey.isOn = shifted
        shiftKey.setSymbol(s.shift == .locked ? "capslock.fill" : shifted ? "shift.fill" : "shift")
        modeKey.title = s.mode == .seokdok && s.script == 1 ? "混" : ["釋", "한", "A"][Int(s.mode.rawValue)]
        let converts = s.composing && s.mode == .seokdok && (s.spaceConverts || s.candidates != nil)
        spaceKey.title = converts ? "變換" : "띄어쓰기"
        enterKey.title = s.composing ? "確定" : "改行"
        enterKey.look = s.returnIsAction && !s.composing ? .accent : .function
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
        keysTop.constant = pads ? 0 : 9
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
            sizeGlobe(width: 40, height: KeyMetrics.keyHeight)
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
        func key(_ ch: Character) -> Key {
            let k = Key(title: String(ch), look: .letter, height: Self.padKeyHeight)
            k.titleFont = .systemFont(ofSize: 24, weight: .regular)
            k.action = { [weak self] in self?.tap { $0.symbol(ch) } }
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
        let enter = Key(title: "改行", height: Self.padKeyHeight)
        enter.action = { [weak self] in self?.tap { $0.enter() } }
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
            if s.learnMenu && s.runs.isEmpty {
                showLearnControls(s)
            } else if s.mode == .seokdok, let input = s.runs.first(where: { $0.attr == 0 }) {
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

    /// Held 釋 key: what was learned can be switched off and cleared here,
    /// because a keyboard without Full Access is out of the app's reach.
    private func showLearnControls(_ s: KeyboardState) {
        let korean = Locale.preferredLanguages.first?.hasPrefix("ko") == true
        func control(_ title: String, _ act: @escaping (KeyboardViewDelegate) -> Void) {
            let cell = CandidateCell(.init(text: title, reading: ""))
            cell.heightAnchor.constraint(equalToConstant: 36).isActive = true
            cell.addAction(UIAction { [weak self] _ in self?.tap(act) }, for: .touchUpInside)
            bar.addArrangedSubview(cell)
        }
        if let on = s.learnOn {
            control(on ? (korean ? "學習 끄기" : "Stop learning") : (korean ? "學習 켜기" : "Learn picks")) { $0.setLearn(!on) }
        }
        control(korean ? "고른 것 잊기" : "Forget picks") { $0.forgetPicks() }
    }

    /// An initial-only jamo (ᄼ) needs a vowel filler to show on its own.
    private func shown(_ j: String) -> String {
        guard let v = j.unicodeScalars.first?.value, (0x1100...0x115F).contains(v) else { return j }
        return j + "\u{1160}"
    }
}
