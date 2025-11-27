//! Command mapping definitions and lookup tables

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::os::Os;

/// Flag mapping between different operating systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagMapping {
    /// The source flag
    pub source: String,
    /// The target flag
    pub target: String,
    /// Description of what this flag does
    pub description: Option<String>,
}

impl FlagMapping {
    pub fn new(source: &str, target: &str) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            description: None,
        }
    }

    pub fn with_description(source: &str, target: &str, description: &str) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            description: Some(description.to_string()),
        }
    }
}

/// Command mapping between different operating systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMapping {
    /// Source command name
    pub source_cmd: String,
    /// Target command name
    pub target_cmd: String,
    /// Flag mappings for this command
    pub flag_mappings: Vec<FlagMapping>,
    /// Whether to preserve unmapped flags
    pub preserve_unmapped_flags: bool,
    /// Notes about this command translation
    pub notes: Option<String>,
}

impl CommandMapping {
    pub fn new(source_cmd: &str, target_cmd: &str) -> Self {
        Self {
            source_cmd: source_cmd.to_string(),
            target_cmd: target_cmd.to_string(),
            flag_mappings: Vec::new(),
            preserve_unmapped_flags: true,
            notes: None,
        }
    }

    pub fn with_flags(mut self, flags: Vec<FlagMapping>) -> Self {
        self.flag_mappings = flags;
        self
    }

    pub fn add_flag(&mut self, source: &str, target: &str) -> &mut Self {
        self.flag_mappings.push(FlagMapping::new(source, target));
        self
    }
}

/// Key for looking up command mappings
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MappingKey {
    pub command: String,
    pub from_os: Os,
    pub to_os: Os,
}

impl MappingKey {
    pub fn new(command: &str, from_os: Os, to_os: Os) -> Self {
        Self {
            command: command.to_lowercase(),
            from_os,
            to_os,
        }
    }
}

