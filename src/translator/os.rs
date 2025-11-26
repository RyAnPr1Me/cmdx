//! Operating System detection and enumeration module

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// Supported operating systems for command translation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Os {
    Windows,
    Linux,
    MacOS,
    FreeBSD,
    OpenBSD,
    NetBSD,
    Solaris,
    Android,
    Ios,
    Unknown,
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Os::Windows => write!(f, "Windows"),
            Os::Linux => write!(f, "Linux"),
            Os::MacOS => write!(f, "macOS"),
            Os::FreeBSD => write!(f, "FreeBSD"),
            Os::OpenBSD => write!(f, "OpenBSD"),
            Os::NetBSD => write!(f, "NetBSD"),
            Os::Solaris => write!(f, "Solaris"),
            Os::Android => write!(f, "Android"),
            Os::Ios => write!(f, "iOS"),
            Os::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Error returned when parsing an invalid OS string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOsError(String);

impl fmt::Display for ParseOsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown operating system: '{}'", self.0)
    }
}

impl std::error::Error for ParseOsError {}

impl FromStr for Os {
    type Err = ParseOsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "windows" | "win" | "win32" | "win64" => Ok(Os::Windows),
            "linux" | "gnu/linux" => Ok(Os::Linux),
            "macos" | "darwin" | "osx" | "mac" => Ok(Os::MacOS),
            "freebsd" => Ok(Os::FreeBSD),
            "openbsd" => Ok(Os::OpenBSD),
            "netbsd" => Ok(Os::NetBSD),
            "solaris" | "sunos" => Ok(Os::Solaris),
            "android" => Ok(Os::Android),
            "ios" => Ok(Os::Ios),
            _ => Err(ParseOsError(s.to_string())),
        }
    }
}

impl Os {
    /// Parse OS from string (case-insensitive) - convenience method
    pub fn parse(s: &str) -> Option<Os> {
        s.parse().ok()
    }

    /// Check if OS is Unix-like
    pub fn is_unix_like(&self) -> bool {
        matches!(
            self,
            Os::Linux | Os::MacOS | Os::FreeBSD | Os::OpenBSD | Os::NetBSD | Os::Solaris | Os::Android
        )
    }

    /// Check if OS is BSD-based
    pub fn is_bsd(&self) -> bool {
        matches!(self, Os::FreeBSD | Os::OpenBSD | Os::NetBSD | Os::MacOS)
    }

    /// Get all supported OS variants
    pub fn all() -> &'static [Os] {
        &[
            Os::Windows,
            Os::Linux,
            Os::MacOS,
            Os::FreeBSD,
            Os::OpenBSD,
            Os::NetBSD,
            Os::Solaris,
            Os::Android,
            Os::Ios,
        ]
    }
}

/// Detect the current operating system at runtime
#[cfg(target_os = "windows")]
pub fn detect_os() -> Os {
    Os::Windows
}

#[cfg(target_os = "linux")]
pub fn detect_os() -> Os {
    // Check for Android via environment or file system
    if std::path::Path::new("/system/build.prop").exists() {
        return Os::Android;
    }
    Os::Linux
}

#[cfg(target_os = "macos")]
pub fn detect_os() -> Os {
    Os::MacOS
}

#[cfg(target_os = "freebsd")]
pub fn detect_os() -> Os {
    Os::FreeBSD
}

#[cfg(target_os = "openbsd")]
pub fn detect_os() -> Os {
    Os::OpenBSD
}

#[cfg(target_os = "netbsd")]
pub fn detect_os() -> Os {
    Os::NetBSD
}

#[cfg(target_os = "solaris")]
pub fn detect_os() -> Os {
    Os::Solaris
}

#[cfg(target_os = "ios")]
pub fn detect_os() -> Os {
    Os::Ios
}

#[cfg(target_os = "android")]
pub fn detect_os() -> Os {
    Os::Android
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "ios",
    target_os = "android"
)))]
pub fn detect_os() -> Os {
    Os::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_from_str() {
        assert_eq!("windows".parse::<Os>(), Ok(Os::Windows));
        assert_eq!("Windows".parse::<Os>(), Ok(Os::Windows));
        assert_eq!("WINDOWS".parse::<Os>(), Ok(Os::Windows));
        assert_eq!("win".parse::<Os>(), Ok(Os::Windows));
        assert_eq!("linux".parse::<Os>(), Ok(Os::Linux));
        assert_eq!("macos".parse::<Os>(), Ok(Os::MacOS));
        assert_eq!("darwin".parse::<Os>(), Ok(Os::MacOS));
        assert_eq!("freebsd".parse::<Os>(), Ok(Os::FreeBSD));
        assert!("invalid".parse::<Os>().is_err());
    }

    #[test]
    fn test_os_parse() {
        assert_eq!(Os::parse("windows"), Some(Os::Windows));
        assert_eq!(Os::parse("invalid"), None);
    }

    #[test]
    fn test_os_is_unix_like() {
        assert!(!Os::Windows.is_unix_like());
        assert!(Os::Linux.is_unix_like());
        assert!(Os::MacOS.is_unix_like());
        assert!(Os::FreeBSD.is_unix_like());
    }

    #[test]
    fn test_os_is_bsd() {
        assert!(!Os::Windows.is_bsd());
        assert!(!Os::Linux.is_bsd());
        assert!(Os::MacOS.is_bsd());
        assert!(Os::FreeBSD.is_bsd());
        assert!(Os::OpenBSD.is_bsd());
        assert!(Os::NetBSD.is_bsd());
    }

    #[test]
    fn test_os_display() {
        assert_eq!(format!("{}", Os::Windows), "Windows");
        assert_eq!(format!("{}", Os::Linux), "Linux");
        assert_eq!(format!("{}", Os::MacOS), "macOS");
    }

    #[test]
    fn test_detect_os() {
        let os = detect_os();
        // Just make sure it doesn't panic and returns a valid OS
        assert!(Os::all().contains(&os) || os == Os::Unknown);
    }
}
