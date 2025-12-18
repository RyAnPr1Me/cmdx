//! Linux distribution detection and package manager identification

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported Linux distributions and package managers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Distro {
    /// Debian-based (apt/apt-get/dpkg)
    Debian,
    /// Ubuntu-based (apt/apt-get/dpkg)
    Ubuntu,
    /// Red Hat Enterprise Linux (yum/rpm)
    RHEL,
    /// CentOS (yum/rpm)
    CentOS,
    /// Fedora (dnf/rpm)
    Fedora,
    /// Arch Linux (pacman)
    Arch,
    /// Manjaro (pacman)
    Manjaro,
    /// openSUSE (zypper/rpm)
    OpenSUSE,
    /// Alpine Linux (apk)
    Alpine,
    /// Gentoo (emerge/portage)
    Gentoo,
    /// Void Linux (xbps)
    Void,
    /// NixOS (nix)
    NixOS,
    /// Generic Linux with unknown package manager
    Generic,
}

impl fmt::Display for Distro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Distro::Debian => write!(f, "Debian"),
            Distro::Ubuntu => write!(f, "Ubuntu"),
            Distro::RHEL => write!(f, "RHEL"),
            Distro::CentOS => write!(f, "CentOS"),
            Distro::Fedora => write!(f, "Fedora"),
            Distro::Arch => write!(f, "Arch"),
            Distro::Manjaro => write!(f, "Manjaro"),
            Distro::OpenSUSE => write!(f, "openSUSE"),
            Distro::Alpine => write!(f, "Alpine"),
            Distro::Gentoo => write!(f, "Gentoo"),
            Distro::Void => write!(f, "Void"),
            Distro::NixOS => write!(f, "NixOS"),
            Distro::Generic => write!(f, "Generic Linux"),
        }
    }
}

/// Error returned when parsing an invalid distro string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDistroError(String);

impl fmt::Display for ParseDistroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown distribution: '{}'", self.0)
    }
}

impl std::error::Error for ParseDistroError {}

impl FromStr for Distro {
    type Err = ParseDistroError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debian" => Ok(Distro::Debian),
            "ubuntu" => Ok(Distro::Ubuntu),
            "rhel" | "redhat" | "red hat" => Ok(Distro::RHEL),
            "centos" => Ok(Distro::CentOS),
            "fedora" => Ok(Distro::Fedora),
            "arch" | "archlinux" => Ok(Distro::Arch),
            "manjaro" => Ok(Distro::Manjaro),
            "opensuse" | "suse" => Ok(Distro::OpenSUSE),
            "alpine" => Ok(Distro::Alpine),
            "gentoo" => Ok(Distro::Gentoo),
            "void" => Ok(Distro::Void),
            "nixos" | "nix" => Ok(Distro::NixOS),
            "generic" | "linux" => Ok(Distro::Generic),
            _ => Err(ParseDistroError(s.to_string())),
        }
    }
}

impl Distro {
    /// Parse distro from string (case-insensitive)
    pub fn parse(s: &str) -> Option<Distro> {
        s.parse().ok()
    }

    /// Get the primary package manager for this distro
    pub fn package_manager(&self) -> PackageManager {
        match self {
            Distro::Debian | Distro::Ubuntu => PackageManager::Apt,
            Distro::RHEL | Distro::CentOS => PackageManager::Yum,
            Distro::Fedora => PackageManager::Dnf,
            Distro::Arch | Distro::Manjaro => PackageManager::Pacman,
            Distro::OpenSUSE => PackageManager::Zypper,
            Distro::Alpine => PackageManager::Apk,
            Distro::Gentoo => PackageManager::Emerge,
            Distro::Void => PackageManager::Xbps,
            Distro::NixOS => PackageManager::Nix,
            Distro::Generic => PackageManager::Generic,
        }
    }
}

