// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! Integration tests for ntfs-mac-core.
//!
//! These tests exercise the public API of the library, including
//! subprocess plumbing, config persistence, error conversion, and
//! the typed error surface that the CLI/GUI layers depend on.
//!
//! Unlike unit tests (which live inside each module), integration
//! tests here call through the public `ntfs_mac_core::` namespace
//! to catch regressions at the crate boundary.

use std::time::Duration;

use ntfs_mac_core::{
    Config, Error, ExitCode, RunOptions, config, device, error, runner, validate_device_id,
    validate_label, validate_mount_point,
};

// ---------------------------------------------------------------------------
// Subprocess runner
// ---------------------------------------------------------------------------

#[test]
fn runner_returns_zero_for_echo() {
    let out = runner::run("echo", &["hello"], &RunOptions::default()).unwrap();
    assert!(out.success());
    assert_eq!(out.stdout.trim(), "hello");
}

#[test]
fn runner_returns_nonzero_for_false() {
    let out = runner::run("false", &[], &RunOptions::default()).unwrap();
    assert!(!out.success());
    assert!(out.status > 0);
}

#[test]
fn runner_captures_stderr() {
    let out = runner::run(
        "sh",
        &["-c", "echo err 1>&2; echo ok"],
        &RunOptions::default(),
    )
    .unwrap();
    assert!(out.success());
    assert!(out.stderr.contains("err"));
}

#[test]
fn runner_timeout_kills_long_process() {
    let out = runner::run(
        "sleep",
        &["60"],
        &RunOptions {
            timeout: Some(Duration::from_millis(500)),
            ..Default::default()
        },
    )
    .unwrap();
    // The timeout sentinel is exported, not a magic literal.
    assert_eq!(out.status, runner::TIMEOUT_STATUS);
    assert!(
        out.timed_out,
        "a killed child must flag itself as timed out"
    );
}

#[test]
fn runner_timeout_flags_are_cleared_on_success() {
    let out = runner::run("echo", &["hello"], &RunOptions::default()).unwrap();
    assert!(out.success());
    assert!(!out.timed_out);
}

/// The module documents SIGTERM-before-SIGKILL, and the disk tools this
/// runner supervises (`fsck_ntfs`, `newfs_ntfs`, `rsync`, `cp`) hold
/// filesystem state while running — so verify the SIGTERM actually
/// reaches the child. A shell that traps TERM creates a marker file;
/// if we had jumped straight to SIGKILL the marker would never appear.
///
/// The body is a foreground busy loop on purpose: backgrounding a child
/// would orphan it, and an orphan holding the pipe write-ends is what
/// [`runner_timeout_survives_orphaned_grandchildren`] covers separately.
#[test]
fn runner_timeout_sends_sigterm_before_sigkill() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("got-term");
    let script = format!(
        "trap 'touch {}' TERM; while :; do :; done",
        marker.display()
    );
    let out = runner::run(
        "sh",
        &["-c", &script],
        &RunOptions {
            timeout: Some(Duration::from_millis(300)),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.timed_out);
    assert_eq!(out.status, runner::TIMEOUT_STATUS);
    assert!(
        marker.exists(),
        "the child never received SIGTERM, so its trap could not have run"
    );
}

/// A timed-out command that spawns its own children used to hang `run()`
/// until the orphan exited: the orphan inherits the pipe write-ends, so
/// an unconditional `join()` blocked on EOF that never arrives. A real
/// case is `install_dependencies` timing out mid-`brew install` with
/// `curl`/`tar` still running. The timeout ceiling must hold regardless.
#[test]
fn runner_timeout_survives_orphaned_grandchildren() {
    let started = std::time::Instant::now();
    let out = runner::run(
        "sh",
        &["-c", "sleep 30 & wait"],
        &RunOptions {
            timeout: Some(Duration::from_millis(200)),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.timed_out);
    assert_eq!(out.status, runner::TIMEOUT_STATUS);
    // timeout (200ms) + SIGTERM grace (1s) + drain bound (500ms).
    let total = started.elapsed();
    assert!(
        total < Duration::from_secs(10),
        "run() was held hostage by the orphan: {total:?}"
    );
}

#[test]
fn run_expect_success_maps_timeout_to_command_timed_out() {
    let limit = Duration::from_millis(300);
    let err = runner::run_expect_success(
        "sleep",
        &["60"],
        &RunOptions {
            timeout: Some(limit),
            ..Default::default()
        },
    )
    .unwrap_err();
    match err {
        Error::CommandTimedOut { cmd, timeout } => {
            assert_eq!(cmd, "sleep");
            assert_eq!(timeout, limit);
        }
        other => panic!("expected CommandTimedOut, got {other:?}"),
    }
}

#[test]
fn timed_out_error_does_not_leak_the_internal_sentinel() {
    let err = Error::CommandTimedOut {
        cmd: "fsck_ntfs".into(),
        timeout: Duration::from_secs(30),
    };
    let msg = err.to_string();
    assert!(msg.contains("fsck_ntfs"), "{msg}");
    assert!(msg.contains("timed out"), "{msg}");
    assert!(
        !msg.contains("150"),
        "the internal status sentinel must not reach the user: {msg}"
    );
}

#[test]
fn runner_missing_binary_returns_command_failed() {
    let err = runner::run(
        "definitely-not-a-real-binary-xyz",
        &[],
        &RunOptions::default(),
    )
    .unwrap_err();
    match err {
        Error::CommandFailed { status, .. } => {
            assert!(status < 0); // spawn failure
        }
        _ => panic!("expected CommandFailed"),
    }
}

#[test]
fn which_returns_path_for_echo() {
    let p = runner::which("echo").unwrap();
    assert!(p.is_absolute());
    assert!(p.exists());
}

#[test]
fn which_missing_binary_errors() {
    let err = runner::which("definitely-not-a-real-binary-xyz").unwrap_err();
    match err {
        Error::MissingDependency { binary, .. } => {
            assert!(binary.contains("definitely-not-a-real-binary"));
        }
        _ => panic!("expected MissingDependency"),
    }
}

// ---------------------------------------------------------------------------
// Config persistence
// ---------------------------------------------------------------------------

#[test]
fn config_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.toml");

    let cfg = Config {
        mount_base: "/Volumes/Backup".into(),
        require_confirmation: false,
        mount_options: vec!["noowners".into(), "uid=501".into()],
        ..Default::default()
    };
    config::save_config(&cfg, &path).unwrap();

    let loaded = config::load_config(&path).unwrap();
    assert_eq!(loaded.mount_base, "/Volumes/Backup");
    assert!(!loaded.require_confirmation);
    assert_eq!(loaded.mount_options.len(), 2);
}

