import UIKit

/// A key cap. It draws itself and knows what it does; `KeyPad` decides when it
/// is pressed.
final class Key: UIView {
    enum Look { case letter, function, accent
        /// No cap of its own until pressed (the flat keys of the number pads).
        case plain
    }

    enum Fires {
        /// When the finger lifts (the system's letters, Space, Return).
        case onRelease
        /// As the finger lands (Shift, Delete).
        case onPress
    }

    var fires = Fires.onRelease
    /// Delete: fire again and again while held.
    var repeats = false
    /// Delete: each repeat while held, instead of `action`.
    var repeatAction: (() -> Void)?
    /// Delete: what it does once held long enough.
    var repeatsFaster: (() -> Void)?
    /// Show the pop-up preview while held.
    var previews = false
    /// What a long press offers, in order; `action` still types the key itself,
    /// which stands at `alternatesMain`. Empty offers nothing.
    var alternates: [String] = []
    var alternatesMain = 0
    /// Type the alternate at this index.
    var onAlternate: ((Int) -> Void)?
    /// 123 and Shift: a finger that started here can slide onto a letter and type
    /// it where it lifts. `slideReturn` then runs, so 123 restores its layer.
    var slides = false
    var slideReturn: (() -> Void)?
    /// False for a key that changes the keys under the finger as it lands.
    var showsPress = true
    /// An end key of the middle row, which also takes the margin beside it.
    var reachesMargin = false
    /// Space: held, it turns the keyboard into a trackpad.
    var holdsForCursor = false
    var action: (() -> Void)?
    /// The click and tap of a press, for keys VoiceOver activates: those touches
    /// never reach `KeyPad`.
    static var onActivate: (() -> Void)?
    /// Instead of `action`, once held for half a second (the 釋 key).
    var onHold: (() -> Void)?

    private let label = UILabel()
    private let icon = UIImageView()

    /// The number pad's Delete is drawn larger than the others.
    func setSymbolSize(_ size: CGFloat) {
        icon.preferredSymbolConfiguration = UIImage.SymbolConfiguration(pointSize: size, weight: .regular)
    }
    private let hint = UILabel()
    private var symbol: String?
    /// The key's height, changed in place when a row needs another.
    private(set) var heightConstraint: NSLayoutConstraint!
    private var labelCenter: NSLayoutConstraint?

    var look: Look {
        didSet { refill() }
    }

    /// Shift while it is on.
    var isOn = false {
        didSet { refill() }
    }

    /// While the keyboard is a trackpad the labels and shadows go and the caps
    /// fade towards the background, by the amounts measured from a recording
    /// of the system keyboard (iOS 18.5 simulator).
    var isDimmed = false {
        didSet {
            let alpha: CGFloat = isDimmed ? 0 : 1
            label.alpha = alpha
            icon.alpha = alpha
            hint.alpha = alpha
            layer.shadowOpacity = isDimmed ? 0 : KeyMetrics.shadow
            refill()
        }
    }

    /// Instant going down; a short fade coming back, whose length is our choice.
    var isPressed = false {
        didSet {
            guard isPressed != oldValue else { return }
            if isPressed {
                refill()
            } else {
                UIView.animate(withDuration: 0.12, delay: 0, options: [.allowUserInteraction, .beginFromCurrentState]) {
                    self.refill()
                }
            }
        }
    }

