//! All foreign-pointer handling and exported symbols stay in this module.
use crate::registry::{self, INVALID, LIMIT, PANIC};
use nebulax_terminal::{Size, snapshot::SnapshotCell, style::Style};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::Command;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NbBytes {
    pub data: *const u8,
    pub len: usize,
}
#[repr(C)]
pub struct NbStatus {
    pub phase: u32,
    pub exit_code: i32,
    pub failure: u32,
    pub finished: u32,
    pub denied_effects: u64,
}
#[repr(C)]
pub struct NbFrame {
    pub generation: u64,
    pub columns: u32,
    pub lines: u32,
    pub cursor_row: u32,
    pub cursor_column: u32,
    pub wrap_pending: u32,
    pub alternate: u32,
    pub cells: *const SnapshotCell,
    pub cell_count: usize,
    pub text: *const u8,
    pub text_len: usize,
    pub row_versions: *const u64,
    pub row_wraps: *const u8,
    pub row_count: usize,
    pub styles: *const Style,
    pub style_count: usize,
    pub cursor_visible: u32,
    pub reserved: u32,
}
fn boundary(f: impl FnOnce() -> Result<(), i32>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => registry::OK,
        Ok(Err(code)) => code,
        Err(_) => PANIC,
    }
}
fn aligned<T>(pointer: *const T) -> bool {
    !pointer.is_null() && pointer.is_aligned()
}
/// Caller guarantees readable immutable byte storage for the duration of call.
unsafe fn string(bytes: NbBytes) -> Result<String, i32> {
    if bytes.len > 4096 {
        return Err(LIMIT);
    }
    if bytes.len == 0 {
        return Ok(String::new());
    }
    if bytes.data.is_null() {
        return Err(INVALID);
    }
    // SAFETY: foreign caller provides readable len bytes; length is bounded and
    // non-null above. Owned copy is made before returning to foreign code.
    let slice = unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) };
    if slice.contains(&0) {
        return Err(INVALID);
    }
    std::str::from_utf8(slice)
        .map(str::to_owned)
        .map_err(|_| INVALID)
}
// SAFETY: project-private prefixed symbol; declaration matches the checked C header.
#[unsafe(no_mangle)]
pub extern "C" fn nb_abi_version() -> u32 {
    3
}

