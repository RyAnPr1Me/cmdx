//! Package manager command translation between Linux distributions

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::distro::PackageManager;

/// Package manager operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageOperation {
    /// Install a package
    Install,
    /// Remove/uninstall a package
    Remove,
    /// Update package lists
    Update,
    /// Upgrade installed packages
    Upgrade,
    /// Search for packages
    Search,
    /// Show package information
    Info,
    /// List installed packages
    List,
    /// Clean package cache
    Clean,
    /// Auto-remove unused dependencies
    AutoRemove,
}

impl fmt::Display for PackageOperation {
    /// Formats the package operation as its lowercase command name.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::translator::package_manager::PackageOperation;
    /// assert_eq!(format!("{}", PackageOperation::Install), "install");
    /// assert_eq!(format!("{}", PackageOperation::AutoRemove), "autoremove");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageOperation::Install => write!(f, "install"),
            PackageOperation::Remove => write!(f, "remove"),
            PackageOperation::Update => write!(f, "update"),
            PackageOperation::Upgrade => write!(f, "upgrade"),
            PackageOperation::Search => write!(f, "search"),
            PackageOperation::Info => write!(f, "info"),
            PackageOperation::List => write!(f, "list"),
            PackageOperation::Clean => write!(f, "clean"),
            PackageOperation::AutoRemove => write!(f, "autoremove"),
        }
    }
}

/// Result of a package manager command translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTranslationResult {
    /// The translated command
    pub command: String,
    /// Original command
    pub original: String,
    /// Source package manager
    pub from_pm: PackageManager,
    /// Target package manager
    pub to_pm: PackageManager,
    /// Warnings or notes about the translation
    pub warnings: Vec<String>,
    /// Whether the command may require elevated privileges (sudo)
    pub requires_sudo: bool,
}

impl PackageTranslationResult {
    /// Creates a new translation result for a package manager command.
    ///
    /// Initializes `warnings` as an empty list and `requires_sudo` as `false`.
    ///
    /// # Parameters
    ///
    /// - `command`: The translated command string (may be empty until translation is assembled).
    /// - `original`: The original input command provided by the user.
    /// - `from_pm`: The detected or specified source package manager.
    /// - `to_pm`: The target package manager to translate the command into.
    ///
    /// # Returns
    ///
    /// A `PackageTranslationResult` populated with the provided fields and default `warnings` and `requires_sudo`.
    ///
    /// # Examples
    ///
    /// ```
    /// let res = crate::translator::package_manager::PackageTranslationResult::new(
    ///     "apt install foo".to_string(),
    ///     "dnf install foo".to_string(),
    ///     crate::distro::PackageManager::Dnf,
    ///     crate::distro::PackageManager::Apt,
    /// );
    /// assert_eq!(res.original, "dnf install foo");
    /// assert!(res.warnings.is_empty());
    /// assert!(!res.requires_sudo);
    /// ```
    pub fn new(
        command: String,
        original: String,
        from_pm: PackageManager,
        to_pm: PackageManager,
    ) -> Self {
        Self {
            command,
            original,
            from_pm,
            to_pm,
            warnings: Vec::new(),
            requires_sudo: false,
        }
    }
}

impl fmt::Display for PackageTranslationResult {
    /// Formats the translation as its translated command string.
    ///
    /// This `Display` implementation prints only the `command` field of the
    /// `PackageTranslationResult`.
    ///
    /// # Examples
    ///
    /// ```
    /// use translator::package_manager::{PackageTranslationResult, PackageManager};
    ///
    /// let res = PackageTranslationResult {
    ///     command: "dnf install foo".into(),
    ///     original: "apt install foo".into(),
    ///     from_pm: PackageManager::Apt,
    ///     to_pm: PackageManager::Dnf,
    ///     warnings: Vec::new(),
    ///     requires_sudo: false,
    /// };
    ///
    /// assert_eq!(format!("{}", res), "dnf install foo");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.command)
    }
}

/// Errors that can occur during package manager translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageTranslationError {
    /// Command not recognized as a package manager command
    NotPackageManagerCommand(String),
    /// Operation not supported by target package manager
    UnsupportedOperation(String),
    /// Empty command
    EmptyCommand,
    /// Same source and target package manager
    SamePackageManager,
}

impl fmt::Display for PackageTranslationError {
    /// Formats a PackageTranslationError into a human-readable message.
    ///
    /// Each error variant is rendered as a concise string suitable for display to users.
    ///
    /// # Examples
    ///
    /// ```
    /// use translator::package_manager::PackageTranslationError;
    /// let err = PackageTranslationError::EmptyCommand;
    /// assert_eq!(format!("{}", err), "Empty command provided");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageTranslationError::NotPackageManagerCommand(cmd) => {
                write!(f, "Not a recognized package manager command: '{}'", cmd)
            }
            PackageTranslationError::UnsupportedOperation(op) => {
                write!(f, "Operation '{}' not supported by target package manager", op)
            }
            PackageTranslationError::EmptyCommand => {
                write!(f, "Empty command provided")
            }
            PackageTranslationError::SamePackageManager => {
                write!(f, "Source and target package managers are the same")
            }
        }
    }
}

impl std::error::Error for PackageTranslationError {}

/// Flag mapping for package manager operations
#[derive(Debug, Clone)]
pub struct PackageFlagMapping {
    /// The source flag
    pub source: String,
    /// The target flag
    pub target: String,
    /// Description of what this flag does
    pub description: Option<String>,
}

impl PackageFlagMapping {
    /// Creates a `PackageFlagMapping` that maps a source flag to a target flag.
    ///
    /// The `source` is the flag form used by the originating package manager (e.g., `"-y"`),
    /// and the `target` is the equivalent form for the destination package manager (e.g., `"--assumeyes"`).
    ///
    /// # Examples
    ///
    /// ```
    /// let m = PackageFlagMapping::new("-y", "--assumeyes");
    /// assert_eq!(m.source, "-y");
    /// assert_eq!(m.target, "--assumeyes");
    /// assert!(m.description.is_none());
    /// ```
    pub fn new(source: &str, target: &str) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            description: None,
        }
    }

    /// Creates a `PackageFlagMapping` that associates a source flag with a target flag and a human-readable description.
    ///
    /// # Parameters
    ///
    /// - `source`: the flag string used by the source package manager (e.g., "-y").
    /// - `target`: the equivalent flag string or strings for the target package manager (e.g., "--assumeyes").
    /// - `description`: a short explanation of the flag's purpose or behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = PackageFlagMapping::with_description("-y", "--assumeyes", "automatically answer yes to prompts");
    /// assert_eq!(m.source, "-y");
    /// assert_eq!(m.target, "--assumeyes");
    /// assert_eq!(m.description.as_deref(), Some("automatically answer yes to prompts"));
    /// ```
    pub fn with_description(source: &str, target: &str, description: &str) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            description: Some(description.to_string()),
        }
    }
}

