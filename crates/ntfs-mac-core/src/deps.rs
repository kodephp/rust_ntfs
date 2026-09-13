// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! Dependency discovery: ntfs-3g, macFUSE / FUSE-T, diskutil, hdiutil,
//! rsync, mkntfs/newfs_ntfs, ntfsfix, fsck_ntfs.
//!
//! Everything is optional at compile time — this module only reports
//! what is actually present so the CLI can give actionable hints.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::runner;

/// Which FUSE driver the user should install on macOS.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FuseDriver {
    /// `brew install --cask macfuse` (Intel + Apple Silicon, dext-based).
    Macfuse,
    /// `brew install --cask fuse-t` (Apple Silicon native, FUSE-T).
    FuseT,
    /// Let ntfs-3g decide.
    #[default]
    Auto,
}

impl From<&str> for FuseDriver {
    fn from(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "macfuse" => Self::Macfuse,
            "fuse-t" | "fuset" => Self::FuseT,
            _ => Self::Auto,
        }
    }
}

impl FuseDriver {
    /// Homebrew install hint.
    pub fn brew_install_command(&self) -> String {
        match self {
            Self::Macfuse => "brew install --cask macfuse".into(),
            Self::FuseT => "brew install --cask fuse-t".into(),
            Self::Auto => "brew install --cask macfuse  # or fuse-t on Apple Silicon".into(),
        }
    }
}

/// Report for a single dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepStatus {
    /// Display name.
    pub name: String,
    /// Whether it is present.
    pub present: bool,
    /// Location on disk, when present.
    pub path: Option<PathBuf>,
    /// Suggested fix command, when absent.
    pub install_hint: Option<String>,
}

impl DepStatus {
    fn missing(name: &str, hint: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            present: false,
            path: None,
            install_hint: Some(hint.into()),
        }
    }
}

/// Report for all dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepReport {
    /// Every probed dependency.
    pub deps: Vec<DepStatus>,
    /// Overall readiness: `true` only if every dep is present.
    pub ready: bool,
    /// Architecture detection (`x86_64` or `aarch64`).
    pub arch: String,
    /// macOS version string from `sw_vers -productVersion`.
    pub macos_version: Option<String>,
}

impl fmt::Display for DepReport {
    /// Human-readable summary. Used by `ntfs-mac doctor`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Platform: macOS {} on {}",
            self.macos_version.as_deref().unwrap_or("unknown"),
            self.arch
        )?;
        writeln!(f, "\nDependencies:")?;
        for d in &self.deps {
            let icon = if d.present { "✓" } else { "✗" };
            let loc = d
                .path
                .as_ref()
                .map(|p| format!("  ({})", p.display()))
                .unwrap_or_default();
            writeln!(
                f,
                "  {} {:<20} {}",
                icon,
                d.name,
                if d.present {
                    loc
                } else {
                    format!(
                        "missing — {}",
                        d.install_hint.as_deref().unwrap_or("no hint")
                    )
                }
            )?;
        }
        writeln!(f, "\nReady: {}\n", self.ready)?;
        Ok(())
    }
}

