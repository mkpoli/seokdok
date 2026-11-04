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
                ConfigFile.save(config)
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
                Toggle("Option 한 番으로 韓/A 轉換 / Tap Option to switch", isOn: $model.config.input.altToggles)
                Toggle("옛한글 겹자모 (ㅅㄱ→ᄭ, ㅂㅇ→ㅸ) / Old Hangul clusters", isOn: $model.config.input.oldClusters)
                Picker("候補 窓 / Candidates", selection: $model.config.input.candidateLayout) {
                    Text("縱 / Vertical").tag("vertical")
                    Text("橫 / Horizontal  (← → move)").tag("horizontal")
                }
            }
            Section("表記 / Spelling") {
                Picker("文字 / Script", selection: $model.config.conversion.script) {
                    Text("釋讀 / Seokdok").tag("seokdok")
                    Text("國漢文混用 / Hanja for Sino-Korean words only").tag("sino_only")
                    Text("韓㐎 專用 / Hangul only, no conversion").tag("hangul_only")
                }
                Picker("送韓字 / Style", selection: $model.config.conversion.style) {
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
