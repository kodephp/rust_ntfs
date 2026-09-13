// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;
use ntfs_mac_core::{
    Config, DestructiveToken, copy, daemon, default_config_path, deps, device, fix, format,
    license, mount, read_line_interactive, validate_device_id, validate_mount_point,
};

/// Shown by `--version` (the short `-V` keeps just the number). Built with
/// `concat!` so it stays a `&'static str`; it deliberately does not repeat
/// the licence text, which lives in `ntfs-mac license`.
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\nRun `ntfs-mac license` for licence, NOTICE and third-party attribution details."
);

#[derive(Parser, Debug)]
#[command(
    name = "ntfs-mac",
    version,
    long_version = LONG_VERSION,
    about = "NTFS toolkit for macOS",
    after_help = "Run `ntfs-mac license` for licence details.\nShell completions: `ntfs-mac completions <shell>`.",
    propagate_version = true
)]
struct Cli {
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    #[arg(short, long, global = true)]
    json: bool,
    #[arg(short, long, global = true)]
    verbose: bool,
    #[arg(long, global = true)]
    no_color: bool,
    /// Log level: error, warn, info, debug, trace (default: warn)
    #[arg(long, global = true, default_value = "warn")]
    log_level: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Check dependencies and system readiness
    Doctor,
    /// List NTFS volumes
    List,
    /// Mount an NTFS volume (use "all" to mount all unmounted)
    Mount {
        /// Device identifier, volume name, or "all"
        target: String,
        #[arg(long)]
        mount_point: Option<String>,
        #[arg(short = 'r', long)]
        readonly: bool,
        #[arg(long)]
        force_ntfs3g: bool,
    },
    /// Unmount an NTFS volume
    Unmount { target: String },
    /// Switch a mounted volume from read-only to read-write
    Rw { target: String },
    /// Show current mount status summary
    Status,
    /// Auto-mount daemon (watch for new NTFS volumes)
    Daemon {
        /// Install as LaunchAgent (auto-start on login)
        #[arg(long)]
        install: bool,
        /// Uninstall LaunchAgent
        #[arg(long)]
        uninstall: bool,
        /// Show daemon status
        #[arg(long)]
        status: bool,
        /// Poll interval in seconds
        #[arg(long, default_value = "3")]
        interval: u64,
    },
    /// Format a volume as NTFS (destructive)
    Format {
        target: String,
        #[arg(long)]
        label: Option<String>,
        /// Full format: zero the volume and scan for bad sectors
        /// (much slower). Default is a quick format.
        #[arg(long)]
        full: bool,
        /// Sector size in bytes (512 or 4096)
        #[arg(long, default_value_t = 4096u32)]
        sector_size: u32,
        /// Cluster size in bytes (512–65536, power of 2)
        #[arg(long, default_value_t = 4096u32)]
        cluster_size: u32,
        #[arg(long)]
        yes: bool,
    },
    /// Fix NTFS volume
    Fix {
        target: String,
        #[arg(long)]
        fsck: bool,
    },
    /// Copy files to/from NTFS volume
    Copy {
        source: PathBuf,
        destination: PathBuf,
        #[arg(short, long)]
        delete: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Show sponsor / donation QR code location
    Sponsor {
        /// Reveal the QR code file in Finder
        #[arg(long)]
        reveal: bool,
    },
    /// Show licence, NOTICE and third-party attribution details
    License {
        /// Print the complete Apache-2.0 licence text
        #[arg(long)]
        full: bool,
        /// Print the embedded third-party crate inventory
        #[arg(long)]
        third_party: bool,
    },
    /// Generate a shell completion script on stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    Show,
    Set { key: String, value: String },
    Reset,
    Path,
}

fn main() -> ExitCode {
    // Install panic hook before anything else so we capture trace even
    // if clap parsing panics.
    ntfs_mac_core::install_panic_hook();

    let cli = Cli::parse();
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Configure structured logging. Writes to stderr (so stdout stays
    // clean for JSON mode) + a rotating log file.
    init_logging(&cli.log_level);

    match run(&cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "command failed");
            if cli.json {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&e.to_string()).unwrap_or_default()
                );
            } else {
                eprintln!("{} {}", "error:".red().bold(), e);
            }
            ExitCode::from(cli_exit_code(&e))
        }
    }
}

