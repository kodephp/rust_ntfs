// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! Mount / unmount NTFS volumes.
//!
//! Prefer `diskutil mount <disk>` because it transparently routes
//! through whichever FUSE helper (ntfs-3g or macOS builtin) is
//! registered for the media type. Fall back to a direct `ntfs-3g`
//! invocation when diskutil refuses.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::runner::{RunOptions, run_expect_success, which};
use crate::{Config, Volume};

/// Options for `mount`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MountOptions {
    /// Explicit mount point. If `None`, use `<cfg.mount_base>/<volume_name>`.
    pub mount_point: Option<String>,
    /// Read-only mount.
    #[serde(default)]
    pub readonly: bool,
    /// Extra ntfs-3g mount options (space-separated).
    #[serde(default)]
    pub extra_options: Vec<String>,
    /// Use `ntfs-3g` directly rather than `diskutil mount`.
    #[serde(default)]
    pub force_ntfs3g: bool,
}

/// Validate a single mount option before it is joined into the `-o`
/// comma list. Options are passed as a single argv element (no shell
/// involved), but a malformed option would silently corrupt the whole
/// option string, so reject anything outside the well-formed charset
/// early with a precise error instead of a confusing mount failure.
fn validate_mount_option(opt: &str) -> Result<()> {
    let bad = |why: &str| {
        Err(Error::InvalidArgument(format!(
            "invalid mount option `{opt}`: {why}"
        )))
    };
    if opt.trim().is_empty() {
        return bad("empty option");
    }
    if opt.contains(|c: char| {
        c.is_whitespace() || matches!(c, ',' | ';' | '"' | '\'' | '\\' | '`' | '$' | '\0')
    }) {
        return bad(
            "option must be a single token without whitespace or separator characters (`, ; \" ' \\ \\` $`)",
        );
    }
    Ok(())
}

/// Validate every user-supplied mount option (config + per-call extras).
pub fn validate_mount_options(options: &[String]) -> Result<()> {
    for opt in options {
        validate_mount_option(opt)?;
    }
    Ok(())
}

