//! cmdx - Cross-platform command translation layer
//!
//! A translation layer for running scripts from one OS on another, similar to WINE.
//! Automatically translates commands, paths, and environment variables on-the-fly.

use std::env;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
use std::fs;
use std::path::Path;

use cmdx::{translate_full, Os, detect_os, TranslationError};

/// CLI exit codes
const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE_ERROR: i32 = 1;
const EXIT_TRANSLATION_ERROR: i32 = 2;
const EXIT_EXECUTION_ERROR: i32 = 3;

/// CLI configuration flags
#[derive(Debug, Clone)]
struct CliConfig {
    verbose: bool,
    quiet: bool,
    dry_run: bool,
    no_color: bool,
}

impl Default for CliConfig {
    /// Creates a CliConfig with all flags disabled.
    ///
    /// All fields (`verbose`, `quiet`, `dry_run`, `no_color`) are set to `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = CliConfig::default();
    /// assert!(!cfg.verbose && !cfg.quiet && !cfg.dry_run && !cfg.no_color);
    /// ```
    fn default() -> Self {
        Self {
            verbose: false,
            quiet: false,
            dry_run: false,
            no_color: false,
        }
    }
}

/// CLI entry point that parses command-line arguments and dispatches cmdx actions.
///
/// Parses flags and subcommands, auto-detects source/target OS when running a script,
/// and delegates work to the appropriate handler for: running a script, `exec`, `shell`,
/// or `translate`. Exits the process with an appropriate exit code on completion or error.
///
/// # Examples
///
/// ```no_run
/// // Typical invocations (examples — not executed in doctests):
/// // Run a script (auto-detects source OS):
/// // cmdx path/to/script.sh
///
/// // Translate a command from Windows to the current OS:
/// // cmdx translate --from windows "dir C:\\"
///
/// // Start interactive translation shell:
/// // cmdx shell
/// ```
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(EXIT_USAGE_ERROR);
    }

    // Check if first arg is a help/version flag
    match args[1].as_str() {
        "--help" | "-h" => {
            print_usage(&args[0]);
            std::process::exit(EXIT_SUCCESS);
        }
        "--version" | "-v" => {
            println!("cmdx {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(EXIT_SUCCESS);
        }
        _ => {}
    }

    // Parse CLI configuration
    let config = parse_cli_config(&args);
    
    // Find the script path or command (skip flags)
    let mut script_or_cmd: Option<String> = None;
    for arg in &args[1..] {
        if !arg.starts_with("--") && !arg.starts_with("-") && !matches!(arg.as_str(), "exec" | "shell" | "translate") {
            script_or_cmd = Some(arg.clone());
            break;
        }
    }
    
    // Check if we have a script path (Proton-style usage)
    if let Some(script_path) = script_or_cmd {
        if !matches!(script_path.as_str(), "exec" | "shell" | "translate") {
            // This is a script path - auto-detect and run
            let to_os = detect_os();
            
            // Auto-detect source OS from file extension
            let from_os = detect_os_from_script(&script_path, &config);
            
            match run_script_with_config(&script_path, from_os, to_os, &config) {
                Ok(code) => std::process::exit(code),
                Err(e) => {
                    eprintln!("{}", colorize(&format!("Script execution error: {}", e), colors::RED, &config));
                    std::process::exit(EXIT_EXECUTION_ERROR);
                }
            }
        }
    }
    
    // Otherwise handle traditional subcommands
    let first_arg = &args[1];

    // Otherwise, handle traditional subcommand style
    match args[1].as_str() {
        "exec" => {
            let from_os = parse_os_arg(&args, "--from");
            let to_os = detect_os();
            
            let command = extract_command(&args[2..]);
            if command.is_empty() {
                eprintln!("Error: exec requires a command argument");
                print_usage(&args[0]);
                std::process::exit(EXIT_USAGE_ERROR);
            }
            
            match exec_command(&command, from_os, to_os) {
                Ok(code) => std::process::exit(code),
                Err(e) => {
                    eprintln!("Execution error: {}", e);
                    std::process::exit(EXIT_EXECUTION_ERROR);
                }
            }
        }
        "shell" => {
            let from_os = parse_os_arg(&args, "--from");
            let to_os = detect_os();
            
            println!("cmdx interactive shell - translating {} commands to {}", from_os, to_os);
            println!("Type 'exit' or Ctrl+D to quit\n");
            
            if let Err(e) = run_interactive_shell(from_os, to_os) {
                eprintln!("Shell error: {}", e);
                std::process::exit(EXIT_EXECUTION_ERROR);
            }
        }
        "translate" => {
            let from_os = parse_os_arg(&args, "--from");
            let to_os = if has_flag(&args, "--to") {
                parse_os_arg(&args, "--to")
            } else {
                detect_os()
            };
            
            // Find the command after all the flags
            let command = extract_command(&args[2..]);
            if command.is_empty() {
                eprintln!("Error: translate requires a command argument");
                print_usage(&args[0]);
                std::process::exit(EXIT_USAGE_ERROR);
            }
            
            match translate_and_print(&command, from_os, to_os) {
                Ok(_) => std::process::exit(EXIT_SUCCESS),
                Err(e) => {
                    eprintln!("Translation error: {}", e);
                    std::process::exit(EXIT_TRANSLATION_ERROR);
                }
            }
        }
        _ => {
            eprintln!("Error: unknown command or file not found: '{}'", args[1]);
            eprintln!("Use --help for usage information");
            std::process::exit(EXIT_USAGE_ERROR);
        }
    }
}

