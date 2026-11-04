import UIKit

/// A key cap. It draws itself and knows what it does; `KeyPad` decides when it
/// is pressed.
final class Key: UIView {
    enum Look { case letter, function, accent }

    enum Fires {
        /// When the finger lifts (the system's letters, Space, Return).
        case onRelease
        /// As the finger lands (Shift, Delete).
        case onPress
    }

    var fires = Fires.onRelease
    /// Delete: fire again and again while held.
    var repeats = false
    /// Show the pop-up preview while held.
    var previews = false
    /// Space: held, it turns the keyboard into a trackpad.
    var holdsForCursor = false
    var action: (() -> Void)?

    private let label = UILabel()
    private let icon = UIImageView()
    private let hint = UILabel()
    private var symbol: String?
    /// The key's height, changed in place when a row needs another.
    private(set) var heightConstraint: NSLayoutConstraint!

    var look: Look {
        didSet { refill() }
    }

    /// Shift while it is on.
    var isOn = false {
        didSet { refill() }
    }

<<<<<<< HEAD
    /// The labels go while the keyboard is a trackpad.
    var isDimmed = false {
        didSet {
            let alpha: CGFloat = isDimmed ? 0 : 1
            label.alpha = alpha
            icon.alpha = alpha
            hint.alpha = alpha
        }
    }

    /// Instant going down; a short fade coming back, as on the system keyboard.
=======
    /// Instant going down; a short fade coming back, whose length is our choice.
>>>>>>> feat/ios-key-press
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

