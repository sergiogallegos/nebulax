import Foundation
import CoreText
import ImageIO
import CryptoKit
import Darwin

var checks = 0
@MainActor func check(_ condition: @autoclosure () -> Bool, _ message: String) {
    precondition(condition(), message); checks += 1
}
@MainActor func expect(_ error: TextError, _ body: () throws -> Void) {
    do { try body(); preconditionFailure("expected \(error)") }
    catch let actual as TextError { check(actual == error, "wrong error: \(actual)") }
    catch { preconditionFailure("unexpected error: \(error)") }
}
@MainActor func start(_ executable: String, _ peer: String) -> UInt64 {
    let storage = [executable, peer].map { strdup($0)! }
    defer { storage.forEach { free($0) } }
    let bytes = storage.map { NbBytes(data: UnsafeRawPointer($0).assumingMemoryBound(to: UInt8.self), len: strlen($0)) }
    var session: UInt64 = 0
    check(nb_session_start(bytes[0], [bytes[1]], 1, 80, 12, &session) == NB_OK, "start")
    return session
}
func glyphJSON(_ shape: ShapedText) -> [[String: Any]] {
    shape.glyphs.map { ["id": Int($0.id), "font": $0.font, "utf16": [$0.utf16.lowerBound, $0.utf16.upperBound],
                       "columns": [$0.columns.lowerBound, $0.columns.upperBound],
                       "position": [$0.position.x, $0.position.y], "advance": [$0.advance.width, $0.advance.height]] }
}
func signature(_ shape: ShapedText) -> [String] {
    shape.glyphs.map { "\(CTFontCopyPostScriptName(shape.fonts[$0.font])):\($0.id):\($0.position):\($0.advance)" }
}
var fontHashes: [URL: String] = [:]
@MainActor func fontJSON(_ font: CTFont) -> [String: Any] {
    let url = CTFontCopyAttribute(font, kCTFontURLAttribute) as? URL
    if let url, fontHashes[url] == nil, let stream = InputStream(url: url) {
        stream.open()
        defer { stream.close() }
        var hash = SHA256()
        var buffer = [UInt8](repeating: 0, count: 65536)
        while true {
            let count = stream.read(&buffer, maxLength: buffer.count)
            precondition(count >= 0, "font hash read")
            if count == 0 { break }
            hash.update(data: Data(buffer.prefix(count)))
        }
        fontHashes[url] = hash.finalize().map { String(format: "%02x", $0) }.joined()
    }
    // Basenames identify actual faces without recording a user's home path.
    return ["postscript_name": CTFontCopyPostScriptName(font) as String,
            "file": url?.lastPathComponent ?? "unavailable",
            "version": CTFontCopyName(font, kCTFontVersionNameKey) as String? ?? "unavailable",
            "file_sha256": url.flatMap { fontHashes[$0] } ?? "unavailable"]
}

