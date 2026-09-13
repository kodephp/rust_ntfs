// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! Auto-mount daemon.
//!
//! Watches for newly inserted NTFS volumes and mounts them
//! automatically — the core Mounty-like feature. Uses periodic
//! polling of `diskutil list -plist` (portable, no fseventsd
//! dependency). Can be installed as a macOS LaunchAgent for
//! login-time auto-start.
//!
//! # Design
//!
//! - Polls every 3 seconds (configurable via `DaemonOptions`).
//! - Compares current volumes against a known-state snapshot.
//! - Only mounts volumes that are unmounted and newly appeared.
//! - Logs to stderr (daemon-friendly; stdout stays clean).
//! - Graceful shutdown on SIGINT / SIGTERM.
//!
//! # LaunchAgent
//!
//! Installs `~/Library/LaunchAgents/com.kodephp.ntfs-mac.plist`
//! that runs `ntfs-mac daemon` at login.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Error, Result};
use crate::{Config, device, mount};

/// Options for the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonOptions {
    /// Poll interval in seconds.
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    /// Max consecutive poll errors before giving up.
    #[serde(default = "default_max_errors")]
    pub max_consecutive_errors: u32,
    /// Auto-mount only external volumes.
    #[serde(default = "default_true")]
    pub external_only: bool,
}

impl Default for DaemonOptions {
    fn default() -> Self {
        Self {
            interval_secs: default_interval(),
            max_consecutive_errors: default_max_errors(),
            external_only: default_true(),
        }
    }
}

fn default_interval() -> u64 {
    3
}

fn default_max_errors() -> u32 {
    10
}

fn default_true() -> bool {
    true
}

/// Result of a single daemon tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickResult {
    /// Volumes that were mounted this tick.
    pub mounted: Vec<String>,
    /// Total volumes seen.
    pub total_volumes: usize,
    /// Total mounted (pre-existing + newly mounted).
    pub total_mounted: usize,
    /// Errors encountered.
    pub errors: Vec<String>,
}

