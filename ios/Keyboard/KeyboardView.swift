import UIKit

protocol KeyboardViewDelegate: AnyObject {
    func letter(_ key: Character)
    /// The letter this key would type with Shift the other way, for once.
    func letterFlipped(_ key: Character)
    func symbol(_ ch: Character)
    /// Text as it is, past any composition (the .com key).
    func literal(_ text: String)
    func space()
    func enter()
    func backspace()
    /// Held: a whole syllable of what is being composed, or one character.
    func backspaceRepeat()
    /// Held long enough: what is being composed by syllables, or the word before the caret.
    func backspaceWord()
    func toggleShift()
    func cycleMode()
    func pick(_ index: Int)
    func convert()
    func focus(run: Int)
    func resize(_ step: Int)
    /// Space was held: end any composition so the caret can move.
    func beginCursor()
    /// Move the caret this many characters.
    func moveCursor(by steps: Int)
    func toggleLearnMenu()
    func forgetPicks()
    func setLearn(_ on: Bool)
}

/// What the text field asks for (`textDocumentProxy.keyboardType` and secure
/// entry). Only the Hangul layouts compose and convert; the others type their
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
    /// Hangul with the symbols showing first (numbers and punctuation).
    case numbers
    /// Hangul with @ and # by the space bar.
    case twitter
    /// Hangul with a period by the space bar (web search).
    case web

    var converts: Bool { self == .standard || self == .numbers || self == .twitter || self == .web }
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
    /// What the field calls its Return key (`textDocumentProxy.returnKeyType`).
    var returnKey = ReturnKey.plain
    /// The field wants text first (`enablesReturnKeyAutomatically`) and has none.
    var returnDisabled = false

    enum Shift { case off, once, locked }
}

/// The Return key's label and colour by what the field asks for. Before iOS 26 the
/// system's Korean keyboard says 이동, 검색, 연결, 다음, 경로, 전송, 완료, 긴급통화 and
/// 계속, here in the keyboard's own spelling; from iOS 26 it draws a symbol for each.
/// 다음 and 계속 are not blue on either, the rest are.
enum ReturnKey {
    case plain, go, search, join, next, route, send, done, emergency, proceed

    init(_ type: UIReturnKeyType) {
        switch type {
        case .go: self = .go
        case .google, .search, .yahoo: self = .search
        case .join: self = .join
        case .next: self = .next
        case .route: self = .route
        case .send: self = .send
        case .done: self = .done
        case .emergencyCall: self = .emergency
        case .continue: self = .proceed
        default: self = .plain
        }
    }

    /// None for a plain Return, which shows the system's arrow.
    var title: String? {
        switch self {
        case .plain: return nil
        case .go: return "移動"
        case .search: return "檢索"
        case .join: return "連結"
        case .next: return "다음"
        case .route: return "經路"
        case .send: return "傳送"
        case .done: return "完了"
        case .emergency: return "緊急通話"
        case .proceed: return "繼續"
        }
    }

    /// What iOS 26's keyboard draws.
    var symbol: String {
        switch self {
        case .plain: return "return.left"
        case .go, .join, .route: return "arrow.right"
        case .search: return "magnifyingglass"
        case .next, .proceed: return "chevron.right"
        case .send: return "arrow.up"
        case .done: return "checkmark"
        case .emergency: return "phone"
        }
    }

    var isAction: Bool {
        switch self {
        case .plain, .next, .proceed: return false
        default: return true
        }
    }
}

