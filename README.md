# cmdx

A high-performance cross-platform command and path translator library and CLI tool for Rust. Works as a **translation layer for cross-OS script execution**, similar to WINE but for shell commands.

## Features

### 🚀 CLI Translation Layer (NEW!)
- **Script Execution**: Run Windows .bat/.cmd scripts on Linux (and vice versa) with automatic command translation
- **Interactive Shell**: Real-time command translation shell
- **Command Execution**: Execute translated commands directly
- **Translation Preview**: See how commands will be translated before execution

### 📚 Library Features
- **Command Translation**: Translate shell commands between Windows, Linux, macOS, BSD, and more
- **Package Manager Translation**: Translate package manager commands between Linux distros (apt, yum, dnf, pacman, zypper, etc.)
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

### As a CLI tool:

```bash
cargo install cmdx
```

**Optional: Enable shell completions**

For Bash:
```bash
source completions/cmdx.bash
```

For Zsh:
```zsh
source completions/_cmdx
```

See [completions/README.md](completions/README.md) for detailed installation instructions.

### As a library in your project:

```toml
[dependencies]
cmdx = "0.1"
```

## CLI Usage

The `cmdx` CLI works like **Proton/WINE for shell scripts** - just run any script and it automatically detects the OS and translates commands in real-time!

### 🚀 Proton-Style Usage (Easiest!)

Just prefix any script with `cmdx` and it will auto-detect and translate:

```bash
# Run a Windows batch script on Linux (auto-detects from .bat extension)
cmdx install.bat

# Run a Linux shell script on Windows (auto-detects from .sh extension)
cmdx setup.sh

# Run a PowerShell script anywhere (auto-detects from .ps1 extension)
cmdx deploy.ps1
```

**That's it!** No flags needed. cmdx automatically:
- Detects the source OS from file extension (`.bat`, `.cmd`, `.ps1` = Windows; `.sh` = Linux)
- Detects your current OS as the target
- Translates each command line-by-line during execution
- Handles comments, echo statements, and special directives

### Example Output

Running `cmdx install.bat` on Linux:

```
[cmdx] Detected Windows script from extension: .bat
Executing script: install.bat (10 lines)
Translating from Windows to Linux

[1] @echo off [skipped]
[2] REM Installing software [skipped]
[3] echo Starting installation... → echo Starting installation...
Starting installation...
[4] dir /s → ls -R
[files listed]
[5] type readme.txt → cat readme.txt
[file contents]
```

### Advanced Usage

#### Execute a single command with translation

```bash
# Run a Windows command on Linux
cmdx exec --from windows "dir /s"

# Run a Linux command on Windows  
cmdx exec --from linux "ls -la"
```

#### Interactive translation shell

```bash
# Start an interactive shell that translates Windows commands to Linux
cmdx shell --from windows

# In the shell:
cmdx [Windows→Linux]> dir /s
→ ls -R
[directory listing output]

cmdx [Windows→Linux]> type readme.txt
→ cat readme.txt
[file contents]
```

#### Preview translation without execution

```bash
# See how a command would be translated
cmdx translate --from windows --to linux "dir /s"
# Output:
# Original [Windows]: dir /s
# Translated [Linux]: ls -R

# Package manager translation
cmdx translate --from linux --to linux "apt install -y vim"
# Output:
# Original [Linux]: apt install -y vim  
# Translated [Linux]: apt install -y vim
```

### Command-line options

```
USAGE:
    cmdx <script>              Run script with auto-detection (Proton-style)
    cmdx <COMMAND> [OPTIONS]   Advanced usage with explicit options

COMMANDS:
    exec <command>           Execute a command with translation
    shell                    Start interactive translation shell
    translate <command>      Translate and print command without executing

OPTIONS:
    --from <os>             Source OS (windows, linux, macos)
    --to <os>               Target OS (default: auto-detect)
    -n, --dry-run           Preview translations without executing
    -q, --quiet             Suppress informational output
    -v, --verbose           Show detailed translation information
    --no-color              Disable colored output
    -h, --help              Print help message
    --version               Print version information
```

### Advanced Features

**Dry-run mode**: Preview what will be translated and executed without actually running commands:
```bash
cmdx --dry-run install.bat
```

**Quiet mode**: Suppress informational messages for use in scripts:
```bash
cmdx --quiet setup.sh
```

**Verbose mode**: Show detailed information including skipped comments:
```bash
cmdx --verbose deploy.ps1
```