/// Map a core error to the documented CLI exit codes (see README
/// "退出码"): 2 missing dependency, 3 user cancelled, 4 confirmation
/// mismatch, 5 invalid argument, 1 everything else. Errors that
/// originate in the CLI layer (plain `anyhow!`) keep the generic 1.
fn cli_exit_code(err: &anyhow::Error) -> u8 {
    use ntfs_mac_core::Error as Core;
    match err.downcast_ref::<Core>() {
        Some(Core::MissingDependency { .. }) => 2,
        Some(Core::Cancelled) => 3,
        Some(Core::ConfirmationMismatch { .. }) => 4,
        Some(Core::InvalidArgument(_)) => 5,
        _ => 1,
    }
}

fn init_logging(level: &str) {
    use tracing_subscriber::prelude::*;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    // Stderr layer: human-readable, no timestamps (so `ntfs-mac ... | jq`
    // stays clean when stderr is redirected).
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .without_time()
        .with_ansi(!is_no_color());

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer);

    if let Some(log_file) = log_file_path() {
        let _ = std::fs::create_dir_all(log_file.parent().unwrap_or(Path::new(".")));
        let file_appender = tracing_appender::rolling::never(
            log_file.parent().unwrap_or(Path::new(".")),
            log_file
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("ntfs-mac.log")),
        );
        // Hold the guard in a static so the appender lives for the process
        // lifetime. (Without this, the file is flushed and closed immediately.)
        let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
        leak_appender_guard(_guard);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file_writer)
            .with_target(true)
            .with_file(true)
            .with_line_number(true);

        registry.with(file_layer).init();
    } else {
        registry.init();
    }
}

/// Leak the non-blocking appender guard so the file stays open for the
/// process lifetime. Without this, the guard is dropped at the end of
/// `init_logging` and all subsequent writes are silently discarded.
fn leak_appender_guard<G>(guard: G) {
    // SAFETY: We intentionally leak this value. The guard's only purpose
    // is to keep the file open; leaking it is the documented pattern for
    // `tracing_appender::non_blocking`.
    std::mem::forget(guard);
}

fn is_no_color() -> bool {
    std::env::var("NO_COLOR").is_ok() || !ntfs_mac_core::is_stdout_tty()
}

fn log_file_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join("Library").join("Logs").join("ntfs-mac.log"))
}

fn run(cli: &Cli) -> Result<()> {
    let cfg = if let Some(path) = &cli.config {
        ntfs_mac_core::load_config(path)?
    } else {
        Config::load()?
    };
    match &cli.command {
        Command::Doctor => cmd_doctor(cli, &cfg),
        Command::List => cmd_list(cli, &cfg),
        // Commands that shell out to the ntfs-3g toolchain pre-check the
        // dependency set so the user gets one aggregated, actionable
        // error (exit code 2) instead of a mid-operation failure.
        Command::Mount {
            target,
            mount_point,
            readonly,
            force_ntfs3g,
        } => {
            deps::require_ready()?;
            cmd_mount(cli, &cfg, target, mount_point, *readonly, *force_ntfs3g)
        }
        Command::Unmount { target } => cmd_unmount(cli, &cfg, target),
        Command::Rw { target } => {
            deps::require_ready()?;
            cmd_rw(cli, &cfg, target)
        }
        Command::Status => cmd_status(cli, &cfg),
        Command::Daemon {
            install,
            uninstall,
            status,
            interval,
        } => cmd_daemon(cli, &cfg, *install, *uninstall, *status, *interval),
        Command::Format {
            target,
            label,
            full,
            sector_size,
            cluster_size,
            yes,
        } => {
            deps::require_ready()?;
            cmd_format(
                cli,
                &cfg,
                &FormatArgs {
                    target: target.clone(),
                    label: label.clone(),
                    quick: !*full,
                    sector_size: *sector_size,
                    cluster_size: *cluster_size,
                    yes: *yes,
                },
            )
        }
        Command::Fix { target, fsck } => {
            deps::require_ready()?;
            cmd_fix(cli, &cfg, target, *fsck)
        }
        Command::Copy {
            source,
            destination,
            delete,
            dry_run,
        } => cmd_copy(cli, source, destination, *delete, *dry_run),
        Command::Config { command } => cmd_config(cli, &cfg, command),
        Command::Sponsor { reveal } => cmd_sponsor(cli, *reveal),
        Command::License { full, third_party } => cmd_license(cli, *full, *third_party),
        Command::Completions { shell } => cmd_completions(*shell),
    }
}