/// The keyboard: one candidate bar and 두벌식 keys with the Shift layer in each
/// key's corner. A symbols layer replaces the letters. The bar's chevron opens
/// a drawer with every candidate and the composition's segments.
final class KeyboardView: UIView {
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
    private var alternatesKey: Key?
    private var keysTop: NSLayoutConstraint!
    private var keysLeading: NSLayoutConstraint!
    private var keysTrailing: NSLayoutConstraint!
    private var keysBottom: NSLayoutConstraint!
    private var barHeight: NSLayoutConstraint!
    private let keysColumn = UIStackView()
    private let keysArea = UIStackView()
    private let letters = UIStackView()
    private let symbols = UIStackView()
    private let pad = UIStackView()
    private var barRow: UIView!
    private var bottomLine: UIView!
    private var extraKeys: [Key] = []
    private var extraTexts: [String] = []
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
    private var layout: KeyLayout?
    private var plainRows: [UIStackView] = []
    private var edgeRows: [(row: UIStackView, middle: UIStackView, widths: [NSLayoutConstraint])] = []
    private var insetRow: UIStackView?
    private var layerWidth: NSLayoutConstraint!
    private var modeWidth: NSLayoutConstraint!
    private var bottomStack: UIStackView { bottomLine as! UIStackView }

