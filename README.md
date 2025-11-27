# cmdx

A high-performance cross-platform command and path translator library for Rust. Designed for integration into terminal emulators and cross-platform tools.

## Features

- **Command Translation**: Translate shell commands between Windows, Linux, macOS, BSD, and more
- **Full Translation**: Translate commands AND their file path arguments in one call
- **Smart Passthrough**: Commands already in the target OS format are passed through unchanged
- **Flag Translation**: Automatically translates command flags/options (e.g., `dir /w` → `ls -C`)
- **Compound Commands**: Translate multi-command pipelines with `&&`, `||`, `|`, and `;`
- **Path Translation**: Bidirectional file path translation (e.g., `C:\Users` ↔ `/mnt/c/Users`)
- **Environment Variables**: Translate environment variable syntax (`%VAR%` ↔ `$VAR`)
- **Script Translation**: Convert script extensions (`.bat` ↔ `.sh`) and shebangs
- **OS Detection**: Runtime detection of the current operating system
- **Serde Support**: All result types support serialization/deserialization
- **High Performance**: Static lookup tables with lazy initialization

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cmdx = "0.1"
```

## Usage

### Full Command + Path Translation

The `translate_full` function translates both commands AND file paths in arguments:

```rust
use cmdx::{translate_full, Os};

// Windows command with paths to Linux
let result = translate_full("copy C:\\Users\\file.txt D:\\backup\\", Os::Windows, Os::Linux)?;
println!("{}", result.command);  // "cp /mnt/c/Users/file.txt /mnt/d/backup/"

// Linux command with paths to Windows
let result = translate_full("cp /home/user/file.txt /tmp/backup", Os::Linux, Os::Windows)?;
println!("{}", result.command);  // "copy C:\Users\user\file.txt C:\tmp\backup"

// Flags are also translated
let result = translate_full("copy /y C:\\src\\file.txt D:\\dest\\", Os::Windows, Os::Linux)?;
// => "cp -f /mnt/c/src/file.txt /mnt/d/dest/"
```

### Command Translation

```rust
use cmdx::{translate_command, Os};

// Translate Windows command to Linux
let result = translate_command("dir /w /s", Os::Windows, Os::Linux)?;
println!("{}", result.command);  // "ls -C -R"

// Translate Linux command to Windows
let result = translate_command("grep -i pattern", Os::Linux, Os::Windows)?;
println!("{}", result.command);  // "findstr /i pattern"

// Check for translation warnings
for warning in &result.warnings {
    println!("Warning: {}", warning);
}
```

### Smart Passthrough

Commands that are already in the target OS format are automatically detected and passed through unchanged:

```rust
use cmdx::{translate_command, is_native_command, Os};

// If you're translating to Linux and the command is already a Linux command,
// it passes through unchanged
let result = translate_command("ls -la", Os::Windows, Os::Linux)?;
println!("{}", result.command);  // "ls -la" (passed through, already Linux native)

// Check if a command is native to an OS
assert!(is_native_command("dir", Os::Windows));
assert!(is_native_command("ls", Os::Linux));
assert!(is_native_command("open", Os::MacOS));
```

### Compound Command Translation

```rust
use cmdx::{translate_compound_command, Os};

// Translate commands with operators
let result = translate_compound_command("dir && cls", Os::Windows, Os::Linux)?;
println!("{}", result.command);  // "ls && clear"

// Pipe operators work too
let result = translate_compound_command("dir | findstr test", Os::Windows, Os::Linux)?;
println!("{}", result.command);  // "ls | grep test"
```

### Environment Variable Translation

```rust
use cmdx::{translate_env_vars, Os};

// Windows to Unix
let result = translate_env_vars("echo %USERPROFILE%", Os::Windows, Os::Linux);
println!("{}", result);  // "echo $HOME"

// Unix to Windows
let result = translate_env_vars("echo $HOME", Os::Linux, Os::Windows);
println!("{}", result);  // "echo %USERPROFILE%"
```

### Path Translation

```rust
use cmdx::{translate_path, Os};

// Windows to Linux
let result = translate_path("C:\\Users\\john\\file.txt", Os::Windows, Os::Linux)?;
println!("{}", result.path);  // "/mnt/c/Users/john/file.txt"

// Linux to Windows
let result = translate_path("/mnt/c/Users/john", Os::Linux, Os::Windows)?;
println!("{}", result.path);  // "C:\Users\john"

// Home directory translation
let result = translate_path("~/Documents", Os::Linux, Os::Windows)?;
println!("{}", result.path);  // "%USERPROFILE%\Documents"
```

### Auto-Detect Path Format

```rust
use cmdx::{translate_path_auto, is_windows_path, is_unix_path, Os};

// Auto-detect and translate
let result = translate_path_auto("C:\\Users\\john", Os::Linux)?;
println!("{}", result.path);  // "/mnt/c/Users/john"

// Check path format
assert!(is_windows_path("C:\\Users"));
assert!(is_unix_path("/home/user"));
```

### OS Detection

```rust
use cmdx::detect_os;