/// Mapping for a package manager operation
#[derive(Debug, Clone)]
struct OperationMapping {
    /// The command pattern for this operation
    command: String,
    /// Whether this operation requires sudo
    requires_sudo: bool,
    /// Flag mappings for this operation
    flag_mappings: Vec<PackageFlagMapping>,
    /// Additional flags or notes
    notes: Option<String>,
}

impl OperationMapping {
    /// Create an OperationMapping with the given base command and sudo requirement.
    ///
    /// The mapping is initialized with an empty list of flag mappings and no notes.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = OperationMapping::new("apt install".into(), true);
    /// assert_eq!(m.command, "apt install");
    /// assert!(m.requires_sudo);
    /// assert!(m.flag_mappings.is_empty());
    /// assert!(m.notes.is_none());
    /// ```
    fn new(command: &str, requires_sudo: bool) -> Self {
        Self {
            command: command.to_string(),
            requires_sudo,
            flag_mappings: Vec::new(),
            notes: None,
        }
    }

    /// Sets the operation's flag mappings and returns the updated `OperationMapping`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mapping = OperationMapping::new("apt install".to_string(), true)
    ///     .with_flags(vec![PackageFlagMapping::new("-y", "--yes")]);
    /// assert_eq!(mapping.flag_mappings.len(), 1);
    /// ```
    fn with_flags(mut self, flags: Vec<PackageFlagMapping>) -> Self {
        self.flag_mappings = flags;
        self
    }

    /// Sets supplementary notes for the operation mapping and returns the updated mapping.
    ///
    /// The `notes` string provides human-readable additional information about the operation
    /// (e.g., caveats or behavioral details) and is stored on the mapping.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mapping = OperationMapping::new("apt install".to_string(), true)
    ///     .with_notes("Requires network access and may prompt for confirmation");
    /// ```
    fn with_notes(mut self, notes: &str) -> Self {
        self.notes = Some(notes.to_string());
        self
    }
}

/// Key for looking up operation mappings
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct OperationKey {
    pm: PackageManager,
    op: PackageOperation,
}

/// Key for looking up flag mappings between package managers
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FlagMappingKey {
    from_pm: PackageManager,
    to_pm: PackageManager,
    op: PackageOperation,
}

