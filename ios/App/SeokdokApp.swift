import SwiftUI

/// The container app: how to turn the keyboard on, a field to try it in, and
/// its settings, written as config.toml to the App Group the keyboard reads
/// (crates/seokdok-session/src/config.rs documents every key).
@main
struct SeokdokApp: App {
    var body: some Scene {
        WindowGroup {
            RootView()
        }
    }
}

/// The label in the system's language: Korean on a Korean iPhone, English on
/// any other.
private func L(_ korean: String, _ english: String) -> String {
    Locale.preferredLanguages.first?.hasPrefix("ko") == true ? korean : english
}

private func openSettings() {
    if let url = URL(string: UIApplication.openSettingsURLString) {
        UIApplication.shared.open(url)
    }
}

/// Text that may hold old letters, which the system fonts lack.
private func oldHangul(_ size: CGFloat) -> Font {
    Font(OldHangul.font(ofSize: size))
}

private enum Page: String, Hashable {
    case conversion, typing
}

struct RootView: View {
    @Environment(\.scenePhase) private var phase
    @State private var added = KeyboardStatus.added
    @State private var path: [Page] = {
        // Launch argument for the simulator check: `-page conversion`.
        UserDefaults.standard.string(forKey: "page").flatMap(Page.init(rawValue:)).map { [$0] } ?? []
    }()
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
        case "ascii": return .asciiCapable
        default: return .default
        }
    }

    var body: some View {
        NavigationStack(path: $path) {
            List {
                if added {
                    Section {
                        Button(L("설정에서 열기", "Open in Settings"), action: openSettings)
                            .accessibilityIdentifier("openSettings")
                    } footer: {
                        Text(L(
                            "키 소리와 이 앱의 設定은 「전체 접근 허용」을 켜야 作動합니다. 켜지 않아도 打字와 變換은 됩니다. 키보드에는 네트워크 코드가 없습니다.",
                            "Key clicks and the settings in this app need Allow Full Access. Typing and conversion work without it. The keyboard has no network code."
                        ))
                    }
                } else {
                    Section(L("키보드 추가", "Add the Keyboard")) {
                        Button(L("설정 열기", "Open Settings"), action: openSettings)
                            .accessibilityIdentifier("openSettings")
                        Step(1, L("일반 → 키보드 → 키보드", "General → Keyboard → Keyboards"))
                        Step(2, L("새로운 키보드 추가 → 釋讀", "Add New Keyboard → 釋讀"))
                        Step(3, L("키 소리와 設定이 必要하면 釋讀을 눌러 「전체 접근 허용」을 켭니다", "For key clicks and settings, tap 釋讀 and turn on Allow Full Access"))
                    }
                }

                Section {
                    TextField(L("여기에 打字해 보세요", "Type here"), text: $trial, axis: .vertical)
                        .focused($trialFocused)
                        .keyboardType(Self.keyboard("trialKind"))
                        .accessibilityIdentifier("trial")
                    if UserDefaults.standard.string(forKey: "trialKind2") != nil {
                        TextField("2", text: $trial2)
                            .keyboardType(Self.keyboard("trialKind2"))
                            .accessibilityIdentifier("trial2")
                    }
                } header: {
                    Text(L("試驗", "Try It"))
                } footer: {
                    Text(L("지구본 키를 길게 눌러 釋讀을 고릅니다.", "Touch and hold the globe key to choose 釋讀."))
                }

                if AppGroup.containerURL == nil {
                    Section {
                        Text(L(
                            "이 署名에는 共有 컨테이너가 없어 設定이 키보드에 傳해지지 않습니다. 키보드는 基本 設定으로 作動합니다.",
                            "This signature grants no shared container, so settings do not reach the keyboard. It runs on the defaults."
                        ))
                    }
                }

                Section(L("設定", "Settings")) {
                    NavigationLink(value: Page.conversion) {
                        Label(L("變換", "Conversion"), systemImage: "character.book.closed")
                    }
                    .accessibilityIdentifier("conversion")
                    NavigationLink(value: Page.typing) {
                        Label(L("入力", "Typing"), systemImage: "keyboard")
                    }
                    .accessibilityIdentifier("typing")
                }

                Section {
                    Link("seokdok.mkpo.li", destination: URL(string: "https://seokdok.mkpo.li/")!)
                }
            }
            .listStyle(.insetGrouped)
            .navigationTitle("釋讀")
            .navigationDestination(for: Page.self) { page in
                switch page {
                case .conversion: ConversionPage()
                case .typing: TypingPage()
                }
            }
        }
        .onAppear {
            Settings.write()
            // Launch argument for the simulator check: `-focusTrial YES`.
            trialFocused = UserDefaults.standard.bool(forKey: "focusTrial")
        }
        .onChange(of: phase) { _ in added = KeyboardStatus.added }
        .onReceive(NotificationCenter.default.publisher(for: UserDefaults.didChangeNotification)) { _ in
            Settings.write()
        }
    }
}

private struct Step: View {
    let number: Int
    let text: String

    init(_ number: Int, _ text: String) {
        self.number = number
        self.text = text
    }

