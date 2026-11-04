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
        static let f = CGVector(dx: 367.0 / 920, dy: 0.771) // ㄹ
        static let r = CGVector(dx: 321.0 / 920, dy: 0.7115) // ㄱ
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

        // 갈 has candidates told apart by their dictionary words.
        for key in [Key.r, Key.k, Key.f, Key.space] {
            tap(key)
        }
        sleep(1)
        shot("readings")
        tap(Key.enter)
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

        /// The vertical centre of the candidate bar.
        var barCenter: CGFloat { 56 }

        /// Pad key in column `c` (0-2) of row `r` (0-3 the grid, 4 the bottom row).
        func pad(_ r: Int, _ c: Int) {
            let w = (width - 6 - 12) / 3
            tap(x: 3 + CGFloat(c) * (w + 6) + w / 2, y: 2 + CGFloat(r) * 57 + 25)
        }
    }

    private func launch(kind: String, second: String? = nil) -> (XCUIApplication, XCUIElement) {
        let app = XCUIApplication()
        app.launchArguments = ["-focusTrial", "YES", "-trialKind", kind]
        if let second { app.launchArguments += ["-trialKind2", second] }
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

    /// Hangul pending in one field must not be written into the next field.
    func testComposingThenFocusingAnotherFieldWritesNothingThere() throws {
        let (app, field) = launch(kind: "default", second: "email")
        let second = app.descendants(matching: .any)["trial2"]
        let g = Geometry(app: app)
        let w = (g.width - 6 - 45) / 10
        let r = 3 + 3 * (w + 5) + w / 2 // ㄱ
        tapUntilTyped(field) { g.tap(x: r, y: 107) }
        g.tap(x: g.width * 738 / 920, y: 159) // ㅏ
        XCTAssertEqual(text(field), "가")
        second.tap()
        sleep(2)
        shot("switched-field")
        XCTAssertEqual(text(second), "", "the composition leaked into the next field")
        g.tap(x: g.width * 274 / 920, y: 159) // d, typed as Latin in the email field
        XCTAssertEqual(text(second), "d")
    }

    /// The chevron opens every candidate over the keys, and a tap on one
    /// confirms it.
    func testDrawerOpensAndPicks() throws {
        let (app, field) = launch(kind: "default")
        let g = Geometry(app: app)
        tapUntilTyped(field) { g.tap(x: g.width * 274 / 920, y: 159) } // ㅇ
        for x: CGFloat in [738, 180, 180, 598, 274] {
            g.tap(x: g.width * x / 920, y: x == 598 ? 107 : 159) // ㅏ ㄴ ㄴ ㅕ ㅇ
        }
        g.tap(x: g.width * 474 / 920, y: 259) // Space
        sleep(1)
        g.tap(x: g.width - 25, y: 56)
        usleep(100_000)
        shot("drawer-opening")
        sleep(1)
        shot("drawer")
        g.tap(x: g.width - 22, y: 22) // closes
        sleep(1)
        shot("drawer-closed")
        g.tap(x: g.width - 25, y: 56)
        sleep(1)
        g.tap(x: 40, y: 66) // the second row's first candidate
        sleep(1)
        shot("drawer-picked")
        let value = text(field)
        XCTAssertFalse(value.isEmpty)
        XCTAssertNotEqual(value, "安寧", "the first candidate came out, not the tapped one")
        XCTAssertNotEqual(value, "안녕")
    }

    /// With the keyboard's own globe shown (the workflow forces it on in
    /// Debug builds), the pad still lays out and types.
    func testNumberPadWithGlobeKey() throws {
        let (app, field) = launch(kind: "number")
        let g = Geometry(app: app)
        tapUntilTyped(field) { g.pad(0, 0) }
        g.pad(0, 1)
        shot("number-pad-globe")
        XCTAssertEqual(text(field), "12")
    }
}
