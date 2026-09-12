//! File copy between macOS and an NTFS volume. Uses `rsync` when
//! available for progress reporting + resume semantics; falls back
//! to `cp -a` when rsync is not installed.
//!
//! `src` and `dst` are paths (either local filesystem paths or
//! `/Volumes/<label>/<file>` paths on a mounted NTFS volume). The
//! caller is responsible for ensuring the target is mounted.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::runner::{RunOptions, run_expect_success, which};

/// Options for `copy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyOptions {
    /// Delete files in destination that are not in source (rsync `--delete`).
    #[serde(default)]
    pub delete: bool,
    /// Preserve timestamps, ownership, permissions.
    #[serde(default = "default_true")]
    pub preserve: bool,
    /// Human-readable progress output.
    #[serde(default = "default_true")]
    pub progress: bool,
    /// Dry-run: print what would be copied, but don't touch files.
    #[serde(default)]
    pub dry_run: bool,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            delete: false,
            preserve: true,
            progress: true,
            dry_run: false,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Result of a copy operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyResult {
    pub bytes_transferred: Option<u64>,
    pub files_transferred: Option<u64>,
    pub duration_ms: u64,
    pub tool_used: String,
    pub dry_run: bool,
}

/// Copy `src` to `dst` using rsync (preferred) or cp (fallback).
pub fn copy(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<CopyResult> {
    let started = std::time::Instant::now();
    let result = if which("rsync").is_ok() {
        copy_with_rsync(src, dst, opts)?
    } else {
        copy_with_cp(src, dst, opts)?
    };
    Ok(CopyResult {
        bytes_transferred: result.bytes_transferred,
        files_transferred: result.files_transferred,
        duration_ms: started.elapsed().as_millis() as u64,
        tool_used: result.tool_used,
        dry_run: opts.dry_run,
    })
}

struct CopyOutcome {
    bytes_transferred: Option<u64>,
    files_transferred: Option<u64>,
    tool_used: String,
}

fn copy_with_rsync(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<CopyOutcome> {
    let mut args: Vec<String> = Vec::new();
    if opts.delete {
        args.push("--delete".into());
    }
    if opts.preserve {
        args.push("-a".into());
    } else {
        args.push("-rt".into());
    }
    if opts.progress {
        // macOS ships rsync 2.6.9 which lacks --info=progress2;
        // use --progress (universally supported).
        args.push("--progress".into());
    }
    if opts.dry_run {
        args.push("--dry-run".into());
    }
    args.push(src.to_string_lossy().to_string());
    args.push(dst.to_string_lossy().to_string());

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_expect_success(
        "rsync",
        &refs,
        &RunOptions {
            timeout: Some(Duration::from_secs(3600)),
            ..Default::default()
        },
    )?;
    Ok(CopyOutcome {
        bytes_transferred: None,
        files_transferred: None,
        tool_used: "rsync".into(),
    })
}

fn copy_with_cp(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<CopyOutcome> {
    let mut args: Vec<String> = Vec::new();
    args.push("-a".into());
    if opts.dry_run {
        args.push("-n".into());
    }
    args.push(src.to_string_lossy().to_string());
    args.push(dst.to_string_lossy().to_string());

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_expect_success(
        "cp",
        &refs,
        &RunOptions {
            timeout: Some(Duration::from_secs(3600)),
            ..Default::default()
        },
    )?;
    let _ = opts;
    Ok(CopyOutcome {
        bytes_transferred: None,
        files_transferred: None,
        tool_used: "cp".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_options_defaults() {
        let opts = CopyOptions::default();
        assert!(!opts.delete);
        assert!(opts.preserve);
        assert!(opts.progress);
        assert!(!opts.dry_run);
    }

    #[test]
    fn dry_run_does_not_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("a.txt"), "hello").unwrap();

        let opts = CopyOptions {
            dry_run: true,
            ..Default::default()
        };
        let _r = copy(&src, &dst, &opts);
    }
}
