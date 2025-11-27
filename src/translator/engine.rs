//! Translation engine - core logic for translating commands between operating systems

use serde::{Deserialize, Serialize};
use std::fmt;

use super::command_map::{get_mapping, is_native_command, is_target_command_for_os, CommandMapping};
use super::os::Os;

/// Result of a command translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    /// The translated command
    pub command: String,
    /// Original command
    pub original: String,
    /// Source OS
    pub from_os: Os,
    /// Target OS
    pub to_os: Os,
    /// Warnings or notes about the translation
    pub warnings: Vec<String>,
    /// Whether any flags couldn't be translated
    pub had_unmapped_flags: bool,
}

impl TranslationResult {
    pub fn new(command: String, original: String, from_os: Os, to_os: Os) -> Self {
        Self {
            command,
            original,
            from_os,
            to_os,
            warnings: Vec::new(),
            had_unmapped_flags: false,
        }
    }
}

impl fmt::Display for TranslationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.command)
    }
}

/// Errors that can occur during translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationError {
    /// Command not found in mapping
    CommandNotFound(String),
    /// Empty command
    EmptyCommand,
    /// Invalid source or target OS
    InvalidOs(String),
    /// Same source and target OS
    SameOs,
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranslationError::CommandNotFound(cmd) => {
                write!(f, "No translation found for command '{}'", cmd)
            }
            TranslationError::EmptyCommand => {
                write!(f, "Empty command provided")
            }
            TranslationError::InvalidOs(os) => {
                write!(f, "Invalid operating system: '{}'", os)
            }
            TranslationError::SameOs => {
                write!(f, "Source and target OS are the same")
            }
        }
    }
}

impl std::error::Error for TranslationError {}

/// Parse a command string into command name and arguments
fn parse_command(input: &str) -> (String, Vec<String>) {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    
    let command = parts[0].to_lowercase();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    
    (command, args)
}

/// Translate flags from source to target OS
fn translate_flags(
    args: &[String],
    mapping: &CommandMapping,
    result: &mut TranslationResult,
) -> Vec<String> {
    let mut translated_args = Vec::new();
    
    for arg in args {
        let mut found = false;
        
        // Check if this is a flag that needs translation
        for flag_mapping in &mapping.flag_mappings {
            // Handle exact match
            if arg == &flag_mapping.source || arg.to_lowercase() == flag_mapping.source.to_lowercase() {
                if !flag_mapping.target.is_empty() {
                    // Handle cases where target contains multiple flags
                    for part in flag_mapping.target.split_whitespace() {
                        translated_args.push(part.to_string());
                    }
                }
                found = true;
                break;
            }
            
            // Handle flags with values (e.g., -n 5 or /n:5)
            if arg.starts_with(&flag_mapping.source) {
                let value = &arg[flag_mapping.source.len()..];
                if !flag_mapping.target.is_empty() {
                    if value.is_empty() {
                        translated_args.push(flag_mapping.target.clone());
                    } else {
                        // Handle different flag value formats
                        let value_clean = value.trim_start_matches(':').trim_start_matches('=');
                        translated_args.push(format!("{} {}", flag_mapping.target, value_clean));
                    }
                }
                found = true;
                break;
            }
        }
        
        // If flag wasn't found in mappings
        if !found {
            if mapping.preserve_unmapped_flags {
                // Keep the original arg
                translated_args.push(arg.clone());
                
                // Warn about unmapped flags
                if arg.starts_with('-') || arg.starts_with('/') {
                    result.warnings.push(format!("Flag '{}' was not translated", arg));
                    result.had_unmapped_flags = true;
                }
            } else {
                result.warnings.push(format!("Flag '{}' was dropped", arg));
                result.had_unmapped_flags = true;
            }
        }
    }
    
    translated_args
}