let output = URL(fileURLWithPath: CommandLine.arguments[3], isDirectory: true)
check(nb_abi_version() == 3, "private ABI")
let session = start(CommandLine.arguments[1], CommandLine.arguments[2])
var status = NbStatus()
let deadline = Date().addingTimeInterval(8)
repeat {
    precondition(Date() < deadline, "PTY deadline")
    precondition(nb_session_status(session, &status) == NB_OK, "status")
    if status.finished == 0 { Thread.sleep(forTimeInterval: 0.001) }
} while status.finished == 0
check(status.phase == NB_EXITED && status.exit_code == 0, "peer exit")
var handle: UInt64 = 0
check(nb_frame_acquire(session, 0, &handle) == NB_OK, "acquire")
var frame = NbFrame()
check(nb_frame_view(handle, &frame) == NB_OK, "view")
let plan = try TextPlan.capture(frame)
check(frame.cells![6 * 80 + 79].kind == 3 && frame.cells![7 * 80].width == 2, "wide edge wraps")
check(plan.runs.filter { $0.row == 3 }.count == 3, "color boundary")
check(plan.runs.filter { $0.row == 4 }.count == 2, "italic boundary")
check(plan.runs.filter { $0.row == 5 }.map { $0.columns.lowerBound } == [0, 4], "empty gap boundary")
check(plan.runs.filter { $0.row == 6 }.allSatisfy { $0.columns.upperBound == 79 }, "padding excluded")
let unicode = plan.runs.first { $0.row == 2 }!
check(Array(unicode.text.utf8) == Array("e\u{301} A\u{308} 界 👩🏽‍💻 🇺🇸 1️⃣".utf8), "exact Unicode bytes")
check(unicode.owners.map { $0.column } == [0, 1, 2, 3, 4, 6, 7, 9, 10, 12, 13], "engine columns")
check(unicode.owners.map { $0.width } == [1, 1, 1, 1, 2, 1, 2, 1, 2, 1, 2], "engine widths")
check(unicode.owners.map { $0.utf16.count } == [2, 1, 2, 1, 1, 1, 7, 1, 4, 1, 3], "UTF16 owners including surrogates")
let longCluster = plan.runs.first { $0.row == 9 }!
check(longCluster.owners.count == 1 && longCluster.text.utf16.count == 21, "long combining owner")
let sourceBytes = Array(UnsafeBufferPointer(start: frame.text, count: frame.text_len))
for run in plan.runs {
    for owner in run.owners {
        let range = NSRange(location: owner.utf16.lowerBound, length: owner.utf16.count)
        let copied = (run.text as NSString).substring(with: range)
        check(Array(copied.utf8) == Array(sourceBytes[owner.sourceUTF8]), "owner bytes survive UTF16 conversion exactly")
    }
}

var limits = TextLimits()
limits.runOwners = 1
let singleOwner = try TextPlan.capture(frame, limits: limits)
check(singleOwner.runs.allSatisfy { $0.owners.count == 1 }, "bounded owner splits")
limits = TextLimits(); limits.runUTF8 = 64
let byteSplit = try TextPlan.capture(frame, limits: limits)
check(byteSplit.runs.allSatisfy { $0.text.utf8.count <= 64 }, "bounded byte splits")
limits = TextLimits(); limits.runUTF16 = 32
let unitSplit = try TextPlan.capture(frame, limits: limits)
check(unitSplit.runs.allSatisfy { $0.text.utf16.count <= 32 }, "bounded UTF16 splits")
limits = TextLimits(); limits.runUTF8 = 1
expect(.limit) { _ = try TextPlan.capture(frame, limits: limits) }
limits = TextLimits(); limits.runUTF16 = 1
expect(.limit) { _ = try TextPlan.capture(frame, limits: limits) }
limits = TextLimits(); limits.planBytes = plan.payloadBytes
let exactPlan = try TextPlan.capture(frame, limits: limits)
check(exactPlan.payloadBytes == plan.payloadBytes, "exact plan cap")
limits.planBytes -= 1
expect(.limit) { _ = try TextPlan.capture(frame, limits: limits) }
var bad = frame; bad.columns = UInt32.max
expect(.invalidFrame) { _ = try TextPlan.capture(bad) }
bad = frame; bad.cell_count -= 1
expect(.invalidFrame) { _ = try TextPlan.capture(bad) }
bad = frame; bad.text_len = Int.max
expect(.invalidFrame) { _ = try TextPlan.capture(bad) }
bad = frame; bad.styles = nil
expect(.invalidFrame) { _ = try TextPlan.capture(bad) }
// Corrupt copied wire buffers, never the live immutable Rust allocation.
let sourceCells = Array(UnsafeBufferPointer(start: frame.cells, count: frame.cell_count))
@MainActor func malformed(_ cells: [NbCell], _ text: [UInt8]) {
    cells.withUnsafeBufferPointer { cellBuffer in
        text.withUnsafeBufferPointer { textBuffer in
            var malformed = frame
            malformed.cells = cellBuffer.baseAddress; malformed.text = textBuffer.baseAddress
            expect(.invalidFrame) { _ = try TextPlan.capture(malformed) }
        }
    }
}
var invalidCells = sourceCells; invalidCells[0].kind = 2
malformed(invalidCells, sourceBytes)
invalidCells = sourceCells; invalidCells[0].text_offset = UInt32.max
malformed(invalidCells, sourceBytes)
invalidCells = sourceCells; invalidCells[0].text_len = UInt16.max
malformed(invalidCells, sourceBytes)
invalidCells = sourceCells; invalidCells[0].style_id = UInt16.max
malformed(invalidCells, sourceBytes)
invalidCells = sourceCells; invalidCells[2 * 80 + 5].kind = 0
malformed(invalidCells, sourceBytes)
var invalidBytes = sourceBytes; invalidBytes[0] = 0xff
malformed(sourceCells, invalidBytes)
// All foreign memory is released before any shaping or drawing.
check(nb_session_release(session) == NB_OK, "session teardown")
check(nb_frame_release(handle) == NB_OK, "frame teardown")
check(nb_frame_view(handle, &frame) == NB_INVALID, "stale frame rejected")
// Do not keep even a dangling view in the rest of the probe.
frame = NbFrame()

