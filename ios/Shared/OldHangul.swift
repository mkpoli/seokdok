import CoreText
import UIKit

/// iOS has no glyphs for the old Hangul letters. System fonts cascade to the
/// bundled OldHangul.otf (a subset of Noto Sans CJK KR, made as
/// web/public/fonts/old-sans-400.woff2) so the keys, the strip and the
/// candidates show them.
enum OldHangul {
    private static let postScriptName: String? = {
        guard let url = Bundle.main.url(forResource: "OldHangul", withExtension: "otf"),
              let provider = CGDataProvider(url: url as CFURL),
              let font = CGFont(provider),
              CTFontManagerRegisterGraphicsFont(font, nil)
        else { return nil }
        return font.postScriptName as String?
    }()

    static func font(ofSize size: CGFloat, weight: UIFont.Weight = .regular) -> UIFont {
        let base = UIFont.systemFont(ofSize: size, weight: weight)
        guard let postScriptName else { return base }
        let fallback = UIFontDescriptor(name: postScriptName, size: size)
        return UIFont(descriptor: base.fontDescriptor.addingAttributes([.cascadeList: [fallback]]), size: size)
    }
}
