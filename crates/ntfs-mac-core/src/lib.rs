//! ntfs-mac core.
//!
//! Subprocess wrappers around `ntfs-3g`, `diskutil`, `hdiutil`,
//! `newfs_ntfs`, `ntfsfix` and `fsck_ntfs`, plus a small config layer.
//!
//! Design rules (do not violate — they are the reason this library is
//! kept under ~1.5k LOC):
//!
//! * No `unsafe`, no `unwrap` on production paths, no async runtime.
//! * Every external command goes through [`runner::run`]; failures are
//!   surfaced as [`error::Error`] with the underlying stderr attached.
//! * Pure parsing helpers are pure and unit-testable; only the module
//!   boundaries that touch `std::process::Command` are impure.
//! * Destructive NTFS operations require an explicit [`DestructiveToken`]
//!   to be matched before the API proceeds.
//!
//! Layout:
//!
//! ```text
//! ntfs-mac-core
//!   runner   Subprocess plumbing + timeouts
//!   error    Typed errors, ExitStatus codes
//!   config   ~/.config/ntfs-mac/config.toml
//!   deps     Dependency discovery (ntfs-3g, macFUSE, FUSE-T, ...)
//!   device   diskutil list -plist parsing
//!   mount    ntfs-3g / diskutil mount and unmount
//!   format   mkntfs / newfs_ntfs (destructive, gated)
//!   fix      ntfsfix / fsck_ntfs
//!   copy     rsync-based copy with progress
//! ```

pub mod config;
pub mod copy;
pub mod daemon;
pub mod deps;
pub mod device;
pub mod error;
pub mod fix;
pub mod format;
pub mod hardening;
pub mod mount;
pub mod runner;

pub use config::{Config, default_config_path, load_config, save_config};
pub use device::Volume;
pub use error::{ConfigError, Error, ExitCode, Result};
pub use hardening::{
    install_panic_hook, install_signal_handlers, is_stdin_tty, is_stdout_tty, should_exit,
    validate_device_id, validate_label, validate_mount_point,
};
pub use runner::{RunOptions, read_line_interactive, run_expect_success, which};

/// Build-time constants surfaced to the CLI/GUI.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "ntfs-mac";

/// Destructive-operation token. Callers that confirm a `format` or
/// `erase` action must pass a token whose `device_identifier` field
/// matches the target device exactly.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DestructiveToken {
    /// Device identifier such as `disk2s2`.
    pub device_identifier: String,
    /// Volume label, when available. Empty string if the volume is unlabelled.
    pub volume_label: String,
}

impl DestructiveToken {
    /// Build a token for the given device + label.
    pub fn new(device_identifier: impl Into<String>, volume_label: impl Into<String>) -> Self {
        Self {
            device_identifier: device_identifier.into(),
            volume_label: volume_label.into(),
        }
    }

    /// Human-readable confirmation phrase. Used in CLI prompts.
    pub fn confirm_phrase(&self) -> String {
        if self.volume_label.is_empty() {
            format!("format {}", self.device_identifier)
        } else {
            format!("format {} {}", self.device_identifier, self.volume_label)
        }
    }

    /// Validate that this token is usable as a confirmation: identifier
    /// must be non-empty.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.device_identifier.trim().is_empty() {
            return Err(Error::InvalidArgument(
                "device_identifier cannot be empty".into(),
            ));
        }
        Ok(())
    }
}
