import UIKit

/// A small button of the candidate bar and the drawer.
final class BarButton: UIControl {
    private let image = UIImageView()

    init(symbol: String, label: String, width: CGFloat = 44) {
        super.init(frame: .zero)
        layer.cornerRadius = 6
        image.image = UIImage(systemName: symbol, withConfiguration: UIImage.SymbolConfiguration(pointSize: 15, weight: .medium))
        image.tintColor = .label
        image.isUserInteractionEnabled = false
        image.translatesAutoresizingMaskIntoConstraints = false
        addSubview(image)
        NSLayoutConstraint.activate([
            image.centerXAnchor.constraint(equalTo: centerXAnchor),
            image.centerYAnchor.constraint(equalTo: centerYAnchor),
            widthAnchor.constraint(equalToConstant: width),
        ])
        isAccessibilityElement = true
        accessibilityTraits = .button
        accessibilityLabel = label
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    override var isHighlighted: Bool {
        didSet { backgroundColor = isHighlighted ? UIColor.label.withAlphaComponent(0.14) : .clear }
    }

    override var isEnabled: Bool {
        didSet { image.alpha = isEnabled ? 1 : 0.3 }
    }
}

/// Every candidate at once, covering the keys: a scrolling grid whose cells
/// are as wide as their text, as on Apple's Chinese keyboard. Above it are the
/// composition's segments, and beside it the keys to shorten or lengthen the
/// focused one. Cells exist only while it is open.
final class CandidateDrawer: UIView {
    var onPick: ((Int) -> Void)?
    var onFocus: ((Int) -> Void)?
    var onResize: ((Int) -> Void)?
    let close = BarButton(symbol: "chevron.up", label: "Fewer candidates")

    private let shorter = BarButton(symbol: "minus", label: "Shorter segment")
    private let longer = BarButton(symbol: "plus", label: "Longer segment")
    private let segmentScroll = UIScrollView()
    private let segments = UIStackView()
    private var shownRuns: [String] = []
    private var shownFocus = -1
    private var segmentHeight: NSLayoutConstraint!
    private let segmentLine = UIView()
    private let scroll = UIScrollView()
    private var cells: [CandidateCell] = []
    private var lines: [UIView] = []
    private var laidWidth: CGFloat = 0
    private var shown: [Session.Candidates.Item] = []

    private static let rowHeight: CGFloat = 44
    private static let columns: CGFloat = 5
    private static let margin: CGFloat = 3