    var body: some View {
        Label {
            Text(text)
        } icon: {
            Image(systemName: "\(number).circle")
                .foregroundStyle(.secondary)
        }
    }
}

/// One setting as Settings lays it out: a group of its own, its name above
/// when it is a list of choices, one line about it and a real example below.
private struct SettingGroup<Content: View>: View {
    var name: String?
    var about: String?
    var example: String?
    @ViewBuilder var content: Content

    var body: some View {
        Section {
            content
        } header: {
            if let name { Text(name) }
        } footer: {
            VStack(alignment: .leading, spacing: 4) {
                if let about { Text(about) }
                if let example { Text(example).font(oldHangul(13)) }
            }
        }
    }
}

/// A list of choices with a check mark on the chosen one.
private struct Choices<Value: Hashable>: View {
    let name: String
    @Binding var selection: Value
    let options: [(Value, String)]

    var body: some View {
        Picker(name, selection: $selection) {
            ForEach(options, id: \.0) { option in
                Text(option.1).font(oldHangul(17)).tag(option.0)
            }
        }
        .pickerStyle(.inline)
        .labelsHidden()
    }
}

private struct ConversionPage: View {
    @AppStorage("script", store: Settings.store) private var script = "seokdok"
    @AppStorage("style", store: Settings.store) private var style = "surface"
    @AppStorage("fragment", store: Settings.store) private var fragment = "absorb"
    @AppStorage("itda", store: Settings.store) private var itda = "有"
    @AppStorage("codaAlways", store: Settings.store) private var codaAlways = false
    @AppStorage("resyllabify", store: Settings.store) private var resyllabify = false
    @AppStorage("hadaKun", store: Settings.store) private var hadaKun = false
    @AppStorage("oldHa", store: Settings.store) private var oldHa = false
    @AppStorage("maxCandidates", store: Settings.store) private var maxCandidates = 12

    /// Style and fragment read as one choice: the leftover vowel only exists
    /// in the short ending.
    private var ending: Binding<Int> {
        Binding(
            get: { style == "morpheme" ? 2 : fragment == "letter" ? 1 : 0 },
            set: { value in
                style = value == 2 ? "morpheme" : "surface"
                if value != 2 { fragment = value == 1 ? "letter" : "absorb" }
            }
        )
    }

    /// 하다 in Hanja and 하 in the old spelling both decide how 하 comes out.
    private var ha: Binding<Int> {
        Binding(
            get: { oldHa ? (hadaKun ? 3 : 2) : (hadaKun ? 1 : 0) },
            set: { value in
                hadaKun = value == 1 || value == 3
                oldHa = value >= 2
            }
        )
    }

    var body: some View {
        List {
            let scriptName = L("韓㐎을 무엇으로 變換하나", "What your Hangul becomes")
            SettingGroup(
                name: scriptName,
                about: L("Space를 押면 이렇게 變합니다. 모드 키의 釋 混 한입니다.", "Space turns what you typed into this. The mode key shows it as 釋, 混 or 한."),
                example: "선생님께서 공부하셨다 → 先生主께서 工夫하셨다 · 先生님께서 工夫하셨다 · 선생님께서 공부하셨다"
            ) {
                Choices(name: scriptName, selection: $script, options: [
                    ("seokdok", L("釋讀: 固有語까지 漢字로", "釋讀: native words in Hanja too")),
                    ("sino_only", L("國漢文混用: 漢字語만 漢字로", "國漢文混用: Sino-Korean words only")),
                    ("hangul_only", L("韓㐎 專用: 變換하지 않음", "Hangul only: no conversion")),
                ])
            }

            let endingName = L("漢字 뒤의 語尾", "The ending after a Hanja verb")
            SettingGroup(
                name: endingName,
                about: L("動詞를 漢字로 적을 때 뒤에 남기는 韓㐎", "How much of the verb stays in Hangul"),
                example: "봤다 → 見ㅆ다 · 見ㅏㅆ다 · 見았다"
            ) {
                Choices(name: endingName, selection: ending, options: [
                    (0, L("줄여서 見ㅆ다", "Short: 見ㅆ다")),
                    (1, L("남는 母音도 字母로 見ㅏㅆ다", "With the vowel as a letter: 見ㅏㅆ다")),
                    (2, L("語尾 그대로 見았다", "The whole ending: 見았다")),
                ])
            }

            let itdaName = L("있다를 적는 字", "The Hanja for 있다")
            SettingGroup(
                name: itdaName,
                about: L("먼저 나오는 表記입니다. 다른 表記도 候補에 남습니다.", "Listed first. The others stay among the candidates.")
            ) {
                Choices(name: itdaName, selection: $itda, options: [("有", "有다"), ("在", "在다"), ("在ㅆ", "在ㅆ다")])
            }

            SettingGroup(example: "먹으면 → 食으면 · 食ㄱ으면") {
                Toggle(L("動詞의 받침을 漢字 뒤에 적기", "Write the verb's final consonant after the Hanja"), isOn: $codaAlways)
            }

            SettingGroup(
                about: L("받침을 적을 때만 작용합니다: 위 設定, 또는 있다의 在ㅆ다.", "Only works when the consonant is written: the setting above, or 在ㅆ다 for 있다."),
                example: "먹으면 → 食ㄱ으면 · 食그면"
            ) {
                Toggle(L("그 받침을 다음 字로 넘기기", "Carry that consonant into the next syllable"), isOn: $resyllabify)
                    .disabled(!codaAlways && itda != "在ㅆ")
            }

            let haName = L("하를 적는 法", "How 하 is written")
            SettingGroup(
                name: haName,
                about: L("먼저 나오는 表記입니다. 다른 表記도 候補에 남습니다.", "Listed first. The others stay among the candidates."),
                example: "하고 → 하고 · 爲고 · \u{1112}\u{119E}고"
            ) {
                Choices(name: haName, selection: ha, options: [
                    (0, L("그대로 하고", "As it is: 하고")),
                    (1, L("漢字로 爲고", "In Hanja: 爲고")),
                    (2, L("옛 表記로 \u{1112}\u{119E}고", "Old spelling: \u{1112}\u{119E}고")),
                    (3, L("옛 表記로, 했다만 爲ㅆ다", "Old spelling, and 爲ㅆ다 for 했다")),
                ])
            }

            SettingGroup(
                about: L("句節 全體를 變換한 候補의 上限입니다. 낱말 候補는 그대로여서 目錄의 맨 끝만 조금 달라집니다.", "A limit on conversions of the whole phrase. Word candidates are not affected, so only the end of the list changes a little.")
            ) {
                Stepper(value: $maxCandidates, in: 1...50) {
                    LabeledContent(L("候補 數", "How many candidates"), value: "\(maxCandidates)")
                }
            }
        }
        .listStyle(.insetGrouped)
        .navigationTitle(L("變換", "Conversion"))
        .navigationBarTitleDisplayMode(.large)
    }
}