/// `ntfs-mac license` — licence summary, full text, or crate inventory.
///
/// Everything is resolved from the binary plus [`license`]'s on-disk
/// probes, so the command keeps working when the tool is installed rather
/// than run from a checkout.
fn cmd_license(cli: &Cli, full: bool, third_party: bool) -> Result<()> {
    if full {
        return cmd_license_full(cli);
    }

    if third_party {
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "source": "THIRD_PARTY_LICENSES.md",
                    "embedded": true,
                    "counts": license::third_party_counts(),
                }))?
            );
        } else {
            print!("{}", license::THIRD_PARTY_INVENTORY);
        }
        return Ok(());
    }

    let summary = license::summary();

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    println!(
        "{} {} — {}",
        summary.project.bold(),
        summary.version,
        summary.spdx.green().bold()
    );
    println!("{}", summary.copyright);
    println!("{}", summary.license_url);
    println!();

    for (kind, text) in license::APACHE_CLAUSES {
        println!("  {:<12} {}", kind.bold(), text);
    }
    println!();

    println!("{}", "Bundled files".bold());
    print_resource("LICENSE", summary.license_file.as_deref());
    print_resource("NOTICE", summary.notice_file.as_deref());
    match summary.third_party {
        Some(counts) => println!(
            "  {:<24} {} crates ({} CLI, {} GUI) — embedded in this binary",
            "THIRD_PARTY_LICENSES.md", counts.total, counts.cli, counts.gui
        ),
        None => println!(
            "  {:<24} embedded in this binary",
            "THIRD_PARTY_LICENSES.md"
        ),
    }
    println!();
    println!(
        "Use {} for the complete licence text, {} for the crate inventory.",
        "--full".cyan(),
        "--third-party".cyan()
    );

    Ok(())
}

fn print_resource(label: &str, found: Option<&str>) {
    // Both files are installed next to each other; a miss usually means the
    // binary was copied out of its install layout.
    match found {
        Some(path) => println!("  {label:<24} {path}"),
        None => println!("  {label:<24} {}", "not found on disk".yellow()),
    }
}

fn cmd_license_full(cli: &Cli) -> Result<()> {
    let path = license::license_file().ok_or_else(|| {
        anyhow::anyhow!(
            "LICENSE not found on disk — the canonical text is at {}",
            license::LICENSE_URL
        )
    })?;

    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "spdx": license::SPDX_ID,
                "path": license::display_path(&path),
                "text": text,
            }))?
        );
    } else {
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
    }

    Ok(())
}

/// `ntfs-mac completions <shell>` — write a completion script to stdout.
///
/// stdout only, so `ntfs-mac completions zsh > _ntfs-mac` works.
fn cmd_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}

fn cmd_doctor(cli: &Cli, _cfg: &Config) -> Result<()> {
    let report = deps::report()?;
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{report}");
    }
    Ok(())
}

fn cmd_list(cli: &Cli, _cfg: &Config) -> Result<()> {
    let volumes = device::list_volumes()?;
    if volumes.is_empty() {
        if cli.json {
            println!("[]");
        } else {
            println!("{}", "No NTFS volumes found.".yellow());
        }
        return Ok(());
    }
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&volumes)?);
    } else {
        println!(
            "{:<12} {:<20} {:<10} {:<12} {}",
            "Device".bold(),
            "Name".bold(),
            "Size".bold(),
            "Status".bold(),
            "Mount Point".bold()
        );
        println!("{}", "-".repeat(80));
        for v in &volumes {
            let status = if v.mounted {
                "mounted".green().to_string()
            } else {
                "unmounted".yellow().to_string()
            };
            let mp = v.mount_point.as_deref().unwrap_or("-");
            println!(
                "{:<12} {:<20} {:<10} {:<12} {}",
                v.device_identifier, v.volume_name, v.size_pretty, status, mp
            );
        }
    }
    Ok(())
}