lazy_static! {
    /// Global package manager operation mappings
    static ref OPERATION_MAPPINGS: HashMap<OperationKey, OperationMapping> = {
        let mut m = HashMap::new();

        // ============================================================
        // APT (Debian/Ubuntu) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Install },
            OperationMapping::new("apt install", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Remove },
            OperationMapping::new("apt remove", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Update },
            OperationMapping::new("apt update", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Upgrade },
            OperationMapping::new("apt upgrade", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Search },
            OperationMapping::new("apt search", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Info },
            OperationMapping::new("apt show", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::List },
            OperationMapping::new("apt list --installed", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Clean },
            OperationMapping::new("apt clean", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::AutoRemove },
            OperationMapping::new("apt autoremove", true),
        );

        // ============================================================
        // YUM (RHEL/CentOS) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Install },
            OperationMapping::new("yum install", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Remove },
            OperationMapping::new("yum remove", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Update },
            OperationMapping::new("yum check-update", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Upgrade },
            OperationMapping::new("yum update", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Search },
            OperationMapping::new("yum search", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Info },
            OperationMapping::new("yum info", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::List },
            OperationMapping::new("yum list installed", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Clean },
            OperationMapping::new("yum clean all", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::AutoRemove },
            OperationMapping::new("yum autoremove", true),
        );

        // ============================================================
        // DNF (Fedora) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Install },
            OperationMapping::new("dnf install", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Remove },
            OperationMapping::new("dnf remove", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Update },
            OperationMapping::new("dnf check-update", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Upgrade },
            OperationMapping::new("dnf upgrade", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Search },
            OperationMapping::new("dnf search", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Info },
            OperationMapping::new("dnf info", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::List },
            OperationMapping::new("dnf list installed", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Clean },
            OperationMapping::new("dnf clean all", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::AutoRemove },
            OperationMapping::new("dnf autoremove", true),
        );

        // ============================================================
        // Pacman (Arch) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Install },
            OperationMapping::new("pacman -S", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Remove },
            OperationMapping::new("pacman -R", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Update },
            OperationMapping::new("pacman -Sy", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Upgrade },
            OperationMapping::new("pacman -Syu", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Search },
            OperationMapping::new("pacman -Ss", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Info },
            OperationMapping::new("pacman -Si", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::List },
            OperationMapping::new("pacman -Q", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Clean },
            OperationMapping::new("pacman -Sc", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::AutoRemove },
            OperationMapping::new("pacman -Rs", true).with_notes("Removes package with unused dependencies"),
        );

        // ============================================================
        // Zypper (openSUSE) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Install },
            OperationMapping::new("zypper install", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Remove },
            OperationMapping::new("zypper remove", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Update },
            OperationMapping::new("zypper refresh", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Upgrade },
            OperationMapping::new("zypper update", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Search },
            OperationMapping::new("zypper search", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Info },
            OperationMapping::new("zypper info", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::List },
            OperationMapping::new("zypper search --installed-only", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Clean },
            OperationMapping::new("zypper clean", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::AutoRemove },
            OperationMapping::new("zypper remove --clean-deps", true),
        );

        // ============================================================
        // APK (Alpine) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Install },
            OperationMapping::new("apk add", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Remove },
            OperationMapping::new("apk del", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Update },
            OperationMapping::new("apk update", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Upgrade },
            OperationMapping::new("apk upgrade", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Search },
            OperationMapping::new("apk search", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Info },
            OperationMapping::new("apk info", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::List },
            OperationMapping::new("apk list --installed", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Clean },
            OperationMapping::new("apk cache clean", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::AutoRemove },
            OperationMapping::new("apk del", true).with_notes("Use with package name and dependencies"),
        );

        // ============================================================
        // Emerge (Gentoo) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Install },
            OperationMapping::new("emerge", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Remove },
            OperationMapping::new("emerge --unmerge", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Update },
            OperationMapping::new("emerge --sync", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Upgrade },
            OperationMapping::new("emerge --update --deep --with-bdeps=y @world", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Search },
            OperationMapping::new("emerge --search", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Info },
            OperationMapping::new("emerge --info", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::List },
            OperationMapping::new("qlist -I", false).with_notes("Requires portage-utils"),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Clean },
            OperationMapping::new("emerge --depclean", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::AutoRemove },
            OperationMapping::new("emerge --depclean", true),
        );

        // ============================================================
        // XBPS (Void) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Install },
            OperationMapping::new("xbps-install", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Remove },
            OperationMapping::new("xbps-remove", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Update },
            OperationMapping::new("xbps-install -S", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Upgrade },
            OperationMapping::new("xbps-install -Su", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Search },
            OperationMapping::new("xbps-query -Rs", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Info },
            OperationMapping::new("xbps-query -R", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::List },
            OperationMapping::new("xbps-query -l", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Clean },
            OperationMapping::new("xbps-remove -O", true),
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::AutoRemove },
            OperationMapping::new("xbps-remove -o", true),
        );

        // ============================================================
        // Nix (NixOS) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Install },
            OperationMapping::new("nix-env -i", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Remove },
            OperationMapping::new("nix-env -e", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Update },
            OperationMapping::new("nix-channel --update", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Upgrade },
            OperationMapping::new("nix-env -u", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Search },
            OperationMapping::new("nix search", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Info },
            OperationMapping::new("nix-env -qa --description", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::List },
            OperationMapping::new("nix-env -q", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Clean },
            OperationMapping::new("nix-collect-garbage", false),
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::AutoRemove },
            OperationMapping::new("nix-collect-garbage -d", false),
        );

        m
    };

    /// Global flag mappings between package managers for different operations
    static ref FLAG_MAPPINGS: HashMap<FlagMappingKey, Vec<PackageFlagMapping>> = {
        let mut m = HashMap::new();

        // ============================================================
        // APT to other package managers - Install operation flags
        // ============================================================
        
        // APT -> DNF (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Dnf, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("-y", "-y", "Assume yes to all prompts"),
                PackageFlagMapping::with_description("--yes", "-y", "Assume yes to all prompts"),
                PackageFlagMapping::with_description("--assume-yes", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--no-install-recommends", "--setopt=install_weak_deps=False", "Don't install weak dependencies"),
                PackageFlagMapping::with_description("--reinstall", "--reinstall", "Reinstall package"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet mode"),
                PackageFlagMapping::with_description("--quiet", "--quiet", "Quiet mode"),
            ],
        );

        // APT -> YUM (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Yum, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("-y", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--yes", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--assume-yes", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--reinstall", "reinstall", "Reinstall package"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet mode"),
                PackageFlagMapping::with_description("--quiet", "--quiet", "Quiet mode"),
            ],
        );

        // APT -> Pacman (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Pacman, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("-y", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("--yes", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("--assume-yes", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("--no-install-recommends", "--asdeps", "Install as dependencies"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
                PackageFlagMapping::with_description("--quiet", "--quiet", "Quiet"),
            ],
        );

        // APT -> Zypper (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Zypper, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("-y", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--yes", "--no-confirm", "No confirmation"),
                PackageFlagMapping::with_description("--assume-yes", "--non-interactive", "Non-interactive"),
                PackageFlagMapping::with_description("--reinstall", "--force", "Force reinstall"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
            ],
        );

        // ============================================================
        // DNF/YUM to other package managers - Install operation flags
        // ============================================================

        // DNF -> APT (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Dnf, to_pm: PackageManager::Apt, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("-y", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--assumeyes", "--assume-yes", "Assume yes"),
                PackageFlagMapping::with_description("--reinstall", "--reinstall", "Reinstall"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
                PackageFlagMapping::with_description("--quiet", "--quiet", "Quiet"),
            ],
        );

        // YUM -> APT (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Yum, to_pm: PackageManager::Apt, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("-y", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--assumeyes", "--assume-yes", "Assume yes"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
                PackageFlagMapping::with_description("--quiet", "--quiet", "Quiet"),
            ],
        );

        // DNF -> Pacman (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Dnf, to_pm: PackageManager::Pacman, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("-y", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("--assumeyes", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
            ],
        );

        // ============================================================
        // Pacman to other package managers - Install operation flags
        // ============================================================

        // Pacman -> APT (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Pacman, to_pm: PackageManager::Apt, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("--noconfirm", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--asdeps", "", "Install as dependency (no direct equivalent)"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
                PackageFlagMapping::with_description("--quiet", "--quiet", "Quiet"),
            ],
        );

        // Pacman -> DNF (Install)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Pacman, to_pm: PackageManager::Dnf, op: PackageOperation::Install },
            vec![
                PackageFlagMapping::with_description("--noconfirm", "-y", "Assume yes"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
            ],
        );

        // ============================================================
        // Remove operation flags
        // ============================================================

        // APT -> DNF (Remove)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Dnf, op: PackageOperation::Remove },
            vec![
                PackageFlagMapping::with_description("-y", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--yes", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--purge", "", "Purge config files (no direct equivalent)"),
                PackageFlagMapping::with_description("--auto-remove", "--noautoremove", "Don't auto-remove dependencies"),
            ],
        );

        // APT -> Pacman (Remove)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Pacman, op: PackageOperation::Remove },
            vec![
                PackageFlagMapping::with_description("-y", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("--yes", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("--purge", "-n", "Remove config files"),
            ],
        );

        // Pacman -> APT (Remove)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Pacman, to_pm: PackageManager::Apt, op: PackageOperation::Remove },
            vec![
                PackageFlagMapping::with_description("--noconfirm", "-y", "Assume yes"),
                PackageFlagMapping::with_description("-n", "--purge", "Remove config files"),
                PackageFlagMapping::with_description("-s", "--auto-remove", "Remove unused dependencies"),
            ],
        );

        // ============================================================
        // Update/Upgrade operation flags
        // ============================================================

        // APT -> DNF (Upgrade)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Dnf, op: PackageOperation::Upgrade },
            vec![
                PackageFlagMapping::with_description("-y", "-y", "Assume yes"),
                PackageFlagMapping::with_description("--yes", "-y", "Assume yes"),
                PackageFlagMapping::with_description("-q", "-q", "Quiet"),
            ],
        );

        // APT -> Pacman (Upgrade)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Pacman, op: PackageOperation::Upgrade },
            vec![
                PackageFlagMapping::with_description("-y", "--noconfirm", "No confirmation"),
                PackageFlagMapping::with_description("--yes", "--noconfirm", "No confirmation"),
            ],
        );

        // Pacman -> APT (Upgrade)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Pacman, to_pm: PackageManager::Apt, op: PackageOperation::Upgrade },
            vec![
                PackageFlagMapping::with_description("--noconfirm", "-y", "Assume yes"),
            ],
        );

        // ============================================================
        // Search operation flags
        // ============================================================

        // APT -> DNF (Search)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Dnf, op: PackageOperation::Search },
            vec![
                PackageFlagMapping::with_description("-n", "", "Search names only (different in DNF)"),
                PackageFlagMapping::with_description("--names-only", "", "Search names only"),
            ],
        );

        // APT -> Pacman (Search)
        m.insert(
            FlagMappingKey { from_pm: PackageManager::Apt, to_pm: PackageManager::Pacman, op: PackageOperation::Search },
            vec![
                PackageFlagMapping::with_description("-n", "", "Search names only"),
                PackageFlagMapping::with_description("--names-only", "", "Search names only"),
            ],
        );

        // ============================================================
        // Cross-compatible flags (work similarly across package managers)
        // ============================================================

        // Verbose flags
        for from_pm in [PackageManager::Apt, PackageManager::Dnf, PackageManager::Yum, PackageManager::Zypper] {
            for to_pm in [PackageManager::Apt, PackageManager::Dnf, PackageManager::Yum, PackageManager::Zypper] {
                if from_pm != to_pm {
                    for op in [PackageOperation::Install, PackageOperation::Remove, PackageOperation::Upgrade] {
                        m.entry(FlagMappingKey { from_pm, to_pm, op })
                            .or_insert_with(Vec::new)
                            .push(PackageFlagMapping::with_description("-v", "-v", "Verbose output"));
                    }
                }
            }
        }

        m
    };
}

