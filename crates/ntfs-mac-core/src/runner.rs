// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! Small subprocess runner.
//!
//! Wraps `std::process::Command` with:
//!
//! * A `timeout` guard that SIGTERMs then SIGKILLs on overrun.
//! * Preserved stderr so [`crate::Error::CommandFailed`] can surface the
//!   underlying tool's own diagnostics.
//! * `CommandRunner` trait so tests can inject a fake binary (see
//!   `#[cfg(test)]` in each module).

use std::io::{BufRead, Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

/// Status reported for a subprocess that was killed by its timeout.
/// Exposed so callers and tests can branch on it without magic numbers.
pub const TIMEOUT_STATUS: i32 = 150;

/// Grace given after SIGTERM before escalating to SIGKILL.
const SIGTERM_GRACE: Duration = Duration::from_secs(1);

/// Poll interval used while racing against a deadline.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Upper bound on waiting for the stdout/stderr readers to flush.
const READ_DRAIN_TIMEOUT: Duration = Duration::from_millis(500);

/// Output of a completed subprocess.
#[derive(Debug, Default)]
pub struct RunResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    /// `true` when the configured timeout fired and the child was
    /// terminated. `status` is then [`TIMEOUT_STATUS`].
    pub timed_out: bool,
}

impl RunResult {
    pub fn success(&self) -> bool {
        self.status == 0
    }
}

/// Configuration for a single subprocess.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Hard timeout. `None` disables the timeout.
    pub timeout: Option<Duration>,
    /// Additional environment variables (additive).
    pub env_extra: Vec<(String, String)>,
    /// Working directory override.
    pub cwd: Option<std::path::PathBuf>,
    /// Capture stdout/stderr? Defaults to `true`. When `false`, the
    /// child inherits the parent's stdio (progress displays stay visible).
    pub capture: bool,
    /// Stream stdin (only meaningful when `capture` is `false`).
    pub stdin_data: Option<String>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            env_extra: Vec::new(),
            cwd: None,
            // Manual Default (not derive): a bool derives to `false`,
            // but capturing is the long-standing default behaviour —
            // error reporting depends on the captured stderr.
            capture: true,
            stdin_data: None,
        }
    }
}