/// Detects the source operating system of a script file.
///
/// Detection is performed (in priority order) by:
/// 1. File extension (e.g., `.bat`, `.cmd`, `.ps1` → Windows; `.sh`, `.bash`, `.zsh` → Linux).
/// 2. Shebang (`#!`) indicating a Unix-like script.
/// 3. Common Windows batch content markers (e.g., `@echo off`, `REM `).
///
/// When detection succeeds, a notice is printed to stderr unless `config.quiet` is true.
/// If no match is found or the file cannot be read, the host OS returned by `detect_os()` is used
/// as a fallback and a warning is printed (unless `config.quiet`).
///
/// # Parameters
///
/// - `script_path`: filesystem path to the script to inspect.
/// - `config`: CLI configuration controlling verbosity and colorization.
///
/// # Returns
///
/// The `Os` variant representing the detected source operating system.
///
/// # Examples
///
/// ```
/// let cfg = CliConfig::default();
/// let os = detect_os_from_script("example.sh", &cfg);
/// assert_eq!(os, Os::Linux);
/// ```
fn detect_os_from_script(script_path: &str, config: &CliConfig) -> Os {
    let path = Path::new(script_path);
    
    // First, try to detect from file extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "bat" | "cmd" | "ps1" => {
                if !config.quiet {
                    eprintln!("{}", colorize(
                        &format!("[cmdx] Detected Windows script from extension: .{}", ext_str),
                        colors::CYAN,
                        config
                    ));
                }
                return Os::Windows;
            }
            "sh" | "bash" | "zsh" => {
                if !config.quiet {
                    eprintln!("{}", colorize(
                        &format!("[cmdx] Detected Linux/Unix script from extension: .{}", ext_str),
                        colors::CYAN,
                        config
                    ));
                }
                return Os::Linux;
            }
            _ => {}
        }
    }
    
    // Try to detect from shebang
    if let Ok(content) = fs::read_to_string(path) {
        let first_line = content.lines().next().unwrap_or("");
        if first_line.starts_with("#!") {
            if !config.quiet {
                eprintln!("{}", colorize(
                    &format!("[cmdx] Detected Linux/Unix script from shebang: {}", first_line),
                    colors::CYAN,
                    config
                ));
            }
            return Os::Linux;
        }
        
        // Check for Windows batch markers
        if first_line.starts_with("@echo off") || first_line.starts_with("REM ") {
            if !config.quiet {
                eprintln!("{}", colorize("[cmdx] Detected Windows batch script from content", colors::CYAN, config));
            }
            return Os::Windows;
        }
    }
    
    // Default to current OS if can't detect
    let current = detect_os();
    if !config.quiet {
        eprintln!("{}", colorize(
            &format!("[cmdx] Could not detect script type, assuming {} script", current),
            colors::YELLOW,
            config
        ));
    }
    current
}