var records: [[String: Any]] = []
var usedFonts: [String: CTFont] = [:]
var maximumShapeBytes = 0
var rejectedOrder = 0
var missingGlyphs = 0
var pictures: [(TextRun, CTFont, Bool)] = []
@MainActor func shape(_ input: TextRun, name: String, enabled: Bool) throws -> ShapedText {
    let font = try shapingFont(name, ligatures: enabled)
    let result = try autoreleasepool {
        try mapLine(makeLine(input, font: font, ligatures: enabled), input: input)
    }
    check(!result.glyphs.isEmpty, "nonempty shaping")
    for owner in input.owners {
        check(result.glyphs.contains { $0.utf16.overlaps(owner.utf16) && $0.columns.contains(owner.column) }, "owner coverage")
    }
    for glyph in result.glyphs {
        let covered = input.owners.filter { $0.utf16.overlaps(glyph.utf16) }
        check(glyph.columns == covered.first!.column..<(covered.last!.column + covered.last!.width), "glyph span")
    }
    maximumShapeBytes = max(maximumShapeBytes, result.payloadBytes)
    missingGlyphs += result.glyphs.filter { $0.id == 0 }.count
    for font in result.fonts { usedFonts[CTFontCopyPostScriptName(font) as String] = font }
    records.append(["row": input.row, "base_font": name, "ligatures": enabled, "text": input.text,
                    "fonts": result.fonts.map(fontJSON), "glyphs": glyphJSON(result), "payload_bytes": result.payloadBytes])
    return result
}

for input in plan.runs {
    for enabled in [false, true] {
        if input.row == 8 {
            expect(.unsupportedOrder) { _ = try shape(input, name: "Menlo-Regular", enabled: enabled) }
            rejectedOrder += 1
        } else { _ = try shape(input, name: "Menlo-Regular", enabled: enabled) }
    }
}
let prose = plan.runs.first { $0.row == 1 }!
let proseOff = try shape(prose, name: "Times-Roman", enabled: false)
let proseOn = try shape(prose, name: "Times-Roman", enabled: true)
check(proseOn.glyphs.count < proseOff.glyphs.count, "actual typographic ligatures required")
check(proseOn.glyphs.contains { $0.columns.count > 1 }, "ligature spans multiple cells")
pictures += [(prose, try shapingFont("Times-Roman", ligatures: false), false),
             (prose, try shapingFont("Times-Roman", ligatures: true), true)]