    /// The number pads, measured from the system keyboard's in iOS 18.5 and iOS
    /// 26.5 simulators; the landscape one is our choice, since none was captured.
    private struct PadMetrics: Equatable {
        let side, gap, key, between, top, bottom: CGFloat
        static var current: PadMetrics {
            if UIScreen.main.bounds.width > UIScreen.main.bounds.height {
                return PadMetrics(side: 3, gap: 6, key: 36, between: 4, top: 4, bottom: 3)
            }
            if #available(iOS 26, *) {
                return PadMetrics(side: 2.3, gap: 7.3, key: 48, between: 6, top: 6, bottom: 0)
            }
            return PadMetrics(side: 3, gap: 6, key: 47, between: 6.33, top: 6, bottom: 3)
        }
    }

    private static let padLetters: [Character: String] = [
        "2": "ABC", "3": "DEF", "4": "GHI", "5": "JKL", "6": "MNO", "7": "PQRS", "8": "TUV", "9": "WXYZ",
    ]
    private static let rows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"]
    /// What a long press on a symbol offers, and which of them is the key: read from
    /// the system's Korean keyboard, iOS 26.5 simulator.
    private static let symbolAlternates: [Character: (list: [String], main: Int)] = [
        "0": (["°", "0"], 1),
        "-": (["-", "–", "—", "•"], 0),
        "/": (["/", "\\"], 0),
        "$": (["₽", "¥", "€", "$", "¢", "£", "₩"], 3),
        "₩": (["₽", "¥", "€", "$", "¢", "£", "₩"], 6),
        "&": (["&", "§"], 0),
        "\"": (["«", "»", "„", "“", "”", "\""], 5),
        ".": ([".", "…"], 0),
        "?": (["?", "¿"], 0),
        "!": (["!", "¡"], 0),
        "'": (["`", "‘", "’", "'"], 3),
    ]
    /// Two pages of symbols, three rows each, as on the system keyboard.
    private static let symbolRows: [[String]] = [
        ["1234567890", "-/:;()₩&@\"", ".,?!'"],
        ["[]{}#%^*+=", "_\\|~<>$€£·", ".,?!'"],
    ]

    /// Clicks and haptics for everything pressed in the keyboard.
    let feedback = KeyFeedback()

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
        keysColumn.axis = .vertical
        keysColumn.spacing = 12
        keysColumn.translatesAutoresizingMaskIntoConstraints = false
        keyPad.addSubview(keysColumn)
        keysTop = keysColumn.topAnchor.constraint(equalTo: keyPad.topAnchor, constant: 9)
        keysLeading = keysColumn.leadingAnchor.constraint(equalTo: keyPad.leadingAnchor, constant: 3)
        keysTrailing = keysColumn.trailingAnchor.constraint(equalTo: keyPad.trailingAnchor, constant: -3)
        keysBottom = keysColumn.bottomAnchor.constraint(equalTo: keyPad.bottomAnchor, constant: -3)
        NSLayoutConstraint.activate([keysTop, keysLeading, keysTrailing, keysBottom])
        keyPad.onPress = { [weak self] in self?.feedback.key() }
        Key.onActivate = { [weak self] in self?.feedback.key() }
        keyPad.onPreview = { [weak self] key in self?.showPopup(key) }
        keyPad.onAlternatesShow = { [weak self] key in
            guard let self else { return }
            self.alternatesKey = key
            self.popup.showAlternates(key, over: key.convert(key.bounds, to: self), within: self.bounds)
        }
        keyPad.onAlternatesMove = { [weak self] x in self?.popup.moveAlternates(to: x) }
        keyPad.onAlternatesEnd = { [weak self] typed in
            guard let self, let key = self.alternatesKey else { return }
            let chosen = self.popup.selectedAlternate
            self.alternatesKey = nil
            self.popup.hide()
            guard typed else { return }
            if chosen == key.alternatesMain { key.action?() } else { key.onAlternate?(chosen) }
        }
        keyPad.onCursorBegin = { [weak self] in
            self?.feedback.cursor()
            self?.delegate?.beginCursor()
        }
        keyPad.onCursorMove = { [weak self] steps in self?.delegate?.moveCursor(by: steps) }

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
        // The pad layouts replace the bar and the keys.
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
            self?.setDrawer(open: true)
        }, for: .touchUpInside)
        more.addAction(UIAction { [weak self] _ in self?.feedback.key() }, for: .touchDown)
        drawer.onTouch = { [weak self] in self?.feedback.key() }
        drawer.onPick = { [weak self] i in
            self?.setDrawer(open: false)
            self?.tap { $0.pick(i) }
        }
        drawer.onFocus = { [weak self] run in self?.tap { $0.focus(run: run) } }
        drawer.onResize = { [weak self] step in self?.tap { $0.resize(step) } }
        drawer.close.addAction(UIAction { [weak self] _ in
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
        barHeight = row.heightAnchor.constraint(equalToConstant: height)
        barHeight.isActive = true
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

    private var enterWidth: NSLayoutConstraint!
    private var shownPad: PadMetrics?
    private var extraWidths: [NSLayoutConstraint] = []
    private var extraEqual: [NSLayoutConstraint] = []

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
            if r == 1 {
                (keys.first as? Key)?.reachesMargin = true
                (keys.last as? Key)?.reachesMargin = true
            }
            if r == 2 {
                shiftKey = Key(symbol: "shift")
                shiftKey.spoken = "Shift"
                shiftKey.fires = .onPress
                shiftKey.slides = true
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
                    guard i < row.count else { return }
                    self.tap { $0.symbol(row[i]) }
                    // An apostrophe leads back into a word, as on the system keyboard.
                    if row[i] == "'" { self.showLayer(letters: true) }
                }
                symbolKeys.append(k)
                keys.append(k)
            }
            if r == 2 {
                symbolPageKey = Key(title: "#+=")
                symbolPageKey.fires = .onPress
                if KeyMetrics.flat { symbolPageKey.titleFont = .systemFont(ofSize: 13, weight: .medium) }
                symbolPageKey.action = { [weak self] in
                    guard let self else { return }
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
            let offered = Self.symbolAlternates[ch]
            k.alternates = offered?.list ?? []
            k.alternatesMain = offered?.main ?? 0
            k.onAlternate = { [weak self] i in
                guard let list = offered?.list, let ch = list[i].first else { return }
                self?.tap { $0.symbol(ch) }
            }
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
            let widths = [first.widthAnchor.constraint(equalToConstant: 45), last.widthAnchor.constraint(equalToConstant: 45)]
            NSLayoutConstraint.activate(widths)
            row.spacing = 15
            edgeRows.append((row, middle, widths))
        } else {
            row.distribution = .fillEqually
            keys.forEach(row.addArrangedSubview)
            plainRows.append(row)
        }
        guard inset > 0 else { return row }
        // The middle letter row is set in by half a key, as on a real keyboard.
        let wrap = UIStackView(arrangedSubviews: [row])
        wrap.isLayoutMarginsRelativeArrangement = true
        wrap.directionalLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: inset, bottom: 0, trailing: inset)
        insetRow = wrap
        return wrap
    }

    /// Backspace fires as the finger lands and repeats while held.
    private func backspaceKey(height: CGFloat = 42) -> Key {
        let k = Key(symbol: "delete.left", height: height)
        k.spoken = "Delete"
        k.fires = .onPress
        k.repeats = true
        k.action = { [weak self] in self?.tap { $0.backspace() } }
        k.repeatAction = { [weak self] in self?.tap { $0.backspaceRepeat() } }
        k.repeatsFaster = { [weak self] in self?.tap { $0.backspaceWord() } }
        return k
    }

    private func bottomRow(_ needsGlobe: Bool) -> UIView {
        let bottom = UIStackView()
        bottom.axis = .horizontal
        bottom.spacing = 6
        // As on the system keyboard, 123 changes the keys as the finger lands, and
        // a finger slid on to a symbol returns to the letters when it lifts.
        layerKey = Key(title: "123")
        layerKey.fires = .onPress
        layerKey.slides = true
        layerKey.showsPress = false
        layerKey.slideReturn = { [weak self] in
            guard let self else { return }
            self.showLayer(letters: self.lettersBeforeLayerKey)
        }
        layerKey.action = { [weak self] in
            guard let self else { return }
            self.lettersBeforeLayerKey = self.lettersShown
            self.showLayer(letters: !self.lettersShown)
        }
        layerWidth = layerKey.widthAnchor.constraint(equalToConstant: 46)
        layerWidth.isActive = true
        bottom.addArrangedSubview(layerKey)
        if needsGlobe {
            let g = GlobeKey()
            globe = g
            keyPad.passThrough = g
            g.addAction(UIAction { [weak self] _ in self?.feedback.key() }, for: .touchDown)
            bottom.addArrangedSubview(g)
            globeWidth = g.widthAnchor.constraint(equalToConstant: 40)
            globeWidth?.isActive = true
        }
        for i in 0..<3 {
            let k = letterKey { [weak self] in
                guard let self, i < self.extraTexts.count else { return }
                let text = self.extraTexts[i]
                if text.count == 1, let ch = text.first {
                    self.tap { $0.symbol(ch) }
                } else {
                    self.tap { $0.literal(text) }
                }
            }
            k.isHidden = true
            extraKeys.append(k)
        }
        modeKey = Key(title: Product.hasKun ? "釋" : "混")
        modeKey.spoken = "Input mode"
        modeKey.action = { [weak self] in self?.tap { $0.cycleMode() } }
        modeWidth = modeKey.widthAnchor.constraint(equalToConstant: 46)
        modeWidth.isActive = true
        modeKey.onHold = { [weak self] in self?.tap { $0.toggleLearnMenu() } }
        spaceKey = Key(title: "띄어쓰기", look: .letter)
        spaceKey.action = { [weak self] in
            self?.tap { $0.space() }
            // Space ends a run of symbols, as on the system keyboard.
            self?.showLayer(letters: true)
        }
        spaceKey.holdsForCursor = true
        enterKey = Key(title: "改行")
        enterKey.action = { [weak self] in self?.tap { $0.enter() } }
        enterWidth = enterKey.widthAnchor.constraint(equalToConstant: 94)
        enterWidth.isActive = true
        for k in [modeKey!, spaceKey!] + extraKeys + [enterKey!] {
            bottom.addArrangedSubview(k)
        }
        // By the space bar they have a key's width; with no space bar, a third each.
        extraWidths = extraKeys.map {
            let width = $0.widthAnchor.constraint(equalToConstant: 40)
            // On the narrowest screens they give way before the space bar does.
            width.priority = UILayoutPriority(999)
            return width
        }
        extraEqual = [
            extraKeys[1].widthAnchor.constraint(equalTo: extraKeys[0].widthAnchor),
            extraKeys[2].widthAnchor.constraint(equalTo: extraKeys[0].widthAnchor),
        ]
        return bottom
    }

    private var lettersBeforeLayerKey = true

    /// Sets the layer rather than toggling it: a slide, Space and an apostrophe can
    /// each ask for the letters on the same lift.
    private func showLayer(letters showLetters: Bool) {
        guard showLetters != lettersShown else { return }
        symbolPage = 0
        labelSymbols()
        letters.isHidden = !showLetters
        symbols.isHidden = showLetters
        relabelLayerKey()
    }

    /// Sizes for this width and orientation, as the system keyboard has them.
    func apply(_ l: KeyLayout, width: CGFloat) {
        guard l != layout || width != appliedWidth else { return }
        layout = l
        appliedWidth = width
        if !shownLayout.isPad {
            keysLeading.constant = l.margin
            keysTrailing.constant = -l.margin
            keysBottom.constant = -l.bottom
        }
        barHeight.constant = l.barHeight
        if !shownLayout.isPad { keysTop.constant = l.firstRow - l.barHeight }
        let between = l.rowPitch - l.keyHeight
        if !shownLayout.isPad { keysColumn.spacing = between }
        letters.spacing = between
        symbols.spacing = between
        for row in plainRows { row.spacing = l.gap }
        let edge = l.edgeKeyWidth(in: width)
        for e in edgeRows {
            e.row.spacing = l.edgeGap
            e.middle.spacing = l.gap
            e.widths.forEach { $0.constant = edge }
        }
        let halfColumn = l.columnPitch(in: width) / 2
        insetRow?.directionalLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: halfColumn, bottom: 0, trailing: halfColumn)
        bottomStack.spacing = l.gap
        let slot = l.bottomSlotWidth(in: width)
        // 123 and the mode key share the slot the system's 123 has alone.
        modeWidth.constant = (slot - l.gap) / 2
        sizeBottomSlots(shownLayout, width: width)
        for k in rowKeys() { k.heightConstraint.constant = l.keyHeight }
        for w in extraWidths { w.constant = l.keyWidth(in: width) }
        if !shownLayout.isPad { sizeGlobe(width: l.keyWidth(in: width), height: l.keyHeight) }
    }

    private var appliedWidth: CGFloat = 0

    /// 123 and Return, narrower with a globe in the row as on the system's iPhone SE.
    private func sizeBottomSlots(_ kind: Layout, width: CGFloat) {
        let slot = layout?.bottomSlotWidth(in: width) ?? 94
        let gap = layout?.gap ?? 6
        layerWidth.constant = kind.converts || globe != nil ? (slot - gap) / 2 : slot
        enterWidth.constant = globe != nil && (kind == .twitter || kind == .web) ? slot * 0.75 : slot
    }

    private func rowKeys() -> [Key] {
        var out: [Key] = []
        func walk(_ v: UIView) {
            for sub in v.subviews {
                if let k = sub as? Key { out.append(k) } else { walk(sub) }
            }
        }
        walk(keysArea)
        walk(bottomLine)
        return out
    }

    private func tap(_ act: (KeyboardViewDelegate) -> Void) {
        if let d = delegate { act(d) }
    }

    private var lettersShown: Bool { !letters.isHidden }
    private var lastMode: Session.Mode = .seokdok
    private var layoutConverts = true

    private func relabelLayerKey() {
        layerKey.title = lettersShown ? "123" : (lastMode == .latin || !layoutConverts ? "ABC" : "한글")
    }

    /// Show the keyboard state: key labels and the candidate bar.
    func render(_ s: KeyboardState) {
        lastMode = s.mode
        layoutConverts = s.layout.converts
        showLayout(s.layout)
        relabelLayerKey()
        let shifted = s.shift != .off
        let latin = s.mode == .latin || !s.layout.converts
        for (ch, key) in letterKeys {
            let up = Character(ch.uppercased())
            if latin {
                key.set(main: String(shifted ? up : ch), hint: nil)
                key.alternates = []
                continue
            }
            let main = Session.jamo(shifted ? up : ch) ?? String(ch)
            let other = Session.jamo(shifted ? ch : up)
            let hint = other == main ? nil : other
            key.set(main: shown(main), hint: hint.map(shown))
            // Held, a key offers its other letter, old ones included; the system's
            // Korean keyboard does this for its double consonants and ㅒ ㅖ.
            key.alternates = hint.map { [shown(main), shown($0)] } ?? []
            key.onAlternate = { [weak self] _ in self?.tap { $0.letterFlipped(ch) } }
        }
        shiftKey.isOn = shifted
        shiftKey.setSymbol(s.shift == .locked ? "capslock.fill" : shifted ? "shift.fill" : "shift")
        modeKey.title = s.mode == .seokdok && s.script == 1 ? "混" : ["釋", "한", "A"][Int(s.mode.rawValue)]
        let converts = s.composing && s.mode == .seokdok && (s.spaceConverts || s.candidates != nil)
        spaceKey.title = converts ? "變換" : (KeyMetrics.flat ? "" : "띄어쓰기")
        spaceKey.spoken = converts ? "變換" : "Space"
        // 釋 and 混 confirm what is being composed first, since that is where a
        // conversion is chosen; Hangul-only and Latin go straight to the field's
        // Return, as the system's Korean keyboard does.
        let confirms = s.composing && s.mode == .seokdok
        let drawn: Bool
        if #available(iOS 26, *) { drawn = true } else { drawn = s.returnKey == .plain }
        enterKey.title = confirms ? "確定" : drawn ? nil : s.returnKey.title
        enterKey.setSymbol(confirms || !drawn ? nil : s.returnKey.symbol)
        enterKey.spoken = confirms ? "確定" : s.returnKey.title ?? "Return"
        enterKey.look = s.returnKey.isAction && !confirms ? .accent : .function
        enterKey.isDisabledLook = s.returnDisabled && !confirms
        shownRuns = s.runs
        showBar(s)
    }

    /// Switch between the Hangul keyboard, the Latin one for ASCII fields and
    /// the number pads. A field that cannot take Hangul has no mode key or
    /// candidates.
    private func showLayout(_ kind: Layout) {
        guard kind != shownLayout else { return }
        shownLayout = kind
        let layout = kind
        let pads = layout.isPad
        let metrics = PadMetrics.current
        barRow.isHidden = pads
        keysTop.constant = pads ? metrics.top : (self.layout.map { $0.firstRow - $0.barHeight } ?? 8)
        keysBottom.constant = pads ? -metrics.bottom : -(self.layout?.bottom ?? 3)
        keysColumn.spacing = pads ? metrics.between : (self.layout.map { $0.rowPitch - $0.keyHeight } ?? 12)
        keysLeading.constant = pads ? 3 : (self.layout?.margin ?? 3)
        keysTrailing.constant = -keysLeading.constant
        keysArea.isHidden = pads
        bottomLine.isHidden = pads
        pad.isHidden = !pads
        modeKey.isHidden = !layout.converts
        // With a globe in the row the system's 123 and Return are narrower too.
        sizeBottomSlots(layout, width: bounds.width)
        switch layout {
        case .email: extraTexts = ["@", "."]
        case .url: extraTexts = [".", "/", ".com"]
        case .twitter: extraTexts = ["@", "#"]
        case .web: extraTexts = ["."]
        default: extraTexts = []
        }
        // A web address has no spaces; the system's has no space bar there.
        spaceKey.isHidden = layout == .url
        NSLayoutConstraint.deactivate(extraWidths + extraEqual)
        for (i, k) in extraKeys.enumerated() {
            k.isHidden = i >= extraTexts.count
            if i < extraTexts.count {
                k.set(main: extraTexts[i], hint: nil)
                // The balloon is a key wider than its key: too small for .com.
                k.previews = extraTexts[i].count == 1
            }
        }
        if layout == .url {
            NSLayoutConstraint.activate(extraEqual)
        } else {
            NSLayoutConstraint.activate(Array(extraWidths.prefix(extraTexts.count)))
        }
        // The symbols first, for a field of numbers and punctuation.
        let wantsSymbols = layout == .numbers
        if wantsSymbols || lettersShown == wantsSymbols {
            symbolPage = 0
            labelSymbols()
            letters.isHidden = wantsSymbols
            symbols.isHidden = !wantsSymbols
        }
        if pads {
            buildPad(layout, metrics)
        } else if let g = globe, let bottom = bottomLine as? UIStackView {
            g.removeFromSuperview()
            bottom.insertArrangedSubview(g, at: 1)
            sizeGlobe(width: self.layout?.keyWidth(in: bounds.width) ?? 40, height: self.layout?.keyHeight ?? 42)
        }
    }

    /// A pad follows the rotation; the letters' rows have their own sizes.
    override func layoutSubviews() {
        super.layoutSubviews()
        let metrics = PadMetrics.current
        guard shownLayout.isPad, shownPad != metrics else { return }
        keysTop.constant = metrics.top
        keysBottom.constant = -metrics.bottom
        keysColumn.spacing = metrics.between
        buildPad(shownLayout, metrics)
    }

    /// Four rows of three, as on the system's pads: no Return, letters under the
    /// digits, and flat keys beside the 0. With a globe to show it takes the empty
    /// cell, or shares the separator's.
    private func buildPad(_ layout: Layout, _ m: PadMetrics) {
        shownPad = m
        pad.arrangedSubviews.forEach { $0.removeFromSuperview() }
        pad.spacing = m.between
        pad.isLayoutMarginsRelativeArrangement = true
        pad.directionalLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: m.side, bottom: 0, trailing: m.side)
        func row(_ views: [UIView]) -> UIStackView {
            let r = UIStackView(arrangedSubviews: views)
            r.axis = .horizontal
            r.spacing = m.gap
            r.distribution = .fillEqually
            return r
        }
        func key(_ ch: Character, look: Key.Look = .letter) -> Key {
            let k = Key(title: String(ch), look: look, height: m.key)
            k.titleFont = .systemFont(ofSize: 25, weight: .regular)
            k.setPadLetters(Self.padLetters[ch])
            let zeroRise: CGFloat
            if #available(iOS 26, *) { zeroRise = 5 } else { zeroRise = 2 }
            k.labelRise = ch == "0" ? zeroRise : ch.isNumber ? 6 : 0
            k.action = { [weak self] in self?.tap { $0.symbol(ch) } }
            return k
        }
        for digits in ["123", "456", "789"] {
            pad.addArrangedSubview(row(digits.map { key($0) }))
        }
        var corner: UIView
        switch layout {
        case .phone:
            let signs = UIStackView(arrangedSubviews: ["+", "*", "#"].map { key(Character($0), look: .plain) })
            signs.axis = .horizontal
            signs.distribution = .fillEqually
            corner = signs
        case .decimal:
            corner = key(Character(Locale.current.decimalSeparator ?? "."), look: .plain)
        default:
            corner = UIView()
        }
        if let g = globe {
            g.removeFromSuperview()
            if layout == .number {
                corner = g
            } else {
                let pair = UIStackView(arrangedSubviews: [g, corner])
                pair.axis = .horizontal
                pair.spacing = m.gap
                pair.distribution = .fillEqually
                corner = pair
            }
            globeWidth?.isActive = false
            globe?.heightConstraint.constant = m.key
        }
        let delete = backspaceKey(height: m.key)
        delete.look = .plain
        delete.setSymbolSize(22.5)
        pad.addArrangedSubview(row([corner, key("0"), delete]))
    }

    /// The globe is one button that moves between the letter row and the pad
    /// row, so its two size constraints change with the row instead of new
    /// ones piling up against them.
    private func sizeGlobe(width: CGFloat, height: CGFloat) {
        globeWidth?.isActive = true
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
                cell.addAction(UIAction { [weak self] _ in self?.feedback.key() }, for: .touchDown)
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
                cell.addAction(UIAction { [weak self] _ in self?.feedback.key() }, for: .touchDown)
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
            cell.addAction(UIAction { [weak self] _ in self?.feedback.key() }, for: .touchDown)
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