    init(title: String? = nil, symbol: String? = nil, look: Look = .function, height: CGFloat = KeyMetrics.keyHeight) {
        self.look = look
        super.init(frame: .zero)
        layer.cornerRadius = KeyMetrics.corner
        layer.shadowColor = UIColor.black.cgColor
        layer.shadowOpacity = 0.35
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
        NSLayoutConstraint.activate([
            heightConstraint,
            label.centerXAnchor.constraint(equalTo: centerXAnchor),
            label.centerYAnchor.constraint(equalTo: centerYAnchor),
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
        layer.shadowPath = UIBezierPath(roundedRect: bounds, cornerRadius: layer.cornerRadius).cgPath
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
        let down = isPressed && !previews
        switch look {
        case .letter:
            backgroundColor = down ? KeyMetrics.functionFill : KeyMetrics.letterFill
            label.textColor = .label
        case .function:
            backgroundColor = down != isOn ? KeyMetrics.letterFill : KeyMetrics.functionFill
            label.textColor = .label
        case .accent:
            backgroundColor = down ? UIColor.systemBlue.withAlphaComponent(0.7) : .systemBlue
            label.textColor = .white
        }
        icon.tintColor = look == .accent ? .white : .label
    }

    override func accessibilityActivate() -> Bool {
        action?()
        return true
    }
}

/// Sizes and colours of the keys. The key height and both fills are measured
/// from the system keyboard's screenshots on an iOS 18.5 simulator; the corner
/// radius and font sizes are our choice.
enum KeyMetrics {
    static let keyHeight: CGFloat = 42
    static let corner: CGFloat = 5
    static let letterSize: CGFloat = 22
    static let titleSize: CGFloat = 16
    static let letterFill = UIColor { $0.userInterfaceStyle == .dark ? UIColor(white: 107.0 / 255, alpha: 1) : .white }
    static let functionFill = UIColor {
        $0.userInterfaceStyle == .dark
            ? UIColor(white: 75.0 / 255, alpha: 1)
            : UIColor(red: 169.0 / 255, green: 175.0 / 255, blue: 186.0 / 255, alpha: 1)
    }
}

/// The next-keyboard key: a real control, because the system's
/// `handleInputModeList(from:with:)` wants its touch events.
final class GlobeKey: UIButton {
    private(set) var heightConstraint: NSLayoutConstraint!

    init() {
        super.init(frame: .zero)
        layer.cornerRadius = KeyMetrics.corner
        layer.shadowColor = UIColor.black.cgColor
        layer.shadowOpacity = 0.35
        layer.shadowOffset = CGSize(width: 0, height: 1)
        layer.shadowRadius = 0
        backgroundColor = KeyMetrics.functionFill
        setImage(UIImage(systemName: "globe"), for: .normal)
        tintColor = .label
        accessibilityLabel = "Next keyboard"
        heightConstraint = heightAnchor.constraint(equalToConstant: KeyMetrics.keyHeight)
        heightConstraint.isActive = true
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    override var isHighlighted: Bool {
        didSet { backgroundColor = isHighlighted ? KeyMetrics.letterFill : KeyMetrics.functionFill }
    }
}

/// All keys of the keyboard, tracked in one place: a touch belongs to the
/// nearest key, so the gaps between keys type too (the system keyboard's key
/// frames touch as well); a finger sliding across letters moves the preview
/// along and types where it lifts; and a new touch types the letter still held,
/// so fast two-thumb typing stays in order.
final class KeyPad: UIView {
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
    }

    private var tracks: [ObjectIdentifier: Track] = [:]
    private var previewing: Key?
    private var repeatTimer: Timer?
<<<<<<< HEAD
    private var holdTimer: Timer?
    /// The touch steering the caret, and what it has moved short of a character.
    private var cursorTouch: ObjectIdentifier?
    private var cursorX: CGFloat = 0
=======
    /// How far a finger may stray from a function key; our choice.
>>>>>>> feat/ios-key-press
    private static let slop: CGFloat = 40
    /// Our choices: the half second follows Hamster's default, and 8 pt per
    /// character was set by eye.
    private static let cursorHold: TimeInterval = 0.5
    private static let cursorStep: CGFloat = 8

    override init(frame: CGRect) {
        super.init(frame: frame)
        isMultipleTouchEnabled = true
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

    private func distance(_ p: CGPoint, _ key: Key) -> CGFloat {
        let r = key.convert(key.bounds, to: self)
        let dx = max(r.minX - p.x, 0, p.x - r.maxX)
        let dy = max(r.minY - p.y, 0, p.y - r.maxY)
        return (dx * dx + dy * dy).squareRoot()
    }

    private func key(at p: CGPoint) -> Key? {
        var keys: [Key] = []
        visibleKeys(in: self, into: &keys)
        return keys.min { distance(p, $0) < distance(p, $1) }
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        // Other fingers do nothing while one steers the caret.
        guard cursorTouch == nil else { return }
        for touch in touches {
            rollOver()
            guard let k = key(at: touch.location(in: self)) else { continue }
            let id = ObjectIdentifier(touch)
            tracks[id] = Track(key: k)
            press(k)
            if k.holdsForCursor {
                holdTimer?.invalidate()
                holdTimer = Timer.scheduledTimer(withTimeInterval: Self.cursorHold, repeats: false) { [weak self, weak touch] _ in
                    guard let self, let touch else { return }
                    self.beginCursor(id, at: touch.location(in: self).x)
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
            guard var t = tracks[id], t.key.fires == .onRelease else { continue }
            if t.key.previews {
                if let next = key(at: p), next !== t.key, next.previews {
                    t.key.isPressed = false
                    t.key = next
                    next.isPressed = true
                    preview(next)
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
            guard let t = tracks.removeValue(forKey: id) else { continue }
            release(t.key)
            if t.inside, t.key.fires == .onRelease { t.key.action?() }
        }
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let id = ObjectIdentifier(touch)
            if id == cursorTouch {
                endCursor()
                continue
            }
            guard let t = tracks.removeValue(forKey: id) else { continue }
            release(t.key)
        }
    }

    /// A new touch while a letter is held: type that letter now.
    private func rollOver() {
        for (id, t) in tracks where t.key.previews {
            tracks[id] = nil
            release(t.key)
            t.key.action?()
        }
    }

    private func press(_ k: Key) {
        k.isPressed = true
        if k.previews { preview(k) }
        if k.fires == .onPress { k.action?() }
        if k.repeats {
            repeatTimer?.invalidate()
            repeatTimer = Timer.scheduledTimer(withTimeInterval: 0.45, repeats: false) { [weak self, weak k] _ in
                self?.repeatTimer = Timer.scheduledTimer(withTimeInterval: 0.08, repeats: true) { [weak k] _ in
                    k?.action?()
                }
            }
        }
    }

    private func beginCursor(_ id: ObjectIdentifier, at x: CGFloat) {
        guard let t = tracks.removeValue(forKey: id) else { return }
        t.key.isPressed = false
        // Other keys still down (a thumb on a letter) end with it.
        for (other, held) in tracks {
            tracks[other] = nil
            release(held.key)
        }
        cursorTouch = id
        cursorX = x
        setDimmed(true)
        onCursorBegin?()
    }

    private func endCursor() {
        cursorTouch = nil
        setDimmed(false)
        onCursorEnd?()
    }

    private func setDimmed(_ dimmed: Bool) {
        var keys: [Key] = []
        visibleKeys(in: self, into: &keys)
        UIView.animate(withDuration: 0.2, delay: 0, options: [.allowUserInteraction, .beginFromCurrentState]) {
            keys.forEach { $0.isDimmed = dimmed }
        }
    }

    private func release(_ k: Key) {
        k.isPressed = false
        if k.holdsForCursor {
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
/// cap with the letter in it. One instance moves from key to key.
final class KeyPopup: UIView {
    private let shape = CAShapeLayer()
    private let label = UILabel()

    override init(frame: CGRect) {
        super.init(frame: frame)
        isUserInteractionEnabled = false
        layer.addSublayer(shape)
        shape.shadowColor = UIColor.black.cgColor
        shape.shadowOpacity = 0.3
        shape.shadowRadius = 3
        shape.shadowOffset = CGSize(width: 0, height: 1)
        label.textAlignment = .center
        label.font = OldHangul.font(ofSize: 32)
        addSubview(label)
        isHidden = true
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    /// Show `key`'s letter over `rect`, the key's frame in this view's
    /// superview, kept within `bounds` of the keyboard.
    func show(_ key: Key, over rect: CGRect, within outer: CGRect) {
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        frame = outer
        let grow: CGFloat = 11
        let bodyHeight = rect.height + 12
        var body = CGRect(x: rect.minX - grow, y: rect.minY - bodyHeight, width: rect.width + 2 * grow, height: bodyHeight)
        body.origin.x = min(max(body.minX, outer.minX + 1), outer.maxX - body.width - 1)
        body.origin.y = max(body.minY, outer.minY)
        let path = Self.path(body: body, key: rect)
        shape.path = path
        shape.shadowPath = path
        shape.fillColor = KeyMetrics.letterFill.resolvedColor(with: traitCollection).cgColor
        label.frame = body
        label.text = key.title
        isHidden = false
        CATransaction.commit()
    }

    func hide() {
        isHidden = true
    }

    /// A rounded body, a neck narrowing to the key, and the key's bottom.
    private static func path(body b: CGRect, key k: CGRect) -> CGPath {
        let r: CGFloat = 8
        let kr = KeyMetrics.corner
        let neck: CGFloat = 8
        let p = UIBezierPath()
        p.move(to: CGPoint(x: b.minX + r, y: b.minY))
        p.addLine(to: CGPoint(x: b.maxX - r, y: b.minY))
        p.addQuadCurve(to: CGPoint(x: b.maxX, y: b.minY + r), controlPoint: CGPoint(x: b.maxX, y: b.minY))
        p.addLine(to: CGPoint(x: b.maxX, y: b.maxY - neck))
        p.addCurve(to: CGPoint(x: k.maxX, y: b.maxY + neck),
                   controlPoint1: CGPoint(x: b.maxX, y: b.maxY),
                   controlPoint2: CGPoint(x: k.maxX, y: b.maxY))
        p.addLine(to: CGPoint(x: k.maxX, y: k.maxY - kr))
        p.addQuadCurve(to: CGPoint(x: k.maxX - kr, y: k.maxY), controlPoint: CGPoint(x: k.maxX, y: k.maxY))
        p.addLine(to: CGPoint(x: k.minX + kr, y: k.maxY))
        p.addQuadCurve(to: CGPoint(x: k.minX, y: k.maxY - kr), controlPoint: CGPoint(x: k.minX, y: k.maxY))
        p.addLine(to: CGPoint(x: k.minX, y: b.maxY + neck))
        p.addCurve(to: CGPoint(x: b.minX, y: b.maxY - neck),
                   controlPoint1: CGPoint(x: k.minX, y: b.maxY),
                   controlPoint2: CGPoint(x: b.minX, y: b.maxY))
        p.addLine(to: CGPoint(x: b.minX, y: b.minY + r))
        p.addQuadCurve(to: CGPoint(x: b.minX + r, y: b.minY), controlPoint: CGPoint(x: b.minX, y: b.minY))
        p.close()
        return p.cgPath
    }
}