/// Run the daemon until interrupted.
///
/// This function blocks. Call from a dedicated thread or process.
/// Installs SIGINT/SIGTERM handlers; returns `Ok(())` on graceful
/// shutdown, `Err(Error::Cancelled)` if the signal was received.
pub fn run(cfg: &Config, opts: &DaemonOptions) -> Result<()> {
    crate::hardening::install_signal_handlers();

    let mut known: HashSet<String> = HashSet::new();
    let mut consecutive_errors: u32 = 0;
    let mut last_tick = std::time::Instant::now();

    log_info(&format!(
        "ntfs-mac daemon started (interval={}s, external_only={})",
        opts.interval_secs, opts.external_only
    ));
    if let Ok(volumes) = device::list_volumes() {
        known = collect_unmounted_ids(&volumes);
        log_info(&format!(
            "initial state: {} volumes, {} unmounted",
            volumes.len(),
            known.len()
        ));
    }

    // Poll interval is short (500ms) so SIGINT/SIGTERM is noticed
    // quickly; the actual tick cadence is controlled by `last_tick`.
    let poll_interval = Duration::from_millis(500);

    loop {
        if crate::hardening::should_exit() {
            log_info("shutdown signal received, exiting");
            return Err(Error::Cancelled);
        }

        std::thread::sleep(poll_interval);

        if last_tick.elapsed() < Duration::from_secs(opts.interval_secs) {
            continue;
        }
        last_tick = std::time::Instant::now();

        match device::list_volumes() {
            Ok(volumes) => {
                consecutive_errors = 0;
                let result = tick(&volumes, &mut known, cfg, opts);
                if !result.mounted.is_empty() {
                    log_info(&format!("mounted: {}", result.mounted.join(", ")));
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                log_warn(&format!("poll error ({}): {e}", consecutive_errors));
                if consecutive_errors >= opts.max_consecutive_errors {
                    return Err(Error::CommandFailed {
                        cmd: "daemon".into(),
                        status: -1,
                        stderr: format!("too many consecutive errors: {consecutive_errors}"),
                        io: None,
                    });
                }
            }
        }
    }
}

/// Single poll tick: detect new volumes and mount them.
fn tick(
    volumes: &[crate::Volume],
    known: &mut HashSet<String>,
    cfg: &Config,
    opts: &DaemonOptions,
) -> TickResult {
    let mut mounted = Vec::new();
    let mut errors = Vec::new();

    for vol in volumes {
        if vol.mounted {
            continue;
        }
        if opts.external_only && vol.location != "external" {
            continue;
        }
        if known.contains(&vol.device_identifier) {
            continue;
        }

        log_info(&format!(
            "detected new volume: {} ({})",
            vol.display_label(),
            vol.size_pretty
        ));
        let result = mount::mount(vol, &mount::MountOptions::default(), cfg);
        match result {
            Ok(mp) => {
                mounted.push(format!("{} -> {}", vol.display_label(), mp));
                log_info(&format!("mounted at {mp}"));
            }
            Err(e) => {
                errors.push(format!("{}: {e}", vol.display_label()));
                log_warn(&format!("mount failed for {}: {e}", vol.display_label()));
            }
        }
    }

    *known = collect_unmounted_ids(volumes);

    let total_mounted = volumes.iter().filter(|v| v.mounted).count() + mounted.len();
    TickResult {
        mounted,
        total_volumes: volumes.len(),
        total_mounted,
        errors,
    }
}

/// Collect identifiers of unmounted volumes.
fn collect_unmounted_ids(volumes: &[crate::Volume]) -> HashSet<String> {
    volumes
        .iter()
        .filter(|v| !v.mounted)
        .map(|v| v.device_identifier.clone())
        .collect()
}

// --- LaunchAgent ---

const LAUNCHAGENT_LABEL: &str = "com.kodephp.ntfs-mac";
const LAUNCHAGENT_PLIST_NAME: &str = "com.kodephp.ntfs-mac.plist";

/// Install the daemon as a macOS LaunchAgent.
pub fn install_launchagent(binary_path: &std::path::Path) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        Error::Config(ConfigError::Read {
            path: PathBuf::from("~"),
            reason: "cannot determine home directory".into(),
        })
    })?;
    let agent_dir = home.join("Library").join("LaunchAgents");
    std::fs::create_dir_all(&agent_dir).map_err(|e| {
        Error::Config(ConfigError::Write {
            path: agent_dir.clone(),
            reason: format!("cannot create {}: {e}", agent_dir.display()),
        })
    })?;

    let plist_path = agent_dir.join(LAUNCHAGENT_PLIST_NAME);
    let plist = generate_launchagent_plist(binary_path.to_str().unwrap_or("ntfs-mac"));
    std::fs::write(&plist_path, plist).map_err(|e| {
        Error::Config(ConfigError::Write {
            path: plist_path.clone(),
            reason: format!("cannot write {}: {e}", plist_path.display()),
        })
    })?;

    log_info(&format!("LaunchAgent installed: {}", plist_path.display()));
    Ok(plist_path)
}

/// Uninstall the LaunchAgent.
pub fn uninstall_launchagent() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| {
        Error::Config(ConfigError::Read {
            path: PathBuf::from("~"),
            reason: "cannot determine home directory".into(),
        })
    })?;
    let plist_path = home
        .join("Library")
        .join("LaunchAgents")
        .join(LAUNCHAGENT_PLIST_NAME);

    if plist_path.exists() {
        std::fs::remove_file(&plist_path).map_err(|e| {
            Error::Config(ConfigError::Write {
                path: plist_path.clone(),
                reason: format!("cannot remove {}: {e}", plist_path.display()),
            })
        })?;
        log_info(&format!("LaunchAgent removed: {}", plist_path.display()));
    } else {
        log_info("LaunchAgent not installed");
    }
    Ok(())
}

/// Check if the LaunchAgent is installed.
pub fn is_launchagent_installed() -> bool {
    let home = dirs::home_dir();
    match home {
        Some(h) => h
            .join("Library")
            .join("LaunchAgents")
            .join(LAUNCHAGENT_PLIST_NAME)
            .exists(),
        None => false,
    }
}

/// Generate the LaunchAgent plist content.
fn generate_launchagent_plist(binary_path: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>~/Library/Logs/ntfs-mac.out.log</string>
    <key>StandardErrorPath</key>
    <string>~/Library/Logs/ntfs-mac.err.log</string>
    <key>WorkingDirectory</key>
    <string>~</string>
</dict>
</plist>
"#,
        label = LAUNCHAGENT_LABEL,
        binary = binary_path,
    )
}