/// Package manager types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManager {
    /// APT (Debian/Ubuntu)
    Apt,
    /// YUM (RHEL/CentOS)
    Yum,
    /// DNF (Fedora)
    Dnf,
    /// Pacman (Arch)
    Pacman,
    /// Zypper (openSUSE)
    Zypper,
    /// APK (Alpine)
    Apk,
    /// Emerge/Portage (Gentoo)
    Emerge,
    /// XBPS (Void)
    Xbps,
    /// Nix (NixOS)
    Nix,
    /// Generic/Unknown
    Generic,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Apt => write!(f, "apt"),
            PackageManager::Yum => write!(f, "yum"),
            PackageManager::Dnf => write!(f, "dnf"),
            PackageManager::Pacman => write!(f, "pacman"),
            PackageManager::Zypper => write!(f, "zypper"),
            PackageManager::Apk => write!(f, "apk"),
            PackageManager::Emerge => write!(f, "emerge"),
            PackageManager::Xbps => write!(f, "xbps"),
            PackageManager::Nix => write!(f, "nix"),
            PackageManager::Generic => write!(f, "generic"),
        }
    }
}

impl FromStr for PackageManager {
    type Err = ParseDistroError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "apt" | "apt-get" | "aptitude" => Ok(PackageManager::Apt),
            "yum" => Ok(PackageManager::Yum),
            "dnf" => Ok(PackageManager::Dnf),
            "pacman" => Ok(PackageManager::Pacman),
            "zypper" => Ok(PackageManager::Zypper),
            "apk" => Ok(PackageManager::Apk),
            "emerge" | "portage" => Ok(PackageManager::Emerge),
            "xbps" | "xbps-install" | "xbps-remove" => Ok(PackageManager::Xbps),
            "nix" | "nix-env" => Ok(PackageManager::Nix),
            "generic" => Ok(PackageManager::Generic),
            _ => Err(ParseDistroError(s.to_string())),
        }
    }
}

impl PackageManager {
    /// Parse package manager from string
    pub fn parse(s: &str) -> Option<PackageManager> {
        s.parse().ok()
    }

