import AppKit
import SwiftUI

/// The settings window, opened from the input menu. Every change is written
/// to config.toml at once; the input method applies it the next time a text
/// field gains focus.
final class SettingsWindow {
    static let shared = SettingsWindow()

    private let model = SettingsModel()
    private var window: NSWindow?

    func show() {
        model.reload()
        if window == nil {
            let w = NSWindow(contentViewController: NSHostingController(rootView: SettingsView(model: model)))
            w.title = "釋讀 設定 / Settings"
            w.styleMask = [.titled, .closable]
            w.isReleasedWhenClosed = false
            w.center()
            window = w
        }
        NSApp.activate(ignoringOtherApps: true)
        window?.makeKeyAndOrderFront(nil)
    }
}

final class SettingsModel: ObservableObject {
    @Published var config = ConfigFile.load() {
        didSet {
            if config != oldValue {
                // Unless chosen here, the script stays what the chord left in the file.
                var saved = config
                if saved.conversion.script == oldValue.conversion.script {
                    saved.conversion.script = ConfigFile.load().conversion.script
                }
                ConfigFile.save(saved)
            }
        }
    }

    func reload() {
        config = ConfigFile.load()
    }
}

struct SettingsView: View {
    @ObservedObject var model: SettingsModel

    var body: some View {
        Form {
            Section("入力 / Input") {
                Toggle("Space로 變換 / Space converts (off: Option-Return, F7, Tab)", isOn: $model.config.input.spaceConverts)
                Toggle("옛한글 겹자모 (ㅅㄱ→ᄭ, ㅂㅇ→ㅸ) / Old Hangul clusters", isOn: $model.config.input.oldClusters)
                Picker("候補 窓 / Candidates", selection: $model.config.input.candidateLayout) {
                    Text("縱 / Vertical").tag("vertical")
                    Text("橫 / Horizontal  (← → move)").tag("horizontal")
                }
            }
            Section("鍵 / Keys") {
                KeyListField(label: "韓 ↔ A / Korean ↔ Latin", keys: $model.config.input.keys.toggle)
                KeyListField(label: "釋讀 → 國漢文混用 → 韓㐎 專用", keys: $model.config.input.keys.scriptCycle)
                KeyListField(label: "變換 / Convert", keys: $model.config.input.keys.convert)
                Text("Comma-separated, such as Alt, F7, Alt+Enter. A modifier alone means tapping it. Other lists are in config.toml. Control-Space, Globe and Caps Lock are set in System Settings › Keyboard.")
                    .foregroundStyle(.secondary)
            }
            Section("表記 / Spelling") {
                Picker("文字 / Script", selection: $model.config.conversion.script) {
                    Text("釋讀 / Seokdok").tag("seokdok")
                    Text("國漢文混用 / Hanja for Sino-Korean words only").tag("sino_only")
                    Text("韓㐎 專用 / Hangul only, no conversion").tag("hangul_only")
                }
                Picker("末音添記 / Style", selection: $model.config.conversion.style) {
                    Text("表音式 / Surface  (見ㅆ다)").tag("surface")
                    Text("形態式 / Morpheme  (見았다)").tag("morpheme")
                }
                Picker("남는 母音 / Leftover vowel", selection: $model.config.conversion.fragment) {
                    Text("漢字에 吸收 / Absorb  (見라)").tag("absorb")
                    Text("字母로 / Letter  (見ㅏ라)").tag("letter")
                }
                Toggle("末音添記 늘 / Always write the final consonant  (有ㅆ다)", isOn: $model.config.conversion.codaAlways)
                Toggle("받침 넘기기 / Resyllabify  (在쓰면)", isOn: $model.config.conversion.resyllabify)
                Toggle("하다를 爲로 / Prefer 爲 for 하다", isOn: $model.config.conversion.hadaKun)
                Picker("있다의 表記 / Spelling of 있다", selection: $model.config.conversion.itda) {
                    Text("有다").tag("有")
                    Text("在다").tag("在")
                    Text("在ㅆ다").tag("在ㅆ")
                }
                Toggle("하 → ᄒᆞ 優先 / Old spelling of 하 first  (하고 → ᄒᆞ고)", isOn: $model.config.conversion.oldHa)
                Stepper(value: $model.config.conversion.maxCandidates, in: 1...99) {
                    Text("最大 候補 數 / Max candidates: \(model.config.conversion.maxCandidates)")
                }
            }
            Section("custom.json") {
                Text("Words, phrases and rewrite rules exported from seokdok.mkpo.li go in custom.json in the settings folder.")
                    .foregroundStyle(.secondary)
                HStack {
                    Button("設定 폴더 열기 / Open the settings folder") {
                        guard let dir = ConfigFile.directory else { return }
                        if !FileManager.default.fileExists(atPath: dir.appendingPathComponent("config.toml").path) {
                            // Write the documented file so there is something to edit.
                            ConfigFile.save(model.config)
                        }
                        NSWorkspace.shared.open(dir)
                    }
                    Link("seokdok.mkpo.li/#custom", destination: URL(string: "https://seokdok.mkpo.li/#custom")!)
                }
            }
        }
        .formStyle(.grouped)
        .frame(width: 560, height: 600)
    }
}

/// A list of `[input.keys]` as comma-separated text. The text stays as typed,
/// so a trailing comma survives until the next key name follows it.
private struct KeyListField: View {
    let label: String
    @Binding var keys: [String]
    @State private var text = ""

    private static func parse(_ text: String) -> [String] {
        text.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
    }

    var body: some View {
        TextField(label, text: $text)
            .onAppear { text = keys.joined(separator: ", ") }
            .onChange(of: text) { new in
                let parsed = Self.parse(new)
                if parsed != keys { keys = parsed }
            }
            .onChange(of: keys) { new in
                if Self.parse(text) != new { text = new.joined(separator: ", ") }
            }
    }
}