/// Translate a command from one OS to another
///
/// # Arguments
///
/// * `input` - The command string to translate
/// * `from_os` - The source operating system
/// * `to_os` - The target operating system
///
/// # Returns
///
/// * `Ok(TranslationResult)` - The translated command
/// * `Err(TranslationError)` - Error if translation fails
///
/// # Example
///
/// ```
/// use cmdx::{translate_command, Os};
///
/// let result = translate_command("dir /w", Os::Windows, Os::Linux);
/// assert!(result.is_ok());
/// println!("{}", result.unwrap());
/// ```
pub fn translate_command(
    input: &str,
    from_os: Os,
    to_os: Os,
) -> Result<TranslationResult, TranslationError> {
    // Check for empty input
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TranslationError::EmptyCommand);
    }
    
    // Same OS - just return the input
    if from_os == to_os {
        return Ok(TranslationResult::new(
            trimmed.to_string(),
            trimmed.to_string(),
            from_os,
            to_os,
        ));
    }
    
    // Parse the command
    let (command_name, args) = parse_command(trimmed);
    
    if command_name.is_empty() {
        return Err(TranslationError::EmptyCommand);
    }
    
    // Check if the command is already native to the target OS
    // If so, pass it through without transformation
    if is_native_command(&command_name, to_os) && !is_native_command(&command_name, from_os) {
        // Command is already in target OS format, pass through
        let mut result = TranslationResult::new(
            trimmed.to_string(),
            trimmed.to_string(),
            from_os,
            to_os,
        );
        result.warnings.push(format!(
            "Command '{}' is already in {} format, passed through unchanged",
            command_name, to_os
        ));
        return Ok(result);
    }
    
    // Check if the command is native to the target OS (same command on both)
    // For example, 'ping' exists on both Windows and Linux
    if is_native_command(&command_name, to_os) && is_native_command(&command_name, from_os) {
        // Command exists on both OSes - check if we have flag translations
        if let Some(mapping) = get_mapping(&command_name, from_os, to_os) {
            // We have flag mappings, so translate the flags
            let mut result = TranslationResult::new(
                String::new(),
                trimmed.to_string(),
                from_os,
                to_os,
            );
            
            let translated_args = translate_flags(&args, mapping, &mut result);
            
            let mut final_command = mapping.target_cmd.clone();
            if !translated_args.is_empty() {
                final_command.push(' ');
                final_command.push_str(&translated_args.join(" "));
            }
            
            result.command = final_command;
            return Ok(result);
        } else {
            // No flag mappings, pass through unchanged
            return Ok(TranslationResult::new(
                trimmed.to_string(),
                trimmed.to_string(),
                from_os,
                to_os,
            ));
        }
    }
    
    // Look up the mapping
    let mapping = match get_mapping(&command_name, from_os, to_os) {
        Some(m) => m,
        None => {
            // Try to find a generic Unix-like mapping if both are Unix-like
            if from_os.is_unix_like() && to_os.is_unix_like() {
                // Unix commands are generally compatible
                let mut result = TranslationResult::new(
                    trimmed.to_string(),
                    trimmed.to_string(),
                    from_os,
                    to_os,
                );
                result.warnings.push(format!(
                    "Command '{}' passed through (Unix-like OS compatibility assumed)",
                    command_name
                ));
                return Ok(result);
            }
            
            // Check if command is already a target OS command
            if is_target_command_for_os(&command_name, to_os) {
                let mut result = TranslationResult::new(
                    trimmed.to_string(),
                    trimmed.to_string(),
                    from_os,
                    to_os,
                );
                result.warnings.push(format!(
                    "Command '{}' appears to already be a {} command, passed through unchanged",
                    command_name, to_os
                ));
                return Ok(result);
            }
            
            return Err(TranslationError::CommandNotFound(command_name));
        }
    };
    
    // Create result
    let mut result = TranslationResult::new(
        String::new(),
        trimmed.to_string(),
        from_os,
        to_os,
    );
    
    // Translate flags
    let translated_args = translate_flags(&args, mapping, &mut result);
    
    // Build the final command
    let mut final_command = mapping.target_cmd.clone();
    
    if !translated_args.is_empty() {
        final_command.push(' ');
        final_command.push_str(&translated_args.join(" "));
    }
    
    result.command = final_command;
    
    // Add notes from mapping if any
    if let Some(notes) = &mapping.notes {
        result.warnings.push(notes.clone());
    }
    
    Ok(result)
}

/// Translate a command with string OS names
pub fn translate_command_str(
    input: &str,
    from_os: &str,
    to_os: &str,
) -> Result<TranslationResult, TranslationError> {
    let from = Os::parse(from_os)
        .ok_or_else(|| TranslationError::InvalidOs(from_os.to_string()))?;
    let to = Os::parse(to_os)
        .ok_or_else(|| TranslationError::InvalidOs(to_os.to_string()))?;
    
    translate_command(input, from, to)
}

/// Batch translate multiple commands
pub fn translate_batch(
    commands: &[&str],
    from_os: Os,
    to_os: Os,
) -> Vec<Result<TranslationResult, TranslationError>> {
    commands
        .iter()
        .map(|cmd| translate_command(cmd, from_os, to_os))
        .collect()
}

/// Operators used in compound commands
const COMPOUND_OPERATORS: &[&str] = &["&&", "||", ";", "|"];