fn cmd_mount(
    cli: &Cli,
    cfg: &Config,
    target: &str,
    mount_point: &Option<String>,
    readonly: bool,
    force_ntfs3g: bool,
) -> Result<()> {
    // Validate mount point if provided.
    if let Some(mp) = mount_point {
        validate_mount_point(mp)?;
    }

    // Allow "all" as a special target; otherwise validate device id.
    if target != "all" {
        validate_device_id(target)?;
    }

    let volumes = device::list_volumes()?;

    // Special case: "all" mounts all unmounted NTFS volumes.
    if target == "all" {
        let unmounted = volumes.iter().filter(|v| !v.mounted).collect::<Vec<_>>();
        if unmounted.is_empty() {
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "mounted": [] }))?
                );
            } else {
                println!("{}", "No unmounted NTFS volumes found.".yellow());
            }
            return Ok(());
        }
        let mut mounted = Vec::new();
        let mut errors = Vec::new();
        for vol in &unmounted {
            let opts = mount::MountOptions {
                mount_point: mount_point.clone(),
                readonly,
                extra_options: Vec::new(),
                force_ntfs3g,
            };
            match mount::mount(vol, &opts, cfg) {
                Ok(mp) => mounted.push(format!("{} → {}", vol.display_label(), mp)),
                Err(e) => errors.push(format!("{}: {e}", vol.display_label())),
            }
        }
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "mounted": mounted,
                    "errors": errors
                }))?
            );
        } else {
            for m in &mounted {
                println!("{} {}", "Mounted".green().bold(), m);
            }
            for e in &errors {
                eprintln!("{} {}", "Failed".red().bold(), e);
            }
        }
        return Ok(());
    }

    let vol = volumes
        .iter()
        .find(|v| v.device_identifier == target || v.volume_name == target)
        .ok_or_else(|| anyhow::anyhow!("volume '{}' not found", target))?
        .clone();
    let opts = mount::MountOptions {
        mount_point: mount_point.clone(),
        readonly,
        extra_options: Vec::new(),
        force_ntfs3g,
    };
    let result = mount::mount(&vol, &opts, cfg)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "mount_point": result }))?
        );
    } else {
        println!("{} {}", "Mounted".green().bold(), result);
    }
    Ok(())
}

fn cmd_unmount(cli: &Cli, _cfg: &Config, target: &str) -> Result<()> {
    let volumes = device::list_volumes()?;

    // Special case: "all" unmounts every mounted NTFS volume, mirroring
    // `mount all`.
    if target == "all" {
        let mounted: Vec<_> = volumes.iter().filter(|v| v.mounted).collect();
        if mounted.is_empty() {
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "unmounted": [] }))?
                );
            } else {
                println!("{}", "No mounted NTFS volumes found.".yellow());
            }
            return Ok(());
        }
        let mut unmounted = Vec::new();
        let mut errors = Vec::new();
        for vol in &mounted {
            match mount::unmount(&vol.device_identifier) {
                Ok(()) => unmounted.push(vol.device_identifier.clone()),
                Err(e) => errors.push(format!("{}: {e}", vol.device_identifier)),
            }
        }
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "unmounted": unmounted,
                    "errors": errors
                }))?
            );
        } else {
            for d in &unmounted {
                println!("{} {}", "Unmounted".green().bold(), d);
            }
            for e in &errors {
                eprintln!("{} {}", "Failed".red().bold(), e);
            }
        }
        return Ok(());
    }

    // Look up by device identifier or volume name, then unmount the device.
    let vol = volumes
        .iter()
        .find(|v| v.device_identifier == target || v.volume_name == target)
        .ok_or_else(|| anyhow::anyhow!("volume '{}' not found", target))?
        .clone();
    mount::unmount(&vol.device_identifier)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({ "unmounted": vol.device_identifier })
            )?
        );
    } else {
        println!("{} {}", "Unmounted".green().bold(), vol.device_identifier);
    }
    Ok(())
}

/// Arguments for the `format` subcommand (avoids >7 params on `cmd_format`).
struct FormatArgs {
    target: String,
    label: Option<String>,
    quick: bool,
    sector_size: u32,
    cluster_size: u32,
    yes: bool,
}