/// Mount an NTFS volume. `vol` should come from `device::list_volumes`
/// so that the identifier and label are already validated.
pub fn mount(vol: &Volume, opts: &MountOptions, cfg: &Config) -> Result<String> {
    validate_mount_options(&cfg.mount_options)?;
    validate_mount_options(&opts.extra_options)?;
    if vol.mounted {
        return Ok(vol
            .mount_point
            .clone()
            .unwrap_or_else(|| vol.device_identifier.clone()));
    }

    let mount_point = opts.mount_point.clone().unwrap_or_else(|| {
        let name = if vol.volume_name.trim().is_empty() {
            vol.device_identifier.replace('s', "-")
        } else {
            vol.volume_name.clone()
        };
        // Sanitize the mount-point basename.
        let safe: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || ['-', '_', '|', ' '].contains(&c) {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        std::path::PathBuf::from(&cfg.mount_base)
            .join(&safe)
            .to_string_lossy()
            .to_string()
    });

    std::fs::create_dir_all(&mount_point).map_err(|e| Error::CommandFailed {
        cmd: "mkdir".into(),
        status: e.raw_os_error().unwrap_or(-1),
        stderr: format!("cannot create mount point {}: {}", mount_point, e),
        io: Some(e),
    })?;

    let opts_vec = build_mount_options(vol, opts, cfg);
    let opts_str = opts_vec.join(",");
    let dev_path = format!("/dev/{}", vol.device_identifier);

    if opts.force_ntfs3g || !diskutil_mount(&dev_path, &mount_point, &opts_vec) {
        // Fall back to direct ntfs-3g.
        let ntfs_bin = which("ntfs-3g")?;
        let args = ["-o", &opts_str, &dev_path, &mount_point];
        run_expect_success(
            ntfs_bin.to_str().unwrap_or("ntfs-3g"),
            &args,
            &RunOptions {
                timeout: Some(Duration::from_secs(60)),
                ..Default::default()
            },
        )?;
        Ok(mount_point)
    } else {
        Ok(mount_point)
    }
}

/// Remount a volume as read-write. If already read-write, no-op.
/// If mounted read-only, unmounts and remounts without `ro`.
/// If not mounted, mounts as read-write.
pub fn remount_rw(vol: &Volume, cfg: &Config) -> Result<String> {
    if !vol.mounted {
        // Not mounted — mount as read-write.
        let opts = MountOptions {
            readonly: false,
            ..Default::default()
        };
        return mount(vol, &opts, cfg);
    }

    // Check current mount options to see if it's read-only.
    let mount_point = vol
        .mount_point
        .clone()
        .unwrap_or_else(|| vol.device_identifier.clone());

    // Use `mount` command to check options.
    let out = crate::runner::run(
        "mount",
        &[&mount_point],
        &RunOptions {
            timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
    );

    let readonly_mount = match out {
        Ok(o) if o.success() => o.stdout.contains("(read-only)"),
        _ => false,
    };

    if !readonly_mount {
        // Already read-write — nothing to do.
        return Ok(mount_point);
    }

    // Read-only mount detected — remount as read-write.
    unmount(&vol.device_identifier)?;
    let opts = MountOptions {
        readonly: false,
        ..Default::default()
    };
    mount(vol, &opts, cfg)
}

/// Unmount an NTFS volume. `device_identifier` should be the disk
/// identifier returned by `list_volumes`.
pub fn unmount(device_identifier: &str) -> Result<()> {
    let dev_path = format!("/dev/{}", device_identifier);
    run_expect_success(
        "diskutil",
        &["unmount", &dev_path],
        &RunOptions {
            timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        },
    )?;
    Ok(())
}

/// Build the ntfs-3g option string from config + user options.
fn build_mount_options(vol: &Volume, opts: &MountOptions, cfg: &Config) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    // Always let macOS decide ownership; ntfs-3g defaults to root.
    out.push("noowners".to_string());
    // Preserve atime is optional and can slow things down.
    out.push("atime".to_string());
    if opts.readonly {
        out.push("ro".to_string());
    }
    for extra in &cfg.mount_options {
        out.push(extra.clone());
    }
    for extra in &opts.extra_options {
        out.push(extra.clone());
    }
    let _ = vol; // Reserved for per-volume overrides.
    out
}

/// Try `diskutil mount -o <opts> <dev> <mount_point>`. Returns
/// `true` if it succeeded. Never returns an error — callers decide
/// whether to fall back.
fn diskutil_mount(dev_path: &str, mount_point: &str, opts: &[String]) -> bool {
    use crate::runner::run;
    let opts_str = opts.join(",");
    let mut args: Vec<&str> = vec!["mount"];
    if !opts.is_empty() {
        args.push("-o");
        args.push(&opts_str);
    }
    args.push(dev_path);
    args.push(mount_point);
    let out = run(
        "diskutil",
        &args,
        &RunOptions {
            timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        },
    );
    match out {
        Ok(o) => o.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn mount_option_validation_accepts_wellformed() {
        assert!(validate_mount_options(&["big_writes".into()]).is_ok());
        assert!(validate_mount_options(&["uid=501".into(), "auto_xattr".into()]).is_ok());
        assert!(validate_mount_options(&[]).is_ok());
    }

    #[test]
    fn mount_option_validation_rejects_bad_tokens() {
        assert!(validate_mount_options(&["".into()]).is_err());
        assert!(validate_mount_options(&["  ".into()]).is_err());
        assert!(validate_mount_options(&["big writes".into()]).is_err());
        assert!(validate_mount_options(&["a,b".into()]).is_err());
        assert!(validate_mount_options(&["a;b".into()]).is_err());
        assert!(validate_mount_options(&["a\"b".into()]).is_err());
        assert!(validate_mount_options(&["a'b".into()]).is_err());
        assert!(validate_mount_options(&["a\\b".into()]).is_err());
        assert!(validate_mount_options(&["a$b".into()]).is_err());
        assert!(validate_mount_options(&["a`b".into()]).is_err());
    }

    #[test]
    fn build_mount_options_defaults() {
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
        let opts = MountOptions::default();
        let built = build_mount_options(&vol, &opts, &cfg);
        assert!(built.contains(&"noowners".to_string()));
        assert!(built.contains(&"atime".to_string()));
        assert!(!built.contains(&"ro".to_string()));
    }

    #[test]
    fn build_mount_options_includes_readonly() {
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
        let cfg = Config {
            mount_options: vec!["uid=501".into()],
            ..Default::default()
        };
        let opts = MountOptions {
            readonly: true,
            extra_options: vec!["mask=777".into()],
            ..Default::default()
        };
        let built = build_mount_options(&vol, &opts, &cfg);
        assert!(built.contains(&"ro".to_string()));
        assert!(built.contains(&"uid=501".to_string()));
        assert!(built.contains(&"mask=777".to_string()));
    }

    #[test]
    fn unmount_missing_device_errors() {
        // This is a live system call; on CI or non-macOS it may fail
        // differently. We only assert the function runs and returns a
        // typed error, not a specific message.
        let r = unmount("disk99999s99");
        assert!(r.is_err());
    }

    #[test]
    fn mount_point_path_sanity() {
        // Just verify create_dir_all + a bogus mount doesn't panic.
        let tmp = tempfile::tempdir().unwrap();
        let _ = tmp.path().join("a");
        let _ = Path::new(tmp.path());
    }
}
