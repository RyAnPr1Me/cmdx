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
        
        // dir -> ls
        m.insert(
            MappingKey::new("dir", Os::Windows, Os::Linux),
            CommandMapping::new("dir", "ls")
                .with_flags(vec![
                    FlagMapping::with_description("/w", "-C", "Wide list format"),
                    FlagMapping::with_description("/s", "-R", "Recursive listing"),
                    FlagMapping::with_description("/b", "-1", "Bare format (names only)"),
                    FlagMapping::with_description("/a", "-la", "All files including hidden"),
                    FlagMapping::with_description("/o:n", "--sort=name", "Sort by name"),
                    FlagMapping::with_description("/o:s", "--sort=size", "Sort by size"),
                    FlagMapping::with_description("/o:d", "--sort=time", "Sort by date"),
                    FlagMapping::with_description("/p", "", "Pause (not directly supported)"),
                    FlagMapping::with_description("/q", "-l", "Show owner"),
                ]),
        );
        
        // Also add macOS mapping (similar to Linux)
        m.insert(
            MappingKey::new("dir", Os::Windows, Os::MacOS),
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
            MappingKey::new("copy", Os::Windows, Os::Linux),
            CommandMapping::new("copy", "cp")
                .with_flags(vec![
                    FlagMapping::with_description("/y", "-f", "Force overwrite"),
                    FlagMapping::with_description("/v", "-v", "Verbose"),
                    FlagMapping::with_description("/a", "", "ASCII mode (N/A)"),
                    FlagMapping::with_description("/b", "", "Binary mode (default)"),
                ]),
        );
        
        m.insert(
            MappingKey::new("copy", Os::Windows, Os::MacOS),
            CommandMapping::new("copy", "cp")
                .with_flags(vec![
                    FlagMapping::new("/y", "-f"),
                    FlagMapping::new("/v", "-v"),
                ]),
        );
        
        // xcopy -> cp -r
        m.insert(
            MappingKey::new("xcopy", Os::Windows, Os::Linux),
            CommandMapping::new("xcopy", "cp -r")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "", "Copy subdirs (implied by -r)"),
                    FlagMapping::with_description("/e", "", "Copy empty dirs too"),
                    FlagMapping::with_description("/y", "-f", "Force overwrite"),
                    FlagMapping::with_description("/i", "", "Assume destination is directory"),
                    FlagMapping::with_description("/q", "-q", "Quiet mode"),
                ]),
        );
        
        // move -> mv
        m.insert(
            MappingKey::new("move", Os::Windows, Os::Linux),
            CommandMapping::new("move", "mv")
                .with_flags(vec![
                    FlagMapping::with_description("/y", "-f", "Force overwrite"),
                ]),
        );
        
        m.insert(
            MappingKey::new("move", Os::Windows, Os::MacOS),
            CommandMapping::new("move", "mv")
                .with_flags(vec![
                    FlagMapping::new("/y", "-f"),
                ]),
        );
        
        // del/erase -> rm
        m.insert(
            MappingKey::new("del", Os::Windows, Os::Linux),
            CommandMapping::new("del", "rm")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "-r", "Recursive"),
                    FlagMapping::with_description("/q", "-f", "Quiet/Force"),
                    FlagMapping::with_description("/f", "-f", "Force"),
                    FlagMapping::with_description("/p", "-i", "Prompt before delete"),
                ]),
        );
        
        m.insert(
            MappingKey::new("erase", Os::Windows, Os::Linux),
            CommandMapping::new("erase", "rm")
                .with_flags(vec![
                    FlagMapping::new("/s", "-r"),
                    FlagMapping::new("/q", "-f"),
                    FlagMapping::new("/f", "-f"),
                    FlagMapping::new("/p", "-i"),
                ]),
        );
        
        // rmdir/rd -> rm -r or rmdir
        m.insert(
            MappingKey::new("rmdir", Os::Windows, Os::Linux),
            CommandMapping::new("rmdir", "rm -r")
                .with_flags(vec![
                    FlagMapping::with_description("/s", "", "Recursive (implied)"),
                    FlagMapping::with_description("/q", "-f", "Quiet"),
                ]),
        );
        
        m.insert(
            MappingKey::new("rd", Os::Windows, Os::Linux),
            CommandMapping::new("rd", "rm -r")
                .with_flags(vec![
                    FlagMapping::new("/s", ""),
                    FlagMapping::new("/q", "-f"),
                ]),
        );
        
        // mkdir/md -> mkdir
        m.insert(
            MappingKey::new("mkdir", Os::Windows, Os::Linux),
            CommandMapping::new("mkdir", "mkdir")
                .with_flags(vec![
                    FlagMapping::with_description("/p", "-p", "Create parent directories"),
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
            CommandMapping::new("chkdsk", "fsck"),
        );
        
        // ============================================================
        // Linux/Unix -> Windows mappings
        // ============================================================
        
        // ls -> dir
        m.insert(
            MappingKey::new("ls", Os::Linux, Os::Windows),
            CommandMapping::new("ls", "dir")
                .with_flags(vec![
                    FlagMapping::with_description("-l", "", "Long format (default)"),
                    FlagMapping::with_description("-a", "/a", "All files"),
                    FlagMapping::with_description("-la", "/a", "All files long format"),
                    FlagMapping::with_description("-R", "/s", "Recursive"),
                    FlagMapping::with_description("-1", "/b", "One file per line"),
                    FlagMapping::with_description("-S", "/o:s", "Sort by size"),
                    FlagMapping::with_description("-t", "/o:d", "Sort by time"),
                    FlagMapping::with_description("-r", "/o:-n", "Reverse order"),
                    FlagMapping::with_description("--sort=size", "/o:s", "Sort by size"),
                    FlagMapping::with_description("--sort=time", "/o:d", "Sort by time"),
                ]),
        );
        
        m.insert(
            MappingKey::new("ls", Os::MacOS, Os::Windows),
            CommandMapping::new("ls", "dir")
                .with_flags(vec![
                    FlagMapping::new("-l", ""),
                    FlagMapping::new("-a", "/a"),
                    FlagMapping::new("-la", "/a"),
                    FlagMapping::new("-R", "/s"),
                ]),
        );
        
        // cp -> copy/xcopy
        m.insert(
            MappingKey::new("cp", Os::Linux, Os::Windows),
            CommandMapping::new("cp", "copy")
                .with_flags(vec![
                    FlagMapping::with_description("-r", "xcopy /s /e", "Recursive copy"),
                    FlagMapping::with_description("-R", "xcopy /s /e", "Recursive copy"),
                    FlagMapping::with_description("-f", "/y", "Force overwrite"),
                    FlagMapping::with_description("-v", "/v", "Verbose"),
                    FlagMapping::with_description("-i", "/-y", "Interactive/confirm"),
                ]),
        );
        
        // mv -> move
        m.insert(
            MappingKey::new("mv", Os::Linux, Os::Windows),
            CommandMapping::new("mv", "move")
                .with_flags(vec![
                    FlagMapping::with_description("-f", "/y", "Force overwrite"),
                    FlagMapping::with_description("-i", "/-y", "Interactive"),
                ]),
        );
        
        // rm -> del
        m.insert(
            MappingKey::new("rm", Os::Linux, Os::Windows),
            CommandMapping::new("rm", "del")
                .with_flags(vec![
                    FlagMapping::with_description("-r", "/s", "Recursive"),
                    FlagMapping::with_description("-R", "/s", "Recursive"),
                    FlagMapping::with_description("-f", "/q /f", "Force/quiet"),
                    FlagMapping::with_description("-rf", "/s /q", "Recursive force"),
                    FlagMapping::with_description("-i", "/p", "Interactive"),
                ]),
        );
        
        // cat -> type
        m.insert(
            MappingKey::new("cat", Os::Linux, Os::Windows),
            CommandMapping::new("cat", "type"),
        );
        
        m.insert(
            MappingKey::new("cat", Os::MacOS, Os::Windows),
            CommandMapping::new("cat", "type"),
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
        
        // grep -> findstr
        m.insert(
            MappingKey::new("grep", Os::Linux, Os::Windows),
            CommandMapping::new("grep", "findstr")
                .with_flags(vec![
                    FlagMapping::with_description("-i", "/i", "Case insensitive"),
                    FlagMapping::with_description("-r", "/s", "Recursive"),
                    FlagMapping::with_description("-R", "/s", "Recursive"),
                    FlagMapping::with_description("-n", "/n", "Line numbers"),
                    FlagMapping::with_description("-v", "/v", "Invert match"),
                    FlagMapping::with_description("-c", "/c:", "Count matches"),
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
}
