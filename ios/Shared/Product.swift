import Foundation

/// What products/<id>.json says of the product this was built as; ios/product.sh
/// puts it in Info.plist.
enum Product {
    private static func text(_ key: String, _ fallback: String) -> String {
        Bundle.main.object(forInfoDictionaryKey: key) as? String ?? fallback
    }

    static let hanja = text("SeokdokProductHanja", "釋讀")
    /// The folder of a keyboard running without a shared container.
    static let dir = text("SeokdokProductDir", "seokdok")
    static let guide = text("SeokdokProductGuide", "https://seokdok.mkpo.li/")
    /// Whether it writes 訓讀 at all.
    static let hasKun = Bundle.main.object(forInfoDictionaryKey: "SeokdokProductHasKun") as? Bool ?? true
    static var defaultScript: String { hasKun ? "seokdok" : "sino_only" }
}
