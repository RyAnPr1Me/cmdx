# cmdx Examples

This directory contains example scripts demonstrating the cross-OS translation capabilities of cmdx.

## 🚀 Proton-Style Usage (Recommended)

Just run any script with `cmdx` - it auto-detects everything!

### Running Windows Scripts on Linux

```bash
# Simple! Just run it like Proton/WINE
cmdx windows_script.bat
```

The tool automatically:
- Detects it's a Windows script from `.bat` extension
- Translates Windows commands to Linux equivalents:
  - `dir /b` → `ls -1`
  - `type file.txt` → `cat file.txt`
  - `del file.txt` → `rm file.txt`
  - `@echo off` → [skipped]
  - `echo.` → [skipped]

### Running Linux Scripts on Windows

```bash
# Works the same way!
cmdx linux_script.sh
```

The tool automatically:
- Detects it's a Linux script from `.sh` extension
- Translates Linux commands to Windows equivalents:
  - `ls -l` → `dir`
  - `cat file.txt` → `type file.txt`
  - `rm file.txt` → `del file.txt`

## Try It Yourself

```bash
# Clone the repo
git clone https://github.com/RyAnPr1Me/cmdx
cd cmdx

# Build the tool
cargo build --release

# Run the Windows example on Linux
./target/release/cmdx examples/windows_script.bat

# Run the Linux example (works on any OS)
./target/release/cmdx examples/linux_script.sh
```

## Interactive Mode

Start an interactive shell that translates commands in real-time:

```bash
# Windows to Linux translation shell
cmdx shell --from windows

# Now you can type Windows commands and see them translated:
cmdx [Windows→Linux]> dir /s
→ ls -R
[output]

cmdx [Windows→Linux]> type readme.txt  
→ cat readme.txt
[output]
```

## Command Translation Preview

See how commands will be translated without executing them:

```bash
# Preview Windows to Linux translation
cmdx translate --from windows --to linux "dir /s /b"

# Preview Linux to Windows translation
cmdx translate --from linux --to windows "ls -la"

# Package manager translation
cmdx translate --from linux --to linux "apt install -y vim"
```

## Direct Execution

Execute individual commands with automatic translation:

```bash
# Execute a Windows command on Linux
cmdx exec --from windows "dir /s"

# Execute a Linux command on Windows
cmdx exec --from linux "ls -la"
```
