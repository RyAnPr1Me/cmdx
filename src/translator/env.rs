//! Environment variable translation module
//!
//! This module provides translation of environment variable references between
//! Windows and Unix-like operating systems.
//!
//! ## Examples
//!
//! ```
//! use cmdx::{translate_env_vars, Os};
//!
//! // Windows to Unix
//! let result = translate_env_vars("echo %PATH%", Os::Windows, Os::Linux);
//! assert_eq!(result, "echo $PATH");
//!
//! // Unix to Windows (HOME maps to USERPROFILE)
//! let result = translate_env_vars("echo $HOME", Os::Linux, Os::Windows);
//! assert_eq!(result, "echo %USERPROFILE%");
//! ```

use super::os::Os;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Common environment variable name mappings between Windows and Unix
    /// Variables without direct equivalents are passed through with the original name.
    static ref ENV_VAR_MAPPINGS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Windows -> Unix mappings (exact equivalents)
        m.insert("USERPROFILE", "HOME");
        m.insert("USERNAME", "USER");
        m.insert("APPDATA", "XDG_CONFIG_HOME");
        m.insert("LOCALAPPDATA", "XDG_DATA_HOME");
        m.insert("TEMP", "TMPDIR");
        m.insert("TMP", "TMPDIR");
        m.insert("COMPUTERNAME", "HOSTNAME");
        m.insert("CD", "PWD");
        m.insert("COMSPEC", "SHELL");
        m
    };

    /// Reverse mappings (Unix -> Windows)
    static ref ENV_VAR_MAPPINGS_REVERSE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("HOME", "USERPROFILE");
        m.insert("USER", "USERNAME");
        m.insert("XDG_CONFIG_HOME", "APPDATA");
        m.insert("XDG_DATA_HOME", "LOCALAPPDATA");
        m.insert("XDG_CACHE_HOME", "LOCALAPPDATA");
        m.insert("TMPDIR", "TEMP");
        m.insert("HOSTNAME", "COMPUTERNAME");
        m.insert("PWD", "CD");
        m.insert("SHELL", "COMSPEC");
        m
    };
}

/// Translate environment variable references in a string from one OS format to another.
///
/// Windows uses `%VAR%` syntax, Unix uses `$VAR` or `${VAR}` syntax.
///
/// # Arguments
///
/// * `input` - The string containing environment variable references
/// * `from_os` - The source operating system
/// * `to_os` - The target operating system
///
/// # Returns
///
/// The string with translated environment variable references.
///
/// # Example
///
/// ```
/// use cmdx::{translate_env_vars, Os};
///
/// // Windows to Unix (USERPROFILE maps to HOME)
/// let result = translate_env_vars("cd %USERPROFILE%\\Documents", Os::Windows, Os::Linux);
/// assert_eq!(result, "cd $HOME\\Documents");
///
/// // Unix to Windows (HOME maps to USERPROFILE)
/// let result = translate_env_vars("cd $HOME/Documents", Os::Linux, Os::Windows);
/// assert_eq!(result, "cd %USERPROFILE%/Documents");
/// ```
pub fn translate_env_vars(input: &str, from_os: Os, to_os: Os) -> String {
    // Same OS - no translation needed
    if from_os == to_os {
        return input.to_string();
    }

    // Determine translation direction
    if from_os == Os::Windows && to_os.is_unix_like() {
        translate_windows_to_unix_env(input)
    } else if from_os.is_unix_like() && to_os == Os::Windows {
        translate_unix_to_windows_env(input)
    } else if from_os.is_unix_like() && to_os.is_unix_like() {
        // Unix to Unix - no translation needed
        input.to_string()
    } else {
        input.to_string()
    }
}