    init(title: String? = nil, symbol: String? = nil, look: Look = .function, height: CGFloat = 42) {
        self.look = look
        super.init(frame: .zero)
        layer.cornerRadius = KeyMetrics.corner
        layer.cornerCurve = .continuous
        layer.shadowColor = UIColor.black.cgColor
        layer.shadowOpacity = KeyMetrics.shadow
        layer.shadowOffset = CGSize(width: 0, height: 1)
        layer.shadowRadius = 0
        isUserInteractionEnabled = false

        label.text = title
        label.textAlignment = .center
        label.font = OldHangul.font(ofSize: title == nil ? KeyMetrics.letterSize : KeyMetrics.titleSize, weight: .regular)
        label.adjustsFontSizeToFitWidth = true
        label.minimumScaleFactor = 0.6
        label.translatesAutoresizingMaskIntoConstraints = false
        addSubview(label)

        icon.tintColor = .label
        icon.contentMode = .center
        icon.preferredSymbolConfiguration = UIImage.SymbolConfiguration(pointSize: 18, weight: .regular)
        icon.translatesAutoresizingMaskIntoConstraints = false
        addSubview(icon)

        hint.font = OldHangul.font(ofSize: 11, weight: .medium)
        hint.textColor = .secondaryLabel
        hint.translatesAutoresizingMaskIntoConstraints = false
        addSubview(hint)

        heightConstraint = heightAnchor.constraint(equalToConstant: height)
        let labelY = label.centerYAnchor.constraint(equalTo: centerYAnchor)
        labelCenter = labelY
        NSLayoutConstraint.activate([
            heightConstraint,
            label.centerXAnchor.constraint(equalTo: centerXAnchor),
            labelY,
            label.widthAnchor.constraint(lessThanOrEqualTo: widthAnchor, constant: -2),
            icon.centerXAnchor.constraint(equalTo: centerXAnchor),
            icon.centerYAnchor.constraint(equalTo: centerYAnchor),
            hint.topAnchor.constraint(equalTo: topAnchor, constant: 2),
            hint.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -4),
        ])
        isAccessibilityElement = true
        accessibilityTraits = .keyboardKey
        setSymbol(symbol)
        refill()
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        if KeyMetrics.shadow > 0 {
            layer.shadowPath = UIBezierPath(roundedRect: bounds, cornerRadius: layer.cornerRadius).cgPath
        }
    }

    var title: String? {
        get { label.text }
        set { label.text = newValue }
    }

    var titleFont: UIFont {
        get { label.font }
        set { label.font = newValue }
    }

    func setSymbol(_ name: String?) {
        guard name != symbol || (name != nil && icon.image == nil) else { return }
        symbol = name
        icon.image = name.flatMap { UIImage(systemName: $0) }
    }

    /// How far above the middle the label sits, on a number pad.
    var labelRise: CGFloat = 0 {
        didSet { labelCenter?.constant = -labelRise }
    }

    /// The letters under a digit on a number pad.
    func setPadLetters(_ text: String?) {
        guard let text else { return }
        let sub = UILabel()
        sub.font = .systemFont(ofSize: 10, weight: .semibold)
        sub.attributedText = NSAttributedString(string: text, attributes: [.kern: 1.6])
        sub.translatesAutoresizingMaskIntoConstraints = false
        addSubview(sub)
        NSLayoutConstraint.activate([
            sub.centerXAnchor.constraint(equalTo: centerXAnchor),
            sub.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -5),
        ])
    }

    /// Spoken by VoiceOver; the visible label unless set.
    var spoken: String?

    override var accessibilityLabel: String? {
        get { spoken ?? title }
        set { spoken = newValue }
    }

    /// The main label and an optional corner hint, larger and darker when the
    /// hint is an old letter.
    func set(main: String, hint text: String?) {
        title = main
        hint.text = text
        let old = text.map(Self.isOld) ?? false
        hint.font = OldHangul.font(ofSize: old ? 14 : 11, weight: old ? .bold : .medium)
        hint.textColor = old ? .label : .secondaryLabel
    }

    private static func isOld(_ letter: String) -> Bool {
        guard let v = letter.unicodeScalars.first?.value else { return false }
        return "ㆍㅿㆁㆆ".unicodeScalars.contains { $0.value == v } || (0x1100...0x115F).contains(v)
    }

    private func refill() {
        let down = isPressed && !previews && showsPress
        switch look {
        case .letter:
            backgroundColor = down ? KeyMetrics.pressedFill : KeyMetrics.letterFill
            label.textColor = .label
        case .function:
            if KeyMetrics.flat {
                backgroundColor = down ? KeyMetrics.pressedFill : KeyMetrics.functionFill
            } else {
                backgroundColor = down != isOn ? KeyMetrics.letterFill : KeyMetrics.functionFill
            }
            label.textColor = .label
        case .accent:
            backgroundColor = down ? UIColor.systemBlue.withAlphaComponent(0.7) : .systemBlue
            label.textColor = .white
        case .plain:
            backgroundColor = isPressed ? KeyMetrics.letterFill : UIColor.white.withAlphaComponent(0.001)
            label.textColor = .label
        }
        icon.tintColor = look == .accent ? .white : .label
        if look == .plain { layer.shadowOpacity = 0 }
        if isDimmed {
            backgroundColor = backgroundColor?.withAlphaComponent(look == .letter ? 0.45 : 0.6)
        }
    }

    override func accessibilityActivate() -> Bool {
        Self.onActivate?()
        action?()
        return true
    }
}