var programming: [String: Any] = ["status": "not_requested"]
if CommandLine.arguments.count > 4 {
    let name = CommandLine.arguments[4]
    if CommandLine.arguments.count > 5 {
        let url = URL(fileURLWithPath: CommandLine.arguments[5])
        check(CTFontManagerRegisterFontsForURL(url as CFURL, .process, nil), "process-only font registration")
    }
    let code = plan.runs.first { $0.row == 0 }!
    let off = try shape(code, name: name, enabled: false)
    let on = try shape(code, name: name, enabled: true)
    check(on.glyphs.map { $0.id } != off.glyphs.map { $0.id }, "programming font must change glyph identities")
    programming = ["status": "passed", "font": name, "off_glyphs": off.glyphs.count, "on_glyphs": on.glyphs.count]
    pictures += [(code, try shapingFont(name, ligatures: false), false), (code, try shapingFont(name, ligatures: true), true)]
}
let unicodeShape = try shape(unicode, name: "Menlo-Regular", enabled: true)
check(unicodeShape.fonts.contains { CTFontCopyPostScriptName($0) as String != "Menlo-Regular" }, "fallback font exercised")
pictures += [(unicode, try shapingFont("Menlo-Regular", ligatures: false), false),
             (unicode, try shapingFont("Menlo-Regular", ligatures: true), true)]
limits = TextLimits(); limits.glyphs = proseOn.glyphs.count - 1
let proseFont = try shapingFont("Times-Roman", ligatures: true)
expect(.limit) { _ = try mapLine(makeLine(prose, font: proseFont, ligatures: true), input: prose, limits: limits) }
limits = TextLimits(); limits.fonts = 0
expect(.limit) { _ = try mapLine(makeLine(prose, font: proseFont, ligatures: true), input: prose, limits: limits) }
limits = TextLimits(); limits.planBytes = proseOn.payloadBytes
let exactShape = try mapLine(makeLine(prose, font: proseFont, ligatures: true), input: prose, limits: limits)
check(exactShape.payloadBytes == proseOn.payloadBytes, "exact shaped payload cap")
limits.planBytes -= 1
expect(.limit) { _ = try mapLine(makeLine(prose, font: proseFont, ligatures: true), input: prose, limits: limits) }
check(missingGlyphs == 0, "fixture has no missing glyphs")
let stableSignature = signature(proseOn)
for _ in 0..<64 {
    let repeated = try autoreleasepool {
        try mapLine(makeLine(prose, font: proseFont, ligatures: true), input: prose)
    }
    check(signature(repeated) == stableSignature, "repeated independent shaping after teardown")
}

