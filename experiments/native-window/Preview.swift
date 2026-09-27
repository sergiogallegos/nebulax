import AppKit
import CoreText
import Darwin

@MainActor
final class FrameLease {
    let handle: UInt64
    let view: NbFrame
    init(_ handle: UInt64) {
        self.handle = handle
        var value = NbFrame()
        precondition(nb_frame_view(handle, &value) == NB_OK)
        view = value
    }
    func release() { precondition(nb_frame_release(handle) == NB_OK) }
    var text: String { String(decoding: UnsafeBufferPointer(start: view.text, count: view.text_len), as: UTF8.self) }
}

@MainActor
final class GridView: NSView {
    var frameLease: FrameLease?
    var submitText: ((String) -> Void)?
    var submitKey: ((UInt32) -> Void)?
    var geometryChanged: ((UInt32, UInt32) -> Void)?
    let cellWidth: CGFloat = 10
    let cellHeight: CGFloat = 22
    let inset: CGFloat = 24
    let font = NSFont.monospacedSystemFont(ofSize: 15, weight: .regular)
    let defaultForeground = NSColor(srgbRed: 0.85, green: 0.89, blue: 0.94, alpha: 1)
    let defaultBackground = NSColor(srgbRed: 0.045, green: 0.063, blue: 0.087, alpha: 1)
    lazy var fonts: [NSFont] = [font,
        NSFontManager.shared.convert(font, toHaveTrait: .boldFontMask),
        NSFontManager.shared.convert(font, toHaveTrait: .italicFontMask),
        NSFontManager.shared.convert(font, toHaveTrait: [.boldFontMask, .italicFontMask])]
    func color(_ encoded: UInt32, fallback: NSColor) -> NSColor {
        if encoded == 0 { return fallback }
        let rgb: UInt32
        if encoded >> 24 == 2 { rgb = encoded & 0xffffff }
        else {
            let index = Int(encoded & 255)
            let basic: [UInt32] = [0x18212b, 0xe06c75, 0x98c379, 0xe5c07b, 0x61afef, 0xc678dd, 0x56b6c2, 0xabb2bf,
                                   0x5c6370, 0xff8992, 0xb5e890, 0xffd68f, 0x85caff, 0xe5a0ff, 0x7edce8, 0xf2f5f9]
            if index < 16 { rgb = basic[index] }
            else if index < 232 {
                let n = index - 16
                let levels: [UInt32] = [0, 95, 135, 175, 215, 255]
                rgb = levels[n / 36] << 16 | levels[(n / 6) % 6] << 8 | levels[n % 6]
            } else { let v = UInt32(8 + 10 * (index - 232)); rgb = v << 16 | v << 8 | v }
        }
        return NSColor(srgbRed: CGFloat((rgb >> 16) & 255) / 255, green: CGFloat((rgb >> 8) & 255) / 255,
                       blue: CGFloat(rgb & 255) / 255, alpha: 1)
    }
    func colors(_ style: NbStyle) -> (NSColor, NSColor) {
        var fg = color(style.foreground, fallback: defaultForeground)
        var bg = color(style.background, fallback: defaultBackground)
        if style.attributes & 16 != 0 { swap(&fg, &bg) }
        if style.attributes & 2 != 0 { fg = fg.blended(withFraction: 0.5, of: bg)! }
        return (fg, bg)
    }
    var paintCount = 0
    override var acceptsFirstResponder: Bool { true }
    override var isOpaque: Bool { true }
    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        let columns = max(2, min(512, Int((newSize.width - 2 * inset) / cellWidth)))
        let lines = max(1, min(128, Int((newSize.height - 2 * inset) / cellHeight)))
        geometryChanged?(UInt32(columns), UInt32(lines))
        needsDisplay = true
    }
    func display(_ next: FrameLease) {
        let old = frameLease
        frameLease = next
        // Full redraw for this prototype. No second mutable grid or cached text.
        needsDisplay = true
        old?.release()
    }
    func clearFrame() { frameLease?.release(); frameLease = nil }
    override func draw(_ dirtyRect: NSRect) {
        defaultBackground.setFill()
        bounds.fill()
        guard let lease = frameLease, let context = NSGraphicsContext.current?.cgContext else { return }
        let snapshot = lease.view
        context.saveGState()
        context.textMatrix = .identity
        // Paint every background first: a wide continuation must not cover its glyph.
        for row in 0..<Int(snapshot.lines) {
            let bottom = bounds.height - inset - CGFloat(row + 1) * cellHeight
            for column in 0..<Int(snapshot.columns) {
                let cell = snapshot.cells![row * Int(snapshot.columns) + column]
                let style = snapshot.styles![Int(cell.style_id)]
                context.setFillColor(colors(style).1.cgColor)
                context.fill(CGRect(x: inset + CGFloat(column) * cellWidth, y: bottom, width: cellWidth, height: cellHeight))
            }
        }
        for row in 0..<Int(snapshot.lines) {
            let bottom = bounds.height - inset - CGFloat(row + 1) * cellHeight
            if bottom + cellHeight < 0 { break }
            for column in 0..<Int(snapshot.columns) {
                let cell = snapshot.cells![row * Int(snapshot.columns) + column]
                if cell.kind != 1 || cell.text_len == 0 { continue }
                let style = snapshot.styles![Int(cell.style_id)]
                if style.attributes & 32 != 0 { continue }
                let left = inset + CGFloat(column) * cellWidth
                if left >= bounds.width { break }
                let foreground = colors(style).0
                let fontIndex = (style.attributes & 1 != 0 ? 1 : 0) + (style.attributes & 4 != 0 ? 2 : 0)
                let attrs: [NSAttributedString.Key: Any] = [.font: fonts[fontIndex], .foregroundColor: foreground]
                let text = String(decoding: UnsafeBufferPointer(start: snapshot.text!.advanced(by: Int(cell.text_offset)), count: Int(cell.text_len)), as: UTF8.self)
                let line = CTLineCreateWithAttributedString(NSAttributedString(string: text, attributes: attrs))
                context.saveGState()
                let width = cellWidth * CGFloat(cell.width)
                context.clip(to: CGRect(x: left, y: bottom, width: width, height: cellHeight))
                context.textPosition = CGPoint(x: left, y: bottom + 5)
                CTLineDraw(line, context)
                context.setFillColor(foreground.cgColor)
                if style.attributes & (8 | 128) != 0 { context.fill(CGRect(x: left, y: bottom + 3, width: width, height: 1)) }
                if style.attributes & 128 != 0 { context.fill(CGRect(x: left, y: bottom + 1, width: width, height: 1)) }
                if style.attributes & 64 != 0 { context.fill(CGRect(x: left, y: bottom + 10, width: width, height: 1)) }
                context.restoreGState()
            }
        }
        let cursor = CGRect(x: inset + CGFloat(snapshot.cursor_column) * cellWidth,
                            y: bounds.height - inset - CGFloat(snapshot.cursor_row + 1) * cellHeight,
                            width: cellWidth, height: 2)
        context.setFillColor(NSColor(calibratedRed: 0.40, green: 0.83, blue: 0.73, alpha: 1).cgColor)
        if snapshot.cursor_visible != 0 { context.fill(cursor) }
        context.restoreGState()
        paintCount += 1
    }
    override func keyDown(with event: NSEvent) {
        if event.modifierFlags.contains(.command) { super.keyDown(with: event); return }
        let keys: [UInt16: UInt32] = [36: 1, 76: 1, 51: 2, 48: 3, 53: 4, 126: 5, 125: 6, 123: 7, 124: 8, 115: 9, 119: 10, 117: 11]
        if let key = keys[event.keyCode] { submitKey?(key); return }
        if event.modifierFlags.contains(.control), let c = event.charactersIgnoringModifiers?.unicodeScalars.first, c.value < 128 {
            let value = c.value
            if (64...127).contains(value) || value == 32 { submitKey?(32 + (value & 31)); return }
        }
        if let text = event.characters, !text.isEmpty, !text.unicodeScalars.contains(where: { CharacterSet.controlCharacters.contains($0) }) { submitText?(text) }
    }
}

