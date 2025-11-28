use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use cmdx::{translate_full, Os};

#[no_mangle]
pub extern "C" fn preprocess_command(cmd: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(cmd) };
    let cmd_str = c_str.to_str().unwrap_or("");

    // Translate from Windows → Linux (adjust source/target as needed)
    let result = translate_full(cmd_str, Os::Windows, Os::Linux).unwrap_or_else(|_| cmd_str.into());
    let c_result = CString::new(result.command).unwrap();
    c_result.into_raw()
}

// Free the string after use
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() { return }
    unsafe { CString::from_raw(s) };
}