let os = detect_os();
println!("Running on: {}", os);
println!("Unix-like: {}", os.is_unix_like());
println!("BSD-based: {}", os.is_bsd());
```

### Batch Translation

```rust
use cmdx::{translate_batch, translate_paths, Os};

// Batch command translation
let commands = vec!["dir", "cls", "copy"];
let results = translate_batch(&commands, Os::Windows, Os::Linux);

// Batch path translation
let paths = vec!["C:\\Users", "D:\\Documents"];
let results = translate_paths(&paths, Os::Windows, Os::Linux);
```

### Script File Translation

```rust
use cmdx::{translate_script_extension, translate_shebang, Os};

// Translate script file extensions
let result = translate_script_extension("build.bat", Os::Windows, Os::Linux);
assert_eq!(result, "build.sh");

let result = translate_script_extension("deploy.sh", Os::Linux, Os::Windows);
assert_eq!(result, "deploy.bat");

// Translate shebangs
let result = translate_shebang("#!/bin/bash", Os::Linux, Os::Windows);
assert_eq!(result, "@echo off");

let result = translate_shebang("@echo off", Os::Windows, Os::Linux);
assert_eq!(result, "#!/bin/bash");
```

### Terminal Emulator Integration

```rust
use cmdx::{translate_command, translate_path, translate_env_vars, detect_os, Os};

/// Process user input and translate for target OS
fn process_input(input: &str, target_os: Os) -> String {
    let current_os = detect_os();
    
    // Translate environment variables first
    let input = translate_env_vars(input, current_os, target_os);
    
    // Try command translation
    if let Ok(result) = translate_command(&input, current_os, target_os) {
        return result.command;
    }
    
    // Fall back to path translation if it looks like a path
    if let Ok(result) = translate_path(&input, current_os, target_os) {
        return result.path;
    }
    
    // Return the env-translated input if no other translation needed
    input
}
```

## Supported Commands

### Windows → Linux

| Windows | Linux | Notes |
|---------|-------|-------|
| `dir` | `ls` | Flags: `/w`→`-C`, `/s`→`-R`, `/a`→`-la` |
| `copy` | `cp` | Flags: `/y`→`-f`, `/v`→`-v` |
| `move` | `mv` | Flags: `/y`→`-f` |
| `del` | `rm` | Flags: `/s`→`-r`, `/q`→`-f` |
| `cls` | `clear` | |
| `type` | `cat` | |
| `findstr` | `grep` | Flags: `/i`→`-i`, `/n`→`-n` |
| `tasklist` | `ps aux` | |
| `ipconfig` | `ip addr` | |
| `ping -n` | `ping -c` | Count flag translation |
| `start` | `xdg-open` | Open files/URLs |
| `clip` | `xclip` | Clipboard |
| And 40+ more... | | |

### Linux → Windows

| Linux | Windows | Notes |
|-------|---------|-------|
| `ls` | `dir` | Flags: `-la`→`/a`, `-R`→`/s` |
| `cp` | `copy` | Flags: `-r`→`xcopy /s /e` |
| `mv` | `move` | |
| `rm` | `del` | Flags: `-r`→`/s`, `-f`→`/q` |
| `clear` | `cls` | |
| `cat` | `type` | |
| `grep` | `findstr` | |
| `ps` | `tasklist` | |
| `xdg-open` | `start` | Open files/URLs |
| `xclip` | `clip` | Clipboard |
| And 40+ more... | | |

### macOS Commands

| macOS | Windows | Linux |
|-------|---------|-------|
| `open` | `start` | `xdg-open` |
| `pbcopy` | `clip` | `xclip` |
| `pbpaste` | `Get-Clipboard` | `xclip -o` |

## Script Extension Mappings

| Windows | Unix |
|---------|------|
| `.bat` | `.sh` |
| `.cmd` | `.sh` |
| `.ps1` | `.sh` |
| `.exe` | (no extension) |

## Environment Variable Mappings

| Windows | Unix |
|---------|------|
| `%USERPROFILE%` | `$HOME` |
| `%USERNAME%` | `$USER` |
| `%TEMP%` / `%TMP%` | `$TMPDIR` |
| `%APPDATA%` | `$XDG_CONFIG_HOME` |
| `%COMPUTERNAME%` | `$HOSTNAME` |
| `%CD%` | `$PWD` |
| `%COMSPEC%` | `$SHELL` |

## Path Translation Mappings

| Unix Path | Windows Path |
|-----------|--------------|
| `/mnt/c/...` | `C:\...` |
| `/mnt/d/...` | `D:\...` |
| `/home/user` | `C:\Users\user` |
| `~` | `%USERPROFILE%` |
| `//server/share` | `\\server\share` |

## Supported Operating Systems

- Windows
- Linux
- macOS (Darwin)
- FreeBSD, OpenBSD, NetBSD
- Solaris
- Android
- iOS

## License

MIT License