lazy_static! {
    /// Global command mapping table
    pub static ref COMMAND_MAPPINGS: HashMap<MappingKey, CommandMapping> = {
        let mut m = HashMap::new();
        
        // ============================================================
        // Windows -> Linux/Unix mappings
        // ============================================================
        
        // dir -> ls (comprehensive flag mapping)
        m.insert(
            MappingKey::new("dir", Os::Windows, Os::Linux),
            CommandMapping::new("dir", "ls")
                .with_flags(vec![
                    FlagMapping::with_description("/w", "-C", "Wide list format"),
                    FlagMapping::with_description("/d", "-C", "Wide list format (same as /w)"),
                    FlagMapping::with_description("/s", "-R", "Recursive listing"),
                    FlagMapping::with_description("/b", "-1", "Bare format (names only)"),
                    FlagMapping::with_description("/a", "-la", "All files including hidden"),
                    FlagMapping::with_description("/a:h", "-a", "Hidden files only"),
                    FlagMapping::with_description("/a:d", "-d */", "Directories only"),
                    FlagMapping::with_description("/a:r", "-l", "Read-only files"),
                    FlagMapping::with_description("/a:-h", "", "Not hidden"),
                    FlagMapping::with_description("/o", "", "Sorted"),
                    FlagMapping::with_description("/o:n", "--sort=name", "Sort by name"),
                    FlagMapping::with_description("/o:-n", "--sort=name -r", "Sort by name reversed"),
                    FlagMapping::with_description("/o:s", "--sort=size", "Sort by size"),
                    FlagMapping::with_description("/o:-s", "--sort=size -r", "Sort by size reversed"),
                    FlagMapping::with_description("/o:d", "--sort=time", "Sort by date"),
                    FlagMapping::with_description("/o:-d", "--sort=time -r", "Sort by date reversed"),
                    FlagMapping::with_description("/o:e", "--sort=extension", "Sort by extension"),
                    FlagMapping::with_description("/o:g", "", "Group directories first"),
                    FlagMapping::with_description("/t:c", "--time=ctime", "Use creation time"),
                    FlagMapping::with_description("/t:a", "--time=atime", "Use access time"),
                    FlagMapping::with_description("/t:w", "", "Use write time (default)"),
                    FlagMapping::with_description("/p", "", "Pause (not directly supported)"),
                    FlagMapping::with_description("/q", "-l", "Show owner"),
                    FlagMapping::with_description("/n", "-1", "New long list format"),
                    FlagMapping::with_description("/x", "-lX", "Show short names"),
                    FlagMapping::with_description("/4", "", "Four-digit years"),
                    FlagMapping::with_description("/l", "-l", "Lowercase"),
                    FlagMapping::with_description("/c", "--block-size=1", "Thousand separator"),
                ]),
        );
        
        // Also add macOS mapping (similar to Linux but some GNU options differ)
        m.insert(
            MappingKey::new("dir", Os::Windows, Os::MacOS),
            CommandMapping::new("dir", "ls")
                .with_flags(vec![
                    FlagMapping::with_description("/w", "-C", "Wide list format"),
                    FlagMapping::with_description("/s", "-R", "Recursive listing"),
                    FlagMapping::with_description("/b", "-1", "Bare format (names only)"),
                    FlagMapping::with_description("/a", "-la", "All files including hidden"),
                    FlagMapping::with_description("/a:h", "-a", "Hidden files"),
                    FlagMapping::with_description("/o:n", "", "Sort by name (default)"),
                    FlagMapping::with_description("/o:s", "-S", "Sort by size"),
                    FlagMapping::with_description("/o:d", "-t", "Sort by time"),
                    FlagMapping::with_description("/q", "-l", "Show owner"),
                ]),
        );
        
        // copy -> cp (comprehensive flag mapping)
        m.insert(
            MappingKey::new("copy", Os::Windows, Os::Linux),
            CommandMapping::new("copy", "cp")
                .with_flags(vec![
                    FlagMapping::with_description("/y", "-f", "Force overwrite without prompting"),
                    FlagMapping::with_description("/-y", "-i", "Prompt before overwrite"),
                    FlagMapping::with_description("/v", "-v", "Verbose"),
                    FlagMapping::with_description("/z", "", "Network resilient mode (N/A)"),
                    FlagMapping::with_description("/a", "", "ASCII mode (N/A in Unix)"),
                    FlagMapping::with_description("/b", "", "Binary mode (default in Unix)"),
                    FlagMapping::with_description("/d", "", "Allow decrypted destination"),
                    FlagMapping::with_description("/n", "", "Use short filename"),
                    FlagMapping::with_description("/l", "-s", "Create symbolic link"),
                ]),
        );
        
        m.insert(
            MappingKey::new("copy", Os::Windows, Os::MacOS),
            CommandMapping::new("copy", "cp")
                .with_flags(vec![
                    FlagMapping::with_description("/y", "-f", "Force overwrite"),
                    FlagMapping::with_description("/-y", "-i", "Interactive"),
                    FlagMapping::with_description("/v", "-v", "Verbose"),
                ]),
        );
        
        // xcopy -> cp -r (comprehensive flag mapping)
        m.insert(
            MappingKey::new("xcopy", Os::Windows, Os::Linux),
            CommandMapping::new("xcopy", "cp -r")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "", "Copy subdirs (implied by -r)"),
                    FlagMapping::with_description("/e", "", "Copy empty dirs too (implied by -r)"),
                    FlagMapping::with_description("/y", "-f", "Force overwrite"),
                    FlagMapping::with_description("/-y", "-i", "Prompt before overwrite"),
                    FlagMapping::with_description("/i", "", "Assume destination is directory"),
                    FlagMapping::with_description("/q", "", "Quiet mode"),
                    FlagMapping::with_description("/f", "-v", "Display full source/dest names"),
                    FlagMapping::with_description("/l", "-n", "List files (dry run)"),
                    FlagMapping::with_description("/h", "", "Copy hidden and system files"),
                    FlagMapping::with_description("/r", "", "Overwrite read-only files"),
                    FlagMapping::with_description("/t", "", "Create directory structure only"),
                    FlagMapping::with_description("/u", "-u", "Update only newer files"),
                    FlagMapping::with_description("/k", "-p", "Keep attributes"),
                    FlagMapping::with_description("/n", "", "Copy using short names"),
                    FlagMapping::with_description("/o", "-p", "Copy ownership and ACL"),
                    FlagMapping::with_description("/x", "-p", "Copy audit settings"),
                    FlagMapping::with_description("/v", "", "Verify each file"),
                    FlagMapping::with_description("/c", "", "Continue on errors"),
                    FlagMapping::with_description("/g", "", "Copy encrypted files"),
                    FlagMapping::with_description("/d", "", "Copy only files changed on date"),
                    FlagMapping::with_description("/a", "", "Archive attribute files only"),
                    FlagMapping::with_description("/m", "", "Archive files, reset archive attr"),
                    FlagMapping::with_description("/z", "", "Network resilient mode"),
                    FlagMapping::with_description("/b", "-a", "Copy symbolic link itself"),
                    FlagMapping::with_description("/j", "", "Copy using unbuffered I/O"),
                ]),
        );
        
        // move -> mv (comprehensive flag mapping)
        m.insert(
            MappingKey::new("move", Os::Windows, Os::Linux),
            CommandMapping::new("move", "mv")
                .with_flags(vec![
                    FlagMapping::with_description("/y", "-f", "Force overwrite without prompting"),
                    FlagMapping::with_description("/-y", "-i", "Prompt before overwrite"),
                ]),
        );
        
        m.insert(
            MappingKey::new("move", Os::Windows, Os::MacOS),
            CommandMapping::new("move", "mv")
                .with_flags(vec![
                    FlagMapping::with_description("/y", "-f", "Force overwrite"),
                    FlagMapping::with_description("/-y", "-i", "Interactive"),
                ]),
        );
        
        // del/erase -> rm (comprehensive flag mapping)
        m.insert(
            MappingKey::new("del", Os::Windows, Os::Linux),
            CommandMapping::new("del", "rm")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "-r", "Recursive delete"),
                    FlagMapping::with_description("/q", "-f", "Quiet mode (no confirmation)"),
                    FlagMapping::with_description("/f", "-f", "Force delete read-only files"),
                    FlagMapping::with_description("/p", "-i", "Prompt before each delete"),
                    FlagMapping::with_description("/a", "", "Delete by attributes"),
                    FlagMapping::with_description("/a:r", "", "Read-only files"),
                    FlagMapping::with_description("/a:h", "", "Hidden files"),
                    FlagMapping::with_description("/a:s", "", "System files"),
                    FlagMapping::with_description("/a:a", "", "Archive files"),
                ]),
        );
        
        m.insert(
            MappingKey::new("del", Os::Windows, Os::MacOS),
            CommandMapping::new("del", "rm")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "-r", "Recursive delete"),
                    FlagMapping::with_description("/q", "-f", "Quiet mode"),
                    FlagMapping::with_description("/f", "-f", "Force delete"),
                    FlagMapping::with_description("/p", "-i", "Interactive"),
                ]),
        );
        
        m.insert(
            MappingKey::new("erase", Os::Windows, Os::Linux),
            CommandMapping::new("erase", "rm")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "-r", "Recursive"),
                    FlagMapping::with_description("/q", "-f", "Quiet"),
                    FlagMapping::with_description("/f", "-f", "Force"),
                    FlagMapping::with_description("/p", "-i", "Interactive"),
                ]),
        );
        
        // rmdir/rd -> rm -r (comprehensive flag mapping)
        m.insert(
            MappingKey::new("rmdir", Os::Windows, Os::Linux),
            CommandMapping::new("rmdir", "rm -r")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "", "Recursive (implied by -r)"),
                    FlagMapping::with_description("/q", "-f", "Quiet mode"),
                ]),
        );
        
        m.insert(
            MappingKey::new("rmdir", Os::Windows, Os::MacOS),
            CommandMapping::new("rmdir", "rm -r")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "", "Recursive (implied)"),
                    FlagMapping::with_description("/q", "-f", "Quiet mode"),
                ]),
        );
        
        m.insert(
            MappingKey::new("rd", Os::Windows, Os::Linux),
            CommandMapping::new("rd", "rm -r")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "", "Recursive (implied)"),
                    FlagMapping::with_description("/q", "-f", "Quiet"),
                ]),
        );
        
        // mkdir/md -> mkdir (comprehensive flag mapping)
        m.insert(
            MappingKey::new("mkdir", Os::Windows, Os::Linux),
            CommandMapping::new("mkdir", "mkdir")
                .with_flags(vec![
                    FlagMapping::with_description("", "-p", "Create parent directories automatically"),
                ]),
        );
        
        m.insert(
            MappingKey::new("mkdir", Os::Windows, Os::MacOS),
            CommandMapping::new("mkdir", "mkdir")
                .with_flags(vec![
                    FlagMapping::with_description("", "-p", "Create parent directories"),
                ]),
        );
        
        m.insert(
            MappingKey::new("md", Os::Windows, Os::Linux),
            CommandMapping::new("md", "mkdir -p"),
        );
        
        // type -> cat
        m.insert(
            MappingKey::new("type", Os::Windows, Os::Linux),
            CommandMapping::new("type", "cat"),
        );
        
        m.insert(
            MappingKey::new("type", Os::Windows, Os::MacOS),
            CommandMapping::new("type", "cat"),
        );
        
        // cls -> clear
        m.insert(
            MappingKey::new("cls", Os::Windows, Os::Linux),
            CommandMapping::new("cls", "clear"),
        );
        
        m.insert(
            MappingKey::new("cls", Os::Windows, Os::MacOS),
            CommandMapping::new("cls", "clear"),
        );
        
        // echo -> echo
        m.insert(
            MappingKey::new("echo", Os::Windows, Os::Linux),
            CommandMapping::new("echo", "echo"),
        );
        
        // findstr -> grep
        m.insert(
            MappingKey::new("findstr", Os::Windows, Os::Linux),
            CommandMapping::new("findstr", "grep")
                .with_flags(vec![
                    FlagMapping::with_description("/i", "-i", "Case insensitive"),
                    FlagMapping::with_description("/s", "-r", "Recursive"),
                    FlagMapping::with_description("/n", "-n", "Line numbers"),
                    FlagMapping::with_description("/v", "-v", "Invert match"),
                    FlagMapping::with_description("/c:", "-c", "Count matches"),
                    FlagMapping::with_description("/r", "-E", "Regular expressions"),
                ]),
        );
        
        // find -> grep (Windows find is different from Unix find)
        m.insert(
            MappingKey::new("find", Os::Windows, Os::Linux),
            CommandMapping::new("find", "grep")
                .with_flags(vec![
                    FlagMapping::with_description("/i", "-i", "Case insensitive"),
                    FlagMapping::with_description("/v", "-v", "Invert match"),
                    FlagMapping::with_description("/c", "-c", "Count lines"),
                    FlagMapping::with_description("/n", "-n", "Line numbers"),
                ]),
        );
        
        // tasklist -> ps
        m.insert(
            MappingKey::new("tasklist", Os::Windows, Os::Linux),
            CommandMapping::new("tasklist", "ps aux"),
        );
        
        m.insert(
            MappingKey::new("tasklist", Os::Windows, Os::MacOS),
            CommandMapping::new("tasklist", "ps aux"),
        );
        
        // taskkill -> kill/pkill
        m.insert(
            MappingKey::new("taskkill", Os::Windows, Os::Linux),
            CommandMapping::new("taskkill", "kill")
                .with_flags(vec![
                    FlagMapping::with_description("/f", "-9", "Force kill"),
                    FlagMapping::with_description("/pid", "", "Process ID (use directly)"),
                    FlagMapping::with_description("/im", "-pkill ", "Image name -> use pkill"),
                ]),
        );
        
        // ipconfig -> ip/ifconfig
        m.insert(
            MappingKey::new("ipconfig", Os::Windows, Os::Linux),
            CommandMapping::new("ipconfig", "ip addr")
                .with_flags(vec![
                    FlagMapping::with_description("/all", "show", "Show all info"),
                    FlagMapping::with_description("/release", "", "Release DHCP"),
                    FlagMapping::with_description("/renew", "", "Renew DHCP"),
                ]),
        );
        
        m.insert(
            MappingKey::new("ipconfig", Os::Windows, Os::MacOS),
            CommandMapping::new("ipconfig", "ifconfig"),
        );
        
        // systeminfo -> uname -a
        m.insert(
            MappingKey::new("systeminfo", Os::Windows, Os::Linux),
            CommandMapping::new("systeminfo", "uname -a && cat /etc/os-release"),
        );
        
        // hostname -> hostname
        m.insert(
            MappingKey::new("hostname", Os::Windows, Os::Linux),
            CommandMapping::new("hostname", "hostname"),
        );
        
        // whoami -> whoami
        m.insert(
            MappingKey::new("whoami", Os::Windows, Os::Linux),
            CommandMapping::new("whoami", "whoami"),
        );
        
        // set -> env/export
        m.insert(
            MappingKey::new("set", Os::Windows, Os::Linux),
            CommandMapping::new("set", "env"),
        );
        
        // attrib -> chmod/chattr
        m.insert(
            MappingKey::new("attrib", Os::Windows, Os::Linux),
            CommandMapping::new("attrib", "chmod"),
        );
        
        // fc -> diff
        m.insert(
            MappingKey::new("fc", Os::Windows, Os::Linux),
            CommandMapping::new("fc", "diff")
                .with_flags(vec![
                    FlagMapping::with_description("/b", "", "Binary compare"),
                    FlagMapping::with_description("/c", "-i", "Ignore case"),
                    FlagMapping::with_description("/n", "-n", "Show line numbers"),
                    FlagMapping::with_description("/w", "-w", "Ignore whitespace"),
                ]),
        );
        
        // more -> less/more
        m.insert(
            MappingKey::new("more", Os::Windows, Os::Linux),
            CommandMapping::new("more", "less"),
        );
        
        // ren/rename -> mv
        m.insert(
            MappingKey::new("ren", Os::Windows, Os::Linux),
            CommandMapping::new("ren", "mv"),
        );
        
        m.insert(
            MappingKey::new("rename", Os::Windows, Os::Linux),
            CommandMapping::new("rename", "mv"),
        );
        
        // tree -> tree
        m.insert(
            MappingKey::new("tree", Os::Windows, Os::Linux),
            CommandMapping::new("tree", "tree")
                .with_flags(vec![
                    FlagMapping::with_description("/f", "", "Show files (default in Linux)"),
                    FlagMapping::with_description("/a", "--charset=ascii", "ASCII characters"),
                ]),
        );
        
        // sort -> sort
        m.insert(
            MappingKey::new("sort", Os::Windows, Os::Linux),
            CommandMapping::new("sort", "sort")
                .with_flags(vec![
                    FlagMapping::with_description("/r", "-r", "Reverse order"),
                    FlagMapping::with_description("/n", "-n", "Numeric sort"),
                ]),
        );
        
        // where -> which/whereis
        m.insert(
            MappingKey::new("where", Os::Windows, Os::Linux),
            CommandMapping::new("where", "which"),
        );
        
        // ping -> ping
        m.insert(
            MappingKey::new("ping", Os::Windows, Os::Linux),
            CommandMapping::new("ping", "ping")
                .with_flags(vec![
                    FlagMapping::with_description("-n", "-c", "Count of pings"),
                    FlagMapping::with_description("-t", "", "Continuous ping (use Ctrl+C)"),
                    FlagMapping::with_description("-l", "-s", "Packet size"),
                    FlagMapping::with_description("-w", "-W", "Timeout"),
                ]),
        );
        
        // tracert -> traceroute
        m.insert(
            MappingKey::new("tracert", Os::Windows, Os::Linux),
            CommandMapping::new("tracert", "traceroute")
                .with_flags(vec![
                    FlagMapping::with_description("-h", "-m", "Max hops"),
                    FlagMapping::with_description("-w", "-w", "Wait timeout"),
                ]),
        );
        
        // netstat -> netstat/ss
        m.insert(
            MappingKey::new("netstat", Os::Windows, Os::Linux),
            CommandMapping::new("netstat", "ss")
                .with_flags(vec![
                    FlagMapping::with_description("-a", "-a", "All sockets"),
                    FlagMapping::with_description("-n", "-n", "Numeric addresses"),
                    FlagMapping::with_description("-o", "-p", "Show process"),
                    FlagMapping::with_description("-b", "-p", "Show process name"),
                ]),
        );
        
        // chkdsk -> fsck
        m.insert(
            MappingKey::new("chkdsk", Os::Windows, Os::Linux),
            CommandMapping::new("chkdsk", "fsck")
                .with_flags(vec![
                    FlagMapping::with_description("/f", "-y", "Fix errors automatically"),
                    FlagMapping::with_description("/r", "-c", "Locate bad sectors"),
                    FlagMapping::with_description("/x", "", "Force dismount"),
                    FlagMapping::with_description("/i", "", "Less vigorous check"),
                    FlagMapping::with_description("/c", "", "Skip cycle checking"),
                    FlagMapping::with_description("/b", "", "Re-evaluate bad clusters"),
                ]),
        );
        
        // ============================================================
        // Linux/Unix -> Windows mappings
        // ============================================================
        
        // ls -> dir (comprehensive flag mapping)
        m.insert(
            MappingKey::new("ls", Os::Linux, Os::Windows),
            CommandMapping::new("ls", "dir")
                .with_flags(vec![
                    FlagMapping::with_description("-l", "", "Long format (default in dir)"),
                    FlagMapping::with_description("-a", "/a", "All files including hidden"),
                    FlagMapping::with_description("-A", "/a", "Almost all (exclude . and ..)"),
                    FlagMapping::with_description("-la", "/a", "All files long format"),
                    FlagMapping::with_description("-lA", "/a", "Almost all, long format"),
                    FlagMapping::with_description("-R", "/s", "Recursive listing"),
                    FlagMapping::with_description("-1", "/b", "One file per line"),
                    FlagMapping::with_description("-C", "/w", "Multi-column output"),
                    FlagMapping::with_description("-S", "/o:s", "Sort by size"),
                    FlagMapping::with_description("-t", "/o:d", "Sort by time"),
                    FlagMapping::with_description("-r", "/o:-n", "Reverse sort order"),
                    FlagMapping::with_description("-X", "/o:e", "Sort by extension"),
                    FlagMapping::with_description("-h", "", "Human readable sizes"),
                    FlagMapping::with_description("-d", "", "List directories themselves"),
                    FlagMapping::with_description("-F", "", "Append indicator"),
                    FlagMapping::with_description("-i", "", "Show inode numbers"),
                    FlagMapping::with_description("-n", "", "Numeric UID/GID"),
                    FlagMapping::with_description("-o", "", "Long without group"),
                    FlagMapping::with_description("-g", "", "Long without owner"),
                    FlagMapping::with_description("-p", "", "Append / to directories"),
                    FlagMapping::with_description("-Q", "", "Quote names"),
                    FlagMapping::with_description("-s", "", "Show block size"),
                    FlagMapping::with_description("-u", "/o:d", "Sort by access time"),
                    FlagMapping::with_description("-U", "", "Unsorted"),
                    FlagMapping::with_description("-v", "/o:n", "Natural sort of version"),
                    FlagMapping::with_description("--sort=size", "/o:s", "Sort by size"),
                    FlagMapping::with_description("--sort=time", "/o:d", "Sort by time"),
                    FlagMapping::with_description("--sort=name", "/o:n", "Sort by name"),
                    FlagMapping::with_description("--sort=extension", "/o:e", "Sort by extension"),
                    FlagMapping::with_description("--sort=none", "", "Unsorted"),
                    FlagMapping::with_description("--color", "", "Color output (N/A)"),
                    FlagMapping::with_description("--color=auto", "", "Color when terminal"),
                    FlagMapping::with_description("--color=never", "", "No color"),
                ]),
        );
        
        m.insert(
            MappingKey::new("ls", Os::MacOS, Os::Windows),
            CommandMapping::new("ls", "dir")
                .with_flags(vec![
                    FlagMapping::with_description("-l", "", "Long format"),
                    FlagMapping::with_description("-a", "/a", "All files"),
                    FlagMapping::with_description("-A", "/a", "Almost all"),
                    FlagMapping::with_description("-la", "/a", "All files long"),
                    FlagMapping::with_description("-R", "/s", "Recursive"),
                    FlagMapping::with_description("-1", "/b", "One per line"),
                    FlagMapping::with_description("-S", "/o:s", "Sort by size"),
                    FlagMapping::with_description("-t", "/o:d", "Sort by time"),
                    FlagMapping::with_description("-r", "/o:-n", "Reverse order"),
                ]),
        );
        
        // cp -> copy/xcopy (comprehensive flag mapping)
        m.insert(
            MappingKey::new("cp", Os::Linux, Os::Windows),
            CommandMapping::new("cp", "copy")
                .with_flags(vec![
                    FlagMapping::with_description("-r", "xcopy /s /e /y", "Recursive copy"),
                    FlagMapping::with_description("-R", "xcopy /s /e /y", "Recursive copy"),
                    FlagMapping::with_description("-f", "/y", "Force overwrite"),
                    FlagMapping::with_description("-i", "/-y", "Interactive/confirm"),
                    FlagMapping::with_description("-n", "/-y", "No clobber"),
                    FlagMapping::with_description("-v", "/v", "Verbose"),
                    FlagMapping::with_description("-u", "", "Update only"),
                    FlagMapping::with_description("-p", "", "Preserve attributes"),
                    FlagMapping::with_description("-a", "xcopy /s /e /h /k", "Archive mode"),
                    FlagMapping::with_description("-l", "", "Create hard links"),
                    FlagMapping::with_description("-s", "", "Create symbolic links"),
                    FlagMapping::with_description("-L", "", "Follow symlinks"),
                    FlagMapping::with_description("-P", "", "Don't follow symlinks"),
                    FlagMapping::with_description("-d", "", "Copy symlinks as symlinks"),
                    FlagMapping::with_description("-x", "", "Stay on filesystem"),
                    FlagMapping::with_description("-t", "", "Target directory"),
                    FlagMapping::with_description("-T", "", "Treat dest as normal file"),
                    FlagMapping::with_description("--backup", "", "Make backup"),
                    FlagMapping::with_description("--preserve", "", "Preserve attributes"),
                ]),
        );
        
        m.insert(
            MappingKey::new("cp", Os::MacOS, Os::Windows),
            CommandMapping::new("cp", "copy")
                .with_flags(vec![
                    FlagMapping::with_description("-r", "xcopy /s /e /y", "Recursive"),
                    FlagMapping::with_description("-R", "xcopy /s /e /y", "Recursive"),
                    FlagMapping::with_description("-f", "/y", "Force"),
                    FlagMapping::with_description("-i", "/-y", "Interactive"),
                    FlagMapping::with_description("-v", "/v", "Verbose"),
                    FlagMapping::with_description("-p", "", "Preserve attributes"),
                ]),
        );
        
        // mv -> move (comprehensive flag mapping)
        m.insert(
            MappingKey::new("mv", Os::Linux, Os::Windows),
            CommandMapping::new("mv", "move")
                .with_flags(vec![
                    FlagMapping::with_description("-f", "/y", "Force overwrite"),
                    FlagMapping::with_description("-i", "/-y", "Interactive"),
                    FlagMapping::with_description("-n", "/-y", "No clobber"),
                    FlagMapping::with_description("-u", "", "Update only"),
                    FlagMapping::with_description("-v", "", "Verbose"),
                    FlagMapping::with_description("-t", "", "Target directory"),
                    FlagMapping::with_description("-T", "", "Treat dest as file"),
                    FlagMapping::with_description("--backup", "", "Make backup"),
                ]),
        );
        
        m.insert(
            MappingKey::new("mv", Os::MacOS, Os::Windows),
            CommandMapping::new("mv", "move")
                .with_flags(vec![
                    FlagMapping::with_description("-f", "/y", "Force"),
                    FlagMapping::with_description("-i", "/-y", "Interactive"),
                    FlagMapping::with_description("-n", "/-y", "No clobber"),
                    FlagMapping::with_description("-v", "", "Verbose"),
                ]),
        );
        
        // rm -> del (comprehensive flag mapping)
        m.insert(
            MappingKey::new("rm", Os::Linux, Os::Windows),
            CommandMapping::new("rm", "del")
                .with_flags(vec![
                    FlagMapping::with_description("-r", "/s", "Recursive delete"),
                    FlagMapping::with_description("-R", "/s", "Recursive delete"),
                    FlagMapping::with_description("-f", "/q /f", "Force delete, quiet mode"),
                    FlagMapping::with_description("-i", "/p", "Interactive prompt"),
                    FlagMapping::with_description("-I", "/p", "Prompt once"),
                    // Both -rf and -fr are commonly used variants (order doesn't matter in rm)
                    FlagMapping::with_description("-rf", "/s /q /f", "Recursive force (common shorthand)"),
                    FlagMapping::with_description("-fr", "/s /q /f", "Force recursive (equivalent to -rf)"),
                    FlagMapping::with_description("-v", "", "Verbose"),
                    FlagMapping::with_description("-d", "rmdir", "Remove empty directories"),
                    FlagMapping::with_description("--preserve-root", "", "Don't delete /"),
                    FlagMapping::with_description("--no-preserve-root", "", "Allow deleting /"),
                    FlagMapping::with_description("--one-file-system", "", "Stay on filesystem"),
                ]),
        );
        
        m.insert(
            MappingKey::new("rm", Os::MacOS, Os::Windows),
            CommandMapping::new("rm", "del")
                .with_flags(vec![
                    FlagMapping::with_description("-r", "/s", "Recursive"),
                    FlagMapping::with_description("-R", "/s", "Recursive"),
                    FlagMapping::with_description("-f", "/q /f", "Force"),
                    FlagMapping::with_description("-i", "/p", "Interactive"),
                    FlagMapping::with_description("-rf", "/s /q /f", "Recursive force"),
                    FlagMapping::with_description("-v", "", "Verbose"),
                    FlagMapping::with_description("-d", "rmdir", "Remove directories"),
                    FlagMapping::with_description("-P", "", "Overwrite before delete"),
                ]),
        );
        
        // cat -> type (with additional handling)
        m.insert(
            MappingKey::new("cat", Os::Linux, Os::Windows),
            CommandMapping::new("cat", "type")
                .with_flags(vec![
                    FlagMapping::with_description("-n", "", "Number lines (N/A)"),
                    FlagMapping::with_description("-b", "", "Number non-blank lines"),
                    FlagMapping::with_description("-s", "", "Squeeze blank lines"),
                    FlagMapping::with_description("-E", "", "Show line endings"),
                    FlagMapping::with_description("-T", "", "Show tabs"),
                    FlagMapping::with_description("-A", "", "Show all"),
                    FlagMapping::with_description("-v", "", "Show non-printing"),
                ]),
        );
        
        m.insert(
            MappingKey::new("cat", Os::MacOS, Os::Windows),
            CommandMapping::new("cat", "type")
                .with_flags(vec![
                    FlagMapping::with_description("-n", "", "Number lines"),
                    FlagMapping::with_description("-b", "", "Number non-blank"),
                    FlagMapping::with_description("-s", "", "Squeeze blanks"),
                ]),
        );
        
        // clear -> cls
        m.insert(
            MappingKey::new("clear", Os::Linux, Os::Windows),
            CommandMapping::new("clear", "cls"),
        );
        
        m.insert(
            MappingKey::new("clear", Os::MacOS, Os::Windows),
            CommandMapping::new("clear", "cls"),
        );
        
        // grep -> findstr (comprehensive flag mapping)
        m.insert(
            MappingKey::new("grep", Os::Linux, Os::Windows),
            CommandMapping::new("grep", "findstr")
                .with_flags(vec![
                    FlagMapping::with_description("-i", "/i", "Case insensitive"),
                    FlagMapping::with_description("-I", "/i", "Case insensitive"),
                    FlagMapping::with_description("-r", "/s", "Recursive search"),
                    FlagMapping::with_description("-R", "/s", "Recursive search"),
                    FlagMapping::with_description("-n", "/n", "Show line numbers"),
                    FlagMapping::with_description("-v", "/v", "Invert match"),
                    FlagMapping::with_description("-c", "/c:", "Count matches"),
                    FlagMapping::with_description("-l", "/m", "List files only"),
                    FlagMapping::with_description("-L", "", "List non-matching files"),
                    FlagMapping::with_description("-E", "/r", "Extended regex"),
                    FlagMapping::with_description("-e", "", "Pattern"),
                    FlagMapping::with_description("-f", "/g:", "Patterns from file"),
                    FlagMapping::with_description("-w", "", "Whole word"),
                    FlagMapping::with_description("-x", "/x", "Whole line"),
                    FlagMapping::with_description("-o", "", "Only matching"),
                    FlagMapping::with_description("-h", "", "No filename"),
                    FlagMapping::with_description("-H", "", "With filename"),
                    FlagMapping::with_description("-q", "", "Quiet mode"),
                    FlagMapping::with_description("-s", "", "Suppress errors"),
                    FlagMapping::with_description("-m", "", "Max count"),
                    FlagMapping::with_description("-A", "", "After context"),
                    FlagMapping::with_description("-B", "", "Before context"),
                    FlagMapping::with_description("-C", "", "Context lines"),
                    FlagMapping::with_description("--color", "", "Color output"),
                    FlagMapping::with_description("--include", "", "Include pattern"),
                    FlagMapping::with_description("--exclude", "", "Exclude pattern"),
                ]),
        );
        
        m.insert(
            MappingKey::new("grep", Os::MacOS, Os::Windows),
            CommandMapping::new("grep", "findstr")
                .with_flags(vec![
                    FlagMapping::with_description("-i", "/i", "Case insensitive"),
                    FlagMapping::with_description("-r", "/s", "Recursive"),
                    FlagMapping::with_description("-R", "/s", "Recursive"),
                    FlagMapping::with_description("-n", "/n", "Line numbers"),
                    FlagMapping::with_description("-v", "/v", "Invert match"),
                    FlagMapping::with_description("-c", "/c:", "Count"),
                    FlagMapping::with_description("-l", "/m", "List files"),
                    FlagMapping::with_description("-E", "/r", "Extended regex"),
                ]),
        );
        
        // ps -> tasklist
        m.insert(
            MappingKey::new("ps", Os::Linux, Os::Windows),
            CommandMapping::new("ps", "tasklist"),
        );
        
        // kill -> taskkill
        m.insert(
            MappingKey::new("kill", Os::Linux, Os::Windows),
            CommandMapping::new("kill", "taskkill /pid")
                .with_flags(vec![
                    FlagMapping::with_description("-9", "/f", "Force kill"),
                    FlagMapping::with_description("-SIGKILL", "/f", "Force kill"),
                    FlagMapping::with_description("-SIGTERM", "", "Terminate"),
                ]),
        );
        
        // pkill -> taskkill /im
        m.insert(
            MappingKey::new("pkill", Os::Linux, Os::Windows),
            CommandMapping::new("pkill", "taskkill /im")
                .with_flags(vec![
                    FlagMapping::with_description("-9", "/f", "Force kill"),
                ]),
        );
        
        // ifconfig/ip -> ipconfig
        m.insert(
            MappingKey::new("ifconfig", Os::Linux, Os::Windows),
            CommandMapping::new("ifconfig", "ipconfig"),
        );
        
        m.insert(
            MappingKey::new("ip", Os::Linux, Os::Windows),
            CommandMapping::new("ip", "ipconfig")
                .with_flags(vec![
                    FlagMapping::with_description("addr", "/all", "Show addresses"),
                    FlagMapping::with_description("link", "", "Link info"),
                    FlagMapping::with_description("route", "", "Routing table"),
                ]),
        );
        
        // uname -> systeminfo
        m.insert(
            MappingKey::new("uname", Os::Linux, Os::Windows),
            CommandMapping::new("uname", "systeminfo")
                .with_flags(vec![
                    FlagMapping::with_description("-a", "", "All info"),
                    FlagMapping::with_description("-r", "", "Release"),
                ]),
        );
        
        // env/printenv -> set
        m.insert(
            MappingKey::new("env", Os::Linux, Os::Windows),
            CommandMapping::new("env", "set"),
        );
        
        m.insert(
            MappingKey::new("printenv", Os::Linux, Os::Windows),
            CommandMapping::new("printenv", "set"),
        );
        
        // chmod -> attrib
        m.insert(
            MappingKey::new("chmod", Os::Linux, Os::Windows),
            CommandMapping::new("chmod", "attrib"),
        );
        
        // diff -> fc
        m.insert(
            MappingKey::new("diff", Os::Linux, Os::Windows),
            CommandMapping::new("diff", "fc")
                .with_flags(vec![
                    FlagMapping::with_description("-i", "/c", "Ignore case"),
                    FlagMapping::with_description("-w", "/w", "Ignore whitespace"),
                    FlagMapping::with_description("-n", "/n", "Show line numbers"),
                ]),
        );
        
        // less/more -> more
        m.insert(
            MappingKey::new("less", Os::Linux, Os::Windows),
            CommandMapping::new("less", "more"),
        );
        
        // which -> where
        m.insert(
            MappingKey::new("which", Os::Linux, Os::Windows),
            CommandMapping::new("which", "where"),
        );
        
        m.insert(
            MappingKey::new("whereis", Os::Linux, Os::Windows),
            CommandMapping::new("whereis", "where"),
        );
        
        // touch -> type nul >
        m.insert(
            MappingKey::new("touch", Os::Linux, Os::Windows),
            CommandMapping::new("touch", "type nul >"),
        );
        
        // head/tail -> more (limited)
        m.insert(
            MappingKey::new("head", Os::Linux, Os::Windows),
            CommandMapping::new("head", "more"),
        );
        
        m.insert(
            MappingKey::new("tail", Os::Linux, Os::Windows),
            CommandMapping::new("tail", "more"),
        );
        
        // ping -> ping (different flags)
        m.insert(
            MappingKey::new("ping", Os::Linux, Os::Windows),
            CommandMapping::new("ping", "ping")
                .with_flags(vec![
                    FlagMapping::with_description("-c", "-n", "Count"),
                    FlagMapping::with_description("-s", "-l", "Packet size"),
                    FlagMapping::with_description("-W", "-w", "Timeout"),
                ]),
        );
        
        // traceroute -> tracert
        m.insert(
            MappingKey::new("traceroute", Os::Linux, Os::Windows),
            CommandMapping::new("traceroute", "tracert")
                .with_flags(vec![
                    FlagMapping::with_description("-m", "-h", "Max hops"),
                    FlagMapping::with_description("-w", "-w", "Wait timeout"),
                ]),
        );
        
        // ss/netstat -> netstat
        m.insert(
            MappingKey::new("ss", Os::Linux, Os::Windows),
            CommandMapping::new("ss", "netstat")
                .with_flags(vec![
                    FlagMapping::with_description("-a", "-a", "All sockets"),
                    FlagMapping::with_description("-n", "-n", "Numeric"),
                    FlagMapping::with_description("-p", "-o", "Show process"),
                    FlagMapping::with_description("-t", "", "TCP only"),
                    FlagMapping::with_description("-u", "", "UDP only"),
                ]),
        );
        
        // tar -> tar (Windows 10+ has tar)
        m.insert(
            MappingKey::new("tar", Os::Linux, Os::Windows),
            CommandMapping::new("tar", "tar"),
        );
        
        // curl -> curl (Windows 10+ has curl)
        m.insert(
            MappingKey::new("curl", Os::Linux, Os::Windows),
            CommandMapping::new("curl", "curl"),
        );
        
        // wget -> curl (use curl as wget alternative)
        m.insert(
            MappingKey::new("wget", Os::Linux, Os::Windows),
            CommandMapping::new("wget", "curl -O")
                .with_flags(vec![
                    FlagMapping::with_description("-O", "-o", "Output file"),
                    FlagMapping::with_description("-q", "-s", "Quiet/silent"),
                ]),
        );
        
        // df -> wmic logicaldisk
        m.insert(
            MappingKey::new("df", Os::Linux, Os::Windows),
            CommandMapping::new("df", "wmic logicaldisk get size,freespace,caption"),
        );
        
        // du -> dir (approximation)
        m.insert(
            MappingKey::new("du", Os::Linux, Os::Windows),
            CommandMapping::new("du", "dir /s"),
        );
        
        // ln -> mklink
        m.insert(
            MappingKey::new("ln", Os::Linux, Os::Windows),
            CommandMapping::new("ln", "mklink")
                .with_flags(vec![
                    FlagMapping::with_description("-s", "", "Symbolic link (default in mklink)"),
                ]),
        );
        
        // man -> help
        m.insert(
            MappingKey::new("man", Os::Linux, Os::Windows),
            CommandMapping::new("man", "help"),
        );
        
        // ============================================================
        // macOS specific additions
        // ============================================================
        
        // open -> start (macOS to Windows)
        m.insert(
            MappingKey::new("open", Os::MacOS, Os::Windows),
            CommandMapping::new("open", "start")
                .with_flags(vec![
                    FlagMapping::with_description("-a", "", "Open with application"),
                    FlagMapping::with_description("-R", "", "Reveal in Finder"),
                ]),
        );
        
        // start -> open (Windows to macOS)
        m.insert(
            MappingKey::new("start", Os::Windows, Os::MacOS),
            CommandMapping::new("start", "open"),
        );
        
        // pbcopy -> clip (macOS to Windows)
        m.insert(
            MappingKey::new("pbcopy", Os::MacOS, Os::Windows),
            CommandMapping::new("pbcopy", "clip"),
        );
        
        // pbpaste -> powershell -command Get-Clipboard (macOS to Windows)
        // Note: Requires PowerShell. For cmd.exe, there's no direct equivalent.
        m.insert(
            MappingKey::new("pbpaste", Os::MacOS, Os::Windows),
            CommandMapping::new("pbpaste", "powershell -command Get-Clipboard"),
        );
        
        // Linux xclip -> pbcopy/pbpaste (Linux to macOS)
        m.insert(
            MappingKey::new("xclip", Os::Linux, Os::MacOS),
            CommandMapping::new("xclip", "pbcopy"),
        );
        
        // xdg-open -> open (Linux to macOS)
        m.insert(
            MappingKey::new("xdg-open", Os::Linux, Os::MacOS),
            CommandMapping::new("xdg-open", "open"),
        );
        
        // open -> xdg-open (macOS to Linux)
        m.insert(
            MappingKey::new("open", Os::MacOS, Os::Linux),
            CommandMapping::new("open", "xdg-open"),
        );
        
        // pbcopy -> xclip (macOS to Linux)
        m.insert(
            MappingKey::new("pbcopy", Os::MacOS, Os::Linux),
            CommandMapping::new("pbcopy", "xclip -selection clipboard"),
        );
        
        // pbpaste -> xclip -o (macOS to Linux)
        m.insert(
            MappingKey::new("pbpaste", Os::MacOS, Os::Linux),
            CommandMapping::new("pbpaste", "xclip -selection clipboard -o"),
        );
        
        // Additional common commands
        
        // start -> xdg-open (Windows to Linux)
        m.insert(
            MappingKey::new("start", Os::Windows, Os::Linux),
            CommandMapping::new("start", "xdg-open"),
        );
        
        // clip -> xclip (Windows to Linux)
        m.insert(
            MappingKey::new("clip", Os::Windows, Os::Linux),
            CommandMapping::new("clip", "xclip -selection clipboard"),
        );
        
        // xdg-open -> start (Linux to Windows)
        m.insert(
            MappingKey::new("xdg-open", Os::Linux, Os::Windows),
            CommandMapping::new("xdg-open", "start"),
        );
        
        // xclip -> clip (Linux to Windows)
        m.insert(
            MappingKey::new("xclip", Os::Linux, Os::Windows),
            CommandMapping::new("xclip", "clip"),
        );
        
        // shutdown commands
        m.insert(
            MappingKey::new("shutdown", Os::Windows, Os::Linux),
            CommandMapping::new("shutdown", "shutdown")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "-h now", "Shutdown"),
                    FlagMapping::with_description("/r", "-r now", "Restart"),
                    FlagMapping::with_description("/t", "-t", "Timeout"),
                    FlagMapping::with_description("/a", "-c", "Abort"),
                ]),
        );
        
        m.insert(
            MappingKey::new("shutdown", Os::Linux, Os::Windows),
            CommandMapping::new("shutdown", "shutdown")
                .with_flags(vec![
                    FlagMapping::with_description("-h", "/s", "Shutdown"),
                    FlagMapping::with_description("-r", "/r", "Restart"),
                    FlagMapping::with_description("-c", "/a", "Abort"),
                ]),
        );
        
        // ============================================================
        // BSD specific mappings
        // ============================================================
        
        // FreeBSD/OpenBSD/NetBSD use similar commands to Linux
        for bsd in [Os::FreeBSD, Os::OpenBSD, Os::NetBSD] {
            // dir -> ls (Windows to BSD)
            m.insert(
                MappingKey::new("dir", Os::Windows, bsd),
                CommandMapping::new("dir", "ls")
                    .with_flags(vec![
                        FlagMapping::new("/w", "-C"),
                        FlagMapping::new("/s", "-R"),
                        FlagMapping::new("/b", "-1"),
                        FlagMapping::new("/a", "-la"),
                    ]),
            );
            
            // copy -> cp
            m.insert(
                MappingKey::new("copy", Os::Windows, bsd),
                CommandMapping::new("copy", "cp")
                    .with_flags(vec![
                        FlagMapping::new("/y", "-f"),
                        FlagMapping::new("/v", "-v"),
                    ]),
            );
            
            // ls -> dir (BSD to Windows)
            m.insert(
                MappingKey::new("ls", bsd, Os::Windows),
                CommandMapping::new("ls", "dir")
                    .with_flags(vec![
                        FlagMapping::new("-a", "/a"),
                        FlagMapping::new("-R", "/s"),
                        FlagMapping::new("-1", "/b"),
                    ]),
            );
        }
        
        m
    };
}

