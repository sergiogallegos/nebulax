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
        NSColor(calibratedRed: 0.045, green: 0.063, blue: 0.087, alpha: 1).setFill()
        bounds.fill()
        guard let lease = frameLease, let context = NSGraphicsContext.current?.cgContext else { return }
        let snapshot = lease.view
        context.saveGState()
        context.textMatrix = .identity
        let foreground = NSColor(calibratedRed: 0.85, green: 0.89, blue: 0.94, alpha: 1)
        let attrs: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: foreground]
        for row in 0..<Int(snapshot.lines) {
            let bottom = bounds.height - inset - CGFloat(row + 1) * cellHeight
            if bottom + cellHeight < 0 { break }
            for column in 0..<Int(snapshot.columns) {
                let cell = snapshot.cells![row * Int(snapshot.columns) + column]
                if cell.kind != 1 || cell.text_len == 0 { continue }
                let left = inset + CGFloat(column) * cellWidth
                if left >= bounds.width { break }
                let text = String(decoding: UnsafeBufferPointer(start: snapshot.text!.advanced(by: Int(cell.text_offset)), count: Int(cell.text_len)), as: UTF8.self)
                let line = CTLineCreateWithAttributedString(NSAttributedString(string: text, attributes: attrs))
                context.saveGState()
                context.clip(to: CGRect(x: left, y: bottom, width: cellWidth * CGFloat(cell.width), height: cellHeight))
                context.textPosition = CGPoint(x: left, y: bottom + 5)
                CTLineDraw(line, context)
                context.restoreGState()
            }
        }
        let cursor = CGRect(x: inset + CGFloat(snapshot.cursor_column) * cellWidth,
                            y: bounds.height - inset - CGFloat(snapshot.cursor_row + 1) * cellHeight,
                            width: cellWidth, height: 2)
        context.setFillColor(NSColor(calibratedRed: 0.40, green: 0.83, blue: 0.73, alpha: 1).cgColor)
        context.fill(cursor)
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
    var inputMessage: String?
    let arguments = CommandLine.arguments
    var testing: Bool { arguments.contains("--self-test") }
    var shell: Bool { arguments.contains("--shell") }
    func option(_ name: String) -> String? { guard let i = arguments.firstIndex(of: name), arguments.indices.contains(i+1) else { return nil }; return arguments[i+1] }
    func applicationDidFinishLaunching(_ notification: Notification) {
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
        } else if testStage == 3 && text.contains("PROTOCOL OK") {
            precondition(text.contains("APP KEY Up") && text.contains("FIXED TOP") && text.contains("FIXED BOTTOM"))
            window.setContentSize(NSSize(width: 948, height: 630))
            testStage = 4
        } else if testStage == 4 && frame.view.columns == 90 && frame.view.lines == 25 {
            precondition(text.contains("Key: Up") && text.contains("You typed: Rust 界"))
            precondition(text.contains("PROTOCOL OK") && text.contains("APP KEY Up") && text.contains("REGION OK"))
            precondition(text.contains("FIXED TOP") && text.contains("FIXED BOTTOM"))
            grid.displayIfNeeded()
            precondition(grid.paintCount > 0)
            renderedText = text
            let output = option("--output")!
            let bitmap = grid.bitmapImageRepForCachingDisplay(in: grid.bounds)!
            grid.cacheDisplay(in: grid.bounds, to: bitmap)
            try! bitmap.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: output).appendingPathComponent("window.png"))
            testStage = 5
            window.performClose(nil)
        }
    }
    func recordSuccess() {
        let result: [String: Any] = ["success": true, "native_key_events": 6, "resize_columns": 90, "resize_lines": 25,
                                  "paint_count": grid.paintCount, "close_heartbeat_progress": heartbeat - closeHeartbeat,
                                  "typed_text_seen": renderedText.contains("You typed: Rust 界"), "arrow_seen": renderedText.contains("Key: Up"),
                                  "application_cursor_reply_and_region_seen": renderedText.contains("PROTOCOL OK") && renderedText.contains("APP KEY Up") && renderedText.contains("REGION OK"),
                                  "outside_region_rows_preserved": renderedText.contains("FIXED TOP") && renderedText.contains("FIXED BOTTOM"),
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
