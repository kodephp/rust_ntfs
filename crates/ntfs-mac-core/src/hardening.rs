// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! Production hardening: panic hook, signal handling, input validation, TTY detection.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{Error, Result};

static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);

pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info.location().map(|l| l.to_string()).unwrap_or_default();
        let message = panic_message(info);
        let backtrace = std::backtrace::Backtrace::force_capture().to_string();

        let report = format!(
            "\n=== ntfs-mac panic ===\nlocation: {location}\nmessage: {message}\nbacktrace:\n{backtrace}\n=== end of report ===\n"
        );

        eprintln!("{report}");

        if let Some(log_path) = panic_log_path() {
            let _ = std::fs::create_dir_all(log_path.parent().unwrap_or(std::path::Path::new(".")));
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
            {
                use std::io::Write;
                let _ = f.write_all(report.as_bytes());
            }
        }

        default_hook(info);
    }));
}

fn panic_message(info: &std::panic::PanicHookInfo<'_>) -> String {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn panic_log_path() -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join("Library")
            .join("Logs")
            .join("ntfs-mac.panics.log"),
    )
}

pub fn install_signal_handlers() {
    #[cfg(unix)]
    {
        use signal_hook::consts::signal::{SIGINT, SIGTERM};
        use signal_hook::iterator::Signals;

        let mut signals = match Signals::new([SIGINT, SIGTERM]) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("failed to install signal handlers: {e}");
                return;
            }
        };

        std::thread::spawn(move || {
            for _ in signals.forever() {
                SIGNAL_RECEIVED.store(true, Ordering::Relaxed);
            }
        });
    }
}

pub fn should_exit() -> bool {
    SIGNAL_RECEIVED.load(Ordering::Relaxed)
}

pub fn is_stdin_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

pub fn is_stdout_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

pub fn validate_device_id(id: &str) -> Result<&str> {
    let id = id.trim();
    if id.is_empty() {
        return Err(Error::InvalidArgument("device id cannot be empty".into()));
    }
    if id.contains('/') || id.contains('\\') || id.contains(' ') {
        return Err(Error::InvalidArgument(format!(
            "device id '{id}' contains invalid characters"
        )));
    }

    let stripped = id.strip_prefix("disk").ok_or_else(|| {
        Error::InvalidArgument(format!("device id '{id}' must start with 'disk'"))
    })?;
    if stripped.is_empty() {
        return Err(Error::InvalidArgument(format!(
            "device id '{id}' has no disk number"
        )));
    }

    let (disk_num, partition) = match stripped.find('s') {
        Some(idx) => (&stripped[..idx], Some(&stripped[idx + 1..])),
        None => (stripped, None),
    };
    if disk_num.is_empty() || !disk_num.chars().all(|c| c.is_ascii_digit()) {
        return Err(Error::InvalidArgument(format!(
            "device id '{id}' has invalid disk number '{disk_num}'"
        )));
    }
    if let Some(p) = partition {
        if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::InvalidArgument(format!(
                "device id '{id}' has invalid partition number"
            )));
        }
    }
    Ok(id)
}

pub fn validate_mount_point(mp: &str) -> Result<()> {
    if mp.is_empty() {
        return Err(Error::InvalidArgument("mount point cannot be empty".into()));
    }
    if !mp.starts_with('/') {
        return Err(Error::InvalidArgument(format!(
            "mount point '{mp}' must be an absolute path"
        )));
    }
    if mp.contains('\0') || mp.contains('\n') {
        return Err(Error::InvalidArgument(format!(
            "mount point '{mp}' contains invalid characters"
        )));
    }
    Ok(())
}

pub fn validate_label(label: &str) -> Result<()> {
    if label.is_empty() {
        return Err(Error::InvalidArgument("label cannot be empty".into()));
    }
    if label.len() > 32 {
        return Err(Error::InvalidArgument(format!(
            "label '{label}' is too long (max 32 chars)"
        )));
    }
    if label.contains('/') || label.contains('\\') || label.contains('\0') {
        return Err(Error::InvalidArgument(format!(
            "label '{label}' contains invalid characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_device_id_accepts_valid() {
        assert!(validate_device_id("disk2s2").is_ok());
        assert!(validate_device_id("disk0").is_ok());
        assert!(validate_device_id("disk10s3").is_ok());
    }

    #[test]
    fn validate_device_id_rejects_invalid() {
        assert!(validate_device_id("").is_err());
        assert!(validate_device_id("notadisk").is_err());
        assert!(validate_device_id("disk").is_err());
        assert!(validate_device_id("disk/2s2").is_err());
        assert!(validate_device_id("disk 2s2").is_err());
        assert!(validate_device_id("diskas2").is_err());
        assert!(validate_device_id("/dev/disk2s2").is_err());
        assert!(validate_device_id("disk2s").is_err());
    }

    #[test]
    fn validate_mount_point_accepts_absolute() {
        assert!(validate_mount_point("/Volumes/MyDisk").is_ok());
        assert!(validate_mount_point("/tmp/mount").is_ok());
    }

    #[test]
    fn validate_mount_point_rejects_relative() {
        assert!(validate_mount_point("").is_err());
        assert!(validate_mount_point("Volumes/MyDisk").is_err());
        assert!(validate_mount_point("relative").is_err());
    }

    #[test]
    fn validate_label_accepts_valid() {
        assert!(validate_label("MyData").is_ok());
        assert!(validate_label("a").is_ok());
        assert!(validate_label("very-long-label-here").is_ok());
    }

    #[test]
    fn validate_label_rejects_invalid() {
        assert!(validate_label("").is_err());
        assert!(validate_label("a/b").is_err());
        assert!(validate_label("a\\b").is_err());
        assert!(validate_label(&"a".repeat(33)).is_err());
    }

    #[test]
    fn signal_flag_defaults_false() {
        assert!(!should_exit());
    }

    #[test]
    fn panic_hook_installs() {
        install_panic_hook();
    }
}