/// Determines the package manager, the requested operation, and the remaining arguments from a command string.
///
/// The input is trimmed and may start with an optional leading `sudo`. Known package manager tokens (e.g., `apt`, `dnf`, `pacman`, etc.) and their operations are recognized; the function returns a tuple of `(PackageManager, PackageOperation, Vec<String>)` where the vector contains the arguments (package names or remaining tokens) following the detected operation. Returns `PackageTranslationError::EmptyCommand` for empty input, `PackageTranslationError::NotPackageManagerCommand` if no known package manager is found, or `PackageTranslationError::UnsupportedOperation` if the detected package manager does not support the parsed operation.
///
/// # Examples
///
/// ```
/// let (pm, op, args) = parse_package_command("sudo apt install curl").unwrap();
/// assert_eq!(pm, PackageManager::Apt);
/// assert_eq!(op, PackageOperation::Install);
/// assert_eq!(args, vec!["curl".to_string()]);
/// ```
fn parse_package_command(input: &str) -> Result<(PackageManager, PackageOperation, Vec<String>), PackageTranslationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PackageTranslationError::EmptyCommand);
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return Err(PackageTranslationError::EmptyCommand);
    }

    // Detect package manager
    let (pm, start_idx) = if parts[0] == "sudo" && parts.len() > 1 {
        // Strip sudo prefix
        detect_package_manager(parts[1])?
    } else {
        detect_package_manager(parts[0])?
    };

    let start_idx = if parts[0] == "sudo" { start_idx + 1 } else { start_idx };

    // Detect operation
    if start_idx >= parts.len() {
        return Err(PackageTranslationError::NotPackageManagerCommand(input.to_string()));
    }

    let operation = detect_operation(pm, &parts[start_idx..])?;

    // Collect remaining arguments (package names, etc.)
    let args: Vec<String> = parts[start_idx + 1..].iter().map(|s| s.to_string()).collect();

    Ok((pm, operation, args))
}

/// Identify the package manager represented by the first command token.
///
/// Returns the detected `PackageManager` and the token index where the operation
/// is expected (typically `1` for a single-token manager name). Returns
/// `PackageTranslationError::NotPackageManagerCommand` when the token is not a
/// recognized package manager.
///
/// # Examples
///
/// ```
/// let (pm, idx) = super::detect_package_manager("apt").unwrap();
/// assert_eq!(pm, super::distro::PackageManager::Apt);
/// assert_eq!(idx, 1);
/// ```
fn detect_package_manager(cmd: &str) -> Result<(PackageManager, usize), PackageTranslationError> {
    match cmd.to_lowercase().as_str() {
        "apt" | "apt-get" | "aptitude" => Ok((PackageManager::Apt, 1)),
        "yum" => Ok((PackageManager::Yum, 1)),
        "dnf" => Ok((PackageManager::Dnf, 1)),
        "pacman" => Ok((PackageManager::Pacman, 1)),
        "zypper" => Ok((PackageManager::Zypper, 1)),
        "apk" => Ok((PackageManager::Apk, 1)),
        "emerge" => Ok((PackageManager::Emerge, 1)),
        "xbps-install" | "xbps-remove" | "xbps-query" => Ok((PackageManager::Xbps, 1)),
        "nix-env" | "nix" => Ok((PackageManager::Nix, 1)),
        _ => Err(PackageTranslationError::NotPackageManagerCommand(cmd.to_string())),
    }
}

