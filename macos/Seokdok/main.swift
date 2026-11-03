import AppKit
import Carbon
import InputMethodKit

/// `Seokdok --register`: add the input method from where the app is now and
/// enable its modes, without logging out.
func registerInputSource() -> Int32 {
    let status = TISRegisterInputSource(Bundle.main.bundleURL as CFURL)
    guard status == 0 else {
        FileHandle.standardError.write("TISRegisterInputSource failed: \(status)\n".data(using: .utf8)!)
        return 1
    }
    let filter = [kTISPropertyBundleID as String: Bundle.main.bundleIdentifier ?? ""] as CFDictionary
    guard let sources = TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource] else {
        return 1
    }
    for source in sources {
        TISEnableInputSource(source)
    }
    return 0
}

if CommandLine.arguments.contains("--register") {
    exit(registerInputSource())
}

let connectionName = Bundle.main.object(forInfoDictionaryKey: "InputMethodConnectionName") as? String
    ?? "li.mkpo.inputmethod.seokdok_Connection"
let server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)

// Start loading the dictionaries now, so the first text field converts at
// once; later sessions share the engine.
_ = Session()

withExtendedLifetime(server) {
    NSApplication.shared.run()
}
