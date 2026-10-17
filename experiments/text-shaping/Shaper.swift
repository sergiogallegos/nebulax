import Foundation
import CoreText

struct MappedGlyph {
    let id: CGGlyph
    let font: Int
    let utf16: Range<Int>
    let columns: Range<Int>
    let position: CGPoint
    let advance: CGSize
}

struct ShapedText {
    let glyphs: [MappedGlyph]
    let fonts: [CTFont]
    let payloadBytes: Int
}

func shapingFont(_ name: String, ligatures: Bool) throws -> CTFont {
    let base = CTFontCreateWithName(name as CFString, 20, nil)
    guard CTFontCopyPostScriptName(base) as String == name else {
        FileHandle.standardError.write(Data("Font unavailable: requested \(name), resolved \(CTFontCopyPostScriptName(base))\n".utf8))
        throw TextError.invalidFrame
    }
    // Programming fonts may use contextual alternates rather than 'liga'.
    let settings: [[CFString: Any]] = [[kCTFontOpenTypeFeatureTag: "calt", kCTFontOpenTypeFeatureValue: ligatures ? 1 : 0]]
    let descriptor = CTFontDescriptorCreateWithAttributes([kCTFontFeatureSettingsAttribute: settings] as CFDictionary)
    return CTFontCreateCopyWithAttributes(base, 20, nil, descriptor)
}

func makeLine(_ input: TextRun, font: CTFont, ligatures: Bool) -> CTLine {
    let attributes: [NSAttributedString.Key: Any] = [
        NSAttributedString.Key(kCTFontAttributeName as String): font,
        NSAttributedString.Key(kCTLigatureAttributeName as String): ligatures ? 2 : 0,
        NSAttributedString.Key(kCTForegroundColorFromContextAttributeName as String): true,
    ]
    return CTLineCreateWithAttributedString(NSAttributedString(string: input.text, attributes: attributes))
}

// Native output is copied while its CTLine lives; no Core Text pointer escapes.
// Only logical-LTR monotonic runs are accepted by this first mapping contract.
func mapLine(_ line: CTLine, input: TextRun, limits: TextLimits = TextLimits()) throws -> ShapedText {
    let total = CTLineGetGlyphCount(line)
    guard total >= 0, total <= limits.glyphs, limits.planBytes >= 0,
          total <= limits.planBytes / MemoryLayout<MappedGlyph>.stride else { throw TextError.limit }
    var glyphs: [MappedGlyph] = []
    var fonts: [CTFont] = []
    for run in CTLineGetGlyphRuns(line) as! [CTRun] {
        let status = CTRunGetStatus(run)
        guard status.intersection([.rightToLeft, .nonMonotonic, .hasNonIdentityMatrix]).isEmpty else {
            throw TextError.unsupportedOrder
        }
        let count = CTRunGetGlyphCount(run)
        let range = CTRunGetStringRange(run)
        guard range.location >= 0, range.length > 0, range.location <= input.text.utf16.count,
              range.length <= input.text.utf16.count - range.location else { throw TextError.invalidFrame }
        guard fonts.count < limits.fonts,
              total * MemoryLayout<MappedGlyph>.stride + (fonts.count + 1) * MemoryLayout<CTFont>.stride <= limits.planBytes
        else { throw TextError.limit }
        let attrs = CTRunGetAttributes(run) as NSDictionary
        let font = attrs[kCTFontAttributeName] as! CTFont
        let fontIndex = fonts.count
        fonts.append(font)
        var ids = [CGGlyph](repeating: 0, count: count)
        var indices = [CFIndex](repeating: 0, count: count)
        var positions = [CGPoint](repeating: .zero, count: count)
        var advances = [CGSize](repeating: .zero, count: count)
        CTRunGetGlyphs(run, CFRange(), &ids)
        CTRunGetStringIndices(run, CFRange(), &indices)
        CTRunGetPositions(run, CFRange(), &positions)
        CTRunGetAdvances(run, CFRange(), &advances)
        let end = range.location + range.length
        let starts = Array(Set(indices)).sorted()
        guard starts.allSatisfy({ $0 >= range.location && $0 < end }) else { throw TextError.invalidFrame }
        let ends = Dictionary(uniqueKeysWithValues: zip(starts, starts.dropFirst() + [end]))
        for i in 0..<count {
            let source = indices[i]..<ends[indices[i]]!
            // Fallback can split a combining/emoji owner into several runs.
            // Every intersecting glyph retains the complete authoritative span.
            let owners = input.owners.filter { $0.utf16.overlaps(source) }
            guard let first = owners.first, let last = owners.last,
                  positions[i].x.isFinite, positions[i].y.isFinite,
                  advances[i].width.isFinite, advances[i].height.isFinite else { throw TextError.invalidFrame }
            glyphs.append(MappedGlyph(id: ids[i], font: fontIndex, utf16: source,
                                     columns: first.column..<(last.column + last.width),
                                     position: positions[i], advance: advances[i]))
        }
    }
    let payload = glyphs.count * MemoryLayout<MappedGlyph>.stride + fonts.count * MemoryLayout<CTFont>.stride
    guard payload <= limits.planBytes else { throw TextError.limit }
    return ShapedText(glyphs: glyphs, fonts: fonts, payloadBytes: payload)
}