/// Map the first token of a command to the corresponding `PackageOperation` for a given package manager.
///
/// This inspects `parts[0]` (case-insensitive) and returns the operation that token represents for `pm`.
/// Returns `PackageTranslationError::NotPackageManagerCommand` if `parts` is empty, and
/// `PackageTranslationError::UnsupportedOperation(token)` if the token is not recognized for the provided package manager.
///
/// # Examples
///
/// ```
/// let parts = ["install", "vim"];
/// let op = detect_operation(PackageManager::Apt, &parts).unwrap();
/// assert_eq!(op, PackageOperation::Install);
/// ```
fn detect_operation(pm: PackageManager, parts: &[&str]) -> Result<PackageOperation, PackageTranslationError> {
    if parts.is_empty() {
        return Err(PackageTranslationError::NotPackageManagerCommand("".to_string()));
    }

    let op_str = parts[0].to_lowercase();

    match pm {
        PackageManager::Apt => match op_str.as_str() {
            "install" => Ok(PackageOperation::Install),
            "remove" | "uninstall" | "purge" => Ok(PackageOperation::Remove),
            "update" => Ok(PackageOperation::Update),
            "upgrade" | "full-upgrade" | "dist-upgrade" => Ok(PackageOperation::Upgrade),
            "search" => Ok(PackageOperation::Search),
            "show" | "info" => Ok(PackageOperation::Info),
            "list" => Ok(PackageOperation::List),
            "clean" | "autoclean" => Ok(PackageOperation::Clean),
            "autoremove" => Ok(PackageOperation::AutoRemove),
            _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
        },
        PackageManager::Yum | PackageManager::Dnf => match op_str.as_str() {
            "install" => Ok(PackageOperation::Install),
            "remove" | "erase" => Ok(PackageOperation::Remove),
            "check-update" => Ok(PackageOperation::Update),
            "update" | "upgrade" => Ok(PackageOperation::Upgrade),
            "search" => Ok(PackageOperation::Search),
            "info" => Ok(PackageOperation::Info),
            "list" => Ok(PackageOperation::List),
            "clean" => Ok(PackageOperation::Clean),
            "autoremove" => Ok(PackageOperation::AutoRemove),
            _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
        },
        PackageManager::Pacman => {
            // Pacman uses flags rather than subcommands
            // Common flags: -S (sync/install), -R (remove), -Sy (update), -Syu (upgrade)
            // -Ss (search), -Si/-Qi (info), -Q (query/list), -Sc (clean)
            if op_str.starts_with('-') {
                match op_str.as_str() {
                    "-syu" => Ok(PackageOperation::Upgrade),
                    "-sy" => Ok(PackageOperation::Update),
                    "-s" => Ok(PackageOperation::Install),
                    "-rs" => Ok(PackageOperation::Remove),
                    "-r" => Ok(PackageOperation::Remove),
                    "-ss" => Ok(PackageOperation::Search),
                    "-si" | "-qi" => Ok(PackageOperation::Info),
                    "-q" => Ok(PackageOperation::List),
                    "-sc" | "-scc" => Ok(PackageOperation::Clean),
                    _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
                }
            } else {
                Err(PackageTranslationError::UnsupportedOperation(op_str))
            }
        },
        PackageManager::Zypper => match op_str.as_str() {
            "install" | "in" => Ok(PackageOperation::Install),
            "remove" | "rm" => Ok(PackageOperation::Remove),
            "refresh" | "ref" => Ok(PackageOperation::Update),
            "update" | "up" => Ok(PackageOperation::Upgrade),
            "search" | "se" => Ok(PackageOperation::Search),
            "info" | "if" => Ok(PackageOperation::Info),
            "packages" => Ok(PackageOperation::List),
            "clean" => Ok(PackageOperation::Clean),
            _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
        },
        PackageManager::Apk => match op_str.as_str() {
            "add" => Ok(PackageOperation::Install),
            "del" => Ok(PackageOperation::Remove),
            "update" => Ok(PackageOperation::Update),
            "upgrade" => Ok(PackageOperation::Upgrade),
            "search" => Ok(PackageOperation::Search),
            "info" => Ok(PackageOperation::Info),
            "list" => Ok(PackageOperation::List),
            "cache" => Ok(PackageOperation::Clean),
            _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
        },
        PackageManager::Emerge => match op_str.as_str() {
            "" | "-av" | "-a" => Ok(PackageOperation::Install), // emerge pkg or emerge -av pkg
            "--unmerge" | "-C" => Ok(PackageOperation::Remove),
            "--sync" => Ok(PackageOperation::Update),
            "--update" | "-u" => Ok(PackageOperation::Upgrade),
            "--search" | "-s" => Ok(PackageOperation::Search),
            "--info" => Ok(PackageOperation::Info),
            "--depclean" => Ok(PackageOperation::Clean),
            _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
        },
        PackageManager::Xbps => {
            // XBPS commands can vary (xbps-install, xbps-remove, xbps-query)
            match op_str.as_str() {
                "-su" => Ok(PackageOperation::Upgrade),
                "-s" => Ok(PackageOperation::Install),
                "-rs" => Ok(PackageOperation::Search),
                "-r" => Ok(PackageOperation::Info),
                "-l" => Ok(PackageOperation::List),
                "-o" => Ok(PackageOperation::Clean),
                "" => Ok(PackageOperation::Remove), // xbps-remove pkg
                _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
            }
        },
        PackageManager::Nix => match op_str.as_str() {
            "-i" | "-iA" => Ok(PackageOperation::Install),
            "-e" => Ok(PackageOperation::Remove),
            "-u" | "-uA" => Ok(PackageOperation::Upgrade),
            "-qa" => Ok(PackageOperation::List),
            _ if op_str.starts_with("search") => Ok(PackageOperation::Search),
            _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
        },
        PackageManager::Generic => Err(PackageTranslationError::UnsupportedOperation(op_str)),
    }
}

/// Translate a list of CLI flags from one package manager to another for a given operation.
///
/// This function looks up per-operation flag mappings and produces a vector of translated
/// flag tokens appropriate for the target package manager. Flags that have no mapping
/// are preserved in the returned list and a warning describing the missing mapping is
/// appended to `result.warnings`. Flags of the form `--option=value` are translated
/// while preserving their value when the mapping indicates an equivalent `--option=`
/// form; when the mapping separates option and value the value is appended as a separate
/// token. Mappings that expand to multiple target tokens are split on whitespace and
/// all non-empty parts are included.
///
/// # Parameters
///
/// - `flags`: list of input flag tokens (e.g., `-y`, `--no-confirm`, `--opt=val`).
/// - `from_pm`: source package manager of the input flags.
/// - `to_pm`: target package manager to translate flags for.
/// - `operation`: package management operation these flags apply to (install, remove, etc.).
/// - `result`: mutable translation result that will receive warnings about unmapped flags.
///
/// # Returns
///
/// A vector of translated flag tokens (and preserved original tokens for unmapped flags).
///
/// # Examples
///
/// ```no_run
/// use translator::package_manager::{translate_flags, PackageTranslationResult, PackageOperation};
/// use super::distro::PackageManager;
///
/// let flags = vec!["-y".to_string(), "--opt=value".to_string()];
/// let mut result = PackageTranslationResult::new("".into(), "".into(), PackageManager::Apt, PackageManager::Dnf);
/// let translated = translate_flags(&flags, PackageManager::Apt, PackageManager::Dnf, PackageOperation::Install, &mut result);
/// // `translated` now contains target-appropriate tokens or original flags if unmapped,
/// // and `result.warnings` contains any mapping warnings.
/// ```
fn translate_flags(
    flags: &[String],
    from_pm: PackageManager,
    to_pm: PackageManager,
    operation: PackageOperation,
    result: &mut PackageTranslationResult,
) -> Vec<String> {
    let mut translated_flags = Vec::new();
    
    // Get flag mappings for this operation
    let flag_key = FlagMappingKey { from_pm, to_pm, op: operation };
    let flag_mappings = FLAG_MAPPINGS.get(&flag_key);
    
    for flag in flags {
        let mut found = false;
        
        if let Some(mappings) = flag_mappings {
            // Check if this flag has a translation
            for mapping in mappings {
                // Handle exact match
                if flag == &mapping.source || flag.to_lowercase() == mapping.source.to_lowercase() {
                    if !mapping.target.is_empty() {
                        // Handle cases where target contains multiple flags
                        for part in mapping.target.split_whitespace() {
                            if !part.is_empty() {
                                translated_flags.push(part.to_string());
                            }
                        }
                    }
                    found = true;
                    break;
                }
                
                // Handle flags with values (e.g., --option=value)
                if flag.starts_with(&mapping.source) && flag.contains('=') {
                    let parts: Vec<&str> = flag.splitn(2, '=').collect();
                    if parts[0] == mapping.source {
                        if !mapping.target.is_empty() {
                            if mapping.target.contains('=') {
                                translated_flags.push(format!("{}={}", mapping.target, parts[1]));
                            } else {
                                translated_flags.push(mapping.target.clone());
                                translated_flags.push(parts[1].to_string());
                            }
                        }
                        found = true;
                        break;
                    }
                }
            }
        }
        
        // If flag wasn't found in mappings, add a warning but keep it
        if !found {
            result.warnings.push(format!(
                "Flag '{}' has no direct equivalent in {} for {} operation",
                flag, to_pm, operation
            ));
            // Keep the original flag - it might still work or be ignored
            translated_flags.push(flag.clone());
        }
    }
    
    translated_flags
}

