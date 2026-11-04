import Foundation
import SeokdokCore

/// One input session of the Rust core (crates/seokdok-ffi). Strings from the
/// C side are copied at once: they stay valid only until the next call that
/// changes the session.
final class Session {
    private let ime: OpaquePointer

    struct Run {
        let text: String
        /// 0 input, 1 converted, 2 focused segment.
        let attr: Int32
    }

    struct Candidates {
        let items: [(text: String, note: String)]
        let selected: Int
        /// 1-based position of the selected candidate among all of them.
        let position: Int
        let total: Int
        let horizontal: Bool
    }

    enum Mode: Int32, CaseIterable {
        case seokdok = 0, hangul = 1, latin = 2
    }

    /// The dictionaries shipped in the bundle (Resources/dict).
    static let dictDir: String = Bundle.main.resourceURL!.appendingPathComponent("dict").path

    init() {
        ime = Session.dictDir.withCString { seokdok_new($0) }
    }

    deinit {
        seokdok_free(ime)
    }

    /// Re-read the settings (on activation).
    func sync() {
        seokdok_sync(ime)
    }

    /// Feed a key; true when the session used it.
    @discardableResult
    func key(_ kind: Int32, _ ch: UInt32 = 0, shift: Bool = false) -> Bool {
        seokdok_key(ime, kind, ch, shift)
    }

    /// Commit whatever is being composed.
    func flush() {
        seokdok_flush(ime)
    }

    /// Forget the composition (the client ended it).
    func reset() {
        seokdok_reset(ime)
    }

    func takeCommit() -> String? {
        seokdok_take_commit(ime).map { String(cString: $0) }
    }

    func preedit() -> [Run] {
        (0..<seokdok_preedit_len(ime)).compactMap { i in
            var attr: Int32 = 0
            guard let p = seokdok_preedit_run(ime, i, &attr) else { return nil }
            return Run(text: String(cString: p), attr: attr)
        }
    }

    func candidates() -> Candidates? {
        let n = seokdok_popup_len(ime)
        guard n > 0 else { return nil }
        var items: [(text: String, note: String)] = []
        for i in 0..<n {
            var note: UnsafePointer<CChar>?
            guard let p = seokdok_popup_item(ime, i, &note) else { continue }
            items.append((String(cString: p), note.map { String(cString: $0) } ?? ""))
        }
        var selected = 0, position = 0, total = 0
        var horizontal = false
        seokdok_popup_info(ime, &selected, &position, &total, &horizontal)
        return Candidates(items: items, selected: selected, position: position, total: total, horizontal: horizontal)
    }

    var mode: Mode {
        get { Mode(rawValue: seokdok_mode(ime)) ?? .seokdok }
        set { seokdok_set_mode(ime, newValue.rawValue) }
    }

    static var altToggles: Bool {
        seokdok_alt_toggles()
    }

    /// 韓㐎 專用 is on: the session reports Hangul for the 釋讀 mode.
    static var hangulOnly: Bool {
        seokdok_hangul_only()
    }
}

/// config.toml, read and written through the Rust side so that the file
/// keeps every option documented (crates/seokdok-session/src/config.rs).
enum ConfigFile {
    static func load() -> Config {
        guard let p = seokdok_config_json() else { return Config() }
        defer { seokdok_string_free(p) }
        let data = Data(String(cString: p).utf8)
        return (try? decoder.decode(Config.self, from: data)) ?? Config()
    }

    @discardableResult
    static func save(_ config: Config) -> Bool {
        guard let data = try? encoder.encode(config), let json = String(data: data, encoding: .utf8) else { return false }
        return json.withCString { seokdok_config_save_json($0) }
    }

    /// The folder holding config.toml and custom.json.
    static var directory: URL? {
        guard let p = seokdok_config_dir() else { return nil }
        defer { seokdok_string_free(p) }
        return URL(fileURLWithPath: String(cString: p), isDirectory: true)
    }

    private static let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }()

    private static let encoder: JSONEncoder = {
        let e = JSONEncoder()
        e.keyEncodingStrategy = .convertToSnakeCase
        return e
    }()
}

/// The fields of config.toml; the string values are the file's.
struct Config: Codable, Equatable {
    struct Input: Codable, Equatable {
        var defaultMode = "seokdok"
        var spaceConverts = true
        var altToggles = true
        var candidateLayout = "vertical"
        var oldClusters = false
    }

    struct Conversion: Codable, Equatable {
        /// "seokdok", "sino_only" or "hangul_only".
        var script = "seokdok"
        var style = "surface"
        var fragment = "absorb"
        var codaAlways = false
        var resyllabify = false
        var hadaKun = false
        var oldHa = false
        var maxCandidates = 12
    }

    var input = Input()
    var conversion = Conversion()
}
