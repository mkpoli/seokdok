import AppKit

/// The candidate window: one page of candidates numbered 1–9, the selected
/// one highlighted, and below them its note and its place in the list. The
/// session owns paging and selection; this only draws what it is given.
final class CandidateWindow {
    static let shared = CandidateWindow()

    private let panel: NSPanel
    private let view = CandidateView()

    private init() {
        panel = NSPanel(contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: true)
        panel.level = .popUpMenu
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true
        panel.hidesOnDeactivate = false
        panel.isReleasedWhenClosed = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        panel.contentView = view
    }

    /// Show `candidates` below `lineRect` (the line of the marked text, in
    /// screen coordinates); a click calls `pick` with the index on the page.
    func show(_ candidates: Session.Candidates, under lineRect: NSRect, pick: @escaping (Int) -> Void) {
        view.candidates = candidates
        view.pick = pick
        let size = view.preferredSize
        let screen = NSScreen.screens.first { $0.frame.contains(lineRect.origin) } ?? NSScreen.main
        let visible = screen?.visibleFrame ?? NSRect(x: 0, y: 0, width: 10_000, height: 10_000)
        var origin = NSPoint(x: lineRect.minX, y: lineRect.minY - size.height - 4)
        if origin.y < visible.minY {
            // No room below: open above the line.
            origin.y = lineRect.maxY + 4
        }
        origin.x = min(max(origin.x, visible.minX), visible.maxX - size.width)
        panel.setFrame(NSRect(origin: origin, size: size), display: true)
        view.needsDisplay = true
        panel.orderFront(nil)
    }

    func hide() {
        panel.orderOut(nil)
        view.pick = nil
    }
}

private final class CandidateView: NSView {
    var candidates: Session.Candidates? {
        didSet { layoutCells() }
    }

    var pick: ((Int) -> Void)?

    private let textFont = NSFont.systemFont(ofSize: 18)
    private let labelFont = NSFont.monospacedDigitSystemFont(ofSize: 12, weight: .regular)
    private let footerFont = NSFont.systemFont(ofSize: 12)
    private let pad: CGFloat = 6
    private let gap: CGFloat = 6

    private var cells: [NSRect] = []
    private var footerRect = NSRect.zero
    private var size = NSSize.zero

    override var isFlipped: Bool { true }
    var preferredSize: NSSize { size }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    private func label(_ i: Int) -> String { "\(i + 1)" }

    private var footer: String {
        guard let c = candidates else { return "" }
        let note = c.items.indices.contains(c.selected) ? c.items[c.selected].note : ""
        let place = "\(c.position)/\(c.total)"
        return note.isEmpty ? place : "\(note)  \(place)"
    }

    private func layoutCells() {
        cells = []
        guard let c = candidates else {
            size = .zero
            return
        }
        let textHeight = ceil(textFont.ascender - textFont.descender + textFont.leading) + 6
        var x = pad, y = pad, widest: CGFloat = 0
        for (i, item) in c.items.enumerated() {
            let w = ceil(cellWidth(label(i), item.text))
            if c.horizontal {
                cells.append(NSRect(x: x, y: y, width: w, height: textHeight))
                x += w + gap
            } else {
                cells.append(NSRect(x: pad, y: y, width: w, height: textHeight))
                y += textHeight
                widest = max(widest, w)
            }
        }
        if c.horizontal {
            widest = x - gap - pad
            y += textHeight
        }
        let footerSize = (footer as NSString).size(withAttributes: [.font: footerFont])
        widest = max(widest, ceil(footerSize.width) + 8)
        // Rows of a vertical list share the widest row's width.
        if !c.horizontal {
            cells = cells.map { NSRect(x: $0.minX, y: $0.minY, width: widest, height: $0.height) }
        }
        footerRect = NSRect(x: pad, y: y + 2, width: widest, height: ceil(footerSize.height) + 2)
        size = NSSize(width: widest + pad * 2, height: footerRect.maxY + pad)
    }

    private func cellWidth(_ label: String, _ text: String) -> CGFloat {
        let l = (label as NSString).size(withAttributes: [.font: labelFont]).width
        let t = (text as NSString).size(withAttributes: [.font: textFont]).width
        return 8 + l + 6 + t + 8
    }

    override func draw(_ dirtyRect: NSRect) {
        let background = NSBezierPath(roundedRect: bounds, xRadius: 8, yRadius: 8)
        NSColor.windowBackgroundColor.setFill()
        background.fill()
        NSColor.separatorColor.setStroke()
        background.lineWidth = 1
        background.stroke()

        guard let c = candidates else { return }
        for (i, item) in c.items.enumerated() where i < cells.count {
            let r = cells[i]
            let selected = i == c.selected
            if selected {
                NSColor.selectedContentBackgroundColor.setFill()
                NSBezierPath(roundedRect: r, xRadius: 5, yRadius: 5).fill()
            }
            let textColor: NSColor = selected ? .alternateSelectedControlTextColor : .labelColor
            let labelColor: NSColor = selected ? .alternateSelectedControlTextColor : .secondaryLabelColor
            let labelAttrs: [NSAttributedString.Key: Any] = [.font: labelFont, .foregroundColor: labelColor]
            let textAttrs: [NSAttributedString.Key: Any] = [.font: textFont, .foregroundColor: textColor]
            let l = label(i) as NSString
            let lSize = l.size(withAttributes: labelAttrs)
            let tSize = (item.text as NSString).size(withAttributes: textAttrs)
            l.draw(at: NSPoint(x: r.minX + 8, y: r.midY - lSize.height / 2), withAttributes: labelAttrs)
            (item.text as NSString).draw(at: NSPoint(x: r.minX + 8 + lSize.width + 6, y: r.midY - tSize.height / 2), withAttributes: textAttrs)
        }
        (footer as NSString).draw(in: footerRect.insetBy(dx: 4, dy: 0), withAttributes: [.font: footerFont, .foregroundColor: NSColor.secondaryLabelColor])
    }

    override func mouseDown(with event: NSEvent) {
        let p = convert(event.locationInWindow, from: nil)
        if let i = cells.firstIndex(where: { $0.contains(p) }) {
            pick?(i)
        }
    }
}
