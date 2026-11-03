import Foundation

/// The App Group the app and the keyboard share. Sideloading tools re-sign
/// the app under the user's Apple ID and may rename the group, so the name in
/// force is read from the provisioning profile embedded by the signer. The
/// name in Info.plist comes first and the profile's names are sorted, so the
/// app and the keyboard, which have separate profiles, settle on the same one.
enum AppGroup {
    private static let resolved: (id: String, url: URL)? = {
        for id in candidates() {
            if let url = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: id) {
                return (id, url)
            }
        }
        return nil
    }()

    /// The shared directory, or nil when the signature grants no group.
    static var containerURL: URL? { resolved?.url }

    /// The defaults both sides can see, or nil with no shared container.
    static var defaults: UserDefaults? { resolved.flatMap { UserDefaults(suiteName: $0.id) } }

    private static func candidates() -> [String] {
        let configured = Bundle.main.object(forInfoDictionaryKey: "SeokdokAppGroup") as? String
        return [configured].compactMap { $0 } + provisionedGroups().sorted()
    }

    /// `com.apple.security.application-groups` of embedded.mobileprovision,
    /// a CMS envelope with the profile as a plain XML plist inside.
    private static func provisionedGroups() -> [String] {
        guard let url = Bundle.main.url(forResource: "embedded", withExtension: "mobileprovision"),
              let data = try? Data(contentsOf: url, options: .mappedIfSafe),
              let start = data.range(of: Data("<?xml".utf8)),
              let end = data.range(of: Data("</plist>".utf8), in: start.lowerBound..<data.endIndex),
              let plist = try? PropertyListSerialization.propertyList(from: data[start.lowerBound..<end.upperBound], format: nil),
              let root = plist as? [String: Any],
              let entitlements = root["Entitlements"] as? [String: Any]
        else { return [] }
        return entitlements["com.apple.security.application-groups"] as? [String] ?? []
    }
}
