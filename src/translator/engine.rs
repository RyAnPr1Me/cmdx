//! Translation engine - core logic for translating commands between operating systems

use std::fmt;

use super::command_map::{get_mapping, CommandMapping};
use super::os::Os;

/// Result of a command translation
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
}