/// Corners, shadow and colours of the keys, measured from the system keyboard's
/// screenshots: iOS 18.5 and iOS 26.5, iPhone 16 Pro simulators. iOS 26's keys are
/// flat, rounder and share one fill. The sizes are in `KeyLayout`.
enum KeyMetrics {
    static let flat = KeyLayout.flat
    static let corner: CGFloat = flat ? 8 : 5
    static let shadow: Float = flat ? 0 : 0.35
    static let letterSize: CGFloat = 22
    static let titleSize: CGFloat = flat ? 18 : 16
    static let letterFill = UIColor {
        $0.userInterfaceStyle == .dark
            ? UIColor(white: (flat ? 61.0 : 107.0) / 255, alpha: 1)
            : .white
    }
    static let functionFill = flat ? letterFill : UIColor {
        $0.userInterfaceStyle == .dark
            ? UIColor(white: 75.0 / 255, alpha: 1)
            : UIColor(red: 169.0 / 255, green: 175.0 / 255, blue: 186.0 / 255, alpha: 1)
    }
    /// A held key that shows no pop-up.
    static let pressedFill = flat ? UIColor {
        $0.userInterfaceStyle == .dark
            ? UIColor(white: 125.0 / 255, alpha: 1)
            : UIColor(red: 193.0 / 255, green: 193.0 / 255, blue: 198.0 / 255, alpha: 1)
    } : functionFill
}

/// The next-keyboard key: a real control, because the system's
/// `handleInputModeList(from:with:)` wants its touch events.
final class GlobeKey: UIButton {
    private(set) var heightConstraint: NSLayoutConstraint!

    init() {
        super.init(frame: .zero)
        layer.cornerRadius = KeyMetrics.corner
        layer.cornerCurve = .continuous
        layer.shadowColor = UIColor.black.cgColor
        layer.shadowOpacity = KeyMetrics.shadow
        layer.shadowOffset = CGSize(width: 0, height: 1)
        layer.shadowRadius = 0
        backgroundColor = KeyMetrics.functionFill
        setImage(UIImage(systemName: "globe"), for: .normal)
        tintColor = .label
        accessibilityLabel = "Next keyboard"
        heightConstraint = heightAnchor.constraint(equalToConstant: 42)
        heightConstraint.isActive = true
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    override var isHighlighted: Bool {
        didSet {
            let down = KeyMetrics.flat ? KeyMetrics.pressedFill : KeyMetrics.letterFill
            backgroundColor = isHighlighted ? down : KeyMetrics.functionFill
        }
    }
}

/// All keys of the keyboard, tracked in one place: a touch belongs to the
/// nearest key, so the gaps between keys type too (the system keyboard's key
/// frames touch as well); a finger sliding across letters moves the preview
/// along and types where it lifts; and a new touch types the letter still held,
/// so fast two-thumb typing stays in order.
final class KeyPad: UIView {
    /// A key went down.
    var onPress: (() -> Void)?
    /// The key being previewed; nil when none is.
    var onPreview: ((Key?) -> Void)?
    /// Space was held; the keyboard is a trackpad until `onCursorEnd`.
    var onCursorBegin: (() -> Void)?
    /// The finger moved this many characters' worth.
    var onCursorMove: ((Int) -> Void)?
    var onCursorEnd: (() -> Void)?
    /// The system's next-keyboard handling needs a real control.
    weak var passThrough: UIView?

    private struct Track {
        var key: Key
        /// Still close enough to the key for release to fire it.
        var inside = true
        /// The layer key this finger slid from, to run again after the lift.
        var returnTo: Key?
    }