/// Probe all known dependencies. Does not fail — missing deps are
/// reported, not raised. Only `diskutil` / `hdiutil` (missing = fatal
/// system state) can produce an [`Error`].
pub fn report() -> Result<DepReport> {
    let arch = std::env::consts::ARCH.to_string();
    let macos = shell_value("sw_vers", &["-productVersion"]);

    let mut deps: Vec<DepStatus> = Vec::new();

    // Core macOS tools — always present; missing here means the OS is
    // misconfigured.
    for (name, hint) in [
        ("diskutil", "System tool — should be at /usr/sbin/diskutil"),
        ("hdiutil", "System tool — should be at /usr/bin/hdiutil"),
    ] {
        deps.push(match runner::which(name) {
            Ok(p) => DepStatus {
                name: name.to_string(),
                present: true,
                path: Some(p),
                install_hint: None,
            },
            Err(_) => DepStatus::missing(name, hint),
        });
    }

    // ntfs-3g and its sub-tools.
    for (name, hint) in [
        (
            "ntfs-3g",
            "brew install ntfs-3g   (may also require `brew install --cask macfuse`)",
        ),
        ("newfs_ntfs", "ships with `brew install ntfs-3g`"),
        (
            "mkntfs",
            "ships with `brew install ntfs-3g` (alias of newfs_ntfs)",
        ),
        ("ntfsfix", "ships with `brew install ntfs-3g`"),
        ("fsck_ntfs", "ships with `brew install ntfs-3g`"),
    ] {
        deps.push(match runner::which(name) {
            Ok(p) => DepStatus {
                name: name.to_string(),
                present: true,
                path: Some(p),
                install_hint: None,
            },
            Err(_) => DepStatus::missing(name, hint),
        });
    }

    // rsync is highly recommended for `copy` (progress + rsync semantics).
    deps.push(match runner::which("rsync") {
        Ok(p) => DepStatus {
            name: "rsync".to_string(),
            present: true,
            path: Some(p),
            install_hint: None,
        },
        Err(_) => DepStatus::missing("rsync", "brew install rsync"),
    });

    // FUSE driver: macFUSE or FUSE-T. Presence is inferred from the
    // filesystem driver directory.
    let macfuse_path = PathBuf::from("/Library/Filesystems/macfuse.fs/Contents/Resources/ntfs");
    let fuset_path = PathBuf::from("/Library/Filesystems/fuset.fs/Contents/Resources/ntfs");
    let fuse_present = macfuse_path.exists() || fuset_path.exists();
    deps.push(if fuse_present {
        DepStatus {
            name: "fuse-driver".to_string(),
            present: true,
            path: Some(if macfuse_path.exists() {
                macfuse_path
            } else {
                fuset_path
            }),
            install_hint: None,
        }
    } else {
        DepStatus::missing(
            "fuse-driver",
            "brew install --cask macfuse   (Intel + Apple Silicon)   OR   brew install --cask fuse-t   (Apple Silicon)",
        )
    });

    let ready = deps.iter().all(|d| d.present);

    Ok(DepReport {
        deps,
        ready,
        arch,
        macos_version: macos,
    })
}

/// Small helper to capture the stdout of a read-only system probe.
fn shell_value(cmd: &str, args: &[&str]) -> Option<String> {
    use crate::runner::{RunOptions, run};
    match run(cmd, args, &RunOptions::default()) {
        Ok(out) if out.success() => Some(out.stdout.trim().to_string()),
        _ => None,
    }
}

/// Convenience: return [`Error::MissingDependency`] if the report is
/// not ready. Used by CLI commands that need the full stack.
pub fn require_ready() -> Result<()> {
    let report = report()?;
    if report.ready {
        Ok(())
    } else {
        Err(Error::MissingDependency {
            binary: "ntfs-mac dependencies".into(),
            detail: report
                .deps
                .iter()
                .filter(|d| !d.present)
                .map(|d| d.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            hint: Some(
                "Run `ntfs-mac doctor` for details, or `./scripts/install.sh` to install.".into(),
            ),
            io: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuse_driver_from_str_roundtrip() {
        assert_eq!(FuseDriver::from("macfuse"), FuseDriver::Macfuse);
        assert_eq!(FuseDriver::from("fuse-t"), FuseDriver::FuseT);
        assert_eq!(FuseDriver::from("FUSE-T"), FuseDriver::FuseT);
        assert_eq!(FuseDriver::from("auto"), FuseDriver::Auto);
        assert_eq!(FuseDriver::from("??"), FuseDriver::Auto);
    }

    #[test]
    fn dep_report_renders_without_panic() {
        // The test machine may or may not have the deps; we only
        // assert the rendering path is exercised.
        let report = DepReport {
            deps: vec![DepStatus::missing("ntfs-3g", "brew install ntfs-3g")],
            ready: false,
            arch: "aarch64".into(),
            macos_version: Some("26.6.2".into()),
        };
        let text = report.to_string();
        assert!(text.contains("ntfs-3g"));
        assert!(text.contains("aarch64"));
    }
}
