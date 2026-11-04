import Foundation
import SeokdokCore

/// The Rust input-method session behind the keyboard (crates/seokdok-ffi).
/// Strings from the C side are copied at once: they stay valid only until
/// the next call that changes the session.
final class Session {
    private let ime: OpaquePointer

    struct Run {
        let text: String
        /// 0 input, 1 converted, 2 focused segment.
        let attr: Int32
    }

    struct Candidates: Equatable {
        struct Item: Equatable {
            let text: String
            /// What to show beside the candidate; may be empty.
            let reading: String
        }
        /// Every candidate of the focused segment.
        let items: [Item]
        let selected: Int
        let canShorten: Bool
        let canLengthen: Bool
    }

    /// Input modes, as numbered by `seokdok_mode`.
    enum Mode: Int32 {
        case seokdok = 0, hangul = 1, latin = 2
    }

    /// Key kinds of `seokdok_key`.
    enum Key {
        static let letter = Int32(SK_LETTER)
        static let printable = Int32(SK_PRINTABLE)
        static let space = Int32(SK_SPACE)
        static let enter = Int32(SK_ENTER)
        static let esc = Int32(SK_ESC)
        static let backspace = Int32(SK_BACKSPACE)
        static let left = Int32(SK_LEFT)
        static let right = Int32(SK_RIGHT)
        static let up = Int32(SK_UP)
        static let down = Int32(SK_DOWN)
        static let convert = Int32(SK_CONVERT)
    }

    /// `dictDir` holds the *.tsv dictionaries.
    init(dictDir: String) {
        ime = dictDir.withCString { seokdok_new($0)! }
    }

    deinit {
        seokdok_free(ime)
    }

    /// Where config.toml, custom.json and user.tsv are read from: the App
    /// Group container the app writes, or nil for the keyboard's own
    /// container (no settings, so the defaults apply).
    static func setConfigDir(_ dir: String?) {
        if let dir {
            setenv("SEOKDOK_CONFIG_DIR", dir, 1)
        } else {
            unsetenv("SEOKDOK_CONFIG_DIR")
        }
    }

    /// `conversion.script`: 0 釋讀, 1 國漢文混用, 2 韓㐎 專用.
    static var script: Int32 {
        seokdok_script()
    }

    /// The mode key: 釋 → 混 → 한 → A → 釋; what is being composed is committed.
    func modeControl() {
        seokdok_mode_control(ime)
    }

    /// The field wants nothing learned: picks are neither used nor added.
    func setPrivateField(_ privateField: Bool) {
        seokdok_set_private_field(ime, privateField)
    }

    /// `input.learn` in the config.toml in force.
    static var learn: Bool {
        get { seokdok_learn() }
        set { _ = seokdok_set_learn(newValue) }
    }

    /// Delete learned.tsv beside the config.toml in force, and the table in memory.
    static func forgetPicks() {
        _ = seokdok_learned_forget()
    }

    /// Re-read the settings.
    func sync() {
        seokdok_sync(ime)
    }

    /// Feed a key; true when the session used it.
    @discardableResult
    func key(_ kind: Int32, _ ch: Character? = nil, shift: Bool = false) -> Bool {
        seokdok_key(ime, kind, ch?.unicodeScalars.first?.value ?? 0, shift)
    }

    /// Commit whatever is being composed.
    func flush() {
        seokdok_flush(ime)
    }

    /// Forget the composition (the text changed under it).
    func reset() {
        seokdok_reset(ime)
    }

    func takeCommit() -> String? {
        guard let p = seokdok_take_commit(ime) else { return nil }
        return String(cString: p)
    }

    func preedit() -> [Run] {
        (0..<seokdok_preedit_len(ime)).compactMap { i in
            var attr: Int32 = 0
            guard let p = seokdok_preedit_run(ime, i, &attr) else { return nil }
            return Run(text: String(cString: p), attr: attr)
        }
    }

    /// The focused segment's candidates, or nil while nothing is converted.
    func candidates() -> Candidates? {
        let n = seokdok_list_len(ime)
        guard n > 0 else { return nil }
        var items: [Candidates.Item] = []
        items.reserveCapacity(n)
        for i in 0..<n {
            var reading: UnsafePointer<CChar>?
            guard let p = seokdok_list_item(ime, i, &reading) else { continue }
            let text = String(cString: p)
            items.append(.init(text: text, reading: reading.map { String(cString: $0) } ?? ""))
        }
        var shorten = false, lengthen = false
        seokdok_list_can_resize(ime, &shorten, &lengthen)
        return Candidates(items: items, selected: seokdok_list_selected(ime), canShorten: shorten, canLengthen: lengthen)
    }

    /// Choose candidate `index` and confirm through the focused segment.
    func pick(_ index: Int) {
        seokdok_list_pick(ime, index)
    }

    /// Whether digits would pick a candidate (Space has walked the list).
    var digitsPick: Bool {
        seokdok_popup_len(ime) > 0
    }

    var mode: Mode {
        get { Mode(rawValue: seokdok_mode(ime)) ?? .seokdok }
        set { seokdok_set_mode(ime, newValue.rawValue) }
    }

    /// The jamo a key types (upper case = Shift layer), for key labels.
    static func jamo(_ key: Character) -> String? {
        guard let v = key.unicodeScalars.first?.value else { return nil }
        let j = seokdok_key_jamo(v)
        guard j != 0, let s = Unicode.Scalar(j) else { return nil }
        return String(Character(s))
    }

    /// Whether Space converts (a setting); the convert key always does.
    static var spaceConverts: Bool {
        seokdok_space_converts()
    }

    /// Whether the dictionaries have finished loading.
    static var ready: Bool {
        seokdok_ready()
    }
}