    private var tracks: [ObjectIdentifier: Track] = [:]
    private var previewing: Key?
    /// The touch choosing among a held key's alternates.
    private var alternatesTouch: ObjectIdentifier?
    private var alternatesTimer: Timer?
    /// The touch the timer is for.
    private var alternatesArmed: ObjectIdentifier?
    /// The system's alternates came up between 0.3 s and 0.4 s into a press, in
    /// shots on an iOS 26.5 simulator.
    private static let alternatesHold: TimeInterval = 0.35
    /// A key is held long enough for its alternates.
    var onAlternatesShow: ((Key) -> Void)?
    /// The finger moved along them, to this x.
    var onAlternatesMove: ((CGFloat) -> Void)?
    /// The finger lifted (true) or the touch was cancelled.
    var onAlternatesEnd: ((Bool) -> Void)?
    private var repeatTimer: Timer?
    /// Held Delete, measured from the system keyboard's held Delete in iOS 18.5
    /// and iOS 26.5 simulators: on the clock of those runs its first repeat came
    /// 0.1 s after this keyboard's at 0.5 s, then ten times a second, and after
    /// twenty repeats whole words, six a second.
    private static let repeatDelay: TimeInterval = 0.6
    private static let repeatInterval: TimeInterval = 0.1
    private static let repeatsBeforeWords = 20
    private static let wordInterval: TimeInterval = 1.0 / 6
    private var holdTimer: Timer?
    /// The touch steering the caret, and what it has moved short of a character.
    private var cursorTouch: ObjectIdentifier?
    private var cursorX: CGFloat = 0
    private var cursorEnding: DispatchWorkItem?
    /// How far from a key a touch still belongs to it: wider than the widest
    /// gap between keys, narrower than a blank cell of the number pad. Our choice.
    private static let reach: CGFloat = 16
    /// The margins of the middle row type its end keys, as a grid of taps over the
    /// system keyboard showed: this far from the key, level with it.
    private static let sideReach: CGFloat = 36
    /// How far a finger may stray from a function key; our choice.
    private static let slop: CGFloat = 40
    /// The system keyboard's trackpad in recordings on iOS 18.5 and iOS 26.5
    /// simulators: it begins about 0.45 s into a held press and fades in 0.07 s,
    /// and the keys stay faded for about half a second after the lift. A slow
    /// 32 pt slide moved the caret about 11 digits of a 17 pt font, so a step is
    /// about 3 pt there, and Hangul moved about 5 pt to a syllable; the system
    /// counts in text widths and this keyboard cannot see them, so a character is
    /// 4 pt whatever its width.
    private static let cursorHold: TimeInterval = 0.45
    private static let cursorFadeIn: TimeInterval = 0.07
    private static let cursorFadeOut: TimeInterval = 0.14
    private static let cursorLinger: TimeInterval = 0.5
    private static let cursorStep: CGFloat = 4

    override init(frame: CGRect) {
        super.init(frame: frame)
        isMultipleTouchEnabled = true
    }

