//! Persistent configuration at `~/.config/ntfs-mac/config.toml`.
//!
//! Kept intentionally tiny — 5 fields, no schema versioning. Add
//! fields freely; unknown fields are ignored on read.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Error, Result};

/// Default config location. Override via `XDG_CONFIG_HOME` if set.
pub fn default_config_path() -> PathBuf {
    if let Some(home) = dirs::config_dir() {
        home.join("ntfs-mac").join("config.toml")
    } else {
        // Fallback for the (very rare) case where config_dir() returns
        // None — still deterministic so tests can rely on it.
        PathBuf::from("/tmp/ntfs-mac/config.toml")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default mount point base. NTFS volumes mount to
    /// `<base>/<volume_label>` by default.
    #[serde(default = "default_mount_base")]
    pub mount_base: String,

    /// Extra `ntfs-3g` mount options, space-separated.
    /// Examples: `"noowners", "uid=501", "gid=20"`.
    #[serde(default)]
    pub mount_options: Vec<String>,

    /// Preferred FUSE driver when both macFUSE and FUSE-T are
    /// installed. `"fuse-t"` (Apple Silicon default), `"macfuse"`, or
    /// `"auto"` (let ntfs-3g decide).
    #[serde(default = "default_fuse_driver")]
    pub fuse_driver: String,

    /// Require interactive confirmation before `format`/`fix` even
    /// when the caller supplies a token. Set to `false` only in
    /// scripted/CI environments.
    #[serde(default = "default_true")]
    pub require_confirmation: bool,

    /// Emit coloured terminal output. Only affects the CLI.
    #[serde(default = "default_true")]
    pub color: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mount_base: default_mount_base(),
            mount_options: Vec::new(),
            fuse_driver: default_fuse_driver(),
            require_confirmation: default_true(),
            color: default_true(),
        }
    }
}

fn default_mount_base() -> String {
    "/Volumes".to_string()
}

fn default_fuse_driver() -> String {
    "auto".to_string()
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load config from the default path. Missing file → defaults.
    pub fn load() -> Result<Self> {
        load_config(&default_config_path())
    }

    /// Save to the default path. Creates parent dirs.
    pub fn save(&self) -> Result<()> {
        save_config(self, &default_config_path())
    }

    /// Override the mount base (used by CLI `--mount-base`).
    #[must_use]
    pub fn with_mount_base(mut self, base: String) -> Self {
        self.mount_base = base;
        self
    }
}

/// Load a TOML config from `path`. Missing file returns defaults.
pub fn load_config(path: &std::path::Path) -> Result<Config> {
    let Ok(bytes) = std::fs::read(path) else {
        return Ok(Config::default());
    };
    match toml::from_str(&String::from_utf8_lossy(&bytes)) {
        Ok(c) => Ok(c),
        Err(e) => Err(Error::Serde(e.to_string())),
    }
}

/// Save a config to `path`, creating parent dirs.
pub fn save_config(cfg: &Config, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::Write {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
    }
    let text = toml::to_string_pretty(cfg).map_err(|e| ConfigError::Write {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    std::fs::write(path, text).map_err(|e| ConfigError::Write {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let c = Config::default();
        assert_eq!(c.mount_base, "/Volumes");
        assert_eq!(c.fuse_driver, "auto");
        assert!(c.require_confirmation);
        assert!(c.color);
        assert!(c.mount_options.is_empty());
    }

    #[test]
    fn round_trip_persists_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        let mut cfg = Config::default();
        cfg.mount_base = "/tmp/vol".into();
        cfg.mount_options.push("noowners".into());
        cfg.require_confirmation = false;
        save_config(&cfg, &path).unwrap();
        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.mount_base, "/tmp/vol");
        assert_eq!(loaded.mount_options, vec!["noowners".to_string()]);
        assert!(!loaded.require_confirmation);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let cfg = load_config(std::path::Path::new("/nonexistent/path/config.toml")).unwrap();
        assert_eq!(cfg.mount_base, "/Volumes");
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            mount_base = "/Volumes"
            totally_unknown_field = 42
            "#,
        )
        .unwrap();
        let cfg = load_config(&path).unwrap();
        assert_eq!(cfg.mount_base, "/Volumes");
    }
}
