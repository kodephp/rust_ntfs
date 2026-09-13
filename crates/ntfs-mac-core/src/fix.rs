// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! NTFS repair. `ntfsfix -d` unmounts the volume, clears the dirty
//! flag and runs a light chkdsk-like pass. `fsck_ntfs` is a heavier
//! alternative.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::Volume;
use crate::error::{Error, Result};
use crate::runner::{RunOptions, run_expect_success};

/// Which fix tool to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FixTool {
    /// Fast: `ntfsfix -d /dev/diskN`.
    #[default]
    Ntfsfix,
    /// Slower, more thorough: `fsck_ntfs /dev/diskN`.
    FsckNtfs,
}

/// Options for `fix`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixOptions {
    pub tool: FixTool,
    /// Pass the `-d` (drop dirty flag) to ntfsfix.
    #[serde(default = "default_true")]
    pub drop_dirty_flag: bool,
}

impl Default for FixOptions {
    fn default() -> Self {
        Self {
            tool: FixTool::Ntfsfix,
            drop_dirty_flag: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Run a repair pass on `vol`.
pub fn fix(vol: &Volume) -> Result<()> {
    fix_with_opts(vol, &FixOptions::default())
}

/// Run a repair pass with explicit options.
pub fn fix_with_opts(vol: &Volume, opts: &FixOptions) -> Result<()> {
    if vol.mounted {
        return Err(Error::InvalidArgument(format!(
            "volume `{}` is mounted at `{}`; unmount before running fix",
            vol.device_identifier,
            vol.mount_point.as_deref().unwrap_or("<unknown>")
        )));
    }
    let dev_path = format!("/dev/{}", vol.device_identifier);
    match opts.tool {
        FixTool::Ntfsfix => {
            let mut args: Vec<String> = Vec::new();
            if opts.drop_dirty_flag {
                args.push("-d".into());
            }
            args.push(dev_path);
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            run_expect_success(
                "ntfsfix",
                &refs,
                &RunOptions {
                    timeout: Some(Duration::from_secs(1800)),
                    ..Default::default()
                },
            )?;
        }
        FixTool::FsckNtfs => {
            run_expect_success(
                "fsck_ntfs",
                &[dev_path.as_str()],
                &RunOptions {
                    timeout: Some(Duration::from_secs(3600)),
                    ..Default::default()
                },
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_refuses_mounted_volume() {
        let vol = Volume {
            device_identifier: "disk2s2".into(),
            volume_name: "X".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 0,
            mounted: true,
            mount_point: Some("/Volumes/X".into()),
            parent_disk: None,
            size_pretty: "0 B".into(),
            location: "external".into(),
            contents: None,
        };
        let r = fix(&vol);
        assert!(matches!(r, Err(Error::InvalidArgument(_))));
    }

    #[test]
    fn fix_tool_defaults_to_ntfsfix() {
        let opts = FixOptions::default();
        assert_eq!(opts.tool, FixTool::Ntfsfix);
        assert!(opts.drop_dirty_flag);
    }
}