    /// The keys can now go while Delete or Space is held.
    deinit {
        repeatTimer?.invalidate()
        holdTimer?.invalidate()
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    override func hitTest(_ point: CGPoint, with event: UIEvent?) -> UIView? {
        guard bounds.contains(point), isUserInteractionEnabled, !isHidden, alpha > 0.01 else { return nil }
        if let p = passThrough, p.window != nil, !p.isHidden, p.convert(p.bounds, to: self).contains(point) {
            return p
        }
        return self
    }

    /// Keys on show, found through the hierarchy so that hiding a layer is all
    /// a layer switch has to do.
    private func visibleKeys(in view: UIView, into out: inout [Key]) {
        for v in view.subviews where !v.isHidden && v !== passThrough {
            if let k = v as? Key {
                out.append(k)
            } else {
                visibleKeys(in: v, into: &out)
            }
        }
    }

    private func allKeys(in view: UIView, into out: inout [Key]) {
        for v in view.subviews {
            if let k = v as? Key {
                out.append(k)
            } else {
                allKeys(in: v, into: &out)
            }
        }
    }

    private func distance(_ p: CGPoint, _ key: Key) -> CGFloat {
        let r = key.convert(key.bounds, to: self)
        let dx = max(r.minX - p.x, 0, p.x - r.maxX)
        let dy = max(r.minY - p.y, 0, p.y - r.maxY)
        return (dx * dx + dy * dy).squareRoot()
    }

    /// Half the gap between rows, so a margin's height meets the next row's.
    private let rowGapHalf: CGFloat = 6

    private func key(at p: CGPoint) -> Key? {
        var keys: [Key] = []
        visibleKeys(in: self, into: &keys)
        guard let nearest = keys.min(by: { distance(p, $0) < distance(p, $1) }) else { return nil }
        if distance(p, nearest) <= Self.reach { return nearest }
        return keys.first { key in
            guard key.reachesMargin else { return false }
            let r = key.convert(key.bounds, to: self)
            return p.y >= r.minY - rowGapHalf && p.y <= r.maxY + rowGapHalf && distance(p, key) <= Self.sideReach
        }
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        // Other fingers do nothing while one steers the caret.
        guard cursorTouch == nil else { return }
        // A new finger ends a tray, typing what it had chosen, as with a held letter.
        endAlternates(typed: true)
        // A press in the moment the keys are still faded types as usual.
        finishCursorFade()
        for touch in touches.sorted(by: { $0.timestamp < $1.timestamp }) {
            rollOver()
            guard let k = key(at: touch.location(in: self)) else { continue }
            let id = ObjectIdentifier(touch)
            tracks[id] = Track(key: k)
            press(k)
            armAlternates(k, id)
            if k.holdsForCursor {
                holdTimer?.invalidate()
                holdTimer = Timer.scheduledTimer(withTimeInterval: Self.cursorHold, repeats: false) { [weak self, weak touch] _ in
                    guard let self, let touch, self.tracks[id]?.inside == true else { return }
                    self.beginCursor(id, at: touch.location(in: self))
                }
            } else if k.onHold != nil {
                holdTimer?.invalidate()
                holdTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: false) { [weak self] _ in
                    guard let self, self.tracks[id]?.inside == true else { return }
                    self.tracks[id]?.inside = false
                    k.isPressed = false
                    k.onHold?()
                }
            }
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let id = ObjectIdentifier(touch)
            let p = touch.location(in: self)
            if id == cursorTouch {
                let steps = Int((p.x - cursorX) / Self.cursorStep)
                if steps != 0 {
                    cursorX += CGFloat(steps) * Self.cursorStep
                    onCursorMove?(steps)
                }
                continue
            }
            if id == alternatesTouch {
                onAlternatesMove?(p.x)
                continue
            }
            guard var t = tracks[id] else { continue }
            if t.key.slides {
                guard let next = key(at: p), next.previews else { continue }
                t.key.isPressed = false
                if t.key.slideReturn != nil { t.returnTo = t.key }
                t.key = next
                next.isPressed = true
                preview(next)
                tracks[id] = t
                continue
            }
            guard t.key.fires == .onRelease else { continue }
            if t.key.previews {
                // Off the keyboard's keys altogether, a lift types nothing.
                let next = key(at: p)
                t.inside = next != nil
                t.key.isPressed = t.inside
                preview(t.inside ? t.key : nil)
                let wasInside = tracks[id]?.inside == true
                if let next, next !== t.key, next.previews {
                    t.key.isPressed = false
                    t.key = next
                    next.isPressed = true
                    preview(next)
                    armAlternates(next, id)
                } else if next !== t.key {
                    // Off the keys, or over one that offers nothing.
                    if alternatesArmed == id { disarmAlternates() }
                } else if !wasInside {
                    armAlternates(t.key, id)
                }
            } else {
                t.inside = distance(p, t.key) < Self.slop
                t.key.isPressed = t.inside
            }
            tracks[id] = t
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let id = ObjectIdentifier(touch)
            if id == cursorTouch {
                endCursor()
                continue
            }
            if id == alternatesTouch {
                endAlternates(typed: true)
                continue
            }
            guard let t = tracks.removeValue(forKey: id) else { continue }
            release(t.key)
            if t.inside, t.key.fires == .onRelease { t.key.action?() }
            t.returnTo?.slideReturn?()
        }
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let id = ObjectIdentifier(touch)
            if id == cursorTouch {
                endCursor()
                continue
            }
            if id == alternatesTouch {
                endAlternates(typed: false)
                continue
            }
            guard let t = tracks.removeValue(forKey: id) else { continue }
            release(t.key)
            t.returnTo?.slideReturn?()
        }
    }

    private func armAlternates(_ k: Key, _ id: ObjectIdentifier) {
        disarmAlternates()
        guard !k.alternates.isEmpty else { return }
        alternatesArmed = id
        alternatesTimer = Timer.scheduledTimer(withTimeInterval: Self.alternatesHold, repeats: false) { [weak self, weak k] _ in
            guard let self, let k, !k.alternates.isEmpty, let t = self.tracks[id], t.key === k, t.inside else { return }
            self.alternatesArmed = nil
            self.alternatesTouch = id
            self.preview(nil)
            // Keys held by other fingers must not change the tray's key under it.
            for (other, held) in self.tracks where other != id {
                self.tracks[other] = nil
                self.release(held.key)
            }
            self.onAlternatesShow?(k)
        }
    }

    private func disarmAlternates() {
        alternatesTimer?.invalidate()
        alternatesArmed = nil
    }

    private func endAlternates(typed: Bool) {
        guard let id = alternatesTouch else { return }
        alternatesTouch = nil
        let t = tracks.removeValue(forKey: id)
        if let t { release(t.key) }
        onAlternatesEnd?(typed)
        t?.returnTo?.slideReturn?()
    }