/// Runs a command with [`RunOptions`] and returns [`RunResult`].
///
/// On timeout the child is SIGTERMed and, if it does not exit within
/// [`SIGTERM_GRACE`], SIGKILLed. `run` itself never fails on timeout:
/// it returns [`RunResult`] with [`RunResult::timed_out`] set and
/// [`RunResult::status`] = [`TIMEOUT_STATUS`]. [`run_expect_success`]
/// turns that into [`Error::CommandTimedOut`].
pub fn run(cmd: &str, args: &[&str], opts: &RunOptions) -> Result<RunResult> {
    let started = Instant::now();

    let mut command = Command::new(cmd);
    command.args(args).env_remove("TERM"); // Prevent tools from emitting escape codes we cannot parse.
    if opts.capture {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    } else {
        // Inherit mode: the child's output goes straight to the parent's
        // terminals (used for long-running tools with progress output
        // such as `rsync --progress`).
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    for (k, v) in &opts.env_extra {
        command.env(k, v);
    }
    if let Some(cwd) = &opts.cwd {
        command.current_dir(cwd);
    }

    let mut child = command.spawn().map_err(|io| Error::CommandFailed {
        cmd: cmd.to_string(),
        status: -1,
        stderr: String::new(),
        io: Some(io),
    })?;

    // Feed stdin if provided.
    if let Some(data) = &opts.stdin_data {
        use std::io::Write;
        let _ = child.stdin.as_mut().map(|w| w.write_all(data.as_bytes()));
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.flush();
        }
    } else {
        child.stdin.take();
    }

    // Collect stdout/stderr in background threads so we can race against the timeout.
    let stdout_buf = Arc::new(Mutex::new(Vec::new()));
    let stderr_buf = Arc::new(Mutex::new(Vec::new()));

    let stdout_arc = stdout_buf.clone();
    // Take stdout and stderr handles before moving them into threads,
    // so `child` remains fully owned for `try_wait`/`wait`.
    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();

    let stdout_thread = std::thread::spawn(move || {
        if let Some(mut handle) = stdout_handle {
            let mut out = Vec::new();
            let _ = handle.read_to_end(&mut out);
            // A poisoned mutex still holds valid data (the panic happened
            // elsewhere); recovering it is strictly better than panicking.
            let _ = stdout_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .write_all(&out);
        }
    });

    let stderr_arc = stderr_buf.clone();
    let stderr_thread = std::thread::spawn(move || {
        if let Some(mut handle) = stderr_handle {
            let mut err = Vec::new();
            let _ = handle.read_to_end(&mut err);
            let _ = stderr_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .write_all(&err);
        }
    });

    // Wait with timeout.
    let mut waited_status = None;
    let mut timed_out = false;

    if let Some(timeout) = opts.timeout {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(Some(status)) = child.try_wait() {
                waited_status = Some(status);
                break;
            }
            if Instant::now() >= deadline {
                timed_out = true;
                break;
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    if timed_out {
        terminate_gracefully(&mut child);
        waited_status = Some(
            child
                .wait()
                .unwrap_or_else(|_| std::process::ExitStatus::from_raw(137)),
        );
    } else if waited_status.is_none() {
        // Wait unbounded if no timeout.
        waited_status = Some(
            child
                .wait()
                .unwrap_or_else(|_| std::process::ExitStatus::from_raw(137)),
        );
    }

    // Drain whatever the child produced, but never let the drain itself
    // blow past the timeout. `join()` below would otherwise block for as
    // long as *any* descendant holds a pipe write-end: a timed-out
    // `brew install` leaves `curl`/`tar` behind, and they keep the pipe
    // open long after we killed `brew`. The child is already dead or
    // dying at this point, so returning with a partial capture is the
    // correct behaviour — the reader threads keep running detached and
    // finish on their own when the orphan eventually exits.
    let drain_deadline = Instant::now() + READ_DRAIN_TIMEOUT;
    while !stdout_thread.is_finished() || !stderr_thread.is_finished() {
        if Instant::now() >= drain_deadline {
            break;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    if stdout_thread.is_finished() {
        let _ = stdout_thread.join();
    }
    if stderr_thread.is_finished() {
        let _ = stderr_thread.join();
    }

    let status = match waited_status {
        Some(_s) if timed_out => TIMEOUT_STATUS,
        Some(s) if s.success() => 0,
        Some(s) => s.code().unwrap_or(-1),
        None => -1,
    };

    let stdout = String::from_utf8_lossy(
        &stdout_buf
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    )
    .to_string();
    let stderr = String::from_utf8_lossy(
        &stderr_buf
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    )
    .to_string();

    Ok(RunResult {
        status,
        stdout,
        stderr,
        duration: started.elapsed(),
        timed_out,
    })
}

/// Terminate a timed-out child: SIGTERM first, escalating to SIGKILL
/// only if it is still alive after [`SIGTERM_GRACE`].
///
/// The tools this runner supervises hold filesystem state while they run
/// (`fsck_ntfs` checks, `newfs_ntfs` formats, `rsync`/`cp` transfer up
/// to an hour of data), so an unannounced SIGKILL can leave a volume's
/// journal or a transfer in a worse state than a clean shutdown. This
/// makes the module-level contract — "SIGTERMs then SIGKILLs" — true
/// instead of decorative.
fn terminate_gracefully(child: &mut Child) {
    let pid = child.id() as libc::pid_t;
    // SAFETY: `pid` is the pid of a child we spawned moments ago and is
    // still alive (it just failed `try_wait` above), so it cannot have
    // been reaped or recycled by another process into an unrelated task.
    // `SIGTERM` is a valid signal; sending it cannot invalidate any of
    // our own allocations because `child` still owns the process handle.
    // `libc::kill` is `unsafe` solely because signalling an arbitrary
    // pid of unknown ownership is UB-adjacent — both preconditions here
    // are established by construction.
    let _ = unsafe { libc::kill(pid, libc::SIGTERM) };

    let grace_deadline = Instant::now() + SIGTERM_GRACE;
    while Instant::now() < grace_deadline {
        match child.try_wait() {
            // The child honoured SIGTERM; it will not be SIGKILLed.
            Ok(Some(_)) => return,
            Ok(None) => std::thread::sleep(POLL_INTERVAL),
            // A wait error means there is no child left to signal.
            Err(_) => return,
        }
    }

    let _ = child.kill();
}

/// Convenience helper: run a command that should succeed, otherwise
/// wrap into [`Error::CommandFailed`].
pub fn run_expect_success(cmd: &str, args: &[&str], opts: &RunOptions) -> Result<RunResult> {
    let out = run(cmd, args, opts)?;
    if out.timed_out {
        return Err(Error::CommandTimedOut {
            cmd: cmd.to_string(),
            timeout: opts.timeout.unwrap_or_default(),
        });
    }
    if out.success() {
        Ok(out)
    } else {
        Err(Error::CommandFailed {
            cmd: cmd.to_string(),
            status: out.status,
            stderr: if out.stderr.trim().is_empty() {
                String::from("<empty stderr>")
            } else {
                out.stderr.trim().to_string()
            },
            io: None,
        })
    }
}

/// Locate a binary on `PATH`. Returns the full path or
/// [`Error::MissingDependency`].
pub fn which(binary: &str) -> Result<std::path::PathBuf> {
    let path_env = std::env::var_os("PATH").ok_or_else(|| Error::MissingDependency {
        binary: binary.to_string(),
        detail: "PATH environment variable is not set".into(),
        hint: Some("Set PATH to include the directory containing the tool.".into()),
        io: None,
    })?;

    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            // Check executability.
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(&candidate)?;
            if meta.permissions().mode() & 0o111 != 0 {
                return Ok(candidate);
            }
        }
    }

    Err(Error::MissingDependency {
        binary: binary.to_string(),
        detail: format!("`{binary}` not found on PATH"),
        hint: Some(format!(
            "Install via `brew install {binary}` (or run `./scripts/install.sh`)."
        )),
        io: None,
    })
}

/// Read stdin from an interactive TTY, returning a trimmed line.
///
/// Used for the CLI confirmation flow. `prompt` is echoed to stderr
/// so stdout stays clean for JSON mode.
pub fn read_line_interactive(prompt: &str) -> Result<String> {
    eprintln!("{prompt}");
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).is_err() {
        return Err(Error::Cancelled);
    }
    Ok(line.trim().to_string())
}
