//! # cmdx - Cross-Platform Command and Path Translator Library
//!
//! A high-performance library for translating shell commands and file paths between
//! different operating systems. Designed for integration into terminal emulators
//! and cross-platform tools.
//!
//! ## Features
//!
//! - **Command Translation**: Translate shell commands with flag support
//! - **Path Translation**: Bidirectional file path translation (Windows ↔ Unix)
//! - **OS Detection**: Runtime detection of the current operating system
//! - **High Performance**: Static lookup tables with lazy initialization
//!
//! ## Command Translation Example
//!
//! ```
//! use cmdx::{translate_command, Os};
//!
//! // Translate a Windows command to Linux
//! let result = translate_command("dir /w", Os::Windows, Os::Linux);
//! assert!(result.is_ok());
//! assert!(result.unwrap().command.contains("ls"));
//!
//! // Translate a Linux command to Windows
//! let result = translate_command("ls -la", Os::Linux, Os::Windows);
//! assert!(result.is_ok());
//! assert!(result.unwrap().command.contains("dir"));
//! ```
//!
//! ## Path Translation Example
//!
//! ```
//! use cmdx::{translate_path, Os};
//!
//! // Windows to Linux path
//! let result = translate_path("C:\\Users\\john\\file.txt", Os::Windows, Os::Linux);
//! assert!(result.is_ok());
//! assert_eq!(result.unwrap().path, "/mnt/c/Users/john/file.txt");
//!
//! // Linux to Windows path
//! let result = translate_path("/mnt/c/Users/john", Os::Linux, Os::Windows);
//! assert!(result.is_ok());
//! assert_eq!(result.unwrap().path, "C:\\Users\\john");
//! ```
//!
//! ## Terminal Emulator Integration
//!
//! ```
//! use cmdx::{translate_command, translate_path, detect_os, Os};
//!
//! // Detect the current OS at runtime
//! let current_os = detect_os();
//!
//! // Translate user input for a different target OS
//! fn process_input(input: &str, target_os: Os) -> String {
//!     let current = cmdx::detect_os();
//!     
//!     // Try command translation first
//!     if let Ok(result) = cmdx::translate_command(input, current, target_os) {
//!         return result.command;
//!     }
//!     
//!     // Fall back to path translation if it looks like a path
//!     if let Ok(result) = cmdx::translate_path(input, current, target_os) {
//!         return result.path;
//!     }
//!     
//!     input.to_string()
//! }
//! ```

pub mod translator;

// Command translation exports
pub use translator::command_map::{CommandMapping, FlagMapping, is_native_command, is_target_command_for_os};
pub use translator::engine::{translate_command, translate_command_str, translate_batch, translate_compound_command, translate_full, translate_script_extension, translate_shebang, TranslationResult, TranslationError};

// Path translation exports
pub use translator::path::{translate_path, translate_path_str, translate_path_auto, translate_paths, PathTranslation, PathError, is_windows_path, is_unix_path};

// Environment variable translation exports
pub use translator::env::{translate_env_vars, translate_with_env};

// OS detection exports
pub use translator::os::{Os, detect_os};
