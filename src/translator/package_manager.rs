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

/// Mapping for a package manager operation
#[derive(Debug, Clone)]
struct OperationMapping {
    /// The command pattern for this operation
    command: String,
    /// Whether this operation requires sudo
    requires_sudo: bool,
    /// Additional flags or notes
    notes: Option<String>,
}

/// Key for looking up operation mappings
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct OperationKey {
    pm: PackageManager,
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
            OperationMapping { command: "apt install".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Remove },
            OperationMapping { command: "apt remove".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Update },
            OperationMapping { command: "apt update".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Upgrade },
            OperationMapping { command: "apt upgrade".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Search },
            OperationMapping { command: "apt search".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Info },
            OperationMapping { command: "apt show".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::List },
            OperationMapping { command: "apt list --installed".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::Clean },
            OperationMapping { command: "apt clean".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apt, op: PackageOperation::AutoRemove },
            OperationMapping { command: "apt autoremove".to_string(), requires_sudo: true, notes: None },
        );

        // ============================================================
        // YUM (RHEL/CentOS) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Install },
            OperationMapping { command: "yum install".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Remove },
            OperationMapping { command: "yum remove".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Update },
            OperationMapping { command: "yum check-update".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Upgrade },
            OperationMapping { command: "yum update".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Search },
            OperationMapping { command: "yum search".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Info },
            OperationMapping { command: "yum info".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::List },
            OperationMapping { command: "yum list installed".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::Clean },
            OperationMapping { command: "yum clean all".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Yum, op: PackageOperation::AutoRemove },
            OperationMapping { command: "yum autoremove".to_string(), requires_sudo: true, notes: None },
        );

        // ============================================================
        // DNF (Fedora) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Install },
            OperationMapping { command: "dnf install".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Remove },
            OperationMapping { command: "dnf remove".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Update },
            OperationMapping { command: "dnf check-update".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Upgrade },
            OperationMapping { command: "dnf upgrade".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Search },
            OperationMapping { command: "dnf search".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Info },
            OperationMapping { command: "dnf info".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::List },
            OperationMapping { command: "dnf list installed".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::Clean },
            OperationMapping { command: "dnf clean all".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Dnf, op: PackageOperation::AutoRemove },
            OperationMapping { command: "dnf autoremove".to_string(), requires_sudo: true, notes: None },
        );

        // ============================================================
        // Pacman (Arch) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Install },
            OperationMapping { command: "pacman -S".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Remove },
            OperationMapping { command: "pacman -R".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Update },
            OperationMapping { command: "pacman -Sy".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Upgrade },
            OperationMapping { command: "pacman -Syu".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Search },
            OperationMapping { command: "pacman -Ss".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Info },
            OperationMapping { command: "pacman -Si".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::List },
            OperationMapping { command: "pacman -Q".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::Clean },
            OperationMapping { command: "pacman -Sc".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Pacman, op: PackageOperation::AutoRemove },
            OperationMapping { command: "pacman -Rs".to_string(), requires_sudo: true, notes: Some("Removes package with unused dependencies".to_string()) },
        );

        // ============================================================
        // Zypper (openSUSE) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Install },
            OperationMapping { command: "zypper install".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Remove },
            OperationMapping { command: "zypper remove".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Update },
            OperationMapping { command: "zypper refresh".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Upgrade },
            OperationMapping { command: "zypper update".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Search },
            OperationMapping { command: "zypper search".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Info },
            OperationMapping { command: "zypper info".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::List },
            OperationMapping { command: "zypper search --installed-only".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::Clean },
            OperationMapping { command: "zypper clean".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Zypper, op: PackageOperation::AutoRemove },
            OperationMapping { command: "zypper remove --clean-deps".to_string(), requires_sudo: true, notes: None },
        );

        // ============================================================
        // APK (Alpine) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Install },
            OperationMapping { command: "apk add".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Remove },
            OperationMapping { command: "apk del".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Update },
            OperationMapping { command: "apk update".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Upgrade },
            OperationMapping { command: "apk upgrade".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Search },
            OperationMapping { command: "apk search".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Info },
            OperationMapping { command: "apk info".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::List },
            OperationMapping { command: "apk list --installed".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::Clean },
            OperationMapping { command: "apk cache clean".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Apk, op: PackageOperation::AutoRemove },
            OperationMapping { command: "apk del".to_string(), requires_sudo: true, notes: Some("Use with package name and dependencies".to_string()) },
        );

        // ============================================================
        // Emerge (Gentoo) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Install },
            OperationMapping { command: "emerge".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Remove },
            OperationMapping { command: "emerge --unmerge".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Update },
            OperationMapping { command: "emerge --sync".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Upgrade },
            OperationMapping { command: "emerge --update --deep --with-bdeps=y @world".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Search },
            OperationMapping { command: "emerge --search".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Info },
            OperationMapping { command: "emerge --info".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::List },
            OperationMapping { command: "qlist -I".to_string(), requires_sudo: false, notes: Some("Requires portage-utils".to_string()) },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::Clean },
            OperationMapping { command: "emerge --depclean".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Emerge, op: PackageOperation::AutoRemove },
            OperationMapping { command: "emerge --depclean".to_string(), requires_sudo: true, notes: None },
        );

        // ============================================================
        // XBPS (Void) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Install },
            OperationMapping { command: "xbps-install".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Remove },
            OperationMapping { command: "xbps-remove".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Update },
            OperationMapping { command: "xbps-install -S".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Upgrade },
            OperationMapping { command: "xbps-install -Su".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Search },
            OperationMapping { command: "xbps-query -Rs".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Info },
            OperationMapping { command: "xbps-query -R".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::List },
            OperationMapping { command: "xbps-query -l".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::Clean },
            OperationMapping { command: "xbps-remove -O".to_string(), requires_sudo: true, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Xbps, op: PackageOperation::AutoRemove },
            OperationMapping { command: "xbps-remove -o".to_string(), requires_sudo: true, notes: None },
        );

        // ============================================================
        // Nix (NixOS) mappings
        // ============================================================
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Install },
            OperationMapping { command: "nix-env -i".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Remove },
            OperationMapping { command: "nix-env -e".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Update },
            OperationMapping { command: "nix-channel --update".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Upgrade },
            OperationMapping { command: "nix-env -u".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Search },
            OperationMapping { command: "nix search".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Info },
            OperationMapping { command: "nix-env -qa --description".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::List },
            OperationMapping { command: "nix-env -q".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::Clean },
            OperationMapping { command: "nix-collect-garbage".to_string(), requires_sudo: false, notes: None },
        );
        m.insert(
            OperationKey { pm: PackageManager::Nix, op: PackageOperation::AutoRemove },
            OperationMapping { command: "nix-collect-garbage -d".to_string(), requires_sudo: false, notes: None },
        );

        m
    };
}

