import Foundation
import SeokdokCore

/// products/<id>.json, as the Rust library was built.
enum Product {
    private static let definition: [String: Any] = {
        guard let p = seokdok_product_json() else { return [:] }
        defer { seokdok_string_free(p) }
        return (try? JSONSerialization.jsonObject(with: Data(String(cString: p).utf8))) as? [String: Any] ?? [:]
    }()

    static let hanja = definition["hanja"] as? String ?? "釋讀"
    static let latin = definition["latin"] as? String ?? "Seokdok"
    static let scripts = definition["scripts"] as? [String] ?? ["seokdok", "sino_only", "hangul_only"]
    /// Whether it writes 訓讀 at all.
    static var hasKun: Bool { scripts.contains("seokdok") }
    static var defaultScript: String { definition["default_script"] as? String ?? "seokdok" }
}