**Color output**: Enabled by default with helpful color coding:
- 🟢 Green: Translated commands
- 🟡 Yellow: Warnings
- 🔴 Red: Errors
- 🔵 Cyan: Informational messages

Disable with `--no-color` or set `NO_COLOR` environment variable.

## Library Usage

### Package Manager Translation (NEW!)

Translate package manager commands between different Linux distributions, **including flags**:

```rust
use cmdx::{translate_package_command, PackageManager};

// Translate apt to dnf
let result = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Dnf)?;
println!("{}", result.command);  // "dnf install vim"

// Translate yum to pacman
let result = translate_package_command("yum remove httpd", PackageManager::Yum, PackageManager::Pacman)?;
println!("{}", result.command);  // "pacman -R httpd"

// Translate apt to zypper
let result = translate_package_command("apt update", PackageManager::Apt, PackageManager::Zypper)?;
println!("{}", result.command);  // "zypper refresh"

// Flags are automatically translated!
let result = translate_package_command("apt install -y vim", PackageManager::Apt, PackageManager::Pacman)?;
println!("{}", result.command);  // "pacman -S --noconfirm vim"

// Multiple flags work too
let result = translate_package_command("apt install -y -q vim", PackageManager::Apt, PackageManager::Dnf)?;
println!("{}", result.command);  // "dnf install -y -q vim"

// Auto-detect source package manager
use cmdx::translate_package_command_auto;
let result = translate_package_command_auto("apt search nginx", PackageManager::Dnf)?;
println!("{}", result.command);  // "dnf search nginx"

// Works with sudo prefix and flags
let result = translate_package_command("sudo apt install -y vim", PackageManager::Apt, PackageManager::Pacman)?;
println!("{}", result.command);  // "sudo pacman -S --noconfirm vim"
```

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

### Package Manager Operations (Linux Distros)

All major Linux package managers are supported with translation for common operations:

| Operation | APT (Debian/Ubuntu) | YUM (RHEL/CentOS) | DNF (Fedora) | Pacman (Arch) | Zypper (openSUSE) |
|-----------|---------------------|-------------------|--------------|---------------|-------------------|
| **Install** | `apt install` | `yum install` | `dnf install` | `pacman -S` | `zypper install` |
| **Remove** | `apt remove` | `yum remove` | `dnf remove` | `pacman -R` | `zypper remove` |
| **Update** | `apt update` | `yum check-update` | `dnf check-update` | `pacman -Sy` | `zypper refresh` |
| **Upgrade** | `apt upgrade` | `yum update` | `dnf upgrade` | `pacman -Syu` | `zypper update` |
| **Search** | `apt search` | `yum search` | `dnf search` | `pacman -Ss` | `zypper search` |
| **Info** | `apt show` | `yum info` | `dnf info` | `pacman -Si` | `zypper info` |
| **List** | `apt list --installed` | `yum list installed` | `dnf list installed` | `pacman -Q` | `zypper packages` |
| **Clean** | `apt clean` | `yum clean all` | `dnf clean all` | `pacman -Sc` | `zypper clean` |
| **Auto-remove** | `apt autoremove` | `yum autoremove` | `dnf autoremove` | `pacman -Rs` | `zypper remove --clean-deps` |

Additional supported package managers:
- **APK** (Alpine Linux): `apk add`, `apk del`, `apk update`, etc.
- **Emerge** (Gentoo): `emerge`, `emerge --unmerge`, `emerge --sync`, etc.
- **XBPS** (Void Linux): `xbps-install`, `xbps-remove`, `xbps-query`, etc.
- **Nix** (NixOS): `nix-env -i`, `nix-env -e`, `nix-env -u`, etc.

### Package Manager Flag Translation

Common flags are automatically translated between package managers:

| Flag Purpose | APT | DNF/YUM | Pacman | Zypper |
|--------------|-----|---------|--------|--------|
| **Assume yes** | `-y`, `--yes` | `-y` | `--noconfirm` | `-y` |
| **Quiet mode** | `-q`, `--quiet` | `-q` | `-q` | `-q` |
| **Reinstall** | `--reinstall` | `--reinstall` | — | `--force` |
| **No recommends** | `--no-install-recommends` | `--setopt=install_weak_deps=False` | `--asdeps` | — |
| **Purge configs** | `--purge` | — | `-n` | — |
| **Auto-remove deps** | `--auto-remove` | `--noautoremove` | `-s` | `--clean-deps` |
| **Verbose** | `-v` | `-v` | — | `-v` |

### OS-Level Commands

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