    override init(frame: CGRect) {
        super.init(frame: frame)
        scroll.delaysContentTouches = false
        scroll.alwaysBounceVertical = true
        scroll.showsVerticalScrollIndicator = false
        scroll.translatesAutoresizingMaskIntoConstraints = false
        close.translatesAutoresizingMaskIntoConstraints = false
        segmentScroll.delaysContentTouches = false
        segmentScroll.showsHorizontalScrollIndicator = false
        segmentScroll.translatesAutoresizingMaskIntoConstraints = false
        segments.axis = .horizontal
        segments.alignment = .center
        segments.translatesAutoresizingMaskIntoConstraints = false
        segmentScroll.addSubview(segments)
        shorter.translatesAutoresizingMaskIntoConstraints = false
        longer.translatesAutoresizingMaskIntoConstraints = false
        shorter.addAction(UIAction { [weak self] _ in self?.onResize?(-1) }, for: .touchUpInside)
        longer.addAction(UIAction { [weak self] _ in self?.onResize?(1) }, for: .touchUpInside)
        segmentLine.backgroundColor = .separator
        segmentLine.isHidden = true
        segmentLine.translatesAutoresizingMaskIntoConstraints = false
        let line = segmentLine
        [segmentScroll, line, scroll, close, shorter, longer].forEach(addSubview)
        segmentHeight = segmentScroll.heightAnchor.constraint(equalToConstant: 0)
        NSLayoutConstraint.activate([
            segmentScroll.topAnchor.constraint(equalTo: topAnchor),
            segmentScroll.leadingAnchor.constraint(equalTo: leadingAnchor),
            segmentScroll.trailingAnchor.constraint(equalTo: close.leadingAnchor),
            segmentHeight,
            segments.leadingAnchor.constraint(equalTo: segmentScroll.contentLayoutGuide.leadingAnchor, constant: 6),
            segments.trailingAnchor.constraint(equalTo: segmentScroll.contentLayoutGuide.trailingAnchor, constant: -6),
            segments.topAnchor.constraint(equalTo: segmentScroll.contentLayoutGuide.topAnchor),
            segments.bottomAnchor.constraint(equalTo: segmentScroll.contentLayoutGuide.bottomAnchor),
            segments.heightAnchor.constraint(equalTo: segmentScroll.frameLayoutGuide.heightAnchor),
            line.topAnchor.constraint(equalTo: segmentScroll.bottomAnchor),
            line.leadingAnchor.constraint(equalTo: leadingAnchor),
            line.trailingAnchor.constraint(equalTo: close.leadingAnchor),
            line.heightAnchor.constraint(equalToConstant: 1 / UIScreen.main.scale),
            scroll.topAnchor.constraint(equalTo: line.bottomAnchor),
            scroll.bottomAnchor.constraint(equalTo: bottomAnchor),
            scroll.leadingAnchor.constraint(equalTo: leadingAnchor),
            scroll.trailingAnchor.constraint(equalTo: close.leadingAnchor),
            close.trailingAnchor.constraint(equalTo: trailingAnchor),
            close.topAnchor.constraint(equalTo: topAnchor),
            close.heightAnchor.constraint(equalToConstant: Self.rowHeight),
            shorter.trailingAnchor.constraint(equalTo: trailingAnchor),
            shorter.topAnchor.constraint(equalTo: close.bottomAnchor),
            shorter.heightAnchor.constraint(equalToConstant: Self.rowHeight),
            longer.trailingAnchor.constraint(equalTo: trailingAnchor),
            longer.topAnchor.constraint(equalTo: shorter.bottomAnchor),
            longer.heightAnchor.constraint(equalToConstant: Self.rowHeight),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    /// Show this list; the cells stay when only the selection moved.
    func show(_ list: Session.Candidates, runs: [Session.Run]) {
        showSegments(runs)
        shorter.isEnabled = list.canShorten
        longer.isEnabled = list.canLengthen
        if list.items != shown {
            clear()
            shown = list.items
            cells = list.items.enumerated().map { i, item in
                let cell = CandidateCell(item)
                cell.addAction(UIAction { [weak self] _ in self?.onPick?(i) }, for: .touchUpInside)
                scroll.addSubview(cell)
                return cell
            }
            laidWidth = 0
            setNeedsLayout()
        }
        for (i, cell) in cells.enumerated() {
            cell.isSelected = i == list.selected
        }
        layoutIfNeeded()
        if cells.indices.contains(list.selected) {
            scroll.scrollRectToVisible(cells[list.selected].frame, animated: false)
        }
    }

    private func showSegments(_ runs: [Session.Run]) {
        let texts = runs.map(\.text)
        let focus = runs.firstIndex { $0.attr == 2 } ?? -1
        guard texts != shownRuns || focus != shownFocus else { return }
        shownRuns = texts
        shownFocus = focus
        // With one segment there is nothing to choose between, and the grid
        // takes the row.
        segmentHeight.constant = texts.count > 1 ? Self.rowHeight : 0
        segmentLine.isHidden = texts.count <= 1
        segments.arrangedSubviews.forEach { $0.removeFromSuperview() }
        for (i, text) in texts.enumerated() {
            let cell = CandidateCell(.init(text: text, reading: ""))
            cell.isSelected = i == focus
            cell.heightAnchor.constraint(equalToConstant: Self.rowHeight - 8).isActive = true
            cell.addAction(UIAction { [weak self] _ in self?.onFocus?(i) }, for: .touchUpInside)
            segments.addArrangedSubview(cell)
        }
    }

    /// Free the cells while closed.
    func clear() {
        segments.arrangedSubviews.forEach { $0.removeFromSuperview() }
        shownRuns = []
        shownFocus = -1
        cells.forEach { $0.removeFromSuperview() }
        cells = []
        lines.forEach { $0.removeFromSuperview() }
        lines = []
        shown = []
        scroll.contentOffset = .zero
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        guard scroll.bounds.width != laidWidth else { return }
        laidWidth = scroll.bounds.width
        layoutGrid(width: laidWidth)
    }

    /// Fill rows left to right; a full row shares out what is left over, and
    /// the last row stays as it is.
    private func layoutGrid(width: CGFloat) {
        guard width > 0 else { return }
        let inner = width - Self.margin
        let minimum = inner / Self.columns
        var rows: [[(cell: CandidateCell, width: CGFloat)]] = [[]]
        var used: CGFloat = 0
        for cell in cells {
            let w = min(inner, max(minimum, cell.naturalWidth))
            if used + w > inner, !rows[rows.count - 1].isEmpty {
                rows.append([])
                used = 0
            }
            rows[rows.count - 1].append((cell, w))
            used += w
        }
        lines.forEach { $0.removeFromSuperview() }
        lines = []
        for (r, row) in rows.enumerated() {
            let total = row.reduce(0) { $0 + $1.width }
            let extra = r < rows.count - 1 ? (inner - total) / CGFloat(max(row.count, 1)) : 0
            var x = Self.margin
            let y = CGFloat(r) * Self.rowHeight
            for (cell, w) in row {
                cell.frame = CGRect(x: x, y: y + 4, width: w + extra, height: Self.rowHeight - 8)
                x += w + extra
            }
            let line = UIView(frame: CGRect(x: 0, y: y + Self.rowHeight, width: width, height: 1 / max(traitCollection.displayScale, 1)))
            line.backgroundColor = .separator
            scroll.addSubview(line)
            lines.append(line)
        }
        scroll.contentSize = CGSize(width: width, height: CGFloat(rows.count) * Self.rowHeight)
    }
}

/// One candidate: its text with the reading beside it, smaller, on the same
/// baseline, so that candidates line up whether they have a reading or not.
final class CandidateCell: UIControl {
    private let label = UILabel()
    private static let inset: CGFloat = 9
    /// A key's colour, as on the system keyboard's candidate bar.
    private static let selectedFill = UIColor { $0.userInterfaceStyle == .dark ? UIColor(white: 0.42, alpha: 1) : .white }

    init(_ item: Session.Candidates.Item) {
        super.init(frame: .zero)
        layer.cornerRadius = 6
        let text = NSMutableAttributedString(string: item.text, attributes: [
            .font: OldHangul.font(ofSize: 20),
            .foregroundColor: UIColor.label,
        ])
        if !item.reading.isEmpty {
            text.append(NSAttributedString(string: "\u{2009}" + item.reading, attributes: [
                .font: OldHangul.font(ofSize: 12),
                .foregroundColor: UIColor.secondaryLabel,
            ]))
        }
        label.attributedText = text
        label.textAlignment = .center
        label.isUserInteractionEnabled = false
        label.translatesAutoresizingMaskIntoConstraints = false
        addSubview(label)
        NSLayoutConstraint.activate([
            label.leadingAnchor.constraint(equalTo: leadingAnchor, constant: Self.inset),
            label.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -Self.inset),
            label.centerYAnchor.constraint(equalTo: centerYAnchor),
        ])
        isAccessibilityElement = true
        accessibilityTraits = .keyboardKey
        accessibilityLabel = item.reading.isEmpty ? item.text : item.text + ", " + item.reading
    }

    required init?(coder: NSCoder) {
        fatalError("not used")
    }

    /// The width the text asks for.
    var naturalWidth: CGFloat {
        ceil(label.intrinsicContentSize.width) + 2 * Self.inset
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
            : isSelected ? Self.selectedFill : .clear
    }
}