/// Translate a compound command containing operators like `&&`, `||`, `;`, or `|`
///
/// This function splits the input by operators, translates each command individually,
/// and then joins them back together.
///
/// # Arguments
///
/// * `input` - The compound command string to translate
/// * `from_os` - The source operating system
/// * `to_os` - The target operating system
///
/// # Returns
///
/// * `Ok(TranslationResult)` - The translated compound command
/// * `Err(TranslationError)` - Error if any command translation fails
///
/// # Example
///
/// ```
/// use cmdx::{translate_compound_command, Os};
///
/// let result = translate_compound_command("dir && cls", Os::Windows, Os::Linux);
/// assert!(result.is_ok());
/// let result = result.unwrap();
/// assert!(result.command.contains("ls"));
/// assert!(result.command.contains("clear"));
/// ```
pub fn translate_compound_command(
    input: &str,
    from_os: Os,
    to_os: Os,
) -> Result<TranslationResult, TranslationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TranslationError::EmptyCommand);
    }

    // Same OS - just return the input
    if from_os == to_os {
        return Ok(TranslationResult::new(
            trimmed.to_string(),
            trimmed.to_string(),
            from_os,
            to_os,
        ));
    }

    // Split the command by operators while preserving the operators
    let parts = split_compound_command(trimmed);
    
    // If there's only one part, use regular translation
    if parts.len() == 1 {
        return translate_command(trimmed, from_os, to_os);
    }

    let mut result = TranslationResult::new(
        String::new(),
        trimmed.to_string(),
        from_os,
        to_os,
    );

    let mut translated_parts = Vec::new();
    
    for part in &parts {
        let trimmed_part = part.trim();
        
        // Check if this part is an operator
        if COMPOUND_OPERATORS.contains(&trimmed_part) {
            translated_parts.push(trimmed_part.to_string());
        } else if !trimmed_part.is_empty() {
            // Translate the command
            match translate_command(trimmed_part, from_os, to_os) {
                Ok(cmd_result) => {
                    translated_parts.push(cmd_result.command);
                    // Collect warnings
                    result.warnings.extend(cmd_result.warnings);
                    result.had_unmapped_flags |= cmd_result.had_unmapped_flags;
                }
                Err(TranslationError::CommandNotFound(_)) => {
                    // Keep original command if not found (might be a custom/unknown command)
                    translated_parts.push(trimmed_part.to_string());
                    result.warnings.push(format!("Command '{}' was not translated", trimmed_part.split_whitespace().next().unwrap_or(trimmed_part)));
                }
                Err(e) => return Err(e),
            }
        }
    }

    result.command = translated_parts.join(" ");
    Ok(result)
}

