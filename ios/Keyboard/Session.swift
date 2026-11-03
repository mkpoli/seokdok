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

    struct Candidates {
        let items: [(text: String, note: String)]
        let selected: Int
        let position: Int
        let total: Int
    }

    enum Key {
        static let letter = Int32(SK_LETTER)
        static let printable = Int32(SK_PRINTABLE)
        static let space = Int32(SK_SPACE)
        static let enter = Int32(SK_ENTER)
        static let backspace = Int32(SK_BACKSPACE)
        static let left = Int32(SK_LEFT)
        static let right = Int32(SK_RIGHT)
        static let toggle = Int32(SK_TOGGLE)
    }

    /// `dictDir` holds the *.tsv dictionaries; `configDir`, when given, the
    /// config.toml and custom.json the app writes (the shared App Group).
    init(dictDir: String, configDir: String?) {
        if let configDir {
            setenv("SEOKDOK_CONFIG_DIR", configDir, 1)
        }
        ime = dictDir.withCString { seokdok_new($0) }
    }

    deinit {
        seokdok_free(ime)
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
        return Candidates(items: items, selected: selected, position: position, total: total)
    }

    /// 0 釋讀, 1 Hangul, 2 Latin.
    var mode: Int32 {
        seokdok_mode(ime)
    }

    /// The jamo a key types (upper case = Shift layer), for key labels.
    static func jamo(_ key: Character) -> String? {
        guard let v = key.unicodeScalars.first?.value else { return nil }
        let j = seokdok_key_jamo(v)
        return j == 0 ? nil : UnicodeScalar(j).map { String($0) }
    }

    static var ready: Bool {
        seokdok_ready()
    }
}