fn cmd_format(cli: &Cli, cfg: &Config, args: &FormatArgs) -> Result<()> {
    let volumes = device::list_volumes()?;
    let vol = volumes
        .iter()
        .find(|v| v.device_identifier == args.target || v.volume_name == args.target)
        .ok_or_else(|| anyhow::anyhow!("volume '{}' not found", args.target))?
        .clone();
    if vol.mounted {
        return Err(anyhow::anyhow!(
            "volume '{}' is mounted; unmount first",
            vol.device_identifier
        ));
    }
    let label_str = args.label.clone().unwrap_or_default();
    if let Some(w) = format::validate_label(&label_str) {
        eprintln!("{} {}", "warning:".yellow().bold(), w);
    }
    let opts = format::FormatOptions {
        label: label_str,
        quick: args.quick,
        sector_size: args.sector_size,
        cluster_size: args.cluster_size,
        extra_args: Vec::new(),
    };
    let token = DestructiveToken::new(&vol.device_identifier, &vol.volume_name);
    if !args.yes && cfg.require_confirmation {
        // If stdin is not a TTY (piped input), we can't do interactive
        // confirmation — require --yes instead.
        if !ntfs_mac_core::is_stdin_tty() {
            return Err(anyhow::anyhow!(
                "interactive confirmation requires a TTY; use --yes to confirm non-interactively"
            ));
        }
        println!(
            "{}",
            "This will ERASE all data on the selected volume."
                .red()
                .bold()
        );
        println!("Volume: {}", vol.display_label());
        println!("Type '{}' to confirm:", token.confirm_phrase());
        let input = read_line_interactive("? ")?;
        if input != token.confirm_phrase() {
            return Err(anyhow::anyhow!("confirmation failed, operation cancelled"));
        }
    }
    format::format(&vol, &opts, &token, cfg)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({ "formatted": vol.device_identifier })
            )?
        );
    } else {
        println!("{} {}", "Formatted".green().bold(), vol.device_identifier);
    }
    Ok(())
}

fn cmd_fix(cli: &Cli, _cfg: &Config, target: &str, fsck: bool) -> Result<()> {
    validate_device_id(target)?;
    let volumes = device::list_volumes()?;
    let vol = volumes
        .iter()
        .find(|v| v.device_identifier == target || v.volume_name == target)
        .ok_or_else(|| anyhow::anyhow!("volume '{}' not found", target))?
        .clone();
    let opts = fix::FixOptions {
        tool: if fsck {
            fix::FixTool::FsckNtfs
        } else {
            fix::FixTool::Ntfsfix
        },
        drop_dirty_flag: true,
    };
    fix::fix_with_opts(&vol, &opts)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "fixed": vol.device_identifier }))?
        );
    } else {
        println!("{} {}", "Fixed".green().bold(), vol.device_identifier);
    }
    Ok(())
}

fn cmd_copy(
    cli: &Cli,
    source: &Path,
    destination: &Path,
    delete: bool,
    dry_run: bool,
) -> Result<()> {
    let opts = copy::CopyOptions {
        delete,
        preserve: true,
        progress: !cli.json,
        dry_run,
    };
    let result = copy::copy(source, destination, &opts)?;
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let status = if dry_run { "would copy" } else { "copied" };
        println!(
            "{} {} -> {} in {}ms via {}",
            status.green().bold(),
            source.display(),
            destination.display(),
            result.duration_ms,
            result.tool_used
        );
    }
    Ok(())
}

fn cmd_rw(cli: &Cli, cfg: &Config, target: &str) -> Result<()> {
    let volumes = device::list_volumes()?;
    let vol = volumes
        .iter()
        .find(|v| v.device_identifier == target || v.volume_name == target)
        .ok_or_else(|| anyhow::anyhow!("volume '{}' not found", target))?
        .clone();
    let result = mount::remount_rw(&vol, cfg)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "read_write": result }))?
        );
    } else {
        println!("{} {}", "Read-Write".green().bold(), result);
    }
    Ok(())
}

