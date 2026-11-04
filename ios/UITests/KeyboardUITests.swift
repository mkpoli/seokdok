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

    // MARK: The app

    /// Every screen of the app in light and dark, English and Korean.
    func testSettingsScreens() throws {
        for (look, appearance) in [("light", XCUIDevice.Appearance.light), ("dark", .dark)] {
            XCUIDevice.shared.appearance = appearance
            for language in ["en", "ko"] {
                let app = XCUIApplication()
                let name = "settings-\(language)-\(look)"
                app.launchArguments = ["-AppleLanguages", "(\(language))"]
                app.launch()
                XCTAssertTrue(app.cells["conversion"].waitForExistence(timeout: 10) || app.buttons["conversion"].waitForExistence(timeout: 5))
                shot(name + "-root")
                app.descendants(matching: .any)["conversion"].firstMatch.tap()
                sleep(1)
                shot(name + "-conversion")
                app.swipeUp()
                sleep(1)
                shot(name + "-conversion-2")
                app.navigationBars.buttons.firstMatch.tap()
                sleep(1)
                app.descendants(matching: .any)["typing"].firstMatch.tap()
                sleep(1)
                shot(name + "-typing")
                app.terminate()

                // With the keyboard not added yet.
                app.launchArguments = ["-AppleLanguages", "(\(language))", "-AppleKeyboards", "(none)"]
                app.launch()
                XCTAssertTrue(app.descendants(matching: .any)["openSettings"].waitForExistence(timeout: 10))
                shot(name + "-setup")
                app.terminate()
            }
        }
        XCUIDevice.shared.appearance = .light

        // The link leads to this app's page in Settings.
        let app = XCUIApplication()
        app.launch()
        let open = app.descendants(matching: .any)["openSettings"].firstMatch
        XCTAssertTrue(open.waitForExistence(timeout: 10))
        open.tap()
        let settings = XCUIApplication(bundleIdentifier: "com.apple.Preferences")
        _ = settings.wait(for: .runningForeground, timeout: 20)
        sleep(10)
        shot("settings-deep-link")
        // The steps the app gives, followed in Settings itself.
        settings.terminate()
        settings.launch()
        sleep(3)
        settings.staticTexts["General"].firstMatch.tap()
        sleep(1)
        shot("settings-general")
        settings.staticTexts["Keyboard"].firstMatch.tap()
        sleep(2)
        // The page and its first row are both called Keyboards.
        settings.cells.containing(.staticText, identifier: "Keyboards").firstMatch.tap()
        sleep(2)
        shot("settings-keyboards")
        let add = settings.staticTexts.matching(NSPredicate(format: "label BEGINSWITH 'Add New Keyboard'")).firstMatch
        XCTAssertTrue(add.waitForExistence(timeout: 5), "the step the app gives is not in Settings")
        add.tap()
        sleep(2)
        shot("settings-add-keyboard")
        settings.terminate()
        app.terminate()
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

        /// Delete, at the end of the third letter row.
        func delete() {
            tap(x: width - 3 - 23, y: 211)
        }

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

    /// Type a first key once the keyboard has come up. Taps made while the
    /// extension starts can land late and all at once, so the field is emptied
    /// again and the key typed once more on the settled keyboard.
    private func tapUntilTyped(_ field: XCUIElement, delete: () -> Void, _ key: () -> Void) {
        for _ in 0..<10 {
            key()
            sleep(2)
            if !text(field).isEmpty { break }
        }
        sleep(1)
        for _ in 0..<10 where !text(field).isEmpty {
            delete()
            usleep(300_000)
        }
        key()
    }

    /// The field reads `expected`, given a moment: a keyboard's text reaches the
    /// app's accessibility value some time after the tap.
    private func assertText(_ field: XCUIElement, _ expected: String, _ message: String = "", file: StaticString = #filePath, line: UInt = #line) {
        let deadline = Date().addingTimeInterval(6)
        while text(field) != expected, Date() < deadline {
            usleep(200_000)
        }
        XCTAssertEqual(text(field), expected, message, file: file, line: line)
    }

    func testNumberPadTypesDigits() throws {
        let (app, field) = launch(kind: "number")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.pad(3, 2) }) { g.pad(0, 0) } // 1
        g.pad(0, 1) // 2
        g.pad(2, 2) // 9
        g.pad(3, 1) // 0
        shot("number-pad")
        assertText(field, "1290")
        g.pad(3, 2) // delete
        assertText(field, "129")
    }

    func testDecimalPadTypesDecimalPoint() throws {
        let (app, field) = launch(kind: "decimal")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.pad(3, 2) }) { g.pad(0, 2) } // 3
        g.pad(3, 0) // the decimal separator
        g.pad(1, 1) // 5
        shot("decimal-pad")
        assertText(field, "3" + (Locale.current.decimalSeparator ?? ".") + "5")
    }

    /// An email field gets Latin letters and an @ key, never Hangul.
    func testEmailFieldTypesLatinWithoutConverting() throws {
        let (app, field) = launch(kind: "email")
        let g = Geometry(app: app)
        let d = g.width * 274 / 920
        let k = g.width * 738 / 920
        tapUntilTyped(field, delete: { g.delete() }) { g.tap(x: d, y: 159) }
        g.tap(x: 74, y: 259) // @
        g.tap(x: k, y: 159)
        g.tap(x: 119, y: 259) // .
        shot("email")
        assertText(field, "d@k.")
    }

    /// Hangul pending in one field must not be written into the next field.
    func testComposingThenFocusingAnotherFieldWritesNothingThere() throws {
        let (app, field) = launch(kind: "default", second: "email")
        let second = app.descendants(matching: .any)["trial2"]
        let g = Geometry(app: app)
        let w = (g.width - 6 - 45) / 10
        let r = 3 + 3 * (w + 5) + w / 2 // ㄱ
        tapUntilTyped(field, delete: { g.delete() }) { g.tap(x: r, y: 107) }
        g.tap(x: g.width * 738 / 920, y: 159) // ㅏ
        assertText(field, "가")
        second.tap()
        sleep(2)
        shot("switched-field")
        assertText(second, "", "the composition leaked into the next field")
        g.tap(x: g.width * 274 / 920, y: 159) // d, typed as Latin in the email field
        assertText(second, "d")
    }

    /// Two default fields: Hangul pending in the first must stay there, and
    /// the keys tapped right after focusing the second compose in it with
    /// nothing lost.
    func testSwitchingBetweenSameLayoutFieldsKeepsEachCompositionApart() throws {
        let (app, field) = launch(kind: "default", second: "default")
        let second = app.descendants(matching: .any)["trial2"]
        let g = Geometry(app: app)
        let w = (g.width - 6 - 45) / 10
        let r = 3 + 3 * (w + 5) + w / 2 // ㄱ
        tapUntilTyped(field, delete: { g.delete() }) { g.tap(x: r, y: 107) }
        g.tap(x: g.width * 738 / 920, y: 159) // ㅏ
        assertText(field, "가")
        second.tap()
        g.tap(x: g.width * 180 / 920, y: 159) // ㄴ, at once
        g.tap(x: g.width * 738 / 920, y: 159) // ㅏ
        sleep(1)
        shot("switched-same-layout")
        assertText(field, "가", "the first field changed")
        assertText(second, "나", "the second field lost or gained text")
    }

    /// With the keyboard's own globe shown (the workflow forces it on in
    /// Debug builds), the pad still lays out and types.
    func testNumberPadWithGlobeKey() throws {
        let (app, field) = launch(kind: "number")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.pad(3, 2) }) { g.pad(0, 0) }
        g.pad(0, 1)
        shot("number-pad-globe")
        assertText(field, "12")
    }
}
