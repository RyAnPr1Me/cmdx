# cmdx Examples

This directory contains example scripts demonstrating the cross-OS translation capabilities of cmdx.

## Running Windows Scripts on Linux

```bash
# Execute a Windows batch script on Linux with automatic translation
cmdx script --from windows windows_script.bat
```

The tool will automatically translate Windows commands to Linux equivalents:
- `dir /b` → `ls`
- `type file.txt` → `cat file.txt`
- `del file.txt` → `rm file.txt`
- `echo.` → `echo`

## Running Linux Scripts on Windows

```bash
# Execute a Linux shell script on Windows with automatic translation
cmdx script --from linux linux_script.sh
```

The tool will automatically translate Linux commands to Windows equivalents:
- `ls -l` → `dir`
- `cat file.txt` → `type file.txt`
- `rm file.txt` → `del file.txt`

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