fn cmd_status(cli: &Cli, _cfg: &Config) -> Result<()> {
    let volumes = device::list_volumes()?;
    let mounted = volumes.iter().filter(|v| v.mounted).count();
    let unmounted = volumes.iter().filter(|v| !v.mounted).count();
    let report = deps::report()?;
    let launchagent = daemon::is_launchagent_installed();

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "total_volumes": volumes.len(),
                "mounted": mounted,
                "unmounted": unmounted,
                "ready": report.ready,
                "daemon_installed": launchagent,
                "volumes": volumes.iter().map(|v| serde_json::json!({
                    "device": v.device_identifier,
                    "name": v.volume_name,
                    "size": v.size_pretty,
                    "mounted": v.mounted,
                    "mount_point": v.mount_point
                })).collect::<Vec<_>>()
            }))?
        );
    } else {
        println!("{}", "ntfs-mac Status".bold());
        println!(
            "  Dependencies: {}",
            if report.ready {
                "READY".green()
            } else {
                "MISSING".red()
            }
        );
        println!(
            "  Daemon: {}",
            if launchagent {
                "INSTALLED".green()
            } else {
                "not installed".yellow()
            }
        );
        println!("  Volumes: {}", volumes.len());
        println!("    Mounted:   {} {}", mounted, "✓".green());
        println!("    Unmounted: {} {}", unmounted, "○".yellow());
        if !volumes.is_empty() {
            println!();
            for v in &volumes {
                let icon = if v.mounted {
                    "●".green().to_string()
                } else {
                    "○".yellow().to_string()
                };
                let mp = v.mount_point.as_deref().unwrap_or("-");
                println!(
                    "  {} {} ({} — {})",
                    icon,
                    v.display_label(),
                    v.size_pretty,
                    mp
                );
            }
        }
    }
    Ok(())
}

fn cmd_daemon(
    cli: &Cli,
    _cfg: &Config,
    install: bool,
    uninstall: bool,
    status: bool,
    interval: u64,
) -> Result<()> {
    if uninstall {
        daemon::uninstall_launchagent()?;
        println!("{} uninstalled", "LaunchAgent".green().bold());
        return Ok(());
    }
    if install {
        let bin_path = std::env::current_exe()?;
        let result = daemon::install_launchagent(&bin_path)?;
        println!(
            "{} installed: {}",
            "LaunchAgent".green().bold(),
            result.plist_path.display()
        );
        if result.loaded {
            println!("  {}", "Daemon loaded and running now".green());
        } else {
            println!(
                "  {} daemon could not be loaded now; it will start at next login.",
                "warning:".yellow().bold()
            );
            if let Some(err) = &result.load_error {
                println!("  Reason: {err}");
            }
        }
        println!("  Run: ntfs-mac daemon --uninstall to remove");
        println!("  Logs: ~/Library/Logs/ntfs-mac.err.log");
        return Ok(());
    }
    if status {
        let installed = daemon::is_launchagent_installed();
        let loaded = installed && daemon::is_launchagent_loaded();
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "installed": installed,
                    "loaded": loaded
                }))?
            );
        } else {
            println!("{}", "Daemon Status".bold());
            println!(
                "  LaunchAgent: {}",
                if installed {
                    "installed".green()
                } else {
                    "not installed".yellow()
                }
            );
            println!(
                "  Loaded in launchd: {}",
                if loaded {
                    "yes".green()
                } else if installed {
                    "no".yellow()
                } else {
                    "-".dimmed()
                }
            );
        }
        return Ok(());
    }
    // Run daemon in foreground
    let opts = daemon::DaemonOptions {
        interval_secs: interval,
        ..Default::default()
    };
    if cli.json {
        println!("ntfs-mac daemon started");
    } else {
        let msg = format!(
            "ntfs-mac daemon started (interval={}s). Press Ctrl+C to stop.",
            interval
        );
        println!("{}", msg.yellow());
    }
    let cfg = ntfs_mac_core::load_config(&default_config_path())?;
    match daemon::run(&cfg, &opts) {
        Ok(()) => Ok(()),
        Err(ntfs_mac_core::Error::Cancelled) => {
            // Graceful shutdown via SIGINT/SIGTERM — not an error.
            if cli.json {
                println!("daemon stopped");
            } else {
                println!("{}", "Daemon stopped".green().bold());
            }
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("{e}")),
    }
}

