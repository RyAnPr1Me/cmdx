# Shell Completions for cmdx

This directory contains shell completion scripts for `cmdx` to enable tab-completion of commands, options, and file paths.

## Bash

### Installation

**System-wide (requires root):**
```bash
sudo cp cmdx.bash /etc/bash_completion.d/cmdx
```

**User-specific:**
```bash
mkdir -p ~/.local/share/bash-completion/completions
cp cmdx.bash ~/.local/share/bash-completion/completions/cmdx
```

**Temporary (current session only):**
```bash
source completions/cmdx.bash
```

### Features

- Tab-complete commands: `exec`, `shell`, `translate`
- Tab-complete options: `--from`, `--to`, `--dry-run`, `--quiet`, etc.
- Tab-complete OS values: `windows`, `linux`, `macos`, `freebsd`
- Auto-suggest script files with extensions: `.bat`, `.cmd`, `.ps1`, `.sh`, `.bash`, `.zsh`

### Usage Examples

```bash
cmdx <TAB>                  # Shows: exec, shell, translate, --help, and script files
cmdx --from <TAB>           # Shows: windows, linux, macos, freebsd
cmdx --dry-run <TAB>        # Shows script files
```

## Zsh

### Installation

**System-wide (requires root):**
```zsh
sudo cp _cmdx /usr/local/share/zsh/site-functions/_cmdx
```

**User-specific:**
```zsh
# Add to your fpath and source
mkdir -p ~/.zsh/completions
cp _cmdx ~/.zsh/completions/_cmdx

# Add this to your ~/.zshrc if not already present:
fpath=(~/.zsh/completions $fpath)
autoload -U compinit && compinit
```

**Temporary (current session only):**
```zsh
source completions/_cmdx
```

### Features

- Intelligent command completion with descriptions
- Contextual option suggestions based on command
- File completion filtered by script extensions
- OS value completion for --from and --to flags

### Usage Examples

```zsh
cmdx <TAB>                  # Shows commands with descriptions and script files
cmdx --from <TAB>           # Shows: windows, linux, macos, freebsd
cmdx shell --from <TAB>     # Shows OS options
```

## Testing Completions

After installation, open a new terminal or source your shell's RC file:

```bash
# For bash
source ~/.bashrc

# For zsh
source ~/.zshrc
```

Then test:
```bash
cmdx --<TAB><TAB>    # Should show all available options
cmdx <TAB><TAB>      # Should show commands and script files
```

## Troubleshooting

### Bash completion not working

1. Ensure bash-completion is installed:
   ```bash
   # Ubuntu/Debian
   sudo apt install bash-completion
   
   # macOS with Homebrew
   brew install bash-completion@2
   ```

2. Verify completion is enabled in your `~/.bashrc`:
   ```bash
   if [ -f /etc/bash_completion ]; then
       . /etc/bash_completion
   fi
   ```

### Zsh completion not working

1. Verify compinit is running:
   ```zsh
   echo $fpath  # Should include your completions directory
   ```

2. Rebuild completion cache:
   ```zsh
   rm -f ~/.zcompdump
   compinit
   ```

3. Ensure your completions directory is in fpath before calling compinit in `~/.zshrc`.
