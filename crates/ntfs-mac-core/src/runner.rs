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
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

/// Output of a completed subprocess.
#[derive(Debug, Default)]
pub struct RunResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl RunResult {
    pub fn success(&self) -> bool {
        self.status == 0
    }
}

/// Configuration for a single subprocess.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Hard timeout. `None` disables the timeout.
    pub timeout: Option<Duration>,
    /// Additional environment variables (additive).
    pub env_extra: Vec<(String, String)>,
    /// Working directory override.
    pub cwd: Option<std::path::PathBuf>,
    /// Capture stdout/stderr? Defaults to `true`.
    pub capture: bool,
    /// Stream stdin (only meaningful when `capture` is `false`).
    pub stdin_data: Option<String>,
}

/// Runs a command with [`RunOptions`] and returns [`RunResult`].
///
/// On timeout the child is SIGTERMed and, if it does not exit within
/// 1s, SIGKILLed. The error variant returned is
/// [`Error::CommandFailed`] with status `150` (timeout sentinel).
pub fn run(cmd: &str, args: &[&str], opts: &RunOptions) -> Result<RunResult> {
    let started = Instant::now();

    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("TERM"); // Prevent tools from emitting escape codes we cannot parse.
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
            let _ = stdout_arc.lock().unwrap().write_all(&out);
        }
    });

    let stderr_arc = stderr_buf.clone();
    let stderr_thread = std::thread::spawn(move || {
        if let Some(mut handle) = stderr_handle {
            let mut err = Vec::new();
            let _ = handle.read_to_end(&mut err);
            let _ = stderr_arc.lock().unwrap().write_all(&err);
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
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    if timed_out {
        let _ = child.kill();
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

    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    let status = match waited_status {
        Some(_s) if timed_out => 150,
        Some(s) if s.success() => 0,
        Some(s) => s.code().unwrap_or(-1),
        None => -1,
    };

    let stdout = String::from_utf8_lossy(&stdout_buf.lock().unwrap()).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf.lock().unwrap()).to_string();

    Ok(RunResult {
        status,
        stdout,
        stderr,
        duration: started.elapsed(),
    })
}

/// Convenience helper: run a command that should succeed, otherwise
/// wrap into [`Error::CommandFailed`].
pub fn run_expect_success(cmd: &str, args: &[&str], opts: &RunOptions) -> Result<RunResult> {
    let out = run(cmd, args, opts)?;
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