    /// A new touch while a letter or Space is held: type it now, so two thumbs stay
    /// in order and the new key is found on the layer that key leads to.
    private func rollOver() {
        for (id, t) in tracks where t.key.previews || t.key.holdsForCursor {
            tracks[id] = nil
            release(t.key)
            if t.inside { t.key.action?() }
            t.returnTo?.slideReturn?()
        }
    }

    private func press(_ k: Key) {
        k.isPressed = true
        onPress?()
        if k.previews { preview(k) }
        if k.fires == .onPress { k.action?() }
        if k.repeats {
            repeatTimer?.invalidate()
            repeatTimer = Timer.scheduledTimer(withTimeInterval: Self.repeatDelay, repeats: false) { [weak self, weak k] _ in
                guard let self, let k else { return }
                k.repeatAction?()
                var fired = 1
                self.repeatTimer = Timer.scheduledTimer(withTimeInterval: Self.repeatInterval, repeats: true) { [weak self, weak k] _ in
                    guard let self, let k else { return }
                    k.repeatAction?()
                    fired += 1
                    guard fired >= Self.repeatsBeforeWords, let words = k.repeatsFaster else { return }
                    self.repeatTimer?.invalidate()
                    self.repeatTimer = Timer.scheduledTimer(withTimeInterval: Self.wordInterval, repeats: true) { _ in
                        words()
                    }
                }
            }
        }
    }

    private func beginCursor(_ id: ObjectIdentifier, at p: CGPoint) {
        endAlternates(typed: false)
        guard let t = tracks.removeValue(forKey: id) else { return }
        t.key.isPressed = false
        // Other keys still down (a thumb on a letter) end with it.
        for (other, held) in tracks {
            tracks[other] = nil
            release(held.key)
            held.returnTo?.slideReturn?()
        }
        cursorTouch = id
        cursorX = p.x
        setDimmed(true)
        onCursorBegin?()
    }

    private func endCursor() {
        cursorTouch = nil
        onCursorEnd?()
        let work = DispatchWorkItem { [weak self] in self?.finishCursorFade() }
        cursorEnding = work
        DispatchQueue.main.asyncAfter(deadline: .now() + Self.cursorLinger, execute: work)
    }

    private func finishCursorFade() {
        guard let ending = cursorEnding else { return }
        ending.cancel()
        cursorEnding = nil
        setDimmed(false)
    }

    private func setDimmed(_ dimmed: Bool) {
        var keys: [Key] = []
        // Hidden ones too: a field can change the layer during the fade's linger.
        allKeys(in: self, into: &keys)
        UIView.animate(withDuration: dimmed ? Self.cursorFadeIn : Self.cursorFadeOut, delay: 0, options: [.allowUserInteraction, .beginFromCurrentState]) {
            keys.forEach { $0.isDimmed = dimmed }
        }
    }

    private func release(_ k: Key) {
        // Another finger may still hold the same key.
        if tracks.values.contains(where: { $0.key === k }) { return }
        k.isPressed = false
        // Only the finger the timer is for: another's lift must not cancel a tray.
        if let armed = alternatesArmed, tracks[armed] == nil { disarmAlternates() }
        if k.holdsForCursor || k.onHold != nil {
            holdTimer?.invalidate()
            holdTimer = nil
        }
        if k.repeats {
            repeatTimer?.invalidate()
            repeatTimer = nil
        }
        if previewing === k { preview(nil) }
    }

    private func preview(_ k: Key?) {
        previewing = k
        onPreview?(k)
    }
}

/// The balloon over a held key: the key's own shape grown upward into a larger
/// cap with the letter in it. One instance moves from key to key. Its shape and
/// timing are measured from the system keyboard's in iOS 18.5 and iOS 26.5
/// simulators.
final class KeyPopup: UIView {
    private let shape = CAShapeLayer()
    private let label = UILabel()
    private var hideWork: DispatchWorkItem?