/// Determines whether the specified flag is present in the argument list.
///
/// `args` is searched for an exact match of `flag`.
///
/// # Examples
///
/// ```
/// let args = vec!["--verbose".to_string(), "file.txt".to_string()];
/// assert!(has_flag(&args, "--verbose"));
/// assert!(!has_flag(&args, "--help"));
/// ```
///
/// # Returns
///
/// `true` if `flag` is found in `args`, `false` otherwise.
fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

/// Builds a CliConfig from command-line arguments and the environment.
///
/// The returned configuration enables flags when their corresponding command-line
/// options are present or when the `NO_COLOR` environment variable is set for
/// `no_color`.
///
/// Flags recognized:
/// - `--verbose` or `-v` → `verbose`
/// - `--quiet` or `-q` → `quiet`
/// - `--dry-run` or `-n` → `dry_run`
/// - `--no-color` or `NO_COLOR` environment variable → `no_color`
///
/// # Examples
///
/// ```
/// let args = vec!["cmd".to_string(), "--verbose".to_string(), "--no-color".to_string()];
/// let cfg = parse_cli_config(&args);
/// assert!(cfg.verbose);
/// assert!(cfg.no_color);
/// assert!(!cfg.quiet);
/// assert!(!cfg.dry_run);
/// ```
fn parse_cli_config(args: &[String]) -> CliConfig {
    CliConfig {
        verbose: has_flag(args, "--verbose") || has_flag(args, "-v"),
        quiet: has_flag(args, "--quiet") || has_flag(args, "-q"),
        dry_run: has_flag(args, "--dry-run") || has_flag(args, "-n"),
        no_color: has_flag(args, "--no-color") || env::var("NO_COLOR").is_ok(),
    }
}

/// ANSI color codes
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const RED: &str = "\x1b[31m";
}

/// Apply ANSI color codes to `text` when color output is enabled in `config`.
///
/// If `config.no_color` is true, the original `text` is returned unchanged; otherwise
/// the returned string is wrapped with `color` at the start and `colors::RESET` at the end.
///
/// # Examples
///
/// ```
/// let cfg = CliConfig::default(); // colors enabled by default
/// let colored = colorize("ok", colors::GREEN, &cfg);
/// assert_eq!(colored, format!("{}{}{}", colors::GREEN, "ok", colors::RESET));
///
/// let mut no_color_cfg = CliConfig::default();
/// no_color_cfg.no_color = true;
/// let plain = colorize("ok", colors::GREEN, &no_color_cfg);
/// assert_eq!(plain, "ok");
/// ```
fn colorize(text: &str, color: &str, config: &CliConfig) -> String {
    if config.no_color {
        text.to_string()
    } else {
        format!("{}{}{}", color, text, colors::RESET)
    }
}

/// Builds a single command string by joining non-flag arguments, skipping flags and their immediate values.
///
/// This scans `args` in order, ignores any argument that starts with `--` and also skips the argument immediately following that flag (treated as the flag's value), then joins the remaining parts with spaces.
///
/// # Arguments
///
/// * `args` - Slice of arguments to process; items starting with `--` and the subsequent item are omitted.
///
/// # Returns
///
/// Joined command string containing the remaining arguments separated by spaces.
///
/// # Examples
///
/// ```
/// let args = vec![
///     "cmdx".to_string(),
///     "--from".to_string(),
///     "windows".to_string(),
///     "echo".to_string(),
///     "hello".to_string(),
/// ];
/// let cmd = extract_command(&args);
/// assert_eq!(cmd, "cmdx echo hello");
/// ```
fn extract_command(args: &[String]) -> String {
    let mut command_parts = Vec::new();
    let mut skip_next = false;
    
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        
        if arg.starts_with("--") {
            // This is a flag, skip it and its value
            skip_next = true;
            continue;
        }
        
        command_parts.push(arg.as_str());
    }
    
    command_parts.join(" ")
}