/// Parse a boolean config value strictly. `config set` must not guess:
/// an unparsable value for a safety switch (e.g. `require_confirmation`)
/// would otherwise silently disable it. Uses the core `InvalidArgument`
/// error so the CLI exit code is the documented 5.
fn parse_config_bool(key: &str, value: &str) -> Result<bool> {
    value.parse::<bool>().map_err(|_| {
        ntfs_mac_core::Error::InvalidArgument(format!(
            "invalid boolean value `{value}` for config key `{key}` (use `true` or `false`)"
        ))
        .into()
    })
}

fn cmd_config(cli: &Cli, cfg: &Config, command: &ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Show => {
            if cli.json {
                println!("{}", serde_json::to_string_pretty(cfg)?);
            } else {
                println!("Config: {}", default_config_path().display());
                println!("{}", serde_json::to_string_pretty(cfg)?);
            }
            Ok(())
        }
        ConfigCommand::Set { key, value } => {
            let mut cfg = cfg.clone();
            match key.as_str() {
                "mount_base" => cfg.mount_base = value.clone(),
                "fuse_driver" => cfg.fuse_driver = value.clone(),
                // Strict parse: a typo must not silently disable a
                // safety-relevant setting (e.g. `require_confirmation`).
                "require_confirmation" => cfg.require_confirmation = parse_config_bool(key, value)?,
                "color" => cfg.color = parse_config_bool(key, value)?,
                "mount_options" => {
                    let opts: Vec<String> =
                        value.split(',').map(|s| s.trim().to_string()).collect();
                    // Reject malformed options at the entry point so the
                    // user learns immediately, not at mount time.
                    mount::validate_mount_options(&opts)?;
                    cfg.mount_options = opts;
                }
                _ => return Err(anyhow::anyhow!("unknown config key: {}", key)),
            }
            cfg.save()?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "saved": true }))?
                );
            } else {
                println!("{} {}", "Saved".green().bold(), key);
            }
            Ok(())
        }
        ConfigCommand::Reset => {
            Config::default().save()?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "reset": true }))?
                );
            } else {
                println!("{}", "Configuration reset to defaults".green().bold());
            }
            Ok(())
        }
        ConfigCommand::Path => {
            println!("{}", default_config_path().display());
            Ok(())
        }
    }
}

/// Sponsor / donation QR code helper.
///
/// The QR code image lives at `crates/ntfs-mac-tauri/src/assets/sponsor-qr.svg`
/// in the source tree and is bundled into the .app on release builds.
/// Replace that file with your own payment QR code to receive donations.
fn cmd_sponsor(cli: &Cli, reveal: bool) -> Result<()> {
    // Candidate paths: source tree first (works in dev), then bundle locations.
    let candidates = [
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../ntfs-mac-tauri/src/assets/sponsor-qr.svg"
        ),
        "/Applications/ntfs-mac.app/Contents/Resources/assets/sponsor-qr.svg",
        "assets/sponsor-qr.svg",
    ];

    let found = candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|p| p.to_string());

    let path = match found {
        Some(p) => p,
        None => {
            // File not found — print instructions.
            let msg = "Sponsor QR code not found.\n\nTo enable donations:\n  1. Create a QR code image (PNG/SVG/JPG)\n  2. Place it at: crates/ntfs-mac-tauri/src/assets/sponsor-qr.svg\n  3. Rebuild: cargo build --release\n\nCurrent default: src/assets/sponsor-qr.svg (placeholder)";
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({ "found": false, "hint": "Place QR at src/assets/sponsor-qr.svg" })
                    )?
                );
            } else {
                println!("{}", msg.yellow());
            }
            return Ok(());
        }
    };

    if reveal {
        // `open -R` reveals the file in Finder (macOS).
        let status = std::process::Command::new("open")
            .args(["-R", &path])
            .status()
            .map_err(|e| anyhow::anyhow!("cannot launch Finder: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!(
                "Finder could not reveal `{path}` (exit status {status})"
            ));
        }
    }

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({ "qr_path": path, "revealed": reveal })
            )?
        );
    } else {
        println!("{}", "Sponsor QR Code".bold().green());
        println!("  Path: {}", path.cyan());
        if reveal {
            println!("  {}", "Revealed in Finder".green());
        }
        println!();
        println!("{}", "To replace with your own QR code:".bold());
        println!("  1. Create a QR code image (PNG/SVG/JPG)");
        println!("  2. Replace: {}", path.cyan());
        println!("  3. Rebuild: cargo build --release");
    }
    Ok(())
}
