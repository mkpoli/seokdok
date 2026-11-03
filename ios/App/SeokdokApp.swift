import SwiftUI

/// The container app: how to turn the keyboard on, and its settings, written
/// as config.toml to the App Group the keyboard reads
/// (crates/seokdok-session/src/config.rs documents every key).
@main
struct SeokdokApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    @AppStorage("spaceConverts", store: Settings.store) private var spaceConverts = true
    @AppStorage("oldClusters", store: Settings.store) private var oldClusters = false
    @AppStorage("startLatin", store: Settings.store) private var startLatin = false

    var body: some View {
        NavigationStack {
            Form {
                Section("켜기 · Turn on") {
                    Text("설정 → 일반 → 키보드 → 키보드 → 새로운 키보드 추가 → 釋讀. 설정을 읽으려면 「전체 접근 허용」을 켭니다.")
                    Text("Settings → General → Keyboard → Keyboards → Add New Keyboard → 釋讀. Allow Full Access so the keyboard can read these settings.")
                        .foregroundStyle(.secondary)
                    Button("설정 열기 · Open Settings") {
                        if let url = URL(string: UIApplication.openSettingsURLString) {
                            UIApplication.shared.open(url)
                        }
                    }
                }
                Section("入力 · Input") {
                    Toggle("Space로 變換 · Space converts", isOn: $spaceConverts)
                    Toggle("옛한글 겹자모 · Old Hangul clusters (ㅅㄱ→ᄭ)", isOn: $oldClusters)
                    Toggle("A로 始作 · Start in Latin", isOn: $startLatin)
                }
                Section {
                    Link("seokdok.mkpo.li", destination: URL(string: "https://seokdok.mkpo.li/")!)
                }
            }
            .navigationTitle("釋讀")
            .onChange(of: spaceConverts) { _ in Settings.write() }
            .onChange(of: oldClusters) { _ in Settings.write() }
            .onChange(of: startLatin) { _ in Settings.write() }
            .onAppear { Settings.write() }
        }
    }
}

enum Settings {
    static let appGroup = "group.li.mkpo.seokdok"
    static let store = UserDefaults(suiteName: appGroup)

    /// Write config.toml where the keyboard's session reads it.
    static func write() {
        guard let dir = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup),
              let s = store else { return }
        let toml = """
        [input]
        default_mode = "\(s.bool(forKey: "startLatin") ? "latin" : "seokdok")"
        space_converts = \(s.object(forKey: "spaceConverts") as? Bool ?? true)
        old_clusters = \(s.bool(forKey: "oldClusters"))
        """
        try? toml.write(to: dir.appendingPathComponent("config.toml"), atomically: true, encoding: .utf8)
    }
}
