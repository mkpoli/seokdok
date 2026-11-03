import UIKit

protocol KeyboardViewDelegate: AnyObject {
    func letter(_ key: Character)
    func space()
    func enter()
    func backspace()
    func toggleShift()
    func toggleMode()
    func pick(_ index: Int)
}

/// The keys and the candidate bar. Each letter key shows the jamo it types
/// and, in its corner, the one on the other Shift layer; old letters
/// (ㆍ ㅿ ㆁ ㆆ ᄼ …) are tinted.
final class KeyboardView: UIView {
    weak var delegate: KeyboardViewDelegate?
    /// The input-mode switch key, when the system asks for one.
    private(set) var globe: UIButton?

    private let bar = UIStackView()
    private let barScroll = UIScrollView()
    private var letterKeys: [(Character, KeyButton)] = []
    private var shiftKey: KeyButton!
    private var modeKey: KeyButton!

    private static let rows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"]

    init(globe needsGlobe: Bool) {
        super.init(frame: .zero)
        backgroundColor = .clear

        barScroll.showsHorizontalScrollIndicator = false
        bar.axis = .horizontal
        bar.spacing = 4
        bar.translatesAutoresizingMaskIntoConstraints = false
        barScroll.addSubview(bar)
        NSLayoutConstraint.activate([
            bar.leadingAnchor.constraint(equalTo: barScroll.contentLayoutGuide.leadingAnchor, constant: 6),
            bar.trailingAnchor.constraint(equalTo: barScroll.contentLayoutGuide.trailingAnchor, constant: -6),
            bar.topAnchor.constraint(equalTo: barScroll.contentLayoutGuide.topAnchor),
            bar.bottomAnchor.constraint(equalTo: barScroll.contentLayoutGuide.bottomAnchor),
            bar.heightAnchor.constraint(equalTo: barScroll.frameLayoutGuide.heightAnchor),
        ])

        let column = UIStackView()
        column.axis = .vertical
        column.spacing = 8
        column.translatesAutoresizingMaskIntoConstraints = false
        addSubview(column)
        NSLayoutConstraint.activate([
            column.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 3),
            column.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -3),
            column.topAnchor.constraint(equalTo: topAnchor, constant: 2),
            column.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -4),
            barScroll.heightAnchor.constraint(equalToConstant: 40),
        ])
        column.addArrangedSubview(barScroll)

        for (r, row) in Self.rows.enumerated() {
            let line = UIStackView()
            line.axis = .horizontal
            line.spacing = 5
            line.distribution = .fillEqually
            var keys: [UIView] = []
            for ch in row {
                let k = KeyButton()
                k.addAction(UIAction { [weak self] _ in self?.delegate?.letter(ch) }, for: .touchUpInside)
                letterKeys.append((ch, k))
                keys.append(k)
            }
            if r == 2 {
                shiftKey = KeyButton(symbol: "shift")
                shiftKey.addAction(UIAction { [weak self] _ in self?.delegate?.toggleShift() }, for: .touchUpInside)
                let back = KeyButton(symbol: "delete.left")
                back.addAction(UIAction { [weak self] _ in self?.delegate?.backspace() }, for: .touchUpInside)
                keys = [shiftKey] + keys + [back]
            }
            keys.forEach(line.addArrangedSubview)
            if r == 1 {
                // The middle row is set in by half a key, as on a real keyboard.
                let wrap = UIStackView(arrangedSubviews: [line])
                wrap.isLayoutMarginsRelativeArrangement = true
                wrap.directionalLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: 18, bottom: 0, trailing: 18)
                column.addArrangedSubview(wrap)
            } else {
                column.addArrangedSubview(line)
            }
        }

        let bottom = UIStackView()
        bottom.axis = .horizontal
        bottom.spacing = 5
        if needsGlobe {
            let g = KeyButton(symbol: "globe")
            globe = g
            bottom.addArrangedSubview(g)
            g.widthAnchor.constraint(equalToConstant: 44).isActive = true
        }
        modeKey = KeyButton(title: "韓")
        modeKey.addAction(UIAction { [weak self] _ in self?.delegate?.toggleMode() }, for: .touchUpInside)
        modeKey.widthAnchor.constraint(equalToConstant: 52).isActive = true
        let space = KeyButton(title: "變換")
        space.addAction(UIAction { [weak self] _ in self?.delegate?.space() }, for: .touchUpInside)
        let enter = KeyButton(title: "確定", accent: true)
        enter.addAction(UIAction { [weak self] _ in self?.delegate?.enter() }, for: .touchUpInside)
        enter.widthAnchor.constraint(equalToConstant: 84).isActive = true
        bottom.addArrangedSubview(modeKey)
        bottom.addArrangedSubview(space)
        bottom.addArrangedSubview(enter)
        column.addArrangedSubview(bottom)
        update(shift: false, mode: 0)
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    /// Relabel the keys for the Shift state and the input mode (2 = Latin).
    func update(shift: Bool, mode: Int32) {
        let latin = mode == 2
        for (ch, key) in letterKeys {
            let up = Character(ch.uppercased())
            if latin {
                key.set(main: String(shift ? up : ch), hint: nil, old: false)
                continue
            }
            let main = Session.jamo(shift ? up : ch) ?? String(ch)
            let other = Session.jamo(shift ? ch : up)
            let hint = other == main ? nil : other
            key.set(main: shown(main), hint: hint.map(shown), old: isOld(main) || isOld(hint ?? ""))
        }
        shiftKey.isSelected = shift
        modeKey.setTitle(latin ? "A" : "韓", for: .normal)
    }

    /// Fill the candidate bar: the candidates when the list is open, else the
    /// text being composed.
    func show(candidates: Session.Candidates?, preedit: [Session.Run], loading: Bool) {
        bar.arrangedSubviews.forEach { $0.removeFromSuperview() }
        if let c = candidates {
            for (i, item) in c.items.enumerated() {
                let b = UIButton(type: .system)
                b.setTitle(item.text, for: .normal)
                b.titleLabel?.font = .systemFont(ofSize: 20)
                b.tintColor = .label
                b.contentEdgeInsets = UIEdgeInsets(top: 0, left: 10, bottom: 0, right: 10)
                b.backgroundColor = i == c.selected ? UIColor.systemBlue.withAlphaComponent(0.18) : .clear
                b.layer.cornerRadius = 6
                b.addAction(UIAction { [weak self] _ in self?.delegate?.pick(i) }, for: .touchUpInside)
                bar.addArrangedSubview(b)
            }
            let count = UILabel()
            count.text = "\(c.position)/\(c.total)"
            count.font = .systemFont(ofSize: 12)
            count.textColor = .secondaryLabel
            bar.addArrangedSubview(count)
        } else {
            let label = UILabel()
            label.font = .systemFont(ofSize: 18)
            label.textColor = .secondaryLabel
            label.text = loading && preedit.isEmpty ? "辭典 讀는 中…" : preedit.map(\.text).joined()
            bar.addArrangedSubview(label)
        }
    }

    /// An initial-only jamo (ᄼ) needs a vowel filler to show on its own.
    private func shown(_ j: String) -> String {
        guard let v = j.unicodeScalars.first?.value, (0x1100...0x115F).contains(v) else { return j }
        return j + "\u{1160}"
    }

    private func isOld(_ j: String) -> Bool {
        guard let v = j.unicodeScalars.first?.value else { return false }
        return "ㆍㅿㆁㆆ".unicodeScalars.contains { $0.value == v } || (0x1100...0x115F).contains(v)
    }
}

/// A key: the main label, an optional corner hint, an old-letter tint.
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
        titleLabel?.font = .systemFont(ofSize: title == nil ? 22 : 16, weight: .medium)
        if let title { setTitle(title, for: .normal) }
        if let symbol {
            setImage(UIImage(systemName: symbol), for: .normal)
            setImage(UIImage(systemName: symbol + ".fill"), for: .selected)
            tintColor = .label
        }
        heightAnchor.constraint(equalToConstant: 44).isActive = true
        hint.font = .systemFont(ofSize: 11, weight: .medium)
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

    func set(main: String, hint text: String?, old: Bool) {
        setTitle(main, for: .normal)
        hint.text = text
        let tint: UIColor = old ? .systemIndigo : .label
        setTitleColor(accent ? .white : tint, for: .normal)
        hint.textColor = old ? .systemIndigo : .secondaryLabel
        layer.borderWidth = old ? 1 : 0
        layer.borderColor = UIColor.systemIndigo.cgColor
    }
}