/// Get a command mapping if it exists
pub fn get_mapping(command: &str, from_os: Os, to_os: Os) -> Option<&'static CommandMapping> {
    let key = MappingKey::new(command, from_os, to_os);
    COMMAND_MAPPINGS.get(&key)
}

/// Check if a command is native to a specific OS
/// Returns true if the command is known to be a native command for that OS
pub fn is_native_command(command: &str, os: Os) -> bool {
    let cmd_lower = command.to_lowercase();
    
    match os {
        Os::Windows => {
            // Windows native commands
            matches!(cmd_lower.as_str(),
                "dir" | "copy" | "xcopy" | "move" | "del" | "erase" | "rmdir" | "rd" |
                "mkdir" | "md" | "type" | "cls" | "findstr" | "find" | "tasklist" |
                "taskkill" | "ipconfig" | "systeminfo" | "hostname" | "whoami" | "set" |
                "attrib" | "fc" | "more" | "ren" | "rename" | "tree" | "sort" | "where" |
                "ping" | "tracert" | "netstat" | "chkdsk" | "start" | "clip" | "shutdown" |
                "robocopy" | "icacls" | "takeown" | "sfc" | "dism" | "wmic" | "net" |
                "sc" | "reg" | "powershell" | "cmd" | "echo" | "pause" | "exit" | "call" |
                "if" | "for" | "goto" | "setlocal" | "endlocal" | "pushd" | "popd" |
                "mklink" | "assoc" | "ftype" | "path" | "title" | "color" | "prompt" |
                "ver" | "vol" | "label" | "format" | "diskpart" | "bcdedit" | "bootrec"
            )
        }
        Os::Linux | Os::FreeBSD | Os::OpenBSD | Os::NetBSD | Os::Solaris | Os::Android => {
            // Unix/Linux native commands
            matches!(cmd_lower.as_str(),
                "ls" | "cp" | "mv" | "rm" | "cat" | "clear" | "grep" | "ps" | "kill" |
                "pkill" | "ifconfig" | "ip" | "uname" | "env" | "printenv" | "export" |
                "chmod" | "chown" | "chgrp" | "diff" | "less" | "more" | "which" |
                "whereis" | "touch" | "head" | "tail" | "ping" | "traceroute" | "ss" |
                "netstat" | "tar" | "gzip" | "gunzip" | "bzip2" | "xz" | "zip" | "unzip" |
                "curl" | "wget" | "df" | "du" | "ln" | "man" | "info" | "find" | "locate" |
                "xdg-open" | "xclip" | "xsel" | "shutdown" | "reboot" | "halt" | "poweroff" |
                "systemctl" | "service" | "apt" | "apt-get" | "yum" | "dnf" | "pacman" |
                "zypper" | "emerge" | "pkg" | "brew" | "snap" | "flatpak" | "echo" | "printf" |
                "test" | "expr" | "bc" | "awk" | "sed" | "cut" | "sort" | "uniq" | "wc" |
                "tr" | "tee" | "xargs" | "date" | "cal" | "uptime" | "who" | "w" | "last" |
                "id" | "groups" | "sudo" | "su" | "passwd" | "useradd" | "userdel" | "usermod" |
                "groupadd" | "groupdel" | "crontab" | "at" | "jobs" | "fg" | "bg" | "nohup" |
                "screen" | "tmux" | "ssh" | "scp" | "sftp" | "rsync" | "nc" | "telnet" |
                "ftp" | "nmap" | "tcpdump" | "iptables" | "ufw" | "firewalld" | "mount" |
                "umount" | "fdisk" | "parted" | "mkfs" | "fsck" | "dd" | "lsblk" | "blkid" |
                "free" | "top" | "htop" | "vmstat" | "iostat" | "sar" | "strace" | "ltrace" |
                "gdb" | "valgrind" | "make" | "gcc" | "g++" | "clang" | "ld" | "ar" | "nm" |
                "objdump" | "readelf" | "ldd" | "git" | "svn" | "hg" | "cvs" | "patch" |
                "alias" | "unalias" | "history" | "source" | "exit" | "logout" | "cd" | "pwd" |
                "mkdir" | "rmdir" | "basename" | "dirname" | "realpath" | "readlink" | "stat" |
                "file" | "strings" | "hexdump" | "od" | "xxd" | "base64" | "md5sum" | "sha1sum" |
                "sha256sum" | "openssl" | "gpg" | "dmesg" | "journalctl" | "logger" | "syslog"
            )
        }
        Os::MacOS | Os::Ios => {
            // macOS native commands (BSD-based plus macOS specific)
            matches!(cmd_lower.as_str(),
                "ls" | "cp" | "mv" | "rm" | "cat" | "clear" | "grep" | "ps" | "kill" |
                "pkill" | "ifconfig" | "uname" | "env" | "printenv" | "export" |
                "chmod" | "chown" | "chgrp" | "diff" | "less" | "more" | "which" |
                "whereis" | "touch" | "head" | "tail" | "ping" | "traceroute" |
                "netstat" | "tar" | "gzip" | "gunzip" | "bzip2" | "xz" | "zip" | "unzip" |
                "curl" | "df" | "du" | "ln" | "man" | "find" | "locate" | "mdfind" |
                "open" | "pbcopy" | "pbpaste" | "say" | "caffeinate" | "osascript" |
                "defaults" | "launchctl" | "diskutil" | "hdiutil" | "sw_vers" | "system_profiler" |
                "softwareupdate" | "spctl" | "codesign" | "xcode-select" | "xcrun" |
                "brew" | "port" | "shutdown" | "reboot" | "halt" | "echo" | "printf" |
                "test" | "expr" | "bc" | "awk" | "sed" | "cut" | "sort" | "uniq" | "wc" |
                "tr" | "tee" | "xargs" | "date" | "cal" | "uptime" | "who" | "w" | "last" |
                "id" | "groups" | "sudo" | "su" | "passwd" | "dscl" | "dscacheutil" |
                "crontab" | "at" | "jobs" | "fg" | "bg" | "nohup" | "screen" | "tmux" |
                "ssh" | "scp" | "sftp" | "rsync" | "nc" | "telnet" | "ftp" | "nmap" |
                "tcpdump" | "pfctl" | "mount" | "umount" | "dd" |
                "top" | "vm_stat" | "fs_usage" | "dtrace" | "lldb" | "make" | "clang" |
                "git" | "svn" | "hg" | "patch" | "alias" | "unalias" | "history" | "source" |
                "exit" | "logout" | "cd" | "pwd" | "mkdir" | "rmdir" | "basename" | "dirname" |
                "realpath" | "readlink" | "stat" | "file" | "strings" | "hexdump" | "od" |
                "xxd" | "base64" | "md5" | "shasum" | "openssl" | "security" | "keychain"
            )
        }
        Os::Unknown => false,
    }
}