    private static var flat: Bool {
        if #available(iOS 26, *) { return true }
        return false
    }
    /// Over the top of the key.
    private static let rise: CGFloat = flat ? 66 : 64
    /// From the middle of the letter to the top of the key.
    private static let letterRise: CGFloat = flat ? 32.8 : 36.8
    private var letterSize: CGFloat = 0
    /// To each side of the key.
    private static let grow: CGFloat = flat ? 12.3 : 13
    /// The system's pop-up outlives the finger by about this much.
    private static let linger: TimeInterval = 0.05
    /// A little lighter than the keys in the dark on iOS 26.
    private static let fill = UIColor {
        guard flat, $0.userInterfaceStyle == .dark else { return KeyMetrics.letterFill.resolvedColor(with: $0) }
        return UIColor(white: 84.0 / 255, alpha: 1)
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        isUserInteractionEnabled = false
        layer.addSublayer(shape)
        shape.shadowColor = UIColor.black.cgColor
        shape.shadowOpacity = Self.flat ? 0 : 0.3
        shape.shadowRadius = 3
        shape.shadowOffset = CGSize(width: 0, height: 1)
        label.textAlignment = .center
        addSubview(label)
        isHidden = true
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    /// Show `key`'s letter over `rect`, the key's frame in this view's
    /// superview, kept within `bounds` of the keyboard: a custom keyboard can draw
    /// nowhere else, so over the top row its edge shortens the pop-up.
    func show(_ key: Key, over rect: CGRect, within outer: CGRect) {
        hideWork?.cancel()
        hideWork = nil
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        frame = outer
        let top = max(rect.minY - Self.rise, outer.minY)
        var body = CGRect(x: rect.minX - Self.grow, y: top, width: rect.width + 2 * Self.grow, height: rect.minY - Self.shoulder - top)
        body.origin.x = min(max(body.minX, outer.minX + 1), outer.maxX - body.width - 1)
        let path = Self.path(body: body, key: rect)
        shape.path = path
        shape.shadowPath = path
        shape.fillColor = Self.fill.resolvedColor(with: traitCollection).cgColor
        // Where the edge shortens the pop-up, the letter sits in what is left, and
        // in landscape shrinks to fit it.
        let letterY = max(rect.minY - Self.letterRise, body.midY)
        label.frame = CGRect(x: body.minX, y: letterY - 25, width: body.width, height: 50)
        let size = min(37, body.height - 4)
        if size != letterSize {
            letterSize = size
            label.font = OldHangul.font(ofSize: size)
        }
        label.text = key.title
        isHidden = false
        CATransaction.commit()
    }

    func hide() {
        if !cells.isEmpty {
            hideWork?.cancel()
            hideWork = nil
            isHidden = true
            hideAlternates()
            return
        }
        // Each move off the keys asks again; the first request's time stands.
        guard hideWork == nil, !isHidden else { return }
        let work = DispatchWorkItem { [weak self] in
            self?.isHidden = true
            self?.hideWork = nil
        }
        hideWork = work
        DispatchQueue.main.asyncAfter(deadline: .now() + Self.linger, execute: work)
    }

    // MARK: Alternates

    private var cells: [UILabel] = []
    private let selection = CALayer()
    private var order: [Int] = []
    private var tray = CGRect.zero
    private(set) var selectedAlternate = 0

    /// iOS 26's tray floats clear of the key, which stays in view.
    private static var trayFloats: Bool {
        if #available(iOS 26, *) { return true }
        return false
    }
    /// A floating tray's measures, from the system's on iOS 26.5: its top and bottom
    /// over the key, the room around the cells, and the choice's room above and below.
    private static let trayRise: CGFloat = 73
    private static let trayDrop: CGFloat = 5
    private static let trayPad: CGFloat = 8.3
    private static let choicePad: CGFloat = 10

    /// A tray over the key, with the key itself over its own place and chosen. Keys
    /// on the right half read towards the middle, as on the system keyboard.
    func showAlternates(_ key: Key, over rect: CGRect, within outer: CGRect) {
        hideWork?.cancel()
        hideWork = nil
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        frame = outer
        order = Array(key.alternates.indices)
        var main = key.alternatesMain
        if key.alternates.count == 2, main == 0, rect.midX > outer.midX {
            order.reverse()
            main = 1
        }
        let floats = Self.trayFloats
        let side = floats ? Self.trayPad : 0
        let cell = min(max(rect.width + (floats ? 4.5 : 4), 34), (outer.width - 2 - 2 * side) / CGFloat(order.count))
        let width = cell * CGFloat(order.count)
        let x = min(max(rect.midX - cell * (CGFloat(main) + 0.5) - side, outer.minX + 1), outer.maxX - width - 2 * side - 1) + side
        let wanted = rect.minY - (floats ? Self.trayRise : 54)
        let top = max(wanted, outer.minY)
        let bottom = rect.minY - (floats ? Self.trayDrop : 14)
        // Under the keyboard's top edge the choice gives up some of its room.
        let cramped = top > wanted
        let cellTop = floats ? top + (cramped ? 4 : Self.choicePad) : top
        let height = bottom - (floats ? (cramped ? 5 : Self.choicePad) : 0) - cellTop
        tray = CGRect(x: x, y: cellTop, width: width, height: height)
        let path = floats
            ? UIBezierPath(roundedRect: CGRect(x: x - side, y: top, width: width + 2 * side, height: bottom - top), cornerRadius: 11).cgPath
            : Self.path(body: tray, key: rect)
        shape.path = path
        shape.shadowPath = path
        shape.fillColor = KeyMetrics.letterFill.resolvedColor(with: traitCollection).cgColor
        label.isHidden = true
        if selection.superlayer == nil {
            selection.backgroundColor = UIColor.systemBlue.cgColor
            selection.cornerRadius = floats ? 12 : 6
            layer.addSublayer(selection)
        }
        cells.forEach { $0.removeFromSuperview() }
        cells = order.enumerated().map { i, index in
            let l = UILabel(frame: CGRect(x: x + CGFloat(i) * cell, y: cellTop, width: cell, height: height))
            l.text = key.alternates[index]
            l.textAlignment = .center
            l.font = OldHangul.font(ofSize: floats ? 26 : 24)
            l.adjustsFontSizeToFitWidth = true
            addSubview(l)
            return l
        }
        selection.isHidden = false
        isHidden = false
        CATransaction.commit()
        choose(main)
    }

    func moveAlternates(to x: CGFloat) {
        guard !cells.isEmpty else { return }
        let i = Int((x - tray.minX) / (tray.width / CGFloat(cells.count)))
        choose(min(max(i, 0), cells.count - 1))
    }

    private func choose(_ i: Int) {
        selectedAlternate = order[i]
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        selection.frame = Self.trayFloats ? cells[i].frame : cells[i].frame.insetBy(dx: 2, dy: 3)
        CATransaction.commit()
        for (j, l) in cells.enumerated() {
            l.textColor = j == i ? .white : .label
        }
    }

    private func hideAlternates() {
        cells.forEach { $0.removeFromSuperview() }
        cells = []
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        selection.isHidden = true
        CATransaction.commit()
        label.isHidden = false
    }

    /// From the bottom of the body's straight sides to the top of the key.
    private static let shoulder: CGFloat = 14

    /// A rounded body, a short shoulder narrowing to the key, and the key's bottom.
    private static func path(body b: CGRect, key k: CGRect) -> CGPath {
        let r: CGFloat = 10
        let kr = KeyMetrics.corner
        let end = k.minY - 3
        let p = UIBezierPath()
        p.move(to: CGPoint(x: b.minX + r, y: b.minY))
        p.addLine(to: CGPoint(x: b.maxX - r, y: b.minY))
        p.addQuadCurve(to: CGPoint(x: b.maxX, y: b.minY + r), controlPoint: CGPoint(x: b.maxX, y: b.minY))
        p.addLine(to: CGPoint(x: b.maxX, y: b.maxY))
        p.addCurve(to: CGPoint(x: k.maxX, y: end),
                   controlPoint1: CGPoint(x: b.maxX, y: b.maxY + 6),
                   controlPoint2: CGPoint(x: k.maxX, y: end - 6))
        p.addLine(to: CGPoint(x: k.maxX, y: k.maxY - kr))
        p.addQuadCurve(to: CGPoint(x: k.maxX - kr, y: k.maxY), controlPoint: CGPoint(x: k.maxX, y: k.maxY))
        p.addLine(to: CGPoint(x: k.minX + kr, y: k.maxY))
        p.addQuadCurve(to: CGPoint(x: k.minX, y: k.maxY - kr), controlPoint: CGPoint(x: k.minX, y: k.maxY))
        p.addLine(to: CGPoint(x: k.minX, y: end))
        p.addCurve(to: CGPoint(x: b.minX, y: b.maxY),
                   controlPoint1: CGPoint(x: k.minX, y: end - 6),
                   controlPoint2: CGPoint(x: b.minX, y: b.maxY + 6))
        p.addLine(to: CGPoint(x: b.minX, y: b.minY + r))
        p.addQuadCurve(to: CGPoint(x: b.minX + r, y: b.minY), controlPoint: CGPoint(x: b.minX, y: b.minY))
        p.close()
        return p.cgPath
    }
}
