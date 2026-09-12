//! Typed errors + POSIX-style exit codes used by the CLI/GUI layer.

use std::path::PathBuf;

use thiserror::Error;

/// Exit code surface for the CLI. Kept small and stable so scripts can
/// branch on it without parsing stderr.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    /// Command succeeded.
    Ok = 0,
    /// Generic failure (I/O, unexpected stderr).
    Failure = 1,
    /// Missing or unsatisfiable dependency (`ntfs-3g`, macFUSE/FUSE-T).
    MissingDependency = 2,
    /// User declined an interactive prompt (Ctrl-C, "no" to confirm).
    UserCancelled = 3,
    /// Confirmation phrase did not match.
    ConfirmationMismatch = 4,
    /// Input/argument validation error (unknown device, invalid label).
    InvalidArgument = 5,
}

impl From<ExitCode> for u8 {
    fn from(e: ExitCode) -> Self {
        e as u8
    }
}

/// Library error type. Every variant carries enough context that a
/// user-facing message can be derived without re-running the failing
/// command.
#[derive(Debug, Error)]
pub enum Error {
    /// A subprocess exited non-zero. `stderr` is preserved verbatim
    /// so the caller can show the underlying tool's diagnosis.
    #[error("command `{cmd}` exited with status {status}: {stderr}")]
    CommandFailed {
        cmd: String,
        status: i32,
        stderr: String,
        #[source]
        io: Option<std::io::Error>,
    },

    /// The target binary is not on PATH. `hint` is a copy-pastable
    /// fix instruction.
    #[error("missing dependency `{binary}`: {detail}")]
    MissingDependency {
        binary: String,
        detail: String,
        hint: Option<String>,
        #[source]
        io: Option<std::io::Error>,
    },

    /// Config file could not be read or written.
    #[error("config error: {0}")]
    Config(#[from] ConfigError),

    /// The parsed `diskutil` output did not contain an NTFS volume
    /// matching the user's request.
    #[error("no matching NTFS volume for `{pattern}`")]
    NoMatch { pattern: String },

    /// Confirmation phrase did not match.
    #[error("confirmation mismatch: expected `{expected}`, got `{actual}`")]
    ConfirmationMismatch { expected: String, actual: String },

    /// User pressed Ctrl-C or answered "no" to a confirmation prompt.
    #[error("user cancelled")]
    Cancelled,

    /// Argument validation failure (empty label, unknown device, ...).
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Underlying I/O error that does not fit into a more specific
    /// variant. Kept narrow so `?` from `std::fs::read_to_string` and
    /// the like still works.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization / parsing error (plist, toml, json).
    #[error("serialization error: {0}")]
    Serde(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}: {reason}")]
    Read { path: PathBuf, reason: String },
    #[error("failed to write config to {path}: {reason}")]
    Write { path: PathBuf, reason: String },
    #[error("config parse error at {path}: {reason}")]
    Parse { path: PathBuf, reason: String },
}

impl From<ConfigError> for std::io::Error {
    fn from(e: ConfigError) -> Self {
        std::io::Error::other(e.to_string())
    }
}

/// Library result alias.
pub type Result<T> = std::result::Result<T, Error>;
