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

/// Main entry point for the cmdx CLI
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

    // Check if the first argument is a file path (Proton-style usage)
    let first_arg = &args[1];
    if !first_arg.starts_with("--") && !matches!(first_arg.as_str(), "exec" | "shell" | "translate") {
        // This is a script path - auto-detect and run
        let script_path = first_arg;
        let to_os = detect_os();
        
        // Auto-detect source OS from file extension
        let from_os = detect_os_from_script(script_path);
        
        match run_script(script_path, from_os, to_os) {
            Ok(code) => std::process::exit(code),
            Err(e) => {
                eprintln!("Script execution error: {}", e);
                std::process::exit(EXIT_EXECUTION_ERROR);
            }
        }
    }

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

/// Detect source OS from script file extension or shebang
fn detect_os_from_script(script_path: &str) -> Os {
    let path = Path::new(script_path);
    
    // First, try to detect from file extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "bat" | "cmd" | "ps1" => {
                eprintln!("[cmdx] Detected Windows script from extension: .{}", ext_str);
                return Os::Windows;
            }
            "sh" | "bash" | "zsh" => {
                eprintln!("[cmdx] Detected Linux/Unix script from extension: .{}", ext_str);
                return Os::Linux;
            }
            _ => {}
        }
    }
    
    // Try to detect from shebang
    if let Ok(content) = fs::read_to_string(path) {
        let first_line = content.lines().next().unwrap_or("");
        if first_line.starts_with("#!") {
            eprintln!("[cmdx] Detected Linux/Unix script from shebang: {}", first_line);
            return Os::Linux;
        }
        
        // Check for Windows batch markers
        if first_line.starts_with("@echo off") || first_line.starts_with("REM ") {
            eprintln!("[cmdx] Detected Windows batch script from content");
            return Os::Windows;
        }
    }
    
    // Default to current OS if can't detect
    let current = detect_os();
    eprintln!("[cmdx] Could not detect script type, assuming {} script", current);
    current
}

/// Check if a flag exists in arguments
fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

/// Extract command from arguments, skipping flags
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

/// Parse OS argument from command line
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
    println!("    -h, --help              Print this help message");
    println!("    -v, --version           Print version information\n");
    println!("EXAMPLES:");
    println!("    # Proton-style (easiest):");
    println!("    {} install.bat", prog);
    println!("    {} setup.sh", prog);
    println!();
    println!("    # Advanced usage:");
    println!("    {} exec --from windows \"dir /s\"", prog);
    println!("    {} shell --from windows", prog);
    println!("    {} translate --from linux --to windows \"apt install vim\"", prog);
}

/// Translate and print a command
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

/// Execute a translated command
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

/// Run an interactive translation shell
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

/// Execute a shell command using the system shell
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

/// Run a script file with translation
fn run_script(script_path: &str, from_os: Os, to_os: Os) -> Result<i32, Box<dyn std::error::Error>> {
    let path = Path::new(script_path);
    
    if !path.exists() {
        return Err(format!("Script file not found: {}", script_path).into());
    }
    
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    println!("Executing script: {} ({} lines)", script_path, lines.len());
    println!("Translating from {} to {}\n", from_os, to_os);
    
    let mut last_exit_code = EXIT_SUCCESS;
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
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
            println!("[{}] {} [skipped]", line_num + 1, trimmed);
            continue;
        }
        
        // Translate and execute the command
        print!("[{}] {} → ", line_num + 1, trimmed);
        io::stdout().flush()?;
        
        match translate_full(trimmed, from_os, to_os) {
            Ok(result) => {
                println!("{}", result.command);
                
                if !result.warnings.is_empty() {
                    for warning in &result.warnings {
                        eprintln!("     Warning: {}", warning);
                    }
                }
                
                // Execute the translated command
                match execute_shell_command(&result.command) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("     Execution error: {}", e);
                        last_exit_code = EXIT_EXECUTION_ERROR;
                        // Continue executing remaining commands
                    }
                }
            }
            Err(e) => {
                eprintln!("ERROR");
                eprintln!("     Translation error: {}", e);
                last_exit_code = EXIT_TRANSLATION_ERROR;
                // Continue executing remaining commands
            }
        }
    }
    
    println!("\nScript execution completed");
    Ok(last_exit_code)
}
