// entry.rs or lib.rs

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use cmdx::{translate_full, Os};

/// Translates a Windows command string to Linux using cmdx.
/// Returns a newly allocated C string. Must be freed with free_string.
#[no_mangle]
pub extern "C" fn preprocess_command(cmd: *const c_char) -> *mut c_char {
    if cmd.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(cmd) };
    let cmd_str = c_str.to_str().unwrap_or("");

    // Perform translation; fallback to original if translation fails
    let result = translate_full(cmd_str, Os::Windows, Os::Linux)
        .unwrap_or_else(|_| cmd_str.into());

    // Convert Rust String to C string
    let c_result = CString::new(result.command).unwrap_or_else(|_| CString::new("").unwrap());
    c_result.into_raw()
}

/// Frees a C string previously allocated by preprocess_command.
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        // Reconstruct CString so it gets dropped and memory freed
        CString::from_raw(s);
    }
}
