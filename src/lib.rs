//! # cmdx - Advanced Runtime Command Translator
//!
//! A high-performance command translator that converts shell commands between
//! different operating systems (Windows, Linux, macOS, FreeBSD, and more).
//!
//! ## Features
//!
//! - OS detection and automatic command translation
//! - Flag/option translation support
//! - Support for common commands across platforms
//! - High performance using static lookup tables
//!
//! ## Example
//!
//! ```
//! use cmdx::{translate_command, Os};
//!
//! let result = translate_command("ls -la", Os::Windows, Os::Linux);
//! assert!(result.is_err()); // ls is not a Windows command
//!
//! let result = translate_command("dir /w", Os::Windows, Os::Linux);
//! assert!(result.is_ok());
//! assert!(result.unwrap().command.contains("ls"));
//! ```

pub mod translator;

pub use translator::command_map::{CommandMapping, FlagMapping};
pub use translator::os::{Os, detect_os};
pub use translator::engine::{translate_command, translate_command_str, translate_batch, TranslationResult, TranslationError};