/// Determine the source OS from command-line arguments or fall back to auto-detection.
///
/// Checks `args` for the provided `flag` and maps the following token to an `Os` variant:
/// - "windows" or "win" → `Os::Windows`
/// - "linux" → `Os::Linux`
/// - "macos", "mac", or "darwin" → `Os::MacOS`
/// - "freebsd" → `Os::FreeBSD`
/// If the flag is present but the value is unrecognized, prints a warning and returns the detected OS. If the flag is not present, returns the detected OS.
///
/// # Parameters
///
/// - `args`: slice of command-line arguments (typically `std::env::args().collect::<Vec<_>>()`).
/// - `flag`: the flag to look for (for example `"--from"`).
///
/// # Examples
///
/// ```
/// let args = vec!["cmdx".to_string(), "--from".to_string(), "windows".to_string()];
/// assert_eq!(parse_os_arg(&args, "--from"), Os::Windows);
/// ```
fn parse_os_arg(args: &[String], flag: &str) -> Os {
    for i in 0..args.len() - 1 {
        if args[i] == flag {
            return match args[i + 1].to_lowercase().as_str() {
                "windows" | "win" => Os::Windows,
                "linux" => Os::Linux,
                "macos" | "mac" | "darwin" => Os::MacOS,
                "freebsd" => Os::FreeBSD,
                _ => {
                    eprintln!("Warning: unknown OS '{}', using detected OS", args[i + 1]);
                    detect_os()
                }
            };
        }
    }
    detect_os()
}

/// Print usage information
fn print_usage(prog: &str) {
    println!("cmdx - Cross-platform command translation layer (like Proton/WINE for scripts)\n");
    println!("USAGE:");
    println!("    {} <script>              Run script with auto-detection (Proton-style)", prog);
    println!("    {} <COMMAND> [OPTIONS]   Advanced usage with explicit options\n", prog);
    println!("PROTON-STYLE (Recommended):");
    println!("    Just run any script and cmdx will auto-detect the source OS and translate:");
    println!("    {} path/to/script.bat    Run Windows batch script on any OS", prog);
    println!("    {} path/to/script.sh     Run Linux shell script on any OS", prog);
    println!("    {} install.ps1           Run PowerShell script on any OS\n", prog);
    println!("ADVANCED COMMANDS:");
    println!("    exec <command>           Execute a command with translation");
    println!("    shell                    Start interactive translation shell");
    println!("    translate <command>      Translate and print command without executing\n");
    println!("OPTIONS:");
    println!("    --from <os>             Source OS (windows, linux, macos)");
    println!("    --to <os>               Target OS (default: auto-detect)");
    println!("    -n, --dry-run           Preview translations without executing");
    println!("    -q, --quiet             Suppress informational output");
    println!("    -v, --verbose           Show detailed translation information");
    println!("    --no-color              Disable colored output");
    println!("    -h, --help              Print this help message");
    println!("    --version               Print version information\n");
    println!("EXAMPLES:");
    println!("    # Proton-style (easiest):");
    println!("    {} install.bat", prog);
    println!("    {} setup.sh", prog);
    println!();
    println!("    # Dry-run mode (preview only):");
    println!("    {} --dry-run install.bat", prog);
    println!();
    println!("    # Advanced usage:");
    println!("    {} exec --from windows \"dir /s\"", prog);
    println!("    {} shell --from windows", prog);
    println!("    {} translate --from linux --to windows \"apt install vim\"", prog);
}

/// Translates a command from one OS to another and prints the original command, the translated command, and any translation warnings.
///
/// On success prints:
/// - "Original [<from_os>]: <command>"
/// - "Translated [<to_os>]: <translated command>"
/// - A "Warnings:" section with each warning on its own line if any warnings were produced.
///
/// # Errors
///
/// Returns `Err(TranslationError)` if translation fails.
///
/// # Examples
///
/// ```
/// // Translate and print a Windows command to the detected host OS.
/// translate_and_print("dir C:\\", Os::Windows, detect_os()).unwrap();
/// ```
fn translate_and_print(command: &str, from_os: Os, to_os: Os) -> Result<(), TranslationError> {
    let result = translate_full(command, from_os, to_os)?;
    
    println!("Original [{}]: {}", from_os, command);
    println!("Translated [{}]: {}", to_os, result.command);
    
    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  - {}", warning);
        }
    }
    
    Ok(())
}

