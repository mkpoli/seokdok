import XCTest

/// Types into the app's try-it field through the real keyboard extension, by
/// tapping screen positions: a keyboard's keys live in another process. The
/// simulator must have 釋讀 as its only enabled keyboard
/// (.github/workflows/ios.yml sets that up). The positions fit iPhone 16-sized
/// screens in portrait; `Geometry` holds them.
final class KeyboardUITests: XCTestCase {
    /// Keys by point offsets from the top of the keyboard, which is 261 pt high
    /// with its top 336 pt above the bottom of an iPhone 16-sized window. The
    /// bar is 45 pt, and key rows are 54 pt apart from y 54; letter keys have 6
    /// between. Pad rows are 52.8 pt apart in three columns with 6 between.
    private struct Geometry {
        let app: XCUIApplication
        var width: CGFloat { app.windows.firstMatch.frame.width }
        var height: CGFloat { app.windows.firstMatch.frame.height }

        func tap(x: CGFloat, y: CGFloat) {
            let top = height - 336
            app.coordinate(withNormalizedOffset: CGVector(dx: x / width, dy: (top + y) / height)).tap()
            usleep(400_000)
        }

        /// A letter by the QWERTY key that types it.
        func letter(_ ch: Character) {
            let w = (width - 6 - 54) / 10
            if let c = Array("qwertyuiop").firstIndex(of: ch) {
                tap(x: 3 + CGFloat(c) * (w + 6) + w / 2, y: 75)
            } else if let c = Array("asdfghjkl").firstIndex(of: ch) {
                tap(x: 3 + 21 + CGFloat(c) * (w + 6) + w / 2, y: 129)
            } else if let c = Array("zxcvbnm").firstIndex(of: ch) {
                let m = (width - 6 - 90 - 30 - 36) / 7
                tap(x: 3 + 45 + 15 + CGFloat(c) * (m + 6) + m / 2, y: 183)
            }
        }

        /// Hold Space, then drag this far with the finger still down.
        func dragSpace(by dx: CGFloat) {
            let top = height - 336
            let y = (top + 237) / height
            let start = app.coordinate(withNormalizedOffset: CGVector(dx: 0.55, dy: y))
            let end = app.coordinate(withNormalizedOffset: CGVector(dx: 0.55 + dx / width, dy: y))
            start.press(forDuration: 0.9, thenDragTo: end, withVelocity: 60, thenHoldForDuration: 0.5)
            usleep(400_000)
        }

        /// Hold the letter for a while, for the screen recording.
        func hold(_ ch: Character, seconds: TimeInterval) {
            let w = (width - 6 - 54) / 10
            var at = CGPoint.zero
            if let c = Array("qwertyuiop").firstIndex(of: ch) {
                at = CGPoint(x: 3 + CGFloat(c) * (w + 6) + w / 2, y: 75)
            } else if let c = Array("asdfghjkl").firstIndex(of: ch) {
                at = CGPoint(x: 3 + 21 + CGFloat(c) * (w + 6) + w / 2, y: 129)
            }
            let top = height - 336
            app.coordinate(withNormalizedOffset: CGVector(dx: at.x / width, dy: (top + at.y) / height)).press(forDuration: seconds)
            usleep(300_000)
        }

        func type(_ letters: String) {
            letters.forEach(letter)
        }

        func delete() {
            tap(x: width - 3 - 22.5, y: 183)
        }

        func space() {
            tap(x: width / 2, y: 237)
        }

        func enter() {
            tap(x: width - 3 - 47, y: 237)
        }

        /// Of the bottom row, with 123 and no mode key: an email field's @ and .
        func emailExtra(_ i: Int) {
            tap(x: 3 + 46 + 6 + 20 + CGFloat(i) * 46, y: 237)
        }

        /// The vertical centre of the candidate bar.
        var barCenter: CGFloat { 22 }

        /// Pad key in column `c` (0-2) of row `r` (0-3 the grid, 4 the bottom row).
        func pad(_ r: Int, _ c: Int) {
            let w = (width - 6 - 12) / 3
            tap(x: 3 + CGFloat(c) * (w + 6) + w / 2, y: CGFloat(r) * 52.8 + 23.4)
        }
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

        let g = Geometry(app: app)
        // 안녕, then Space to convert.
        g.type("dkssud")
        shot("composing")
        g.space()
        sleep(1)
        shot("candidates")
        g.enter()
        sleep(1)
        shot("confirmed")