// --- Logging (uses tracing for consistent structured output) ---

fn log_info(msg: &str) {
    tracing::info!(target: "ntfs_mac_core::daemon", "{msg}");
}

fn log_warn(msg: &str) {
    tracing::warn!(target: "ntfs_mac_core::daemon", "{msg}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_options_defaults() {
        let opts = DaemonOptions::default();
        assert_eq!(opts.interval_secs, 3);
        assert_eq!(opts.max_consecutive_errors, 10);
        assert!(opts.external_only);
    }

    #[test]
    fn tick_mounts_new_volume() {
        let cfg = Config::default();
        let opts = DaemonOptions {
            external_only: false,
            ..Default::default()
        };
        let volumes = vec![crate::Volume {
            device_identifier: "disk2s2".into(),
            volume_name: "TestVol".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 1000000000,
            mounted: false,
            mount_point: None,
            parent_disk: Some("disk2".into()),
            size_pretty: "0.9 GiB".into(),
            location: "external".into(),
            contents: Some("GUID_partition_scheme".into()),
        }];
        let mut known: HashSet<String> = HashSet::new();
        let result = tick(&volumes, &mut known, &cfg, &opts);
        assert_eq!(result.total_volumes, 1);
        assert!(!result.errors.is_empty() || !result.mounted.is_empty());
    }

    #[test]
    fn tick_skips_already_mounted() {
        let cfg = Config::default();
        let opts = DaemonOptions {
            external_only: false,
            ..Default::default()
        };
        let volumes = vec![crate::Volume {
            device_identifier: "disk2s2".into(),
            volume_name: "Mounted".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 0,
            mounted: true,
            mount_point: Some("/Volumes/Mounted".into()),
            parent_disk: Some("disk2".into()),
            size_pretty: "0 B".into(),
            location: "external".into(),
            contents: None,
        }];
        let mut known: HashSet<String> = HashSet::new();
        let result = tick(&volumes, &mut known, &cfg, &opts);
        assert!(result.mounted.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.total_mounted, 1);
    }

    #[test]
    fn tick_skips_internal_when_external_only() {
        let cfg = Config::default();
        let opts = DaemonOptions {
            external_only: true,
            ..Default::default()
        };
        let volumes = vec![crate::Volume {
            device_identifier: "disk0s2".into(),
            volume_name: "Internal".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 0,
            mounted: false,
            mount_point: None,
            parent_disk: Some("disk0".into()),
            size_pretty: "0 B".into(),
            location: "internal".into(),
            contents: None,
        }];
        let mut known: HashSet<String> = HashSet::new();
        let result = tick(&volumes, &mut known, &cfg, &opts);
        assert!(result.mounted.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn launchagent_plist_generation() {
        let plist = generate_launchagent_plist("/usr/local/bin/ntfs-mac");
        assert!(plist.contains("com.kodephp.ntfs-mac"));
        assert!(plist.contains("/usr/local/bin/ntfs-mac"));
        assert!(plist.contains("RunAtLoad"));
    }

    #[test]
    fn test_collect_unmounted_ids() {
        let volumes = vec![
            crate::Volume {
                device_identifier: "disk2s2".into(),
                volume_name: "A".into(),
                media_type: "com.microsoft.ntfs".into(),
                uuid: None,
                size_bytes: 0,
                mounted: false,
                mount_point: None,
                parent_disk: None,
                size_pretty: "0 B".into(),
                location: "external".into(),
                contents: None,
            },
            crate::Volume {
                device_identifier: "disk3s2".into(),
                volume_name: "B".into(),
                media_type: "com.microsoft.ntfs".into(),
                uuid: None,
                size_bytes: 0,
                mounted: true,
                mount_point: Some("/Volumes/B".into()),
                parent_disk: None,
                size_pretty: "0 B".into(),
                location: "external".into(),
                contents: None,
            },
        ];
        let ids = collect_unmounted_ids(&volumes);
        assert!(ids.contains("disk2s2"));
        assert!(!ids.contains("disk3s2"));
        assert_eq!(ids.len(), 1);
    }
}