private struct TypingPage: View {
    @AppStorage("defaultMode", store: Settings.store) private var defaultMode = "seokdok"
    @AppStorage("spaceConverts", store: Settings.store) private var spaceConverts = true
    @AppStorage("oldClusters", store: Settings.store) private var oldClusters = false
    @AppStorage("learn", store: Settings.store) private var learn = true
    @State private var forgotten = false

    var body: some View {
        List {
            let startName = L("앱에서 처음 打기 始作할 때", "When you start typing in an app")
            SettingGroup(
                name: startName,
                about: L("모드 키로 언제든 바꿀 수 있습니다.", "The mode key switches this at any time."),
                example: "gksmf → 天 · 하늘 · gksmf"
            ) {
                Choices(name: startName, selection: $defaultMode, options: [
                    ("seokdok", L("韓㐎, 變換함", "Hangul, converted")),
                    ("hangul", L("韓㐎, 變換 無이", "Hangul, no conversion")),
                    ("latin", L("英字", "English letters")),
                ])
            }

            SettingGroup(
                about: L("끄면 Space는 띄어쓰기이고, 候補 欄의 글자를 눌러 變換합니다.", "Off: Space types a space, and tapping the text in the candidate bar converts."),
                example: "하늘을 + Space → 天을 · 하늘을␣"
            ) {
                Toggle(L("Space로 變換", "Space converts"), isOn: $spaceConverts)
            }

            SettingGroup(
                about: L("두 번 고른 候補가 그 읽기의 첫 候補가 됩니다. 고른 낱말은 이 iPhone의 learned.tsv에만 남습니다.", "A candidate picked twice comes first for that reading. The words stay on this iPhone in learned.tsv.")
            ) {
                Toggle(L("자주 고른 候補 먼저", "Put candidates you keep picking first"), isOn: $learn)
            }

            SettingGroup(
                about: L("learned.tsv를 지웁니다. 끈 뒤에도 파일은 지울 때까지 남습니다. 전체 접근 없이 쓸 때는 키보드의 釋 키를 길게 눌러 지웁니다.", "Deletes learned.tsv. Turning learning off keeps the file until you delete it. Without Full Access, hold the keyboard's 釋 key to clear it.")
            ) {
                Button(forgotten ? L("지웠습니다", "Forgotten") : L("고른 것 잊기", "Forget what I picked")) {
                    Settings.forgetPicks()
                    forgotten = true
                }
                .disabled(forgotten)
            }

            SettingGroup(example: "ㅅ, ㄱ, ㅏ → ㅅ가 · \u{112D}\u{1161}") {
                Toggle(L("現代 字母를 이어 옛 겹字母로", "Join modern letters into old clusters"), isOn: $oldClusters)
            }
        }
        .listStyle(.insetGrouped)
        .navigationTitle(L("入力", "Typing"))
        .navigationBarTitleDisplayMode(.large)
    }
}

enum KeyboardStatus {
    /// Whether this app's keyboard is among the enabled ones. iOS lists them
    /// under `AppleKeyboards`; a re-signing tool keeps the keyboard's ID an
    /// extension of the app's.
    static var added: Bool {
        guard let id = Bundle.main.bundleIdentifier,
              let keyboards = UserDefaults.standard.array(forKey: "AppleKeyboards") as? [String]
        else { return false }
        return keyboards.contains { $0.hasPrefix(id + ".") }
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
