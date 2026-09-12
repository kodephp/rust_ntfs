//! Format NTFS volumes. Destructive: requires a matching
//! [`DestructiveToken`].
//!
//! Prefers `newfs_ntfs` (from ntfs-3g) over `diskutil eraseVolume`
//! because the latter cannot produce a genuine NTFS volume.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::runner::{RunOptions, run_expect_success};
use crate::{Config, DestructiveToken, Volume};

/// Options for `format`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatOptions {
    /// Volume label (NTFS max 11 chars, but we accept longer and warn).
    pub label: String,
    /// Quick format (no full surface write). Default `true`.
    #[serde(default = "default_quick")]
    pub quick: bool,
    /// Sector size in bytes (512 or 4096). Default `4096` (GPT default).
    #[serde(default = "default_sector_size")]
    pub sector_size: u32,
    /// Cluster size in bytes (usually 4096).
    #[serde(default = "default_cluster_size")]
    pub cluster_size: u32,
    /// Extra ntfs-3g `newfs_ntfs` arguments.
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            label: String::new(),
            quick: true,
            sector_size: 4096,
            cluster_size: 4096,
            extra_args: Vec::new(),
        }
    }
}

fn default_quick() -> bool {
    true
}
fn default_sector_size() -> u32 {
    4096
}
fn default_cluster_size() -> u32 {
    4096
}

/// Format `vol` as NTFS. Requires `token.device_identifier` to match
/// `vol.device_identifier`.
pub fn format(
    vol: &Volume,
    opts: &FormatOptions,
    token: &DestructiveToken,
    _cfg: &Config,
) -> Result<()> {
    if vol.device_identifier != token.device_identifier {
        return Err(Error::ConfirmationMismatch {
            expected: token.device_identifier.clone(),
            actual: vol.device_identifier.clone(),
        });
    }
    if !vol.device_identifier.starts_with("disk") {
        return Err(Error::InvalidArgument(format!(
            "refusing to format `{}` — identifier must start with 'disk'",
            vol.device_identifier
        )));
    }

    // Prefer newfs_ntfs; fall back to mkntfs if not present.
    let (bin_name, bin_path) = match crate::runner::which("newfs_ntfs") {
        Ok(p) => ("newfs_ntfs", p),
        Err(_) => {
            let p = crate::runner::which("mkntfs")?;
            ("mkntfs", p)
        }
    };

    let mut args: Vec<String> = Vec::new();
    if opts.quick {
        args.push("-F".into());
    }
    if opts.sector_size == 4096 {
        args.push("-f".into());
        args.push("4096".into());
    }
    if opts.cluster_size == 4096 {
        args.push("-c".into());
        args.push("4096".into());
    }
    if !opts.label.is_empty() {
        args.push("-L".into());
        args.push(opts.label.clone());
    }
    for a in &opts.extra_args {
        args.push(a.clone());
    }
    args.push(format!("/dev/{}", vol.device_identifier));

    let bin_str = bin_path.to_str().unwrap_or(bin_name);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    run_expect_success(
        bin_str,
        &arg_refs,
        &RunOptions {
            timeout: Some(Duration::from_secs(600)),
            ..Default::default()
        },
    )?;
    Ok(())
}

/// Validate a volume label for NTFS (max 11 chars, no special
/// characters). Returns a warning string, or `None` if clean.
pub fn validate_label(label: &str) -> Option<String> {
    if label.is_empty() {
        return None;
    }
    if label.len() > 11 {
        return Some(format!(
            "NTFS volume labels are limited to 11 characters; your label is {} chars",
            label.chars().count()
        ));
    }
    if label.contains([':', '\\', '/', '*', '?', '"', '<', '>', '|']) {
        return Some("label contains invalid NTFS characters: : \\ / * ? \" < > |".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_label_lengths() {
        assert!(validate_label("MyData").is_none());
        assert!(validate_label("ABCDEFGHIJK").is_none()); // 11
        assert!(validate_label("ABCDEFGHIJKL").is_some()); // 12
        assert!(validate_label("").is_none());
    }

    #[test]
    fn validate_label_invalid_chars() {
        assert!(validate_label("a:b").is_some());
        assert!(validate_label("a\\b").is_some());
        assert!(validate_label("a/b").is_some());
        assert!(validate_label("a*b").is_some());
        assert!(validate_label("a?b").is_some());
        assert!(validate_label("a\"b").is_some());
        assert!(validate_label("a<b").is_some());
        assert!(validate_label("a>b").is_some());
        assert!(validate_label("a|b").is_some());
    }

    #[test]
    fn format_rejects_identifier_mismatch() {
        let vol = Volume {
            device_identifier: "disk2s2".into(),
            volume_name: "X".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 0,
            mounted: false,
            mount_point: None,
            parent_disk: None,
            size_pretty: "0 B".into(),
            location: "external".into(),
            contents: None,
        };
        let cfg = Config::default();
        let opts = FormatOptions::default();
        let token = DestructiveToken::new("disk999s1", "wrong");
        let r = format(&vol, &opts, &token, &cfg);
        assert!(matches!(r, Err(Error::ConfirmationMismatch { .. })));
    }

    #[test]
    fn format_rejects_non_disk_identifier() {
        let vol = Volume {
            device_identifier: "notdisk".into(),
            volume_name: "X".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 0,
            mounted: false,
            mount_point: None,
            parent_disk: None,
            size_pretty: "0 B".into(),
            location: "external".into(),
            contents: None,
        };
        let cfg = Config::default();
        let opts = FormatOptions::default();
        let token = DestructiveToken::new("notdisk", "");
        let r = format(&vol, &opts, &token, &cfg);
        assert!(matches!(r, Err(Error::InvalidArgument(_))));
    }
}
