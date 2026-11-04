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

    // MARK: Keyboard types
    //
    // Keys of the pad layouts and the email layout, by point offsets from the
    // top of the keyboard. The keyboard is 284 pt high and its top sits 359 pt
    // above the bottom of an iPhone 16-sized window. Pad rows are 50 pt with 7
    // between, in three columns with 6 between, 3 pt in from each side.

    private struct Geometry {
        let app: XCUIApplication
        var width: CGFloat { app.windows.firstMatch.frame.width }
        var height: CGFloat { app.windows.firstMatch.frame.height }

        func tap(x: CGFloat, y: CGFloat) {
            let top = height - 359
            app.coordinate(withNormalizedOffset: CGVector(dx: x / width, dy: (top + y) / height)).tap()
            usleep(400_000)
        }

        /// Pad key in column `c` (0-2) of row `r` (0-3 the grid, 4 the bottom row).
        func pad(_ r: Int, _ c: Int) {
            let w = (width - 6 - 12) / 3
            tap(x: 3 + CGFloat(c) * (w + 6) + w / 2, y: 2 + CGFloat(r) * 57 + 25)
        }
    }

    private func launch(kind: String) -> (XCUIApplication, XCUIElement) {
        let app = XCUIApplication()
        app.launchArguments = ["-focusTrial", "YES", "-trialKind", kind]
        app.launch()
        let field = app.descendants(matching: .any)["trial"]
        XCTAssertTrue(field.waitForExistence(timeout: 10))
        return (app, field)
    }

    private func text(_ field: XCUIElement) -> String {
        let v = field.value as? String ?? ""
        return v == field.placeholderValue ? "" : v
    }

    /// Tap until the keyboard has come up and the field changed, so a slow
    /// extension start does not lose the first key.
    private func tapUntilTyped(_ field: XCUIElement, _ key: () -> Void) {
        for _ in 0..<10 {
            key()
            sleep(2)
            if !text(field).isEmpty { return }
        }
    }

    func testNumberPadTypesDigits() throws {
        let (app, field) = launch(kind: "number")
        let g = Geometry(app: app)
        tapUntilTyped(field) { g.pad(0, 0) } // 1
        g.pad(0, 1) // 2
        g.pad(2, 2) // 9
        g.pad(3, 1) // 0
        shot("number-pad")
        XCTAssertEqual(text(field), "1290")
        g.pad(3, 2) // delete
        XCTAssertEqual(text(field), "129")
        XCUIDevice.shared.appearance = .dark
        sleep(2)
        shot("number-pad-dark")
        XCUIDevice.shared.appearance = .light
    }

    func testDecimalPadTypesDecimalPoint() throws {
        let (app, field) = launch(kind: "decimal")
        let g = Geometry(app: app)
        tapUntilTyped(field) { g.pad(0, 2) } // 3
        g.pad(3, 0) // the decimal separator
        g.pad(1, 1) // 5
        shot("decimal-pad")
        XCTAssertEqual(text(field), "3" + (Locale.current.decimalSeparator ?? ".") + "5")
    }

    /// An email field gets Latin letters and an @ key, never Hangul.
    func testEmailFieldTypesLatinWithoutConverting() throws {
        let (app, field) = launch(kind: "email")
        let g = Geometry(app: app)
        let d = g.width * 274 / 920
        let k = g.width * 738 / 920
        tapUntilTyped(field) { g.tap(x: d, y: 159) }
        g.tap(x: 74, y: 259) // @
        g.tap(x: k, y: 159)
        g.tap(x: 119, y: 259) // .
        shot("email")
        XCTAssertEqual(text(field), "d@k.")
    }
}