/// Split a compound command by operators while preserving the operators
fn split_compound_command(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for two-character operators first
        if i + 1 < chars.len() {
            let two_char = format!("{}{}", chars[i], chars[i + 1]);
            if two_char == "&&" || two_char == "||" {
                if !current.is_empty() {
                    parts.push(current);
                    current = String::new();
                }
                parts.push(two_char);
                i += 2;
                continue;
            }
        }
        
        // Check for single-character operators
        if chars[i] == '|' || chars[i] == ';' {
            if !current.is_empty() {
                parts.push(current);
                current = String::new();
            }
            parts.push(chars[i].to_string());
            i += 1;
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let (cmd, args) = parse_command("ls -la /home");
        assert_eq!(cmd, "ls");
        assert_eq!(args, vec!["-la", "/home"]);
    }

    #[test]
    fn test_parse_command_no_args() {
        let (cmd, args) = parse_command("ls");
        assert_eq!(cmd, "ls");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_command_empty() {
        let (cmd, args) = parse_command("");
        assert!(cmd.is_empty());
        assert!(args.is_empty());
    }

    #[test]
    fn test_translate_dir_to_ls() {
        let result = translate_command("dir", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "ls");
    }

    #[test]
    fn test_translate_dir_with_flags() {
        let result = translate_command("dir /w", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("ls"));
        assert!(result.command.contains("-C"));
    }

    #[test]
    fn test_translate_ls_to_dir() {
        let result = translate_command("ls", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "dir");
    }

    #[test]
    fn test_translate_ls_with_flags() {
        let result = translate_command("ls -la", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("dir"));
    }

    #[test]
    fn test_translate_copy_to_cp() {
        let result = translate_command("copy /y", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("cp"));
        assert!(result.command.contains("-f"));
    }

    #[test]
    fn test_translate_cls_to_clear() {
        let result = translate_command("cls", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "clear");
    }

    #[test]
    fn test_translate_clear_to_cls() {
        let result = translate_command("clear", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "cls");
    }

    #[test]
    fn test_translate_grep_to_findstr() {
        let result = translate_command("grep -i pattern", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("findstr"));
        assert!(result.command.contains("/i"));
    }

    #[test]
    fn test_translate_findstr_to_grep() {
        let result = translate_command("findstr /i pattern", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("grep"));
        assert!(result.command.contains("-i"));
    }

    #[test]
    fn test_translate_same_os() {
        let result = translate_command("ls -la", Os::Linux, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "ls -la");
    }

    #[test]
    fn test_translate_empty_command() {
        let result = translate_command("", Os::Windows, Os::Linux);
        assert!(result.is_err());
        match result {
            Err(TranslationError::EmptyCommand) => {}
            _ => panic!("Expected EmptyCommand error"),
        }
    }

    #[test]
    fn test_translate_command_not_found() {
        let result = translate_command("nonexistent", Os::Windows, Os::Linux);
        assert!(result.is_err());
        match result {
            Err(TranslationError::CommandNotFound(_)) => {}
            _ => panic!("Expected CommandNotFound error"),
        }
    }

    #[test]
    fn test_translate_command_str() {
        let result = translate_command_str("dir", "windows", "linux");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().command, "ls");
    }

    #[test]
    fn test_translate_command_str_invalid_os() {
        let result = translate_command_str("dir", "invalid", "linux");
        assert!(result.is_err());
        match result {
            Err(TranslationError::InvalidOs(_)) => {}
            _ => panic!("Expected InvalidOs error"),
        }
    }

    #[test]
    fn test_translate_batch() {
        let commands = vec!["dir", "cls", "copy"];
        let results = translate_batch(&commands, Os::Windows, Os::Linux);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_unix_to_unix_passthrough() {
        let result = translate_command("some_unix_cmd", Os::Linux, Os::MacOS);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "some_unix_cmd");
    }

    #[test]
    fn test_translate_tasklist_to_ps() {
        let result = translate_command("tasklist", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("ps"));
    }

    #[test]
    fn test_translate_ps_to_tasklist() {
        let result = translate_command("ps", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("tasklist"));
    }

    #[test]
    fn test_translate_ping_flags() {
        let result = translate_command("ping -n 5 localhost", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("ping"));
        assert!(result.command.contains("-c"));
    }

    #[test]
    fn test_compound_command_and() {
        let result = translate_compound_command("dir && cls", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("ls"));
        assert!(result.command.contains("&&"));
        assert!(result.command.contains("clear"));
    }

    #[test]
    fn test_compound_command_or() {
        let result = translate_compound_command("dir || cls", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("ls"));
        assert!(result.command.contains("||"));
        assert!(result.command.contains("clear"));
    }

    #[test]
    fn test_compound_command_pipe() {
        let result = translate_compound_command("dir | findstr test", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("ls"));
        assert!(result.command.contains("|"));
        assert!(result.command.contains("grep"));
    }

    #[test]
    fn test_compound_command_semicolon() {
        let result = translate_compound_command("ls; clear", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("dir"));
        assert!(result.command.contains(";"));
        assert!(result.command.contains("cls"));
    }

    #[test]
    fn test_compound_command_single() {
        let result = translate_compound_command("dir", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().command, "ls");
    }

    #[test]
    fn test_split_compound_command() {
        let parts = split_compound_command("dir && cls || type");
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].trim(), "dir");
        assert_eq!(parts[1], "&&");
        assert_eq!(parts[2].trim(), "cls");
        assert_eq!(parts[3], "||");
        assert_eq!(parts[4].trim(), "type");
    }

    #[test]
    fn test_native_command_passthrough() {
        // If we're translating from Linux to Windows, but the command is already
        // a Windows command (like 'dir'), it should pass through unchanged
        let result = translate_command("dir", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "dir");
        assert!(result.warnings.iter().any(|w| w.contains("already")));
    }

    #[test]
    fn test_native_command_passthrough_with_flags() {
        // Windows command with Windows flags should pass through
        let result = translate_command("dir /w", Os::Linux, Os::Windows);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "dir /w");
    }

    #[test]
    fn test_native_unix_command_passthrough_to_linux() {
        // If we're translating from Windows to Linux, but the command is already
        // a Linux command (like 'ls'), it should pass through unchanged
        let result = translate_command("ls", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "ls");
        assert!(result.warnings.iter().any(|w| w.contains("already")));
    }

    #[test]
    fn test_native_unix_command_passthrough_with_flags() {
        // Unix command with Unix flags should pass through
        let result = translate_command("ls -la", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.command, "ls -la");
    }

    #[test]
    fn test_common_command_with_different_flags() {
        // ping exists on both OSes but has different flag syntax
        // When translating from Windows to Linux, flags should be translated
        let result = translate_command("ping -n 5 localhost", Os::Windows, Os::Linux);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.command.contains("ping"));
        assert!(result.command.contains("-c")); // -n becomes -c
    }
}