/// Translate a package manager command string from a specified source package manager to a target package manager.
///
/// Parses the input command, detects the operation and arguments, maps the operation and flags to their equivalents
/// for the target package manager, and returns a `PackageTranslationResult` containing the translated command,
/// any warnings about unmapped flags or mismatches, and whether elevated privileges may be required.
///
/// Returns an `Err(PackageTranslationError)` when translation cannot proceed, for example:
/// - `PackageTranslationError::EmptyCommand` if the trimmed input is empty.
/// - `PackageTranslationError::NotPackageManagerCommand(_)` if the command does not start with a recognized package manager.
/// - `PackageTranslationError::UnsupportedOperation(_)` if the detected operation has no mapping for the target package manager.
///
/// # Examples
///
/// ```
/// use crate::translator::package_manager::{translate_package_command, PackageManager};
///
/// let res = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Dnf).unwrap();
/// assert_eq!(res.command, "dnf install vim");
/// ```
pub fn translate_package_command(
    input: &str,
    from_pm: PackageManager,
    to_pm: PackageManager,
) -> Result<PackageTranslationResult, PackageTranslationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PackageTranslationError::EmptyCommand);
    }

    if from_pm == to_pm {
        return Ok(PackageTranslationResult::new(
            trimmed.to_string(),
            trimmed.to_string(),
            from_pm,
            to_pm,
        ));
    }

    // Parse the command
    let (detected_pm, operation, args) = parse_package_command(trimmed)?;

    // Get the target operation mapping
    let key = OperationKey { pm: to_pm, op: operation };
    let mapping = OPERATION_MAPPINGS.get(&key)
        .ok_or_else(|| PackageTranslationError::UnsupportedOperation(operation.to_string()))?;

    // Build the translated command
    let mut result = PackageTranslationResult::new(
        String::new(),
        trimmed.to_string(),
        from_pm,
        to_pm,
    );

    // Verify detected PM matches expected from_pm
    if detected_pm != from_pm {
        result.warnings.push(format!(
            "Command appears to be for {} but was specified as {}",
            detected_pm, from_pm
        ));
    }

    result.requires_sudo = mapping.requires_sudo;

    // Handle sudo prefix
    let mut command = String::new();
    if trimmed.starts_with("sudo ") && mapping.requires_sudo {
        command.push_str("sudo ");
    }

    command.push_str(&mapping.command);

    // Separate flags from package names
    let (flags, packages): (Vec<String>, Vec<String>) = args.into_iter()
        .partition(|arg| arg.starts_with('-') || arg.starts_with('/'));
    
    // Translate flags
    let translated_flags = translate_flags(&flags, from_pm, to_pm, operation, &mut result);
    
    // Add translated flags
    if !translated_flags.is_empty() {
        command.push(' ');
        command.push_str(&translated_flags.join(" "));
    }
    
    // Add package names
    if !packages.is_empty() {
        command.push(' ');
        command.push_str(&packages.join(" "));
    }

    result.command = command;

    // Add notes if any
    if let Some(notes) = &mapping.notes {
        result.warnings.push(notes.clone());
    }

    Ok(result)
}

