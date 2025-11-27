//! Path translation module - converts file paths between operating systems
//!
//! This module provides bidirectional path translation between Windows and Unix-like
//! operating systems, handling path separators, drive letters, and common path mappings.

use super::os::Os;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of a path translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathTranslation {
    /// The translated path
    pub path: String,
    /// Original path
    pub original: String,
    /// Source OS
    pub from_os: Os,
    /// Target OS
    pub to_os: Os,
    /// Whether drive letter was translated
    pub drive_translated: bool,
    /// Warnings about the translation
    pub warnings: Vec<String>,
}

impl PathTranslation {
    pub fn new(path: String, original: String, from_os: Os, to_os: Os) -> Self {
        Self {
            path,
            original,
            from_os,
            to_os,
            drive_translated: false,
            warnings: Vec::new(),
        }
    }
}

impl fmt::Display for PathTranslation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// Errors that can occur during path translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathError {
    /// Empty path
    EmptyPath,
    /// Invalid path format
    InvalidPath(String),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::EmptyPath => write!(f, "Empty path provided"),
            PathError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
        }
    }
}

impl std::error::Error for PathError {}

/// Common drive letter to Unix path mappings
fn get_drive_mapping(drive: char) -> String {
    // Use lowercase for the mount point (WSL convention)
    format!("/mnt/{}", drive.to_ascii_lowercase())
}

/// Check if a path looks like a Windows path
pub fn is_windows_path(path: &str) -> bool {
    // Check for drive letter (e.g., C:\, D:/)
    if path.len() >= 2 {
        let chars: Vec<char> = path.chars().collect();
        if chars[0].is_ascii_alphabetic() && chars[1] == ':' {
            return true;
        }
    }
    // Check for UNC paths (\\server\share)
    if path.starts_with("\\\\") {
        return true;
    }
    // Check for backslashes
    path.contains('\\')
}

/// Check if a path looks like a Unix path
pub fn is_unix_path(path: &str) -> bool {
    path.starts_with('/') || path.starts_with("~/") || path.starts_with("./") || path.starts_with("../")
}

/// Translate a Windows path to Unix path
fn windows_to_unix(path: &str, result: &mut PathTranslation) -> String {
    let mut unix_path = path.to_string();
    
    // Handle drive letter (C:\Users -> /mnt/c/Users)
    if unix_path.len() >= 2 {
        let chars: Vec<char> = unix_path.chars().collect();
        if chars[0].is_ascii_alphabetic() && chars[1] == ':' {
            let drive = chars[0];
            let mount_point = get_drive_mapping(drive);
            unix_path = format!("{}{}", mount_point, &unix_path[2..]);
            result.drive_translated = true;
        }
    }
    
    // Handle UNC paths (\\server\share -> //server/share or /mnt/server/share)
    if unix_path.starts_with("\\\\") {
        unix_path = unix_path.replacen("\\\\", "//", 1);
        result.warnings.push("UNC path converted to network path format".to_string());
    }
    
    // Convert backslashes to forward slashes
    unix_path = unix_path.replace('\\', "/");
    
    // Normalize multiple slashes (except leading // for network paths)
    if unix_path.starts_with("//") {
        let rest = unix_path[2..].split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("/");
        unix_path = format!("//{}", rest);
    } else {
        let parts: Vec<_> = unix_path.split('/').filter(|s| !s.is_empty()).collect();
        unix_path = if path.starts_with('/') || path.starts_with('\\') || path.chars().nth(1) == Some(':') {
            format!("/{}", parts.join("/"))
        } else {
            parts.join("/")
        };
    }
    
    unix_path
}

