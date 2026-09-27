import Foundation
import Darwin

func start(_ executable: String, _ args: [String]) -> UInt64 {
    let storage = ([executable] + args).map { strdup($0)! }
    defer { storage.forEach { free($0) } }
    let bytes = storage.map { NbBytes(data: UnsafeRawPointer($0).assumingMemoryBound(to: UInt8.self), len: strlen($0)) }
    var handle: UInt64 = 0
    let result = Array(bytes.dropFirst()).withUnsafeBufferPointer { args in
        nb_session_start(bytes[0], args.baseAddress, args.count, 8, 3, &handle)
    }
    precondition(result == NB_OK)
    return handle
}
func view(_ handle: UInt64) -> NbFrame {
    var frame = NbFrame()
    precondition(nb_frame_view(handle, &frame) == NB_OK)
    return frame
}
func text(_ frame: NbFrame) -> [UInt8] {
    Array(UnsafeBufferPointer(start: frame.text, count: frame.text_len))
}
precondition(nb_abi_version() == 2 && MemoryLayout<NbCell>.size == 12 && MemoryLayout<NbFrame>.size == 104 && MemoryLayout<NbStyle>.size == 12)
let session = start(CommandLine.arguments[1], [CommandLine.arguments[2], "roundtrip"])
let deadline = Date().addingTimeInterval(5)
var held: UInt64 = 0
var heldBytes: [UInt8] = []
var resized = false
var status = NbStatus()
while true {
    precondition(Date() < deadline, "worker deadline exceeded")
    precondition(nb_session_status(session, &status) == NB_OK)
    if !resized && status.denied_effects == 2 {
        // Effect accounting may become visible just before the first frame
        // publication. Retry NO_FRAME rather than depending on thread timing.
        let acquired = nb_frame_acquire(session, 0, &held)
        precondition(acquired == NB_OK || acquired == NB_NO_FRAME)
        if acquired == NB_OK {
            heldBytes = text(view(held))
            precondition(nb_session_resize(session, 12, 4) == NB_OK)
            resized = true
        }
    }
    if status.finished != 0 { break }
    Thread.sleep(forTimeInterval: 0.001)
}
precondition(resized && status.phase == NB_EXITED && status.exit_code == 0)
var final: UInt64 = 0
precondition(nb_frame_acquire(session, 0, &final) == NB_OK)
let finalView = view(final)
precondition(finalView.columns == 12 && finalView.lines == 4)
precondition(String(decoding: text(finalView), as: UTF8.self) == "abRDONEERR�")
let heldView = view(held)
precondition(nb_session_release(session) == NB_OK)
precondition(text(heldView) == heldBytes)
precondition(heldView.style_count > 0 && heldView.styles![0].attributes == 0)
precondition(String(decoding: text(finalView), as: UTF8.self) == "abRDONEERR�")
precondition(nb_frame_release(held) == NB_OK && nb_frame_release(final) == NB_OK)
precondition(nb_frame_release(final) == NB_INVALID)

let closing = start("/bin/sleep", ["30"])
precondition(nb_session_release(closing) == NB_BUSY)
precondition(nb_session_close(closing) == NB_OK)
let closeDeadline = Date().addingTimeInterval(5)
repeat {
    precondition(Date() < closeDeadline)
    precondition(nb_session_status(closing, &status) == NB_OK)
    if status.finished == 0 { Thread.sleep(forTimeInterval: 0.001) }
} while status.finished == 0
precondition(status.phase == NB_STOPPED)
precondition(nb_session_release(closing) == NB_OK)
print("Swift ABI: PTY replies, resize, retained frames and asynchronous close passed")