/// Automatically detect the source package manager and translate the given command into the target package manager.
///
/// The function trims the input, detects the source package manager and operation, and then produces a translated
/// command targeting `to_pm`. Errors returned by detection or translation are propagated.
///
/// # Returns
///
/// `Ok(PackageTranslationResult)` with the translated command on success, or `Err(PackageTranslationError)` if the
/// input is empty or the command cannot be parsed or translated.
///
/// # Examples
///
/// ```
/// let res = translate_package_command_auto("sudo apt install curl", PackageManager::Dnf).unwrap();
/// assert!(res.command.contains("dnf"));
/// assert!(res.command.contains("install") || res.command.contains("install"));
/// ```
pub fn translate_package_command_auto(
    input: &str,
    to_pm: PackageManager,
) -> Result<PackageTranslationResult, PackageTranslationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PackageTranslationError::EmptyCommand);
    }

    // Parse to detect source package manager
    let (from_pm, _, _) = parse_package_command(trimmed)?;

    translate_package_command(trimmed, from_pm, to_pm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_to_dnf_install() {
        let result = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("dnf install"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_dnf_to_apt_remove() {
        let result = translate_package_command("dnf remove vim", PackageManager::Dnf, PackageManager::Apt);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("apt remove"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_apt_to_pacman_install() {
        let result = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Pacman);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("pacman -S"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_pacman_to_apt_search() {
        let result = translate_package_command("pacman -Ss vim", PackageManager::Pacman, PackageManager::Apt);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("apt search"));
        assert!(r.command.contains("vim"));
    }

    /// Verifies that a translation preserves a leading `sudo` prefix when the target operation requires elevated privileges.
    ///
    /// # Examples
    ///
    /// ```
    /// let result = translate_package_command("sudo apt install vim", PackageManager::Apt, PackageManager::Dnf).unwrap();
    /// assert!(result.command.starts_with("sudo"));
    /// assert!(result.requires_sudo);
    /// ```
    #[test]
    fn test_sudo_prefix_preserved() {
        let result = translate_package_command("sudo apt install vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.starts_with("sudo"));
        assert!(r.requires_sudo);
    }

    #[test]
    fn test_auto_detection() {
        let result = translate_package_command_auto("apt install vim", PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("dnf install"));
    }

    #[test]
    fn test_same_package_manager() {
        let result = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Apt);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.command, "apt install vim");
    }

    #[test]
    fn test_apt_to_zypper() {
        let result = translate_package_command("apt update", PackageManager::Apt, PackageManager::Zypper);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("zypper refresh"));
    }

    #[test]
    fn test_yum_to_apk() {
        let result = translate_package_command("yum install nginx", PackageManager::Yum, PackageManager::Apk);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("apk add"));
        assert!(r.command.contains("nginx"));
    }

    // ============================================================
    // Flag translation tests
    // ============================================================

    #[test]
    fn test_apt_to_dnf_with_yes_flag() {
        let result = translate_package_command("apt install -y vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("dnf install"));
        assert!(r.command.contains("-y"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_apt_to_pacman_with_yes_flag() {
        let result = translate_package_command("apt install -y vim", PackageManager::Apt, PackageManager::Pacman);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("pacman -S"));
        assert!(r.command.contains("--noconfirm"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_apt_to_dnf_with_quiet_flag() {
        let result = translate_package_command("apt install -q vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("dnf install"));
        assert!(r.command.contains("-q"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_pacman_to_apt_with_noconfirm_flag() {
        let result = translate_package_command("pacman -S --noconfirm vim", PackageManager::Pacman, PackageManager::Apt);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("apt install"));
        assert!(r.command.contains("-y"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_dnf_to_apt_with_assumeyes_flag() {
        let result = translate_package_command("dnf install -y vim", PackageManager::Dnf, PackageManager::Apt);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("apt install"));
        assert!(r.command.contains("-y"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_apt_remove_with_purge_flag_to_pacman() {
        let result = translate_package_command("apt remove --purge vim", PackageManager::Apt, PackageManager::Pacman);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("pacman -R"));
        assert!(r.command.contains("-n")); // --purge translates to -n in pacman
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_apt_to_zypper_with_yes_flag() {
        let result = translate_package_command("apt install -y vim", PackageManager::Apt, PackageManager::Zypper);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("zypper install"));
        assert!(r.command.contains("-y"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_multiple_flags_translation() {
        let result = translate_package_command("apt install -y -q vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.contains("dnf install"));
        assert!(r.command.contains("-y"));
        assert!(r.command.contains("-q"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_sudo_with_flags() {
        let result = translate_package_command("sudo apt install -y vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.command.starts_with("sudo"));
        assert!(r.command.contains("dnf install"));
        assert!(r.command.contains("-y"));
        assert!(r.command.contains("vim"));
    }

    #[test]
    fn test_unmapped_flag_warning() {
        let result = translate_package_command("apt install --some-unknown-flag vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_ok());
        let r = result.unwrap();
        // Should still translate the command and package, but warn about the flag
        assert!(r.command.contains("dnf install"));
        assert!(r.command.contains("vim"));
        assert!(!r.warnings.is_empty()); // Should have a warning about the unmapped flag
    }

    // ============================================================
    // Error case tests
    // ============================================================

    #[test]
    fn test_empty_command_error() {
        let result = translate_package_command("", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_err());
        match result {
            Err(PackageTranslationError::EmptyCommand) => {},
            _ => panic!("Expected EmptyCommand error"),
        }
    }

    #[test]
    fn test_empty_command_auto_error() {
        let result = translate_package_command_auto("", PackageManager::Dnf);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_package_manager_command() {
        let result = translate_package_command("invalid-command install vim", PackageManager::Apt, PackageManager::Dnf);
        assert!(result.is_err());
        match result {
            Err(PackageTranslationError::NotPackageManagerCommand(_)) => {},
            _ => panic!("Expected NotPackageManagerCommand error"),
        }
    }

    #[test]
    fn test_unsupported_operation() {
        // Pacman with invalid flag
        let result = translate_package_command("pacman -X vim", PackageManager::Pacman, PackageManager::Apt);
        assert!(result.is_err());
    }

    // ============================================================
    // All operations coverage tests
    // ============================================================

    #[test]
    fn test_all_operations_apt_to_dnf() {
        // Install
        let r = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf install"));
        
        // Remove
        let r = translate_package_command("apt remove vim", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf remove"));
        
        // Update
        let r = translate_package_command("apt update", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf check-update"));
        
        // Upgrade
        let r = translate_package_command("apt upgrade", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf upgrade"));
        
        // Search
        let r = translate_package_command("apt search vim", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf search"));
        
        // Info
        let r = translate_package_command("apt show vim", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf info"));
        
        // Clean
        let r = translate_package_command("apt clean", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf clean"));
        
        // Autoremove
        let r = translate_package_command("apt autoremove", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf autoremove"));
    }

    // ============================================================
    // Package manager specific tests
    // ============================================================

    #[test]
    fn test_apk_operations() {
        // APK to Apt
        let r = translate_package_command("apk add nginx", PackageManager::Apk, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt install"));
        assert!(r.command.contains("nginx"));
        
        let r = translate_package_command("apk del nginx", PackageManager::Apk, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt remove"));
        
        let r = translate_package_command("apk update", PackageManager::Apk, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt update"));
        
        let r = translate_package_command("apk search nginx", PackageManager::Apk, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt search"));
    }

    #[test]
    fn test_emerge_operations() {
        // Emerge to Apt - emerge uses flags for operations
        let r = translate_package_command("emerge -a vim", PackageManager::Emerge, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt install"));
        
        let r = translate_package_command("emerge --unmerge vim", PackageManager::Emerge, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt remove"));
        
        let r = translate_package_command("emerge --sync", PackageManager::Emerge, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt update"));
        
        let r = translate_package_command("emerge --search vim", PackageManager::Emerge, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt search"));
    }

    #[test]
    fn test_xbps_operations() {
        // XBPS to Apt
        let r = translate_package_command("xbps-install -s vim", PackageManager::Xbps, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt install"));
        
        let r = translate_package_command("xbps-install -su", PackageManager::Xbps, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt upgrade"));
        
        let r = translate_package_command("xbps-query -rs vim", PackageManager::Xbps, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt search"));
    }

    #[test]
    fn test_nix_operations() {
        // Nix to Apt
        let r = translate_package_command("nix-env -i vim", PackageManager::Nix, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt install"));
        
        let r = translate_package_command("nix-env -e vim", PackageManager::Nix, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt remove"));
        
        let r = translate_package_command("nix-env -u", PackageManager::Nix, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt upgrade"));
    }

    #[test]
    fn test_zypper_operations() {
        // Zypper to Apt - test all operations
        let r = translate_package_command("zypper install vim", PackageManager::Zypper, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt install"));
        
        let r = translate_package_command("zypper remove vim", PackageManager::Zypper, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt remove"));
        
        let r = translate_package_command("zypper refresh", PackageManager::Zypper, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt update"));
        
        let r = translate_package_command("zypper update", PackageManager::Zypper, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt upgrade"));
        
        let r = translate_package_command("zypper search vim", PackageManager::Zypper, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt search"));
        
        let r = translate_package_command("zypper info vim", PackageManager::Zypper, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt show"));
    }

    #[test]
    fn test_pacman_all_operations() {
        // Test all Pacman operations
        let r = translate_package_command("pacman -S vim", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt install"));
        
        let r = translate_package_command("pacman -R vim", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt remove"));
        
        let r = translate_package_command("pacman -Sy", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt update"));
        
        let r = translate_package_command("pacman -Syu", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt upgrade"));
        
        let r = translate_package_command("pacman -Ss vim", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt search"));
        
        let r = translate_package_command("pacman -Si vim", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt show"));
        
        let r = translate_package_command("pacman -Q", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt list"));
        
        let r = translate_package_command("pacman -Sc", PackageManager::Pacman, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt clean"));
    }

    #[test]
    fn test_yum_all_operations() {
        // Test all YUM operations
        let r = translate_package_command("yum install vim", PackageManager::Yum, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt install"));
        
        let r = translate_package_command("yum remove vim", PackageManager::Yum, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt remove"));
        
        let r = translate_package_command("yum check-update", PackageManager::Yum, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt update"));
        
        let r = translate_package_command("yum update", PackageManager::Yum, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt upgrade"));
        
        let r = translate_package_command("yum search vim", PackageManager::Yum, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt search"));
        
        let r = translate_package_command("yum info vim", PackageManager::Yum, PackageManager::Apt).unwrap();
        assert!(r.command.contains("apt show"));
    }

    // ============================================================
    // Multiple packages tests
    // ============================================================

    #[test]
    fn test_multiple_packages() {
        let r = translate_package_command("apt install vim nginx git", PackageManager::Apt, PackageManager::Dnf).unwrap();
        assert!(r.command.contains("dnf install"));
        assert!(r.command.contains("vim"));
        assert!(r.command.contains("nginx"));
        assert!(r.command.contains("git"));
    }

    #[test]
    fn test_multiple_packages_with_flags() {
        let r = translate_package_command("apt install -y -q vim nginx", PackageManager::Apt, PackageManager::Pacman).unwrap();
        assert!(r.command.contains("pacman -S"));
        assert!(r.command.contains("--noconfirm"));
        assert!(r.command.contains("-q"));
        assert!(r.command.contains("vim"));
        assert!(r.command.contains("nginx"));
    }

    // ============================================================
    // Display and serialization tests
    // ============================================================

    #[test]
    fn test_translation_result_display() {
        let r = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Dnf).unwrap();
        let display = format!("{}", r);
        assert!(display.contains("dnf install"));
    }

    #[test]
    fn test_translation_error_display() {
        let err = PackageTranslationError::EmptyCommand;
        let display = format!("{}", err);
        assert!(display.contains("Empty command"));
        
        let err = PackageTranslationError::NotPackageManagerCommand("test".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Not a recognized"));
        
        let err = PackageTranslationError::UnsupportedOperation("unknown".to_string());
        let display = format!("{}", err);
        assert!(display.contains("not supported"));
    }

    #[test]
    fn test_package_operation_display() {
        assert_eq!(format!("{}", PackageOperation::Install), "install");
        assert_eq!(format!("{}", PackageOperation::Remove), "remove");
        assert_eq!(format!("{}", PackageOperation::Update), "update");
        assert_eq!(format!("{}", PackageOperation::Upgrade), "upgrade");
        assert_eq!(format!("{}", PackageOperation::Search), "search");
        assert_eq!(format!("{}", PackageOperation::Info), "info");
        assert_eq!(format!("{}", PackageOperation::List), "list");
        assert_eq!(format!("{}", PackageOperation::Clean), "clean");
        assert_eq!(format!("{}", PackageOperation::AutoRemove), "autoremove");
    }

    // ============================================================
    // API and structure tests
    // ============================================================

    #[test]
    fn test_package_flag_mapping_new() {
        let mapping = PackageFlagMapping::new("-y", "--yes");
        assert_eq!(mapping.source, "-y");
        assert_eq!(mapping.target, "--yes");
        assert!(mapping.description.is_none());
    }

    #[test]
    fn test_package_flag_mapping_with_description() {
        let mapping = PackageFlagMapping::with_description("-y", "--yes", "Assume yes");
        assert_eq!(mapping.source, "-y");
        assert_eq!(mapping.target, "--yes");
        assert_eq!(mapping.description, Some("Assume yes".to_string()));
    }

    #[test]
    fn test_translation_result_new() {
        let result = PackageTranslationResult::new(
            "dnf install vim".to_string(),
            "apt install vim".to_string(),
            PackageManager::Apt,
            PackageManager::Dnf,
        );
        assert_eq!(result.command, "dnf install vim");
        assert_eq!(result.original, "apt install vim");
        assert_eq!(result.from_pm, PackageManager::Apt);
        assert_eq!(result.to_pm, PackageManager::Dnf);
        assert!(result.warnings.is_empty());
        assert!(!result.requires_sudo);
    }

    #[test]
    fn test_translation_result_with_warnings() {
        let mut result = PackageTranslationResult::new(
            "dnf install vim".to_string(),
            "apt install vim".to_string(),
            PackageManager::Apt,
            PackageManager::Dnf,
        );
        result.warnings.push("Test warning".to_string());
        result.requires_sudo = true;
        
        assert!(!result.warnings.is_empty());
        assert!(result.requires_sudo);
        assert_eq!(result.warnings[0], "Test warning");
    }

    #[test]
    fn test_package_translation_error_is_error() {
        let err = PackageTranslationError::EmptyCommand;
        let _: &dyn std::error::Error = &err;  // Verify it implements Error trait
    }
}