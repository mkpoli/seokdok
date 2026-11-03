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
    @AppStorage("defaultMode", store: Settings.store) private var defaultMode = "seokdok"
    @AppStorage("spaceConverts", store: Settings.store) private var spaceConverts = true
    @AppStorage("oldClusters", store: Settings.store) private var oldClusters = false
    @AppStorage("style", store: Settings.store) private var style = "surface"
    @AppStorage("fragment", store: Settings.store) private var fragment = "absorb"
    @AppStorage("codaAlways", store: Settings.store) private var codaAlways = false
    @AppStorage("resyllabify", store: Settings.store) private var resyllabify = false
    @AppStorage("hadaKun", store: Settings.store) private var hadaKun = false
    @AppStorage("sinoOnly", store: Settings.store) private var sinoOnly = false
    @AppStorage("maxCandidates", store: Settings.store) private var maxCandidates = 12

    var body: some View {
        NavigationStack {
            Form {
                Section("켜기 · Turn on") {
                    Text("설정 → 일반 → 키보드 → 키보드 → 새로운 키보드 추가 → 釋讀. 아래 設定을 쓰려면 「전체 접근 허용」을 켭니다. 켜지 않아도 기본 設定으로 動作합니다.")
                    Text("Settings → General → Keyboard → Keyboards → Add New Keyboard → 釋讀. Turn on Allow Full Access for the keyboard to use the settings below; without it the keyboard works with the defaults.")
                        .foregroundStyle(.secondary)
                    Button("설정 열기 · Open Settings") {
                        if let url = URL(string: UIApplication.openSettingsURLString) {
                            UIApplication.shared.open(url)
                        }
                    }
                }
                Section("入力 · Input") {
                    Picker("始作 모드 · Start in", selection: $defaultMode) {
                        Text("釋讀").tag("seokdok")
                        Text("한글 · Hangul").tag("hangul")
                        Text("A · Latin").tag("latin")
                    }
                    Toggle("Space로 變換 · Space converts", isOn: $spaceConverts)
                    Toggle("옛한글 겹자모 · Old Hangul clusters (ㅅㄱ→ᄭ)", isOn: $oldClusters)
                }
                Section {
                    Toggle("國漢文混用 · Hanja for Sino-Korean words only", isOn: $sinoOnly)
                    Picker("表記 · Style", selection: $style) {
                        Text("表音式 見ㅆ다").tag("surface")
                        Text("形態式 見았다").tag("morpheme")
                    }
                    Picker("남는 母音 · Leftover vowel", selection: $fragment) {
                        Text("吸收 見라").tag("absorb")
                        Text("글자로 見ㅏ라").tag("letter")
                    }
                    .disabled(style != "surface")
                    Toggle("받침 늘 적기 · Always write the final consonant", isOn: $codaAlways)
                    Toggle("連音 · Carry a final consonant to a following ㅇ", isOn: $resyllabify)
                    Toggle("하다 → 爲 · Prefer 爲 for 하다", isOn: $hadaKun)
                    Stepper("候補 數 · Candidates: \(maxCandidates)", value: $maxCandidates, in: 1...50)
                } header: {
                    Text("變換 · Conversion")
                }
                Section {
                    Link("seokdok.mkpo.li", destination: URL(string: "https://seokdok.mkpo.li/")!)
                }
            }
            .navigationTitle("釋讀")
            .onChange(of: defaultMode) { _ in Settings.write() }
            .onChange(of: spaceConverts) { _ in Settings.write() }
            .onChange(of: oldClusters) { _ in Settings.write() }
            .onChange(of: style) { _ in Settings.write() }
            .onChange(of: fragment) { _ in Settings.write() }
            .onChange(of: codaAlways) { _ in Settings.write() }
            .onChange(of: resyllabify) { _ in Settings.write() }
            .onChange(of: hadaKun) { _ in Settings.write() }
            .onChange(of: sinoOnly) { _ in Settings.write() }
            .onChange(of: maxCandidates) { _ in Settings.write() }
            .onAppear { Settings.write() }
        }
    }
}

enum Settings {
    static let appGroup = "group.li.mkpo.seokdok"
    static let store = UserDefaults(suiteName: appGroup)

    /// Write config.toml where the keyboard's session reads it. Keys left
    /// out (alt_toggles) keep their defaults; the candidate bar is a row of
    /// its own, so the layout stays vertical and ← → move between segments.
    static func write() {
        guard let dir = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup),
              let s = store else { return }
        func bool(_ key: String, _ fallback: Bool) -> Bool { s.object(forKey: key) as? Bool ?? fallback }
        func text(_ key: String, _ fallback: String) -> String { s.string(forKey: key) ?? fallback }
        let toml = """
        [input]
        default_mode = "\(text("defaultMode", "seokdok"))"
        space_converts = \(bool("spaceConverts", true))
        candidate_layout = "vertical"
        old_clusters = \(bool("oldClusters", false))

        [conversion]
        style = "\(text("style", "surface"))"
        fragment = "\(text("fragment", "absorb"))"
        coda_always = \(bool("codaAlways", false))
        resyllabify = \(bool("resyllabify", false))
        hada_kun = \(bool("hadaKun", false))
        sino_only = \(bool("sinoOnly", false))
        max_candidates = \(s.object(forKey: "maxCandidates") as? Int ?? 12)

        """
        try? toml.write(to: dir.appendingPathComponent("config.toml"), atomically: true, encoding: .utf8)
    }
}