/// Translate Windows environment variables to Unix format
fn translate_windows_to_unix_env(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' {
            // Look for closing %
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '%') {
                let end = end + i + 1;
                let var_name: String = chars[i + 1..end].iter().collect();
                
                // Check for known mappings, use original name if not found
                let mapped_name = ENV_VAR_MAPPINGS
                    .get(var_name.to_uppercase().as_str())
                    .copied()
                    .unwrap_or(&var_name);
                
                result.push('$');
                result.push_str(mapped_name);
                i = end + 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Translate Unix environment variables to Windows format
fn translate_unix_to_windows_env(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            // Handle ${VAR} format
            if chars[i + 1] == '{' {
                if let Some(end) = chars[i + 2..].iter().position(|&c| c == '}') {
                    let end = end + i + 2;
                    let var_name: String = chars[i + 2..end].iter().collect();
                    
                    // Check for known mappings, use original name if not found
                    let mapped_name = ENV_VAR_MAPPINGS_REVERSE
                        .get(var_name.to_uppercase().as_str())
                        .copied()
                        .unwrap_or(&var_name);
                    
                    result.push('%');
                    result.push_str(mapped_name);
                    result.push('%');
                    i = end + 1;
                    continue;
                }
            }
            // Handle $VAR format
            else if chars[i + 1].is_alphanumeric() || chars[i + 1] == '_' {
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                    end += 1;
                }
                
                let var_name: String = chars[start..end].iter().collect();
                
                // Check for known mappings, use original name if not found
                let mapped_name = ENV_VAR_MAPPINGS_REVERSE
                    .get(var_name.to_uppercase().as_str())
                    .copied()
                    .unwrap_or(&var_name);
                
                result.push('%');
                result.push_str(mapped_name);
                result.push('%');
                i = end;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Alias for translate_env_vars for convenience.
///
/// This function translates environment variable references in a string
/// from one OS format to another.
#[inline]
pub fn translate_with_env(input: &str, from_os: Os, to_os: Os) -> String {
    translate_env_vars(input, from_os, to_os)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_to_unix_single_var() {
        let result = translate_env_vars("echo %PATH%", Os::Windows, Os::Linux);
        assert_eq!(result, "echo $PATH");
    }

    #[test]
    fn test_windows_to_unix_multiple_vars() {
        let result = translate_env_vars("%USERPROFILE%\\%USERNAME%", Os::Windows, Os::Linux);
        assert_eq!(result, "$HOME\\$USER");
    }

    #[test]
    fn test_windows_to_unix_with_mapping() {
        let result = translate_env_vars("cd %USERPROFILE%", Os::Windows, Os::Linux);
        assert_eq!(result, "cd $HOME");
    }

    #[test]
    fn test_unix_to_windows_dollar_format() {
        let result = translate_env_vars("echo $PATH", Os::Linux, Os::Windows);
        assert_eq!(result, "echo %PATH%");
    }

    #[test]
    fn test_unix_to_windows_braces_format() {
        let result = translate_env_vars("echo ${PATH}", Os::Linux, Os::Windows);
        assert_eq!(result, "echo %PATH%");
    }

    #[test]
    fn test_unix_to_windows_with_mapping() {
        let result = translate_env_vars("cd $HOME", Os::Linux, Os::Windows);
        assert_eq!(result, "cd %USERPROFILE%");
    }

    #[test]
    fn test_same_os_no_change() {
        let result = translate_env_vars("echo %PATH%", Os::Windows, Os::Windows);
        assert_eq!(result, "echo %PATH%");
    }

    #[test]
    fn test_unix_to_unix_no_change() {
        let result = translate_env_vars("echo $PATH", Os::Linux, Os::MacOS);
        assert_eq!(result, "echo $PATH");
    }

    #[test]
    fn test_no_env_vars() {
        let result = translate_env_vars("echo hello world", Os::Windows, Os::Linux);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_mixed_content() {
        let result = translate_env_vars("cd %TEMP% && dir", Os::Windows, Os::Linux);
        assert_eq!(result, "cd $TMPDIR && dir");
    }

    #[test]
    fn test_temp_variable_mapping() {
        let result = translate_env_vars("%TEMP%", Os::Windows, Os::Linux);
        assert_eq!(result, "$TMPDIR");
    }

    #[test]
    fn test_tmpdir_to_temp() {
        let result = translate_env_vars("$TMPDIR", Os::Linux, Os::Windows);
        assert_eq!(result, "%TEMP%");
    }
}
