# cmdx

A high-performance runtime command translator for cross-platform shell commands written in Rust.

## Features

- **Cross-platform translation**: Convert commands between Windows, Linux, macOS, FreeBSD, OpenBSD, NetBSD, Solaris, Android, and iOS
- **Flag translation**: Automatically translates command flags/options between platforms
- **OS detection**: Automatically detect the current operating system at runtime
- **High performance**: Uses static lookup tables with lazy initialization for fast translations
- **CLI tool**: Easy-to-use command-line interface
- **Library**: Use as a Rust library in your own projects
- **JSON output**: Machine-readable output format for scripting
- **Interactive mode**: Continuous translation session
- **Pipe support**: Works with stdin for batch processing

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/RyAnPr1Me/cmdx.git
cd cmdx

# Build with optimizations
cargo build --release

# The binary will be at target/release/cmdx
```

### Using Cargo

```bash
cargo install --path .
```

## Usage

### Basic Translation

```bash
# Translate a Windows command to Linux
cmdx translate "dir /w /s" --from windows --to linux
# Output: ls -C -R

# Translate a Linux command to Windows
cmdx translate "ls -la /home" --from linux --to windows
# Output: dir /a /home

# Translate grep with flags
cmdx translate "grep -i pattern file.txt" --from linux --to windows
# Output: findstr /i pattern file.txt
```

### Using Pipes

```bash
# Translate multiple commands from stdin
echo -e "dir /w\ncls\ncopy file1 file2" | cmdx translate --from windows --to linux
# Output:
# ls -C
# clear
# cp file1 file2
```

### JSON Output

```bash
cmdx translate "grep -i pattern" --from linux --to windows --json
```
```json
{
  "from": "Linux",
  "had_unmapped_flags": false,
  "original": "grep -i pattern",
  "to": "Windows",
  "translated": "findstr /i pattern",
  "warnings": []
}
```

### List Available Commands

```bash
# List all supported commands for Windows -> Linux
cmdx list --from windows --to linux
```

### OS Detection

```bash
# Detect the current operating system
cmdx detect
# Output:
# Detected OS: Linux
# Unix-like: true
# BSD-based: false
```

### List Supported Operating Systems

```bash
cmdx os
```

### Interactive Mode

```bash
cmdx interactive --from windows --to linux
# [Windows->Linux] > dir /w
# => ls -C
# [Windows->Linux] > swap
# Swapped: now translating Linux -> Windows
# [Linux->Windows] > ls -la
# => dir /a
```

## Supported Commands

### Windows to Linux

| Windows | Linux | Description |
|---------|-------|-------------|
| `dir` | `ls` | List directory contents |
| `copy` | `cp` | Copy files |
| `move` | `mv` | Move files |
| `del`, `erase` | `rm` | Delete files |
| `cls` | `clear` | Clear screen |
| `type` | `cat` | Display file contents |
| `findstr` | `grep` | Search text patterns |
| `tasklist` | `ps aux` | List processes |
| `taskkill` | `kill` | Terminate processes |
| `ipconfig` | `ip addr` | Network configuration |
| `ping` | `ping` | Network ping (with flag translation) |
| `tracert` | `traceroute` | Trace route |
| `fc` | `diff` | Compare files |
| And many more... | | |

### Linux to Windows

| Linux | Windows | Description |
|-------|---------|-------------|
| `ls` | `dir` | List directory contents |
| `cp` | `copy` | Copy files |
| `mv` | `move` | Move files |
| `rm` | `del` | Delete files |
| `clear` | `cls` | Clear screen |
| `cat` | `type` | Display file contents |
| `grep` | `findstr` | Search text patterns |
| `ps` | `tasklist` | List processes |
| `kill` | `taskkill /pid` | Terminate processes |
| `ifconfig`, `ip` | `ipconfig` | Network configuration |
| `traceroute` | `tracert` | Trace route |
| `diff` | `fc` | Compare files |
| `which` | `where` | Find command location |
| And many more... | | |

## Library Usage

```rust
use cmdx::{translate_command, Os};

fn main() {
    // Translate a Windows command to Linux
    let result = translate_command("dir /w", Os::Windows, Os::Linux);
    
    match result {
        Ok(translation) => {
            println!("Translated: {}", translation.command);
            for warning in &translation.warnings {
                println!("Warning: {}", warning);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Batch Translation

```rust
use cmdx::{translate_batch, Os};

fn main() {
    let commands = vec!["dir", "cls", "copy"];
    let results = translate_batch(&commands, Os::Windows, Os::Linux);
    
    for result in results {
        match result {
            Ok(t) => println!("{} -> {}", t.original, t.command),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

### OS Detection

```rust
use cmdx::detect_os;

fn main() {
    let os = detect_os();
    println!("Running on: {}", os);
    println!("Unix-like: {}", os.is_unix_like());
    println!("BSD-based: {}", os.is_bsd());
}
```

## Supported Operating Systems

- Windows
- Linux
- macOS (Darwin)
- FreeBSD
- OpenBSD
- NetBSD
- Solaris
- Android
- iOS

## Performance

cmdx is designed for high performance:

- **Lazy static initialization**: Command mappings are initialized once on first use
- **O(1) lookups**: Uses HashMap for constant-time command lookups
- **Zero-copy parsing**: Efficient string handling where possible
- **Release optimizations**: LTO, single codegen unit, and stripping enabled

Build with optimizations:
```bash
cargo build --release
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.