@MainActor
final class Preview: NSObject, NSApplicationDelegate, NSWindowDelegate {
    let grid = GridView()
    let status = NSTextField(labelWithString: "Starting…")
    var window: NSWindow!
    var timer: Timer?
    var session: UInt64 = 0
    var closing = false
    var terminating = false
    var lastGeometry: (UInt32, UInt32) = (80, 24)
    var lastGeneration: UInt64 = 0
    var testStage = 0
    var testDeadline = Date().addingTimeInterval(12)
    var heartbeat = 0
    var closeHeartbeat = 0
    var renderedText = ""
    var completedTest = false
    var hiddenBitmap: NSBitmapImageRep?
    var hiddenVersions: [UInt64] = []
    var hiddenCursor: (UInt32, UInt32) = (0, 0)
    var cursorPixelChanges = 0
    var inputMessage: String?
    let arguments = CommandLine.arguments
    var testing: Bool { arguments.contains("--self-test") }
    var shell: Bool { arguments.contains("--shell") }
    func option(_ name: String) -> String? { guard let i = arguments.firstIndex(of: name), arguments.indices.contains(i+1) else { return nil }; return arguments[i+1] }
    func applicationDidFinishLaunching(_ notification: Notification) {
        precondition(nb_abi_version() == 3)
        NSApp.setActivationPolicy(.regular)
        let menu = NSMenu()
        let item = NSMenuItem()
        menu.addItem(item)
        let submenu = NSMenu()
        submenu.addItem(withTitle: "Quit Nebulax Preview", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        item.submenu = submenu
        NSApp.mainMenu = menu
        window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 848, height: 608), styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
        window.title = "Nebulax Preview"
        window.minSize = NSSize(width: 480, height: 350)
        window.isReleasedWhenClosed = false
        window.delegate = self
        window.appearance = NSAppearance(named: .darkAqua)
        let root = NSView(frame: NSRect(x: 0, y: 0, width: 848, height: 608))
        grid.frame = NSRect(x: 0, y: 32, width: 848, height: 576)
        grid.autoresizingMask = [.width, .height]
        status.frame = NSRect(x: 24, y: 7, width: 800, height: 18)
        status.autoresizingMask = [.width]
        status.font = .systemFont(ofSize: 11)
        status.textColor = .secondaryLabelColor
        root.addSubview(grid); root.addSubview(status)
        window.contentView = root
        let python = option("--python")!
        let peer = option("--peer")!
        if shell {
            start("/usr/bin/env", ["-i", "PATH=/usr/bin:/bin", "TERM=dumb", "HISTFILE=/dev/null", "PS1=nx> ", "/bin/bash", "--noprofile", "--norc", "-i"])
        } else { start(python, [peer] + (testing ? ["--test"] : [])) }
        grid.submitText = { [weak self] text in self?.textInput(text) }
        grid.submitKey = { [weak self] key in self?.keyInput(key) }
        grid.geometryChanged = { [weak self] columns, lines in self?.resize(columns, lines) }
        grid.setFrameSize(grid.frame.size)
        window.center(); window.makeKeyAndOrderFront(nil)
        window.makeFirstResponder(grid)
        NSApp.activate(ignoringOtherApps: true)
        timer = Timer.scheduledTimer(withTimeInterval: 1.0/60, repeats: true) { [weak self] _ in
            MainActor.assumeIsolated { self?.tick() }
        }
        RunLoop.main.add(timer!, forMode: .eventTracking)
    }
    func start(_ executable: String, _ args: [String]) {
        let storage = ([executable] + args).map { strdup($0)! }
        defer { storage.forEach { free($0) } }
        let bytes = storage.map { NbBytes(data: UnsafeRawPointer($0).assumingMemoryBound(to: UInt8.self), len: strlen($0)) }
        let result = Array(bytes.dropFirst()).withUnsafeBufferPointer { args in nb_session_start(bytes[0], args.baseAddress, args.count, 80, 24, &session) }
        precondition(result == NB_OK)
    }
    func resize(_ columns: UInt32, _ lines: UInt32) {
        if closing || session == 0 || (columns == lastGeometry.0 && lines == lastGeometry.1) { return }
        let result = nb_session_resize(session, columns, lines)
        if result == NB_OK { lastGeometry = (columns, lines) }
    }
    func inputResult(_ result: Int32) {
        if result != NB_OK {
            inputMessage = result == NB_LIMIT ? "Input busy — keystroke not sent" : "Session is not accepting input"
            status.stringValue = inputMessage!; NSSound.beep()
        } else { inputMessage = nil }
    }
    func textInput(_ text: String) {
        guard !closing, session != 0 else { return }
        let bytes = Array(text.utf8)
        // No native staging queue. One commit is accepted in full or rejected.
        guard bytes.count <= 4096 else { inputResult(Int32(NB_LIMIT)); return }
        let result = bytes.withUnsafeBufferPointer { nb_session_text(session, NbBytes(data: $0.baseAddress, len: $0.count)) }
        inputResult(result)
    }
    func keyInput(_ key: UInt32) {
        guard !closing, session != 0 else { return }
        inputResult(nb_session_key(session, key))
    }
    func tick() {
        heartbeat += 1
        if testing && Date() > testDeadline { fail("native test deadline exceeded") }
        guard session != 0 else { return }
        var current = NbStatus()
        precondition(nb_session_status(session, &current) == NB_OK)
        if !closing {
            var frame: UInt64 = 0
            let result = nb_frame_acquire(session, lastGeneration, &frame)
            if result == NB_OK {
                let lease = FrameLease(frame)
                lastGeneration = lease.view.generation
                grid.display(lease)
                status.stringValue = inputMessage ?? "\(shell ? "Shell preview" : "Demo session")   ·   \(lease.view.columns) × \(lease.view.lines)   ·   Type and press Return"
            } else { precondition(result == NB_NO_FRAME) }
        }
        if testing && !closing { testStep() }
        if current.finished != 0 {
            if closing {
                precondition(nb_session_release(session) == NB_OK)
                session = 0
                grid.clearFrame()
                timer?.invalidate()
                if testing {
                    precondition(current.phase == NB_STOPPED)
                    precondition(heartbeat > closeHeartbeat)
                    recordSuccess()
                }
                window.close()
                if terminating { NSApp.reply(toApplicationShouldTerminate: true) }
                else { NSApp.terminate(nil) }
            } else {
                status.stringValue = current.phase == NB_SESSION_FAILED ? "Session failed (\(current.failure))" : "Session ended"
            }
        }
    }
    func requestClose() {
        guard !closing else { return }
        closing = true
        closeHeartbeat = heartbeat
        status.stringValue = "Closing session…"
        precondition(nb_session_close(session) == NB_OK)
    }
    func windowShouldClose(_ sender: NSWindow) -> Bool {
        if session == 0 { return true }
        requestClose(); return false
    }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        if session == 0 { return .terminateNow }
        terminating = true; requestClose(); return .terminateLater
    }
    func event(_ characters: String, code: UInt16) {
        let event = NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: [], timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber, context: nil, characters: characters, charactersIgnoringModifiers: characters, isARepeat: false, keyCode: code)!
        window.sendEvent(event)
    }
    func testStep() {
        guard let frame = grid.frameLease else { return }
        let text = frame.text
        if testStage == 0 && text.contains("does not execute commands") {
            event("Rust", code: 0); event("\u{7f}", code: 51); event("t 界", code: 0); event("\r", code: 36)
            testStage = 1
        } else if testStage == 1 && text.contains("You typed: Rust 界") {
            event("\u{f700}", code: 126)
            testStage = 2
        } else if testStage == 2 && text.contains("APP READY") && text.contains("REGION OK") {
            event("\u{f700}", code: 126)
            testStage = 3
        } else if testStage == 3 && text.contains("STYLE OK") && frame.view.cursor_visible == 0 {
            precondition(text.contains("APP KEY Up") && text.contains("FIXED TOP") && text.contains("FIXED BOTTOM"))
            window.setContentSize(NSSize(width: 948, height: 630))
            testStage = 4
        } else if testStage == 4 && frame.view.columns == 90 && frame.view.lines == 25 {
            precondition(text.contains("Key: Up") && text.contains("You typed: Rust 界"))
            precondition(text.contains("PROTOCOL OK") && text.contains("APP KEY Up") && text.contains("REGION OK"))
            precondition(text.contains("FIXED TOP") && text.contains("FIXED BOTTOM"))
            precondition(text.contains("STYLE OK"))
            precondition(text.contains("EDITOK") && !text.contains("INSERTED"))
            precondition(frame.view.cells![15 * 90 + 4].kind == 0)
            let edited = frame.view.cells![15 * 90 + 5]
            precondition(edited.text_len == 1 && frame.view.text![Int(edited.text_offset)] == 79)
            let tabbed = frame.view.cells![20 * 90 + 8]
            precondition(tabbed.text_len == 1 && frame.view.text![Int(tabbed.text_offset)] == 79)
            let countedTab = frame.view.cells![20 * 90 + 16]
            precondition(countedTab.text_len == 1 && frame.view.text![Int(countedTab.text_offset)] == 33)
            precondition(frame.view.cells![20 * 90 + 4].kind == 0)
            let styled = frame.view.cells![22 * 90]
            let rendition = frame.view.styles![Int(styled.style_id)]
            precondition(rendition.foreground == 0x020c2238 && rendition.background == 0x010000e6 && rendition.attributes == 13)
            precondition(frame.view.cells![22 * 90 + 1].style_id == styled.style_id)
            let erased = frame.view.cells![22 * 90 + 60]
            let eraseStyle = frame.view.styles![Int(erased.style_id)]
            precondition(erased.kind == 0 && eraseStyle.background == 0x01000011 && eraseStyle.foreground == 0 && eraseStyle.attributes == 0)
            precondition(frame.view.cursor_visible == 0)
            let edge = frame.view.cells![18 * 90 + 79]
            precondition(edge.text_len == 1 && frame.view.text![Int(edge.text_offset)] == 68)
            precondition(frame.view.row_wraps![18] == 0)
            grid.displayIfNeeded()
            precondition(grid.paintCount > 0)
            renderedText = text
            let output = option("--output")!
            let bitmap = grid.bitmapImageRepForCachingDisplay(in: grid.bounds)!
            grid.cacheDisplay(in: grid.bounds, to: bitmap)
            try! bitmap.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: output).appendingPathComponent("hidden.png"))
            hiddenBitmap = bitmap
            hiddenVersions = Array(UnsafeBufferPointer(start: frame.view.row_versions, count: frame.view.row_count))
            hiddenCursor = (frame.view.cursor_row, frame.view.cursor_column)
            testStage = 5
            event("v", code: 9)
        } else if testStage == 5 && frame.view.cursor_visible == 1 {
            precondition(text == renderedText && frame.view.cursor_row == hiddenCursor.0 && frame.view.cursor_column == hiddenCursor.1)
            precondition(Array(UnsafeBufferPointer(start: frame.view.row_versions, count: frame.view.row_count)) == hiddenVersions)
            grid.displayIfNeeded()
            let bitmap = grid.bitmapImageRepForCachingDisplay(in: grid.bounds)!
            grid.cacheDisplay(in: grid.bounds, to: bitmap)
            let old = hiddenBitmap!
            precondition(old.pixelsWide == bitmap.pixelsWide && old.pixelsHigh == bitmap.pixelsHigh)
            // Inspect rendered pixels, not a drawing counter. Only the cursor's
            // cell may differ after a visibility-only frame with identical rows.
            let sx = CGFloat(bitmap.pixelsWide) / grid.bounds.width
            let sy = CGFloat(bitmap.pixelsHigh) / grid.bounds.height
            let left = Int((grid.inset + CGFloat(hiddenCursor.1) * grid.cellWidth) * sx)
            let top = Int((grid.inset + CGFloat(hiddenCursor.0) * grid.cellHeight) * sy)
            for y in 0..<bitmap.pixelsHigh {
                for x in 0..<bitmap.pixelsWide {
                    if old.colorAt(x: x, y: y) != bitmap.colorAt(x: x, y: y) {
                        precondition(x >= left && x < left + Int(grid.cellWidth * sx) && y >= top && y < top + Int(grid.cellHeight * sy))
                        cursorPixelChanges += 1
                    }
                }
            }
            precondition(cursorPixelChanges > 0)
            try! bitmap.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: option("--output")!).appendingPathComponent("window.png"))
            hiddenBitmap = nil
            testStage = 6
            window.performClose(nil)
        }
    }
    func recordSuccess() {
        let result: [String: Any] = ["success": true, "native_key_events": 7, "resize_columns": 90, "resize_lines": 25,
                                  "paint_count": grid.paintCount, "close_heartbeat_progress": heartbeat - closeHeartbeat,
                                  "typed_text_seen": renderedText.contains("You typed: Rust 界"), "arrow_seen": renderedText.contains("Key: Up"),
                                  "application_cursor_reply_and_region_seen": renderedText.contains("PROTOCOL OK") && renderedText.contains("APP KEY Up") && renderedText.contains("REGION OK"),
                                  "outside_region_rows_preserved": renderedText.contains("FIXED TOP") && renderedText.contains("FIXED BOTTOM"),
                                  "character_and_region_line_edits_verified": true, "tabs_and_full_line_erase_verified": true, "styled_wide_cell_and_erased_background_verified": true, "abi_version": nb_abi_version(),
                                  "autowrap_disabled_edge_verified": true, "visibility_only_cursor_pixels_changed": cursorPixelChanges,
                                  "child_reaped_before_window_close": true]
        try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys]).write(to: URL(fileURLWithPath: option("--output")!).appendingPathComponent("window.json"))
        completedTest = true
        print("AppKit: drawing, native key events, resize and asynchronous close passed")
    }
    func fail(_ message: String) -> Never { fputs(message + "\n", stderr); exit(1) }
}

let app = NSApplication.shared
let preview = Preview()
app.delegate = preview
app.run()