/// Translate a command from one OS to another and run the resulting command.
///
/// On success returns the child process exit code. Translation warnings (if any)
/// are printed to stderr before execution. If the translation produces an empty
/// command or if translation/spawning/waiting fails, an `Err` is returned.
///
/// # Examples
///
/// ```
/// let exit = exec_command("echo hello", Os::Linux, Os::MacOS).unwrap();
/// assert!(exit == 0 || exit > 0);
/// ```
fn exec_command(command: &str, from_os: Os, to_os: Os) -> Result<i32, Box<dyn std::error::Error>> {
    // Translate the command
    let result = translate_full(command, from_os, to_os)?;
    
    if !result.warnings.is_empty() {
        eprintln!("Translation warnings:");
        for warning in &result.warnings {
            eprintln!("  - {}", warning);
        }
    }
    
    // Display what we're executing
    eprintln!("Executing: {}", result.command);
    
    // Parse the command for execution
    let parts: Vec<&str> = result.command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command after translation".into());
    }
    
    let cmd = parts[0];
    let args = &parts[1..];
    
    // Execute the command
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    
    let status = child.wait()?;
    Ok(status.code().unwrap_or(EXIT_EXECUTION_ERROR))
}

/// Start a REPL that reads commands from stdin, translates each line from `from_os` to `to_os`, prints any warnings, and executes the translated command.
///
/// The loop prompts with `cmdx [from→to]>`, ignores empty input, and exits on EOF or when the user enters `exit` or `quit`.
///
/// # Returns
///
/// `Ok(())` when the session ends normally; `Err` if an I/O operation (prompt flush or input read) fails.
///
/// # Examples
///
/// ```
/// // Start an interactive shell translating from Linux to the host OS (example only).
/// // In real use this reads from stdin and blocks until EOF or "exit"/"quit".
/// let _ = run_interactive_shell(Os::Linux, Os::Linux);
/// ```
fn run_interactive_shell(from_os: Os, to_os: Os) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    
    loop {
        // Print prompt
        print!("cmdx [{}→{}]> ", from_os, to_os);
        io::stdout().flush()?;
        
        // Read command
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let command = line.trim();
                
                // Handle special commands
                if command.is_empty() {
                    continue;
                }
                if command == "exit" || command == "quit" {
                    break;
                }
                
                // Translate and execute
                match translate_full(command, from_os, to_os) {
                    Ok(result) => {
                        println!("→ {}", result.command);
                        
                        if !result.warnings.is_empty() {
                            for warning in &result.warnings {
                                eprintln!("Warning: {}", warning);
                            }
                        }
                        
                        // Execute the translated command
                        if let Err(e) = execute_shell_command(&result.command) {
                            eprintln!("Execution error: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Translation error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Input error: {}", e);
                break;
            }
        }
    }
    
    println!("\nGoodbye!");
    Ok(())
}

/// Run a command string through the platform's system shell.
///
/// On Windows this invokes `cmd /C <command>`, on other systems it invokes `sh -c <command>`.
/// The child process inherits stdin, stdout and stderr so interactive programs behave normally.
///
/// # Returns
///
/// `Ok(())` if the command exited with status code 0, `Err` containing an error or the child's exit code otherwise.
///
/// # Examples
///
/// ```
/// let res = execute_shell_command("echo hello");
/// assert!(res.is_ok());
/// ```
fn execute_shell_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    
    let flag = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };
    
    let status = Command::new(shell)
        .arg(flag)
        .arg(command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if !status.success() {
        return Err(format!("Command failed with exit code: {:?}", status.code()).into());
    }
    
    Ok(())
}