/// Translate a Unix path to Windows path
fn unix_to_windows(path: &str, result: &mut PathTranslation) -> String {
    let mut windows_path = path.to_string();
    
    // Handle /mnt/X/ paths (convert to X:\)
    if windows_path.starts_with("/mnt/") && windows_path.len() >= 6 {
        let drive_char = windows_path.chars().nth(5);
        if let Some(drive) = drive_char {
            if drive.is_ascii_alphabetic() {
                // Check if it's followed by / or end of string
                let after_drive = windows_path.chars().nth(6);
                if after_drive.is_none() || after_drive == Some('/') {
                    windows_path = format!("{}:{}", drive.to_ascii_uppercase(), &windows_path[6..]);
                    result.drive_translated = true;
                }
            }
        }
    }
    // Handle /home/username -> C:\Users\username (common mapping)
    else if windows_path.starts_with("/home/") {
        windows_path = format!("C:\\Users{}", &windows_path[5..]);
        result.drive_translated = true;
        result.warnings.push("/home mapped to C:\\Users".to_string());
    }
    // Handle ~ (home directory)
    else if windows_path.starts_with("~/") {
        windows_path = format!("%USERPROFILE%{}", &windows_path[1..]);
        result.warnings.push("~ translated to %USERPROFILE%".to_string());
    }
    else if windows_path == "~" {
        windows_path = "%USERPROFILE%".to_string();
        result.warnings.push("~ translated to %USERPROFILE%".to_string());
    }
    // Handle root paths
    else if windows_path.starts_with('/') && !windows_path.starts_with("//") {
        // Generic Unix root -> C:\
        windows_path = format!("C:{}", windows_path);
        result.drive_translated = true;
        result.warnings.push("Root path mapped to C: drive".to_string());
    }
    // Handle network paths (//server/share -> \\server\share)
    else if windows_path.starts_with("//") {
        windows_path = windows_path.replacen("//", "\\\\", 1);
    }
    
    // Convert forward slashes to backslashes
    windows_path = windows_path.replace('/', "\\");
    
    // Normalize multiple backslashes (but keep UNC prefix)
    if windows_path.starts_with("\\\\") {
        let rest: Vec<_> = windows_path[2..].split('\\').filter(|s| !s.is_empty()).collect();
        windows_path = format!("\\\\{}", rest.join("\\"));
    } else {
        let parts: Vec<_> = windows_path.split('\\').filter(|s| !s.is_empty()).collect();
        windows_path = parts.join("\\");
    }
    
    windows_path
}

/// Translate a file path between operating systems
///
/// # Arguments
///
/// * `path` - The path to translate
/// * `from_os` - The source operating system
/// * `to_os` - The target operating system
///
/// # Returns
///
/// * `Ok(PathTranslation)` - The translated path
/// * `Err(PathError)` - Error if translation fails
///
/// # Example
///
/// ```
/// use cmdx::{translate_path, Os};
///
/// // Windows to Linux
/// let result = translate_path("C:\\Users\\john\\file.txt", Os::Windows, Os::Linux);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().path, "/mnt/c/Users/john/file.txt");
///
/// // Linux to Windows
/// let result = translate_path("/mnt/c/Users/john/file.txt", Os::Linux, Os::Windows);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().path, "C:\\Users\\john\\file.txt");
/// ```
pub fn translate_path(
    path: &str,
    from_os: Os,
    to_os: Os,
) -> Result<PathTranslation, PathError> {
    if path.trim().is_empty() {
        return Err(PathError::EmptyPath);
    }
    
    let path = path.trim();
    
    // Same OS - just return normalized path
    if from_os == to_os {
        return Ok(PathTranslation::new(
            path.to_string(),
            path.to_string(),
            from_os,
            to_os,
        ));
    }
    
    let mut result = PathTranslation::new(
        String::new(),
        path.to_string(),
        from_os,
        to_os,
    );
    
    // Determine translation direction based on OS types
    let translated = if from_os == Os::Windows && to_os.is_unix_like() {
        // Windows -> Unix
        windows_to_unix(path, &mut result)
    } else if from_os.is_unix_like() && to_os == Os::Windows {
        // Unix -> Windows
        unix_to_windows(path, &mut result)
    } else if from_os.is_unix_like() && to_os.is_unix_like() {
        // Unix -> Unix (just normalize)
        path.to_string()
    } else {
        // Fallback: try to auto-detect and convert
        if is_windows_path(path) {
            windows_to_unix(path, &mut result)
        } else {
            unix_to_windows(path, &mut result)
        }
    };
    
    result.path = translated;
    Ok(result)
}

/// Translate a path with string OS names
pub fn translate_path_str(
    path: &str,
    from_os: &str,
    to_os: &str,
) -> Result<PathTranslation, PathError> {
    let from = Os::parse(from_os)
        .ok_or_else(|| PathError::InvalidPath(format!("Unknown OS: {}", from_os)))?;
    let to = Os::parse(to_os)
        .ok_or_else(|| PathError::InvalidPath(format!("Unknown OS: {}", to_os)))?;
    
    translate_path(path, from, to)
}