        let value = field.value as? String ?? ""
        XCTAssertTrue(value.contains("安寧"), "field holds \(value)")

        // 갈 has candidates told apart by their dictionary words.
        g.type("rkf")
        g.space()
        sleep(1)
        shot("readings")
        g.enter()
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
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("d") }
        g.emailExtra(0) // @
        g.letter("k")
        g.emailExtra(1) // .
        shot("email")
        assertText(field, "d@k.")
    }

    /// Hangul pending in one field must not be written into the next field.
    func testComposingThenFocusingAnotherFieldWritesNothingThere() throws {
        let (app, field) = launch(kind: "default", second: "email")
        let second = app.descendants(matching: .any)["trial2"]
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("r") } // ㄱ
        g.letter("k") // ㅏ
        assertText(field, "가")
        second.tap()
        sleep(2)
        shot("switched-field")
        assertText(second, "", "the composition leaked into the next field")
        g.letter("d") // typed as Latin in the email field
        assertText(second, "d")
    }

    /// Space held turns the keyboard into a trackpad that moves the caret, and
    /// what is being composed is committed first.
    func testHoldingSpaceMovesTheCaret() throws {
        let (app, field) = launch(kind: "email")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("a") }
        g.letter("b")
        assertText(field, "ab")
        g.dragSpace(by: -60)
        g.letter("c")
        sleep(1)
        shot("trackpad-moved")
        assertText(field, "cab", "the caret did not move")
        g.dragSpace(by: 16) // two steps to the right of c
        g.letter("d")
        sleep(1)
        assertText(field, "cabd", "the caret did not move back")
        app.terminate()

        let (app2, field2) = launch(kind: "default")
        let g2 = Geometry(app: app2)
        tapUntilTyped(field2, delete: { g2.delete() }) { g2.letter("r") } // ㄱ
        g2.letter("k") // ㅏ: 가 is being composed
        g2.dragSpace(by: -60)
        g2.letter("s") // ㄴ, before the committed 가
        sleep(1)
        shot("trackpad-composition")
        assertText(field2, "ㄴ가", "the composition was not committed before the caret moved")
    }

    /// A held letter shows its pop-up; the recording and these frames show it.
    /// The gaps between keys type, and a slide types where the finger lifts.
    func testKeyPressFeedback() throws {
        let (app, field) = launch(kind: "default")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("d") }
        g.hold("q", seconds: 1.5)
        g.hold("g", seconds: 1.5)
        g.hold("p", seconds: 1.5)
        g.enter() // confirm
        sleep(1)
        // A plain key first, so that a failure below is about the gaps.
        let start = text(field)
        g.letter("k")
        sleep(1)
        XCTAssertEqual(text(field).count, start.count + 1, "the key after Return typed nothing")
        g.enter()
        sleep(1)
        // ㅣ (l) itself, then between ㅓ (j) and ㅏ (k), then between two rows.
        let beforeL = text(field)
        g.letter("l")
        sleep(1)
        shot("key-l")
        XCTAssertEqual(text(field).count, beforeL.count + 1, "the ㅣ key typed nothing")
        g.enter()
        sleep(1)
        let w = (g.width - 6 - 54) / 10
        let gapX = 3 + 21 + 7 * (w + 6) - 3
        let afterKey = text(field)
        g.tap(x: gapX, y: 129)
        sleep(1)
        shot("gap-column")
        XCTAssertEqual(text(field).count, afterKey.count + 1, "a tap in the gap between two keys typed nothing")
        g.enter()
        sleep(1)
        let afterColumn = text(field)
        g.tap(x: g.width / 2, y: 102)
        sleep(1)
        shot("gap-row")
        XCTAssertEqual(text(field).count, afterColumn.count + 1, "a tap in the gap between two rows typed nothing")
        g.enter()
    }

    /// 123 changes the keys as the finger lands; a slide on to a symbol types it
    /// and returns to the letters; Space after a symbol returns too.
    func testSymbolLayerSlideAndSpace() throws {
        let (app, field) = launch(kind: "email")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("a") }
        g.delete()
        assertText(field, "")
        let layer = CGPoint(x: 26, y: 237)
        let digit = CGPoint(x: 20, y: 75)
        let top = g.height - 336
        func point(_ p: CGPoint) -> XCUICoordinate {
            app.coordinate(withNormalizedOffset: CGVector(dx: p.x / g.width, dy: (top + p.y) / g.height))
        }
        point(layer).press(forDuration: 0.3, thenDragTo: point(digit))
        assertText(field, "1", "a slide from 123 typed no digit")
        g.letter("q") // on the letters again
        assertText(field, "1q", "the slide did not return to the letters")
        g.tap(x: layer.x, y: layer.y)
        g.letter("w") // 2
        assertText(field, "1q2")
        g.space()
        g.letter("q")
        assertText(field, "1q2 q", "Space did not return to the letters")
    }

    /// Holding the 釋 key offers to forget what was learned, in the bar.
    func testHoldingTheModeKeyShowsLearnControls() throws {
        let (app, field) = launch(kind: "default")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("d") }
        g.delete()
        sleep(1)
        let top = g.height - 336
        app.coordinate(withNormalizedOffset: CGVector(dx: 78 / g.width, dy: (top + 237) / g.height)).press(forDuration: 1)
        sleep(1)
        shot("learn-controls")
    }

    /// The candidate bar, a held key and the drawer. The workflow runs this again
    /// in dark mode.
    func testDarkScreens() throws {
        let (app, field) = launch(kind: "default")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("d") }
        g.type("kssud")
        sleep(1)
        shot("look-composing")
        g.space()
        sleep(1)
        shot("look-candidates")
        g.tap(x: g.width - 25, y: g.barCenter)
        sleep(1)
        shot("look-drawer")
        g.tap(x: g.width - 22, y: 22)
        sleep(1)
        g.enter()
        g.hold("g", seconds: 1.5)
        g.enter()
    }

    /// The chevron opens every candidate over the keys, and a tap on one
    /// confirms it.
    func testDrawerOpensAndPicks() throws {
        let (app, field) = launch(kind: "default")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("d") }
        g.type("kssud")
        g.space()
        sleep(1)
        g.tap(x: g.width - 25, y: g.barCenter)
        usleep(100_000)
        shot("drawer-opening")
        sleep(1)
        shot("drawer")
        g.tap(x: g.width - 22, y: 22) // closes
        sleep(1)
        shot("drawer-closed")
        shot("drawer-reopened")
        g.tap(x: g.width - 25, y: g.barCenter)
        sleep(1)
        g.tap(x: 40, y: 66) // the grid's second row, first candidate
        sleep(1)
        shot("drawer-picked")
        let value = text(field)
        XCTAssertFalse(value.isEmpty)
        XCTAssertNotEqual(value, "安寧", "the first candidate came out, not the tapped one")
        XCTAssertNotEqual(value, "안녕")
    }

    /// Two default fields: Hangul pending in the first must stay there, and
    /// the keys tapped right after focusing the second compose in it with
    /// nothing lost.
    func testSwitchingBetweenSameLayoutFieldsKeepsEachCompositionApart() throws {
        let (app, field) = launch(kind: "default", second: "default")
        let second = app.descendants(matching: .any)["trial2"]
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("r") } // ㄱ
        g.letter("k") // ㅏ
        assertText(field, "가")
        second.tap()
        g.letter("s") // ㄴ, at once
        g.letter("k") // ㅏ
        sleep(1)
        shot("switched-same-layout")
        assertText(field, "가", "the first field changed")
        assertText(second, "나", "the second field lost or gained text")
    }

    /// A longer composition shows its segments in the drawer, and the − and +
    /// beside the grid change the focused one.
    func testDrawerResizesSegments() throws {
        let (app, field) = launch(kind: "default")
        let g = Geometry(app: app)
        tapUntilTyped(field, delete: { g.delete() }) { g.letter("d") }
        g.type("kssudgktpdy")
        g.space()
        sleep(1)
        let converted = text(field)
        g.tap(x: g.width - 25, y: g.barCenter)
        sleep(1)
        shot("drawer-segments")
        g.tap(x: g.width - 22, y: 66) // shorter
        sleep(1)
        shot("drawer-segments-shorter")
        XCTAssertNotEqual(text(field), "", "resizing lost the composition")
        g.tap(x: g.width - 22, y: 110) // longer
        sleep(1)
        assertText(field, converted, "lengthening did not undo shortening")
        g.tap(x: g.width - 22, y: 22) // closes
        sleep(1)
        g.enter()
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

    // MARK: The app

    /// Every screen of the app in English and Korean. The workflow runs this
    /// again in dark mode.
    func testSettingsScreens() throws {
        do {
            for language in ["en", "ko"] {
                let app = XCUIApplication()
                let name = "settings-\(language)"
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
}