// Independently inspect raster bytes to verify clipping at a one-cell span.
let clipped = CGContext(data: nil, width: 128, height: 64, bitsPerComponent: 8, bytesPerRow: 512,
                        space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
clipped.setFillColor(CGColor(gray: 0, alpha: 1)); clipped.fill(CGRect(x: 0, y: 0, width: 128, height: 64))
let background = Data(bytes: clipped.data!, count: 512 * 64)
clipped.saveGState(); clipped.clip(to: CGRect(x: 10, y: 10, width: 14, height: 34))
clipped.setFillColor(CGColor(gray: 1, alpha: 1)); clipped.textPosition = CGPoint(x: 10, y: 20)
CTLineDraw(makeLine(prose, font: proseFont, ligatures: true), clipped)
clipped.restoreGState()
let raster = Data(bytes: clipped.data!, count: 512 * 64)
var changedInside = 0, changedOutside = 0
for y in 0..<64 {
    for x in 0..<128 {
        let offset = y * 512 + x * 4
        if raster[offset..<(offset + 4)] != background[offset..<(offset + 4)] {
            // Bitmap rows run opposite the CGContext's bottom-origin coordinates.
            if (10..<24).contains(x) && (20..<54).contains(y) { changedInside += 1 }
            else { changedOutside += 1 }
        }
    }
}
check(changedInside > 0 && changedOutside == 0, "glyph raster stays inside clip")

// Offscreen diagnostic: natural Core Text positions with a terminal-run clip.
// Grid fitting and contextual glyph dependencies are deliberately still open.
let width = 1200, height = pictures.count * 80
let context = CGContext(data: nil, width: width, height: height, bitsPerComponent: 8, bytesPerRow: width * 4,
                        space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
context.setFillColor(CGColor(gray: 0.06, alpha: 1)); context.fill(CGRect(x: 0, y: 0, width: width, height: height))
for (index, picture) in pictures.enumerated() {
    let (input, font, enabled) = picture
    let y = CGFloat(height - (index + 1) * 80)
    let label = "\(CTFontCopyPostScriptName(font))   ligatures \(enabled ? "ON" : "OFF")"
    let attrs: [NSAttributedString.Key: Any] = [
        NSAttributedString.Key(kCTFontAttributeName as String): CTFontCreateWithName("Menlo-Regular" as CFString, 12, nil),
        NSAttributedString.Key(kCTForegroundColorFromContextAttributeName as String): true]
    context.setFillColor(CGColor(gray: 0.8, alpha: 1)); context.textPosition = CGPoint(x: 12, y: y + 55)
    CTLineDraw(CTLineCreateWithAttributedString(NSAttributedString(string: label, attributes: attrs)), context)
    context.setStrokeColor(CGColor(gray: 0.25, alpha: 1))
    for column in input.columns.lowerBound...input.columns.upperBound {
        context.stroke(CGRect(x: 12 + column * 14, y: Int(y) + 8, width: 14, height: 34))
    }
    context.saveGState()
    context.clip(to: CGRect(x: 12 + input.columns.lowerBound * 14, y: Int(y) + 8, width: input.columns.count * 14, height: 34))
    context.textPosition = CGPoint(x: 12 + input.columns.lowerBound * 14, y: Int(y) + 15)
    context.setFillColor(CGColor(gray: 0.95, alpha: 1))
    CTLineDraw(makeLine(input, font: font, ligatures: enabled), context)
    context.restoreGState()
}
let destination = CGImageDestinationCreateWithURL(output.appendingPathComponent("shaping.png") as CFURL, "public.png" as CFString, 1, nil)!
CGImageDestinationAddImage(destination, context.makeImage()!, nil)
check(CGImageDestinationFinalize(destination), "PNG")
let summary: [String: Any] = ["schema_version": 1, "success": true, "kind": "core_text_mapping_correctness_not_performance",
    "checks": checks, "runs": plan.runs.count, "plan_payload_bytes": plan.payloadBytes,
    "maximum_shaped_run_payload_bytes": maximumShapeBytes, "input_leases_live_during_shaping": 0,
    "programming_ligatures": programming, "typographic_ligatures": ["off_glyphs": proseOff.glyphs.count, "on_glyphs": proseOn.glyphs.count],
    "unsupported_order_rejections": rejectedOrder, "missing_glyphs": missingGlyphs,
    "repeat_shape_cycles": 64, "clip_changed_inside_pixels": changedInside, "clip_changed_outside_pixels": changedOutside,
    "resolved_fonts": usedFonts.keys.sorted().map { fontJSON(usedFonts[$0]!) },
    "limits": ["Logical payload counts exclude Swift allocator, Core Text/font caches and RSS", "One native line is shaped at a time; no retained CTLine or glyph cache",
               "LTR monotonic mapping only; bidi/nonmonotonic/matrix runs rejected", "Natural positions are diagnostic; terminal grid fitting and Metal are pending"]]
let inputs: [[String: Any]] = plan.runs.map { run in
    ["row": run.row, "text": run.text,
     "style": [run.style.foreground, run.style.background, run.style.attributes],
     "owners": run.owners.map { ["column": $0.column, "width": $0.width,
         "source_utf8": [$0.sourceUTF8.lowerBound, $0.sourceUTF8.upperBound], "utf16": [$0.utf16.lowerBound, $0.utf16.upperBound]] }]
}
for (name, value) in [("summary.json", summary as Any), ("shapes.json", records as Any), ("inputs.json", inputs as Any)] {
    let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
    try (data + Data([10])).write(to: output.appendingPathComponent(name))
}
print("Core Text: \(plan.runs.count) immutable runs; typographic ligatures, Unicode/fallback, limits and lifetime checks passed")