#[test]
fn config_missing_file_returns_default() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("does-not-exist.toml");
    // load_config should create the parent and return defaults.
    let cfg = config::load_config(&path).unwrap();
    assert_eq!(cfg.mount_base, Config::default().mount_base);
}

#[test]
fn config_parse_error_on_garbage() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("garbage.toml");
    std::fs::write(&path, "this is not [valid toml at all").unwrap();
    let err = config::load_config(&path).unwrap_err();
    match err {
        Error::Serde(msg) => {
            assert!(
                msg.contains("toml") || msg.contains("expected"),
                "unexpected: {msg}"
            );
        }
        other => panic!("expected Serde error, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Device parsing
// ---------------------------------------------------------------------------

#[test]
fn device_parse_empty_plist() {
    // The parser should handle an empty plist without panicking.
    let err = device::parse_diskutil_plist("[]").unwrap_err();
    // Empty plist is not a valid NTFS volumes list.
    let _ = err; // Just verify it doesn't panic.
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[test]
fn validate_device_id_full_matrix() {
    // Valid
    for id in ["disk0", "disk2s2", "disk10s3", "disk100s200"] {
        assert!(validate_device_id(id).is_ok(), "{id} should be valid");
    }
    // Invalid
    for id in [
        "",           // empty
        "disk",       // no number
        "disk0s",     // empty partition
        "disk0sabc",  // non-numeric partition
        "notadisk",   // wrong prefix
        "disk 0",     // space
        "/dev/disk0", // absolute path
        "disk0/2",    // slash
    ] {
        assert!(validate_device_id(id).is_err(), "{id} should be invalid");
    }
}

#[test]
fn validate_mount_point_full_matrix() {
    // Valid
    for mp in ["/Volumes/MyDisk", "/tmp/mount", "/"] {
        assert!(validate_mount_point(mp).is_ok(), "{mp} should be valid");
    }
    // Invalid
    for mp in ["", "relative", "relative/path", "/path/\n/newline"] {
        assert!(
            validate_mount_point(mp).is_err(),
            "{mp:?} should be invalid"
        );
    }
}

#[test]
fn validate_label_full_matrix() {
    // Valid
    for label in ["A", "MyData"] {
        assert!(validate_label(label).is_ok(), "{label} should be valid");
    }
    let long_label = "a".repeat(32);
    assert!(
        validate_label(&long_label).is_ok(),
        "32-char label should be valid"
    );
    // Invalid
    assert!(validate_label("").is_err());
    assert!(validate_label("a/b").is_err());
    assert!(validate_label("a\\b").is_err());
    let too_long = "a".repeat(33);
    assert!(validate_label(&too_long).is_err());
}

// ---------------------------------------------------------------------------
// Error conversions
// ---------------------------------------------------------------------------

#[test]
fn exit_code_conversions() {
    assert_eq!(u8::from(ExitCode::Ok), 0);
    assert_eq!(u8::from(ExitCode::Failure), 1);
    assert_eq!(u8::from(ExitCode::MissingDependency), 2);
    assert_eq!(u8::from(ExitCode::UserCancelled), 3);
    assert_eq!(u8::from(ExitCode::ConfirmationMismatch), 4);
    assert_eq!(u8::from(ExitCode::InvalidArgument), 5);
}

#[test]
fn error_display_contains_context() {
    let err = Error::InvalidArgument("bad device".into());
    assert!(err.to_string().contains("bad device"));

    let err = Error::Cancelled;
    assert!(err.to_string().contains("cancel"));

    let err = Error::NoMatch {
        pattern: "disk9999".into(),
    };
    assert!(err.to_string().contains("disk9999"));
}

#[test]
fn error_io_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
}

// ---------------------------------------------------------------------------
// Result alias
// ---------------------------------------------------------------------------

#[test]
fn result_alias_is_stdout() {
    // Just verify the type alias is usable.
    fn foo() -> error::Result<i32> {
        Ok(42)
    }
    assert_eq!(foo().unwrap(), 42);
}