/// Parse a package manager command to determine the operation and arguments
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

/// Detect which package manager is being used
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

/// Detect which operation is being performed
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
            if op_str.starts_with('-') {
                match op_str.as_str() {
                    "-s" => Ok(PackageOperation::Install),
                    "-r" | "-rs" => Ok(PackageOperation::Remove),
                    "-sy" => Ok(PackageOperation::Update),
                    "-syu" => Ok(PackageOperation::Upgrade),
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
        _ => Err(PackageTranslationError::UnsupportedOperation(op_str)),
    }
}

/// Translate a package manager command from one package manager to another
///
/// # Arguments
///
/// * `input` - The package manager command string to translate
/// * `from_pm` - The source package manager
/// * `to_pm` - The target package manager
///
/// # Returns
///
/// * `Ok(PackageTranslationResult)` - The translated command
/// * `Err(PackageTranslationError)` - Error if translation fails
///
/// # Example
///
/// ```
/// use cmdx::translate_package_command;
/// use cmdx::PackageManager;
///
/// let result = translate_package_command("apt install vim", PackageManager::Apt, PackageManager::Dnf);
/// // Result: "dnf install vim"
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
    let (detected_pm, operation, mut args) = parse_package_command(trimmed)?;

    // Verify detected PM matches expected from_pm
    if detected_pm != from_pm {
        let mut result = PackageTranslationResult::new(
            String::new(),
            trimmed.to_string(),
            from_pm,
            to_pm,
        );
        result.warnings.push(format!(
            "Command appears to be for {} but was specified as {}",
            detected_pm, from_pm
        ));
    }

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

    result.requires_sudo = mapping.requires_sudo;

    // Handle sudo prefix
    let mut command = String::new();
    if trimmed.starts_with("sudo ") && mapping.requires_sudo {
        command.push_str("sudo ");
    }

    command.push_str(&mapping.command);

    // Add arguments (package names, etc.)
    // Filter out flags that are PM-specific
    args.retain(|arg| !arg.starts_with('-') && !arg.starts_with('/'));
    
    if !args.is_empty() {
        command.push(' ');
        command.push_str(&args.join(" "));
    }

    result.command = command;

    // Add notes if any
    if let Some(notes) = &mapping.notes {
        result.warnings.push(notes.clone());
    }

    Ok(result)
}

/// Translate a package manager command with automatic detection
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
}