/// # Safety
/// All nonempty input buffers/arrays are readable for their stated lengths;
/// output points to writable aligned u64. Inputs and output must not alias.
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nb_session_start(
    executable: NbBytes,
    args: *const NbBytes,
    argc: usize,
    columns: u32,
    lines: u32,
    out: *mut u64,
) -> i32 {
    boundary(|| {
        if !aligned(out) || argc > 16 || (argc > 0 && !aligned(args)) {
            return Err(INVALID);
        }
        // SAFETY: inputs satisfy this function's foreign storage contract.
        let executable = unsafe { string(executable) }?;
        if executable.is_empty() {
            return Err(INVALID);
        }
        let args = if argc == 0 {
            &[]
        } else {
            // SAFETY: caller supplies argc readable aligned NbBytes values.
            unsafe { std::slice::from_raw_parts(args, argc) }
        };
        let mut total = executable.len();
        let mut command = Command::new(executable);
        for arg in args {
            // SAFETY: each argument obeys the same readable storage contract.
            let value = unsafe { string(*arg) }?;
            total += value.len();
            if total > 16_384 {
                return Err(LIMIT);
            }
            command.arg(value);
        }
        let handle = registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .start(
                command,
                Size {
                    columns: columns as usize,
                    lines: lines as usize,
                },
            )?;
        // SAFETY: caller provides non-aliasing writable aligned output storage.
        unsafe { out.write(handle) };
        Ok(())
    })
}
/// # Safety
/// out must be writable aligned NbStatus storage; no concurrent access to it.
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nb_session_status(handle: u64, out: *mut NbStatus) -> i32 {
    boundary(|| {
        if !aligned(out) {
            return Err(INVALID);
        }
        let (s, finished) = registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .status(handle)?;
        // SAFETY: checked alignment/null and caller guarantees writable storage.
        unsafe {
            out.write(NbStatus {
                phase: s.phase as u32,
                exit_code: s.exit_code,
                failure: s.failure,
                finished: u32::from(finished),
                denied_effects: s.denied_effects,
            })
        };
        Ok(())
    })
}
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub extern "C" fn nb_session_resize(handle: u64, columns: u32, lines: u32) -> i32 {
    boundary(|| {
        registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .resize(
                handle,
                Size {
                    columns: columns as usize,
                    lines: lines as usize,
                },
            )
    })
}
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub extern "C" fn nb_session_close(handle: u64) -> i32 {
    boundary(|| {
        registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .close(handle)
    })
}
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub extern "C" fn nb_session_release(handle: u64) -> i32 {
    boundary(|| {
        registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .release_session(handle)
    })
}
/// # Safety
/// out must be writable aligned u64 storage; no concurrent access to it.
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nb_frame_acquire(
    handle: u64,
    after_generation: u64,
    out: *mut u64,
) -> i32 {
    boundary(|| {
        if !aligned(out) {
            return Err(INVALID);
        }
        let frame = registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .acquire(handle, after_generation)?;
        // SAFETY: writable output contract and checked alignment/null.
        unsafe { out.write(frame) };
        Ok(())
    })
}
/// # Safety
/// out is writable aligned NbFrame storage. Caller must not release this frame
/// concurrently with reading the returned immutable views. Views expire at release.
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nb_frame_view(handle: u64, out: *mut NbFrame) -> i32 {
    boundary(|| {
        if !aligned(out) {
            return Err(INVALID);
        }
        let registry = registry::global().lock().unwrap_or_else(|e| e.into_inner());
        let frame = registry.frame(handle)?;
        // SAFETY: caller supplies writable output, and the checked frame handle
        // retains immutable allocation ownership until explicitly released.
        unsafe {
            out.write(NbFrame {
                generation: frame.generation(),
                columns: frame.size().columns as u32,
                lines: frame.size().lines as u32,
                cursor_row: frame.cursor().row as u32,
                cursor_column: frame.cursor().column as u32,
                wrap_pending: u32::from(frame.cursor().wrap_pending),
                alternate: u32::from(frame.is_alternate()),
                cells: frame.cells().as_ptr(),
                cell_count: frame.cells().len(),
                text: frame.text().as_ptr(),
                text_len: frame.text().len(),
                row_versions: frame.row_versions().as_ptr(),
                row_wraps: frame.row_wraps().as_ptr(),
                row_count: frame.row_versions().len(),
                styles: frame.styles().as_ptr(),
                style_count: frame.styles().len(),
                cursor_visible: u32::from(frame.cursor_visible()),
                reserved: 0,
            })
        };
        Ok(())
    })
}
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub extern "C" fn nb_frame_release(handle: u64) -> i32 {
    boundary(|| {
        registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .release_frame(handle)
    })
}

/// # Safety
/// text.data must be readable for text.len bytes until return. UTF-8 is copied.
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nb_session_text(handle: u64, text: NbBytes) -> i32 {
    boundary(|| {
        // SAFETY: caller provides a readable buffer; string checks bounded length.
        let text = unsafe { string(text) }?;
        registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .input(handle, nebulax_pty_session::input::Input::Text(text))
    })
}
/// # Safety
/// text.data must be readable for text.len bytes until return. UTF-8 is copied.
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nb_session_paste(handle: u64, text: NbBytes) -> i32 {
    boundary(|| {
        // SAFETY: caller provides a readable buffer; string checks bounded length.
        let text = unsafe { string(text) }?;
        registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .input(handle, nebulax_pty_session::input::Input::Paste(text))
    })
}
// SAFETY: unique private ABI export matching the checked header.
#[unsafe(no_mangle)]
pub extern "C" fn nb_session_key(handle: u64, key: u32) -> i32 {
    boundary(|| {
        let key = nebulax_terminal::input::Key::from_code(key).ok_or(INVALID)?;
        registry::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .input(handle, nebulax_pty_session::input::Input::Key(key))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn panics_are_converted_to_status_without_crossing_c() {
        assert_eq!(boundary(|| panic!("synthetic boundary panic")), PANIC);
    }
}
