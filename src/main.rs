//! cmdx - Advanced Runtime Command Translator
//!
//! A CLI tool for translating shell commands between different operating systems.

use clap::{Parser, Subcommand, ValueEnum};
use cmdx::{detect_os, translate_command, Os};
use std::io::{self, BufRead, Write};

/// Advanced runtime command translator for cross-platform shell commands
#[derive(Parser, Debug)]
#[command(name = "cmdx")]
#[command(author = "cmdx contributors")]
#[command(version)]
#[command(about = "Translate shell commands between Windows, Linux, macOS, and more")]
#[command(long_about = "cmdx is a high-performance command translator that converts shell commands \
between different operating systems. It supports Windows, Linux, macOS, FreeBSD, \
OpenBSD, NetBSD, Solaris, and more.\n\n\
Examples:\n\
  cmdx translate \"dir /w\" --from windows --to linux\n\
  cmdx translate \"ls -la\" --from linux --to windows\n\
  echo \"dir\" | cmdx translate --from windows --to linux")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Translate a command from one OS to another
    #[command(visible_alias = "t")]
    Translate {
        /// The command to translate (can also be piped via stdin)
        command: Option<String>,
        
        /// Source operating system
        #[arg(short, long, value_enum)]
        from: OsArg,
        
        /// Target operating system
        #[arg(short, long, value_enum)]
        to: OsArg,
        
        /// Show warnings and notes about the translation
        #[arg(short, long)]
        verbose: bool,
        
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    
    /// List all supported commands for a translation
    #[command(visible_alias = "ls")]
    List {
        /// Source operating system
        #[arg(short, long, value_enum)]
        from: OsArg,
        
        /// Target operating system
        #[arg(short, long, value_enum)]
        to: OsArg,
    },
    
    /// Detect the current operating system
    Detect,
    
    /// List all supported operating systems
    Os,
    
    /// Interactive mode - continuously translate commands
    #[command(visible_alias = "i")]
    Interactive {
        /// Source operating system
        #[arg(short, long, value_enum)]
        from: OsArg,
        
        /// Target operating system
        #[arg(short, long, value_enum)]
        to: OsArg,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OsArg {
    Windows,
    Linux,
    #[value(alias = "mac")]
    Macos,
    Freebsd,
    Openbsd,
    Netbsd,
    Solaris,
    Android,
    Ios,
}

impl From<OsArg> for Os {
    fn from(arg: OsArg) -> Self {
        match arg {
            OsArg::Windows => Os::Windows,
            OsArg::Linux => Os::Linux,
            OsArg::Macos => Os::MacOS,
            OsArg::Freebsd => Os::FreeBSD,
            OsArg::Openbsd => Os::OpenBSD,
            OsArg::Netbsd => Os::NetBSD,
            OsArg::Solaris => Os::Solaris,
            OsArg::Android => Os::Android,
            OsArg::Ios => Os::Ios,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Translate { command, from, to, verbose, json }) => {
            cmd_translate(command, from.into(), to.into(), verbose, json);
        }
        Some(Commands::List { from, to }) => {
            cmd_list(from.into(), to.into());
        }
        Some(Commands::Detect) => {
            cmd_detect();
        }
        Some(Commands::Os) => {
            cmd_os();
        }
        Some(Commands::Interactive { from, to }) => {
            cmd_interactive(from.into(), to.into());
        }
        None => {
            // Default behavior: show help
            println!("cmdx - Advanced Runtime Command Translator\n");
            println!("Use 'cmdx --help' for usage information");
            println!("Use 'cmdx translate --help' for translation options");
        }
    }
}

fn cmd_translate(command: Option<String>, from_os: Os, to_os: Os, verbose: bool, json: bool) {
    let input = match command {
        Some(cmd) => cmd,
        None => {
            // Read from stdin
            let stdin = io::stdin();
            let mut lines = Vec::new();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => lines.push(l),
                    Err(e) => {
                        eprintln!("Error reading stdin: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            lines.join("\n")
        }
    };
    
    // Process each line as a separate command
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        match translate_command(trimmed, from_os, to_os) {
            Ok(result) => {
                if json {
                    let output = serde_json::json!({
                        "original": result.original,
                        "translated": result.command,
                        "from": format!("{}", from_os),
                        "to": format!("{}", to_os),
                        "warnings": result.warnings,
                        "had_unmapped_flags": result.had_unmapped_flags,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                } else {
                    println!("{}", result.command);
                    if verbose && !result.warnings.is_empty() {
                        for warning in &result.warnings {
                            eprintln!("  Note: {}", warning);
                        }
                    }
                }
            }
            Err(e) => {
                if json {
                    let output = serde_json::json!({
                        "error": format!("{}", e),
                        "original": trimmed,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                } else {
                    eprintln!("Error: {}", e);
                }
                if !json {
                    std::process::exit(1);
                }
            }
        }
    }
}

fn cmd_list(from_os: Os, to_os: Os) {
    use cmdx::translator::command_map::get_available_commands;
    
    let commands = get_available_commands(from_os, to_os);
    
    println!("Available commands for {} -> {}:\n", from_os, to_os);
    
    if commands.is_empty() {
        println!("  No specific command translations available.");
        println!("  (Unix-like OS commands may still work via passthrough)");
    } else {
        let mut sorted_commands = commands.clone();
        sorted_commands.sort();
        
        for cmd in sorted_commands {
            // Get the translation to show
            if let Ok(result) = translate_command(cmd, from_os, to_os) {
                println!("  {} -> {}", cmd, result.command);
            }
        }
    }
    
    println!("\nTotal: {} commands", commands.len());
}

fn cmd_detect() {
    let os = detect_os();
    println!("Detected OS: {}", os);
    println!("Unix-like: {}", os.is_unix_like());
    println!("BSD-based: {}", os.is_bsd());
}

fn cmd_os() {
    println!("Supported operating systems:\n");
    for os in Os::all() {
        let mut notes = Vec::new();
        if os.is_unix_like() {
            notes.push("Unix-like");
        }
        if os.is_bsd() {
            notes.push("BSD");
        }
        
        if notes.is_empty() {
            println!("  {}", os);
        } else {
            println!("  {} ({})", os, notes.join(", "));
        }
    }
}

fn cmd_interactive(from_os: Os, to_os: Os) {
    println!("cmdx Interactive Mode");
    println!("Translating from {} to {}", from_os, to_os);
    println!("Type 'exit' or 'quit' to exit, 'swap' to swap source/target OS\n");
    
    let stdin = io::stdin();
    let mut from = from_os;
    let mut to = to_os;
    
    loop {
        print!("[{}->{}] > ", from, to);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = input.trim();
                
                // Special commands
                match trimmed.to_lowercase().as_str() {
                    "exit" | "quit" | "q" => break,
                    "swap" => {
                        std::mem::swap(&mut from, &mut to);
                        println!("Swapped: now translating {} -> {}", from, to);
                        continue;
                    }
                    "help" | "?" => {
                        println!("Commands:");
                        println!("  exit, quit, q - Exit interactive mode");
                        println!("  swap         - Swap source and target OS");
                        println!("  help, ?      - Show this help");
                        println!("\nEnter any shell command to translate it.");
                        continue;
                    }
                    "" => continue,
                    _ => {}
                }
                
                match translate_command(trimmed, from, to) {
                    Ok(result) => {
                        println!("=> {}", result.command);
                        for warning in &result.warnings {
                            println!("   Note: {}", warning);
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    
    println!("\nGoodbye!");
}
