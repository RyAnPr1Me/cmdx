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


/// Translates a Windows command string to Linux using cmdx.
/// Returns a newly allocated C string. Must be freed with free_string.
/// 
/// # Safety
/// 
/// This function is unsafe because it:
/// - Dereferences a raw pointer (`cmd`)
/// - The caller must ensure `cmd` is a valid null-terminated C string
/// - The returned pointer must be freed by calling `free_string`
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

/// Frees a C string previously allocated by preprocess_command.
/// 
/// # Safety
/// 
/// This function is unsafe because it:
/// - Dereferences a raw pointer (`s`)
/// - The caller must ensure `s` was allocated by `preprocess_command`
/// - The pointer must not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    // Reconstruct CString so it gets dropped and memory freed
    let _ = CString::from_raw(s);
}