    /// Get the command name for this package manager
    pub fn command_name(&self) -> &'static str {
        match self {
            PackageManager::Apt => "apt",
            PackageManager::Yum => "yum",
            PackageManager::Dnf => "dnf",
            PackageManager::Pacman => "pacman",
            PackageManager::Zypper => "zypper",
            PackageManager::Apk => "apk",
            PackageManager::Emerge => "emerge",
            PackageManager::Xbps => "xbps-install",
            PackageManager::Nix => "nix-env",
            PackageManager::Generic => "package-manager",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_parse() {
        assert_eq!(Distro::parse("ubuntu"), Some(Distro::Ubuntu));
        assert_eq!(Distro::parse("debian"), Some(Distro::Debian));
        assert_eq!(Distro::parse("fedora"), Some(Distro::Fedora));
        assert_eq!(Distro::parse("arch"), Some(Distro::Arch));
    }

    #[test]
    fn test_distro_package_manager() {
        assert_eq!(Distro::Ubuntu.package_manager(), PackageManager::Apt);
        assert_eq!(Distro::Fedora.package_manager(), PackageManager::Dnf);
        assert_eq!(Distro::Arch.package_manager(), PackageManager::Pacman);
    }

    #[test]
    fn test_package_manager_parse() {
        assert_eq!(PackageManager::parse("apt"), Some(PackageManager::Apt));
        assert_eq!(PackageManager::parse("yum"), Some(PackageManager::Yum));
        assert_eq!(PackageManager::parse("pacman"), Some(PackageManager::Pacman));
    }

    #[test]
    fn test_package_manager_command_name() {
        assert_eq!(PackageManager::Apt.command_name(), "apt");
        assert_eq!(PackageManager::Dnf.command_name(), "dnf");
        assert_eq!(PackageManager::Pacman.command_name(), "pacman");
        assert_eq!(PackageManager::Yum.command_name(), "yum");
        assert_eq!(PackageManager::Zypper.command_name(), "zypper");
        assert_eq!(PackageManager::Apk.command_name(), "apk");
        assert_eq!(PackageManager::Emerge.command_name(), "emerge");
        assert_eq!(PackageManager::Xbps.command_name(), "xbps-install");
        assert_eq!(PackageManager::Nix.command_name(), "nix-env");
    }

    #[test]
    fn test_distro_display() {
        assert_eq!(format!("{}", Distro::Debian), "Debian");
        assert_eq!(format!("{}", Distro::Ubuntu), "Ubuntu");
        assert_eq!(format!("{}", Distro::Fedora), "Fedora");
        assert_eq!(format!("{}", Distro::Arch), "Arch");
        assert_eq!(format!("{}", Distro::Alpine), "Alpine");
    }

    #[test]
    fn test_package_manager_display() {
        assert_eq!(format!("{}", PackageManager::Apt), "apt");
        assert_eq!(format!("{}", PackageManager::Dnf), "dnf");
        assert_eq!(format!("{}", PackageManager::Pacman), "pacman");
        assert_eq!(format!("{}", PackageManager::Zypper), "zypper");
    }

    #[test]
    fn test_all_distros_parse() {
        assert_eq!(Distro::parse("debian"), Some(Distro::Debian));
        assert_eq!(Distro::parse("ubuntu"), Some(Distro::Ubuntu));
        assert_eq!(Distro::parse("rhel"), Some(Distro::RHEL));
        assert_eq!(Distro::parse("centos"), Some(Distro::CentOS));
        assert_eq!(Distro::parse("fedora"), Some(Distro::Fedora));
        assert_eq!(Distro::parse("arch"), Some(Distro::Arch));
        assert_eq!(Distro::parse("manjaro"), Some(Distro::Manjaro));
        assert_eq!(Distro::parse("opensuse"), Some(Distro::OpenSUSE));
        assert_eq!(Distro::parse("alpine"), Some(Distro::Alpine));
        assert_eq!(Distro::parse("gentoo"), Some(Distro::Gentoo));
        assert_eq!(Distro::parse("void"), Some(Distro::Void));
        assert_eq!(Distro::parse("nixos"), Some(Distro::NixOS));
        assert_eq!(Distro::parse("invalid"), None);
    }

    #[test]
    fn test_all_package_managers_parse() {
        assert_eq!(PackageManager::parse("apt"), Some(PackageManager::Apt));
        assert_eq!(PackageManager::parse("apt-get"), Some(PackageManager::Apt));
        assert_eq!(PackageManager::parse("yum"), Some(PackageManager::Yum));
        assert_eq!(PackageManager::parse("dnf"), Some(PackageManager::Dnf));
        assert_eq!(PackageManager::parse("pacman"), Some(PackageManager::Pacman));
        assert_eq!(PackageManager::parse("zypper"), Some(PackageManager::Zypper));
        assert_eq!(PackageManager::parse("apk"), Some(PackageManager::Apk));
        assert_eq!(PackageManager::parse("emerge"), Some(PackageManager::Emerge));
        assert_eq!(PackageManager::parse("xbps"), Some(PackageManager::Xbps));
        assert_eq!(PackageManager::parse("nix"), Some(PackageManager::Nix));
        assert_eq!(PackageManager::parse("invalid"), None);
    }

    #[test]
    fn test_parse_distro_error() {
        let result = "invalid-distro".parse::<Distro>();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(format!("{}", e).contains("Unknown distribution"));
        }
    }

    #[test]
    fn test_parse_package_manager_error() {
        let result = "invalid-pm".parse::<PackageManager>();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(format!("{}", e).contains("Unknown distribution"));
        }
    }

    #[test]
    fn test_all_distros_have_package_manager() {
        assert_eq!(Distro::Debian.package_manager(), PackageManager::Apt);
        assert_eq!(Distro::Ubuntu.package_manager(), PackageManager::Apt);
        assert_eq!(Distro::RHEL.package_manager(), PackageManager::Yum);
        assert_eq!(Distro::CentOS.package_manager(), PackageManager::Yum);
        assert_eq!(Distro::Fedora.package_manager(), PackageManager::Dnf);
        assert_eq!(Distro::Arch.package_manager(), PackageManager::Pacman);
        assert_eq!(Distro::Manjaro.package_manager(), PackageManager::Pacman);
        assert_eq!(Distro::OpenSUSE.package_manager(), PackageManager::Zypper);
        assert_eq!(Distro::Alpine.package_manager(), PackageManager::Apk);
        assert_eq!(Distro::Gentoo.package_manager(), PackageManager::Emerge);
        assert_eq!(Distro::Void.package_manager(), PackageManager::Xbps);
        assert_eq!(Distro::NixOS.package_manager(), PackageManager::Nix);
        assert_eq!(Distro::Generic.package_manager(), PackageManager::Generic);
    }
}
