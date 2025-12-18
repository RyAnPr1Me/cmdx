// entry.rs or lib.rs

mod translator;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Re-export main translator types and functions
pub use translator::engine::{
    translate_command, translate_full, translate_compound_command,
    translate_batch, translate_script_extension, translate_shebang,
    TranslationResult, TranslationError,
};
pub use translator::os::{Os, detect_os};
pub use translator::path::{
    translate_path, translate_path_auto, translate_paths,
    is_windows_path, is_unix_path, PathTranslation,
};
pub use translator::env::translate_env_vars;
pub use translator::command_map::{
    is_native_command, get_mapping, get_available_commands,
    CommandMapping, FlagMapping,
};
pub use translator::distro::{Distro, PackageManager};
pub use translator::package_manager::{
    translate_package_command, translate_package_command_auto,
    PackageTranslationResult, PackageTranslationError, PackageOperation,
};


/// Translate a Windows command string into its Linux equivalent and return it as a newly allocated C string.
///
/// The function accepts a pointer to a null-terminated C string containing a Windows command, attempts to translate it
/// to a Linux command using the library's translator, and returns a pointer to a newly allocated C string containing
/// the translated command. The returned pointer must be released by calling `free_string`.
///
/// # Safety
///
/// - `cmd` must be a valid, non-null, null-terminated C string.
/// - The caller must ensure the pointer is not mutated or freed while this function executes.
/// - The returned pointer is owned by the caller and must be freed with `free_string`; using or freeing it in any
///   other way is undefined behavior.
///
/// # Returns
///
/// A pointer to a newly allocated null-terminated C string containing the translated command, or a null pointer if
/// `cmd` is null. The pointer must be freed with `free_string`.
///
/// # Examples
///
/// ```
/// use std::ffi::CString;
/// use std::os::raw::c_char;
///
/// // Prepare input
/// let input = CString::new("dir C:\\").unwrap();
/// let in_ptr: *const c_char = input.as_ptr();
///
/// // Call the FFI function (unsafe)
/// let out_ptr = unsafe { preprocess_command(in_ptr) };
/// assert!(!out_ptr.is_null());
///
/// // Convert result back to Rust and free it
/// let out_cstr = unsafe { CString::from_raw(out_ptr) };
/// let translated = out_cstr.to_string_lossy();
/// // free_string is not needed here because from_raw already took ownership and freed on drop;
/// // in actual usage, free_string should be used to free the pointer returned by preprocess_command.
///
/// assert!(!translated.is_empty());
/// ```
#[no_mangle]
pub unsafe extern "C" fn preprocess_command(cmd: *const c_char) -> *mut c_char {
    if cmd.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = CStr::from_ptr(cmd);
    let cmd_str = c_str.to_str().unwrap_or("");

    // Perform translation; fallback to original if translation fails
    let result = translate_full(cmd_str, Os::Windows, Os::Linux)
        .unwrap_or_else(|_| TranslationResult::new(
            cmd_str.to_string(),
            cmd_str.to_string(),
            Os::Windows,
            Os::Linux,
        ));

    // Convert Rust String to C string
    let c_result = CString::new(result.command).unwrap_or_else(|_| CString::new("").unwrap());
    c_result.into_raw()
}

/// Frees a C string that was previously allocated and returned to C.
///
/// This function reclaims the memory for a `*mut c_char` by reconstructing a
/// `CString` and dropping it. Passing a null pointer is a no-op.
///
/// # Safety
///
/// - `s` must either be null or be a pointer returned by `preprocess_command`
///   (or otherwise created by `CString::into_raw`).
/// - The caller must ensure the pointer is not used after this call.
/// - Calling this with a pointer not created by `CString::into_raw` or with a
///   pointer that has already been freed is undefined behavior.
///
/// # Examples
///
/// ```
/// use std::ffi::CString;
/// use libc::c_char;
///
/// // Create a C string and take ownership of its raw pointer.
/// let s = CString::new("example").unwrap();
/// let ptr: *mut c_char = s.into_raw();
///
/// // Free the string using the FFI helper.
/// unsafe { crate::free_string(ptr); }
/// ```
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    // Reconstruct CString so it gets dropped and memory freed
    let _ = CString::from_raw(s);
}