import Foundation

enum TextError: Error, Equatable { case invalidFrame, limit, unsupportedOrder }

struct TextLimits {
    var runOwners = 256
    var runUTF8 = 4096
    var runUTF16 = 2048
    var planBytes = 8 * 1024 * 1024
    var glyphs = 8192
    var fonts = 64
}

// These values describe copied presentation input, never mutable terminal state.
struct TextOwner {
    let column: Int
    let width: Int
    let sourceUTF8: Range<Int>
    let utf16: Range<Int>
}

struct TextRun {
    let row: Int
    let style: NbStyle
    let text: String
    let owners: [TextOwner]
    var columns: Range<Int> { owners.first!.column..<(owners.last!.column + owners.last!.width) }
}

struct TextPlan {
    let generation: UInt64
    let columns: Int
    let rows: Int
    let runs: [TextRun]
    // Logical payload charge, including native Swift record strides; not RSS.
    let payloadBytes: Int

    static func capture(_ frame: NbFrame, limits: TextLimits = TextLimits()) throws -> TextPlan {
        let columns = Int(frame.columns), rows = Int(frame.lines)
        guard columns > 0, rows > 0, columns <= 65_536, rows <= 65_536,
              columns * rows <= 65_536, frame.cell_count == columns * rows,
              frame.row_count == rows, frame.text_len >= 0, frame.text_len <= 2 * 1024 * 1024,
              frame.style_count > 0, frame.style_count <= 1024,
              let cells = frame.cells, let styles = frame.styles,
              frame.text_len == 0 || frame.text != nil else { throw TextError.invalidFrame }
        guard limits.runOwners > 0, limits.runUTF8 > 0, limits.runUTF16 > 0,
              limits.planBytes >= MemoryLayout<TextPlan>.stride else { throw TextError.limit }
        var runs: [TextRun] = []
        var payload = MemoryLayout<TextPlan>.stride
        var sourceEnd = 0
        for row in 0..<rows {
            var text = ""
            var owners: [TextOwner] = []
            var style = NbStyle()
            var utf16Count = 0
            func flush() {
                if !owners.isEmpty {
                    runs.append(TextRun(row: row, style: style, text: text, owners: owners))
                    text = ""; owners = []; utf16Count = 0
                }
            }
            var column = 0
            while column < columns {
                let cell = cells[row * columns + column]
                guard cell.reserved == 0, Int(cell.style_id) < frame.style_count else { throw TextError.invalidFrame }
                if cell.kind == 0 || cell.kind == 3 {
                    guard cell.width == 0, cell.text_len == 0 else { throw TextError.invalidFrame }
                    flush(); column += 1; continue
                }
                let width = Int(cell.width), length = Int(cell.text_len), offset = Int(cell.text_offset)
                guard cell.kind == 1, width == 1 || width == 2, column + width <= columns,
                      length > 0, offset == sourceEnd, offset <= frame.text_len,
                      length <= frame.text_len - offset, let bytes = frame.text,
                      let cluster = String(bytes: UnsafeBufferPointer(start: bytes.advanced(by: offset), count: length), encoding: .utf8)
                else { throw TextError.invalidFrame }
                if width == 2 {
                    let tail = cells[row * columns + column + 1]
                    guard tail.kind == 2, tail.width == 0, tail.text_len == 0,
                          tail.style_id == cell.style_id, tail.reserved == 0 else { throw TextError.invalidFrame }
                }
                let nextStyle = styles[Int(cell.style_id)]
                let units = cluster.utf16.count
                guard length <= limits.runUTF8, units <= limits.runUTF16 else { throw TextError.limit }
                let sameStyle = style.foreground == nextStyle.foreground && style.background == nextStyle.background
                    && style.attributes == nextStyle.attributes
                if !owners.isEmpty && (!sameStyle || owners.count == limits.runOwners
                    || text.utf8.count + length > limits.runUTF8 || utf16Count + units > limits.runUTF16) { flush() }
                let charge = length + units * 2 + MemoryLayout<TextOwner>.stride
                    + (owners.isEmpty ? MemoryLayout<TextRun>.stride : 0)
                guard charge <= limits.planBytes - payload else { throw TextError.limit }
                payload += charge
                style = nextStyle
                owners.append(TextOwner(column: column, width: width, sourceUTF8: offset..<(offset + length),
                                        utf16: utf16Count..<(utf16Count + units)))
                text += cluster; utf16Count += units; sourceEnd += length; column += width
            }
            flush()
        }
        guard sourceEnd == frame.text_len else { throw TextError.invalidFrame }
        return TextPlan(generation: frame.generation, columns: columns, rows: rows, runs: runs, payloadBytes: payload)
    }
}