/// Execute a script file line-by-line, translating each non-skipped line from `from_os` to `to_os` and optionally running the translated commands according to `config`.
///
/// The function reads the file at `script_path`, skips empty and OS-specific comment/directive lines, translates each remaining line using the translation layer, prints translation and warnings according to `config`, and executes translated commands unless `config.dry_run` is set. It continues processing remaining lines after translation or execution errors and returns a final exit code representing the overall outcome.
///
/// Returns the final exit code: `EXIT_SUCCESS` when no translation or execution errors occurred, `EXIT_TRANSLATION_ERROR` if any line failed to translate, or `EXIT_EXECUTION_ERROR` if any executed command failed. Returns `Err` for file-not-found and I/O errors encountered while reading the script.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use std::io::Write;
/// // Create a small temporary script file
/// let path = "test_script.sh";
/// let mut f = fs::File::create(path).unwrap();
/// writeln!(f, "echo hello").unwrap();
///
/// let config = crate::CliConfig::default();
/// // Assuming `Os::Linux` is available in scope
/// let code = crate::run_script_with_config(path, crate::Os::Linux, crate::Os::Linux, &config).unwrap();
/// assert_eq!(code, crate::EXIT_SUCCESS);
///
/// // Cleanup
/// let _ = fs::remove_file(path);
/// ```
fn run_script_with_config(script_path: &str, from_os: Os, to_os: Os, config: &CliConfig) -> Result<i32, Box<dyn std::error::Error>> {
    let path = Path::new(script_path);
    
    if !path.exists() {
        return Err(format!("Script file not found: {}", script_path).into());
    }
    
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if !config.quiet {
        println!("{}", colorize(
            &format!("Executing script: {} ({} lines)", script_path, lines.len()),
            colors::BOLD,
            config
        ));
        println!("{}", colorize(
            &format!("Translating from {} to {}", from_os, to_os),
            colors::BLUE,
            config
        ));
        if config.dry_run {
            println!("{}", colorize("[DRY RUN MODE - Commands will not be executed]", colors::YELLOW, config));
        }
        println!();
    }
    
    let mut last_exit_code = EXIT_SUCCESS;
    let mut commands_executed = 0;
    let mut commands_skipped = 0;
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }
        
        // Skip comments and special directives based on source OS
        let should_skip = match from_os {
            Os::Windows => {
                trimmed.starts_with("REM ") || 
                trimmed.starts_with("::") ||
                trimmed == "@echo off" ||
                trimmed == "@echo on" ||
                trimmed.starts_with("@REM") ||
                trimmed == "echo." ||
                trimmed == "echo,"
            },
            _ => trimmed.starts_with('#'),
        };
        
        if should_skip {
            if config.verbose {
                println!("{}", colorize(
                    &format!("[{}] {} [skipped]", line_num + 1, trimmed),
                    colors::DIM,
                    config
                ));
            }
            commands_skipped += 1;
            continue;
        }
        
        // Translate and execute the command
        if !config.quiet {
            print!("[{}] {} {} ", 
                line_num + 1, 
                colorize(trimmed, colors::DIM, config),
                colorize("→", colors::BLUE, config)
            );
            io::stdout().flush()?;
        }
        
        match translate_full(trimmed, from_os, to_os) {
            Ok(result) => {
                if !config.quiet {
                    println!("{}", colorize(&result.command, colors::GREEN, config));
                }
                
                if config.verbose && !result.warnings.is_empty() {
                    for warning in &result.warnings {
                        eprintln!("     {}", colorize(&format!("Warning: {}", warning), colors::YELLOW, config));
                    }
                }
                
                // Execute the translated command (unless dry-run)
                if !config.dry_run {
                    match execute_shell_command(&result.command) {
                        Ok(_) => {
                            commands_executed += 1;
                        }
                        Err(e) => {
                            eprintln!("     {}", colorize(&format!("Execution error: {}", e), colors::RED, config));
                            last_exit_code = EXIT_EXECUTION_ERROR;
                            // Continue executing remaining commands
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", colorize("ERROR", colors::RED, config));
                eprintln!("     {}", colorize(&format!("Translation error: {}", e), colors::RED, config));
                last_exit_code = EXIT_TRANSLATION_ERROR;
                // Continue executing remaining commands
            }
        }
    }
    
    if !config.quiet {
        println!();
        println!("{}", colorize(
            &format!("Script execution completed: {} commands executed, {} skipped", 
                commands_executed, commands_skipped),
            colors::BOLD,
            config
        ));
    }
    
    Ok(last_exit_code)
}

/// Run a script file with translation (legacy function for compatibility)
fn run_script(script_path: &str, from_os: Os, to_os: Os) -> Result<i32, Box<dyn std::error::Error>> {
    run_script_with_config(script_path, from_os, to_os, &CliConfig::default())
}