/// Check if a command exists as a target in mappings TO a specific OS
/// This helps determine if a command is already in the target OS format
pub fn is_target_command_for_os(command: &str, target_os: Os) -> bool {
    let cmd_lower = command.to_lowercase();
    
    // Check if this command appears as a target_cmd in any mapping TO the target_os
    COMMAND_MAPPINGS
        .iter()
        .filter(|(key, _)| key.to_os == target_os)
        .any(|(_, mapping)| {
            // Get the base command from target (handle cases like "cp -r")
            let target_base = mapping.target_cmd.split_whitespace().next().unwrap_or("");
            target_base.to_lowercase() == cmd_lower
        })
}

/// Get all available commands for a specific OS transition
pub fn get_available_commands(from_os: Os, to_os: Os) -> Vec<&'static str> {
    COMMAND_MAPPINGS
        .iter()
        .filter(|(key, _)| key.from_os == from_os && key.to_os == to_os)
        .map(|(_, mapping)| mapping.source_cmd.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_mapping_new() {
        let flag = FlagMapping::new("/w", "-C");
        assert_eq!(flag.source, "/w");
        assert_eq!(flag.target, "-C");
        assert!(flag.description.is_none());
    }

    #[test]
    fn test_flag_mapping_with_description() {
        let flag = FlagMapping::with_description("/w", "-C", "Wide format");
        assert_eq!(flag.source, "/w");
        assert_eq!(flag.target, "-C");
        assert_eq!(flag.description, Some("Wide format".to_string()));
    }

    #[test]
    fn test_command_mapping_new() {
        let cmd = CommandMapping::new("dir", "ls");
        assert_eq!(cmd.source_cmd, "dir");
        assert_eq!(cmd.target_cmd, "ls");
        assert!(cmd.flag_mappings.is_empty());
    }

    #[test]
    fn test_get_mapping() {
        let mapping = get_mapping("dir", Os::Windows, Os::Linux);
        assert!(mapping.is_some());
        let mapping = mapping.unwrap();
        assert_eq!(mapping.source_cmd, "dir");
        assert_eq!(mapping.target_cmd, "ls");
    }

    #[test]
    fn test_get_mapping_reverse() {
        let mapping = get_mapping("ls", Os::Linux, Os::Windows);
        assert!(mapping.is_some());
        let mapping = mapping.unwrap();
        assert_eq!(mapping.source_cmd, "ls");
        assert_eq!(mapping.target_cmd, "dir");
    }

    #[test]
    fn test_get_mapping_not_found() {
        let mapping = get_mapping("nonexistent", Os::Windows, Os::Linux);
        assert!(mapping.is_none());
    }

    #[test]
    fn test_get_available_commands() {
        let commands = get_available_commands(Os::Windows, Os::Linux);
        assert!(commands.contains(&"dir"));
        assert!(commands.contains(&"copy"));
        assert!(commands.contains(&"cls"));
    }

    #[test]
    fn test_is_native_command_windows() {
        assert!(is_native_command("dir", Os::Windows));
        assert!(is_native_command("copy", Os::Windows));
        assert!(is_native_command("cls", Os::Windows));
        assert!(is_native_command("ipconfig", Os::Windows));
        assert!(!is_native_command("ls", Os::Windows));
        assert!(!is_native_command("grep", Os::Windows));
    }

    #[test]
    fn test_is_native_command_linux() {
        assert!(is_native_command("ls", Os::Linux));
        assert!(is_native_command("grep", Os::Linux));
        assert!(is_native_command("cat", Os::Linux));
        assert!(is_native_command("chmod", Os::Linux));
        assert!(!is_native_command("dir", Os::Linux));
        assert!(!is_native_command("ipconfig", Os::Linux));
    }

    #[test]
    fn test_is_native_command_macos() {
        assert!(is_native_command("ls", Os::MacOS));
        assert!(is_native_command("open", Os::MacOS));
        assert!(is_native_command("pbcopy", Os::MacOS));
        assert!(!is_native_command("dir", Os::MacOS));
        assert!(!is_native_command("xdg-open", Os::MacOS));
    }

    #[test]
    fn test_is_native_command_case_insensitive() {
        assert!(is_native_command("DIR", Os::Windows));
        assert!(is_native_command("Dir", Os::Windows));
        assert!(is_native_command("LS", Os::Linux));
        assert!(is_native_command("Grep", Os::Linux));
    }

    #[test]
    fn test_is_target_command_for_os() {
        // ls is a target command for Linux (from Windows -> Linux mappings)
        assert!(is_target_command_for_os("ls", Os::Linux));
        // dir is a target command for Windows (from Linux -> Windows mappings)
        assert!(is_target_command_for_os("dir", Os::Windows));
    }
}
