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
    @AppStorage("learn", store: Settings.store) private var learn = true
    @AppStorage("style", store: Settings.store) private var style = "surface"
    @AppStorage("fragment", store: Settings.store) private var fragment = "absorb"
    @AppStorage("codaAlways", store: Settings.store) private var codaAlways = false
    @AppStorage("resyllabify", store: Settings.store) private var resyllabify = false
    @AppStorage("hadaKun", store: Settings.store) private var hadaKun = false
    @AppStorage("itda", store: Settings.store) private var itda = "有"
    @AppStorage("oldHa", store: Settings.store) private var oldHa = false
    @AppStorage("script", store: Settings.store) private var script = "seokdok"
    @AppStorage("maxCandidates", store: Settings.store) private var maxCandidates = 12
    @State private var trial = ""
    @State private var trial2 = ""
    @FocusState private var trialFocused: Bool

    /// Launch arguments for the simulator check: `-trialKind number` (also
    /// decimal, phone, email, url) gives the try-it field that keyboard type,
    /// and `-trialKind2` adds a second field of the given type below it.
    private static func keyboard(_ argument: String) -> UIKeyboardType {
        switch UserDefaults.standard.string(forKey: argument) {
        case "number": return .numberPad
        case "decimal": return .decimalPad
        case "phone": return .phonePad
        case "email": return .emailAddress
        case "url": return .URL
        default: return .default
        }
    }

    var body: some View {
        NavigationStack {
            Form {
                Section("試驗 · Try it") {
                    TextField("여기에 입력 · Type here", text: $trial, axis: .vertical)
                        .focused($trialFocused)
                        .keyboardType(Self.keyboard("trialKind"))
                        .accessibilityIdentifier("trial")
                    if UserDefaults.standard.string(forKey: "trialKind2") != nil {
                        TextField("2", text: $trial2)
                            .keyboardType(Self.keyboard("trialKind2"))
                            .accessibilityIdentifier("trial2")
                    }
                }
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
                if AppGroup.containerURL == nil {
                    Section {
                        Text("이 署名에는 共有 컨테이너가 없어 아래 設定이 키보드에 傳해지지 않습니다. 키보드는 基本 設定으로 動作합니다.")
                        Text("This signature grants no shared container, so the settings below do not reach the keyboard. It runs on the defaults.")
                            .foregroundStyle(.secondary)
                    }
                }
                Section("入力 · Input") {
                    Picker("始作 모드 · Start in", selection: $defaultMode) {
                        Text("釋讀").tag("seokdok")
                        Text("한글 · Hangul").tag("hangul")
                        Text("A · Latin").tag("latin")
                    }
                    Toggle("Space로 變換 · Space converts", isOn: $spaceConverts)
                    Toggle(isOn: $oldClusters) {
                        Text("옛한글 겹자모 · Old Hangul clusters (ㅅㄱ→ᄭ\u{1160})")
                            .font(Font(OldHangul.font(ofSize: 17)))
                    }
                    Toggle("자주 고른 候補 먼저 · Put candidates you keep picking first", isOn: $learn)
                    Button("고른 것 잊기 · Forget what I picked") { Settings.forgetPicks() }
                    Text("전체 접근 없이 쓸 때는 키보드의 釋 키를 길게 눌러 지웁니다. · Without Full Access, hold the keyboard's 釋 key to clear them.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section {
                    Picker("文字 · Script", selection: $script) {
                        Text("釋讀").tag("seokdok")
                        Text("國漢文混用 · Hanja for Sino-Korean words only").tag("sino_only")
                        Text("韓㐎 專用 · Hangul only").tag("hangul_only")
                    }
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
                    Picker("있다의 表記 · Spelling of 있다", selection: $itda) {
                        Text("有다").tag("有")
                        Text("在다").tag("在")
                        Text("在ㅆ다").tag("在ㅆ")
                    }
                    Toggle("하 → ᄒᆞ 優先 · Old spelling of 하 first (하고 → ᄒᆞ고)", isOn: $oldHa)
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
            .onChange(of: learn) { _ in Settings.write() }
            .onChange(of: style) { _ in Settings.write() }
            .onChange(of: fragment) { _ in Settings.write() }
            .onChange(of: codaAlways) { _ in Settings.write() }
            .onChange(of: resyllabify) { _ in Settings.write() }
            .onChange(of: hadaKun) { _ in Settings.write() }
            .onChange(of: itda) { _ in Settings.write() }
            .onChange(of: oldHa) { _ in Settings.write() }
            .onChange(of: script) { _ in Settings.write() }
            .onChange(of: maxCandidates) { _ in Settings.write() }
            .onAppear {
                Settings.write()
                // Launch argument for the simulator check: `-focusTrial YES`.
                trialFocused = UserDefaults.standard.bool(forKey: "focusTrial")
            }
        }
    }
}

enum Settings {
    /// Falls back to the app's own defaults, so the toggles still remember
    /// their state when the signature grants no App Group.
    static let store = AppGroup.defaults ?? .standard

    /// Delete learned.tsv from the shared folder, with any temporary file a
    /// crash left. The keyboard notices at its next conversion.
    static func forgetPicks() {
        guard let dir = AppGroup.containerURL else { return }
        let manager = FileManager.default
        for name in (try? manager.contentsOfDirectory(atPath: dir.path)) ?? []
        where name == "learned.tsv" || (name.hasPrefix("learned.tsv.") && name.hasSuffix(".tmp")) {
            try? manager.removeItem(at: dir.appendingPathComponent(name))
        }
    }

    /// Write config.toml where the keyboard's session reads it. `[input.keys]`
    /// is left out: a keyboard extension receives no hardware keys. The
    /// candidate bar is a row of its own, so the layout stays vertical and
    /// ← → move between segments.
    static func write() {
        guard let dir = AppGroup.containerURL else { return }
        let s = store
        func bool(_ key: String, _ fallback: Bool) -> Bool { s.object(forKey: key) as? Bool ?? fallback }
        func text(_ key: String, _ fallback: String) -> String { s.string(forKey: key) ?? fallback }
        let toml = """
        [input]
        default_mode = "\(text("defaultMode", "seokdok"))"
        space_converts = \(bool("spaceConverts", true))
        candidate_layout = "vertical"
        old_clusters = \(bool("oldClusters", false))
        learn = \(bool("learn", true))

        [conversion]
        style = "\(text("style", "surface"))"
        fragment = "\(text("fragment", "absorb"))"
        coda_always = \(bool("codaAlways", false))
        resyllabify = \(bool("resyllabify", false))
        hada_kun = \(bool("hadaKun", false))
        itda = "\(text("itda", "有"))"
        old_ha = \(bool("oldHa", false))
        script = "\(text("script", "seokdok"))"
        max_candidates = \(s.object(forKey: "maxCandidates") as? Int ?? 12)

        """
        try? toml.write(to: dir.appendingPathComponent("config.toml"), atomically: true, encoding: .utf8)
    }
}