/// Auto-detect the path format and translate to the target OS
///
/// # Example
///
/// ```
/// use cmdx::{translate_path_auto, Os};
///
/// // Auto-detects Windows path and converts to Linux
/// let result = translate_path_auto("C:\\Users\\john", Os::Linux);
/// assert!(result.is_ok());
/// ```
pub fn translate_path_auto(
    path: &str,
    to_os: Os,
) -> Result<PathTranslation, PathError> {
    if path.trim().is_empty() {
        return Err(PathError::EmptyPath);
    }
    
    // Auto-detect source OS
    let from_os = if is_windows_path(path) {
        Os::Windows
    } else {
        Os::Linux // Default to Linux for Unix-like paths
    };
    
    translate_path(path, from_os, to_os)
}

/// Batch translate multiple paths
pub fn translate_paths(
    paths: &[&str],
    from_os: Os,
    to_os: Os,
) -> Vec<Result<PathTranslation, PathError>> {
    paths
        .iter()
        .map(|path| translate_path(path, from_os, to_os))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_windows_path() {
        assert!(is_windows_path("C:\\Users\\john"));
        assert!(is_windows_path("D:/Documents"));
        assert!(is_windows_path("\\\\server\\share"));
        assert!(is_windows_path("folder\\file.txt"));
        assert!(!is_windows_path("/home/john"));
        assert!(!is_windows_path("./file.txt"));
    }

    #[test]
    fn test_is_unix_path() {
        assert!(is_unix_path("/home/john"));
        assert!(is_unix_path("~/Documents"));
        assert!(is_unix_path("./file.txt"));
        assert!(is_unix_path("../parent/file"));
        assert!(!is_unix_path("C:\\Users"));
    }

    #[test]
    fn test_windows_to_linux_basic() {
        let result = translate_path("C:\\Users\\john\\file.txt", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "/mnt/c/Users/john/file.txt");
        assert!(result.drive_translated);
    }

    #[test]
    fn test_windows_to_linux_drive_d() {
        let result = translate_path("D:\\Documents\\report.pdf", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "/mnt/d/Documents/report.pdf");
    }

    #[test]
    fn test_linux_to_windows_mnt() {
        let result = translate_path("/mnt/c/Users/john/file.txt", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "C:\\Users\\john\\file.txt");
        assert!(result.drive_translated);
    }

    #[test]
    fn test_linux_to_windows_home() {
        let result = translate_path("/home/john/Documents", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "C:\\Users\\john\\Documents");
    }

    #[test]
    fn test_linux_to_windows_tilde() {
        let result = translate_path("~/Documents", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "%USERPROFILE%\\Documents");
    }

    #[test]
    fn test_linux_to_windows_root() {
        let result = translate_path("/etc/config", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "C:\\etc\\config");
    }

    #[test]
    fn test_unc_path_to_unix() {
        let result = translate_path("\\\\server\\share\\file.txt", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "//server/share/file.txt");
    }

    #[test]
    fn test_network_path_to_windows() {
        let result = translate_path("//server/share/file.txt", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.path, "\\\\server\\share\\file.txt");
    }

    #[test]
    fn test_same_os_passthrough() {
        let result = translate_path("/home/john", Os::Linux, Os::Linux);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().path, "/home/john");
    }

    #[test]
    fn test_empty_path_error() {
        let result = translate_path("", Os::Windows, Os::Linux);
        assert!(result.is_err());
    }

    #[test]
    fn test_translate_path_str() {
        let result = translate_path_str("C:\\Users", "windows", "linux");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().path, "/mnt/c/Users");
    }

    #[test]
    fn test_translate_path_auto() {
        let result = translate_path_auto("C:\\Users\\john", Os::Linux);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().path, "/mnt/c/Users/john");
    }

    #[test]
    fn test_translate_paths_batch() {
        let paths = vec!["C:\\Users", "D:\\Documents"];
        let results = translate_paths(&paths, Os::Windows, Os::Linux);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_unix_to_unix() {
        let result = translate_path("/home/john", Os::Linux, Os::MacOS);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().path, "/home/john");
    }

    #[test]
    fn test_macos_to_windows() {
        let result = translate_path("/Users/john/Documents", Os::MacOS, Os::Windows);
        assert!(result.is_ok());
        // macOS /Users maps to C:\Users on Windows
        assert!(result.unwrap().path.contains("Users"));
    }
}
