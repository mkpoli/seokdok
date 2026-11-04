import XCTest

/// Types into the app's try-it field through the real keyboard extension, by
/// tapping screen positions: a keyboard's keys live in another process. The
/// simulator must have 釋讀 as its only enabled keyboard
/// (.github/workflows/ios.yml sets that up). The positions fit iPhone 16-sized
/// screens in portrait.
final class KeyboardUITests: XCTestCase {
    private enum Key {
        static let d = CGVector(dx: 274.0 / 920, dy: 0.771) // ㅇ
        static let k = CGVector(dx: 738.0 / 920, dy: 0.771) // ㅏ
        static let s = CGVector(dx: 180.0 / 920, dy: 0.771) // ㄴ
        static let u = CGVector(dx: 598.0 / 920, dy: 0.7115) // ㅕ
        static let space = CGVector(dx: 474.0 / 920, dy: 0.8855)
        static let enter = CGVector(dx: 816.0 / 920, dy: 0.8855)
    }

    private func shot(_ name: String) {
        guard let dir = ProcessInfo.processInfo.environment["SHOTS"] else { return }
        try? XCUIScreen.main.screenshot().pngRepresentation
            .write(to: URL(fileURLWithPath: dir).appendingPathComponent(name + ".png"))
    }

    func testConvertsHangulToHanja() throws {
        let app = XCUIApplication()
        app.launch()
        shot("app")
        app.terminate()

        app.launchArguments = ["-focusTrial", "YES"]
        app.launch()
        let field = app.descendants(matching: .any)["trial"]
        XCTAssertTrue(field.waitForExistence(timeout: 10))
        // The extension comes up and loads its dictionaries.
        sleep(15)
        shot("keyboard")

        func tap(_ key: CGVector) {
            app.coordinate(withNormalizedOffset: key).tap()
            usleep(300_000)
        }
        // 안녕, then Space to convert.
        for key in [Key.d, Key.k, Key.s, Key.s, Key.u, Key.d] {
            tap(key)
        }
        shot("composing")
        tap(Key.space)
        sleep(1)
        shot("candidates")
        tap(Key.enter)
        sleep(1)
        shot("confirmed")

        let value = field.value as? String ?? ""
        XCTAssertTrue(value.contains("安寧"), "field holds \(value)")
    }
}
