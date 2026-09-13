// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! ntfs-mac GUI backend: a menu-bar (tray) application in the spirit of
//! [mounty](https://mounty.app/).
//!
//! The icon lives in the macOS menu bar and every NTFS volume is reachable as a
//! flat menu entry — mount, open in Finder, unmount, eject. The main window is
//! a fuller-featured console.
//!
//! ## Threading model
//!
//! * **Menu-event handler** runs on the main thread. Each action hands off to a
//!   short-lived [`std::thread`] so the tray menu can close immediately, then
//!   refreshes the menu.
//! * **Background poller** re-scans volumes and dependencies every
//!   [`REFRESH_INTERVAL`]; a signature check prevents rebuilding the `NSMenu`
//!   when nothing changed.
//! * **Subprocess calls never run on the async runtime.** Everything that shells
//!   out goes through [`ntfs_mac_core::runner`], which carries a hard timeout.
//!
//! ## Labels
//!
//! [`ZH`] / [`EN`] mirror the frontend i18n dictionaries in `src/app.js`. Keep
//! the two in sync — the frontend contract test in `scripts/test-frontend.sh`
//! fails if they drift.

use std::sync::Mutex;
use std::time::Duration;

use ntfs_mac_core::{Config, DestructiveToken, deps, device, fix, format, mount, runner};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

// ---------------------------------------------------------------------------
// Menu-bar tray constants
// ---------------------------------------------------------------------------

/// Tray identity. [`AppHandle::tray_by_id`] matches on this string.
const TRAY_ID: &str = "ntfs-mac";

/// Menu identity, so the same `Menu` id is reused across rebuilds.
const MENU_ID: &str = "ntfs-mac-menu";

/// Bundled menu-bar glyph. Pure black + alpha, so macOS tints it for the
/// light/dark menu bar (see [`TrayIconBuilder::icon_as_template`]).
#[cfg_attr(target_os = "macos", allow(dead_code))]
const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/tray/tray-44.png");

/// Background poll interval.
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// Hard timeout for any one shell-out from the UI.
const CMD_TIMEOUT: Duration = Duration::from_secs(30);

/// Emitted to the webview whenever the volume set or language changed.
const EVENT_REFRESH: &str = "ntfs-mac:refresh";

/// Emitted when an action fails, so the window can surface a message.
const EVENT_ERROR: &str = "ntfs-mac:error";

/// Stable menu-item ids. Per-volume ids are prefixed with the device id
/// (e.g. `vol-mount-disk2s2`) so an entry keeps its identity across rebuilds.
const ID_HEADER: &str = "volumes-header";
const ID_EMPTY: &str = "volumes-empty";
const ID_STATUS: &str = "deps-status";
const ID_REFRESH: &str = "refresh";
const ID_SHOW_WINDOW: &str = "show-window";
const ID_INSTALL_DEPS: &str = "install-deps";
const ID_QUIT: &str = "quit";

const VOL_MOUNT_PREFIX: &str = "vol-mount-";
const VOL_OPEN_PREFIX: &str = "vol-open-";
const VOL_UNMOUNT_PREFIX: &str = "vol-unmount-";
const VOL_EJECT_PREFIX: &str = "vol-eject-";

// ---------------------------------------------------------------------------
// Labels — mirror the frontend i18n dictionaries
// ---------------------------------------------------------------------------

/// Menu-bar copy for one language.
#[derive(Debug, Clone, Copy)]
struct Labels<'a> {
    /// Disabled group header above the volume list.
    header: &'a str,
    /// Placeholder row shown when no NTFS volume is attached.
    empty: &'a str,
    /// Mount an unmounted volume.
    mount: &'a str,
    /// Open a mounted volume in Finder.
    open: &'a str,
    /// Unmount (safely detach) a mounted volume.
    unmount: &'a str,
    /// Physically eject the whole enclosing drive.
    eject: &'a str,
    refresh: &'a str,
    show_window: &'a str,
    install_deps: &'a str,
    quit: &'a str,
    ready: &'a str,
    missing: &'a str,
    /// Tooltip volume count; `{n}` is replaced with the total.
    volumes_n: &'a str,
    /// Tooltip mount count; `{n}` is replaced with the mounted total.
    mounted_n: &'a str,
}

const ZH: Labels<'static> = Labels {
    header: "NTFS 卷",
    empty: "未检测到 NTFS 卷",
    mount: "挂载",
    open: "在 Finder 中打开",
    unmount: "卸载",
    eject: "弹出移动硬盘",
    refresh: "刷新",
    show_window: "打开主窗口",
    install_deps: "安装依赖",
    quit: "退出 ntfs-mac",
    ready: "依赖已就绪",
    missing: "依赖缺失",
    volumes_n: "{n} 个 NTFS 卷",
    mounted_n: "已挂载 {n} 个",
};

const EN: Labels<'static> = Labels {
    header: "NTFS volumes",
    empty: "No NTFS volumes found",
    mount: "Mount",
    open: "Open in Finder",
    unmount: "Unmount",
    eject: "Eject drive",
    refresh: "Refresh",
    show_window: "Show main window",
    install_deps: "Install dependencies",
    quit: "Quit ntfs-mac",
    ready: "All dependencies ready",
    missing: "Dependencies missing",
    volumes_n: "{n} volume(s)",
    mounted_n: "{n} mounted",
};

/// Resolve labels for `language`; anything other than `"en"` falls back to
/// Chinese, which is the default UI language.
#[must_use]
fn labels(language: &str) -> Labels<'static> {
    if language == "en" { EN } else { ZH }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

/// Shared state, reachable from commands and from the tray poller.
#[derive(Default)]
struct AppState {
    /// Lazily loaded user config; read once, then cached.
    config: Mutex<Option<Config>>,
    /// UI language: `"zh"` (default) or `"en"`.
    language: Mutex<String>,
}

impl AppState {
    fn language(&self) -> String {
        self.language
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|poisoned| poisoned.into_inner().clone())
    }

    /// User config, loaded and cached on first access. A missing or unreadable
    /// config file degrades to the built-in defaults.
    fn config(&self) -> Config {
        let mut guard = match self.config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard
            .get_or_insert_with(|| Config::load().unwrap_or_default())
            .clone()
    }
}

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// A volume as seen by the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct VolumeInfo {
    device_identifier: String,
    volume_name: String,
    media_type: String,
    size_bytes: u64,
    size_pretty: String,
    mounted: bool,
    mount_point: Option<String>,
    parent_disk: Option<String>,
    location: String,
    display_label: String,
}

/// One dependency probe result.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct DepInfo {
    name: String,
    present: bool,
    path: Option<String>,
    install_hint: Option<String>,
}

/// Full dependency report.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct DepReportInfo {
    deps: Vec<DepInfo>,
    ready: bool,
    arch: String,
    macos_version: Option<String>,
}

/// Package metadata, shown in the window's about block.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppInfo {
    name: String,
    version: String,
    description: String,
    repository: String,
    license: String,
    authors: String,
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// Map any displayable error into a [`tauri::Error`].
///
/// The message is stringified first: `anyhow!` needs `Debug + Send + Sync`,
/// which a bare `Display` bound does not provide.
fn e(err: impl std::fmt::Display) -> tauri::Error {
    tauri::Error::from(anyhow::anyhow!(err.to_string()))
}

/// `disk2` (whole disk) and `disk2s2` (partition) are the only identifiers the
/// CLI accepts; reject anything else before it reaches a subprocess.
///
/// At most one `s` is allowed, and it must be preceded and followed by digits —
/// `disk2s2s3` is rejected even though it is full of digits and `s`.
#[must_use]
fn is_device_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("disk") else {
        return false;
    };
    match rest.find('s') {
        // Whole disk: `disk2`, `disk10`.
        None => !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
        Some(pos) => {
            let (number, tail) = rest.split_at(pos);
            let Some(partition) = tail.strip_prefix('s') else {
                return false;
            };
            !number.is_empty()
                && number.chars().all(|c| c.is_ascii_digit())
                && !partition.is_empty()
                && !partition.contains('s')
                && partition.chars().all(|c| c.is_ascii_digit())
        }
    }
}

/// `disk2s2` -> `disk2`, `disk10s2` -> `disk10`, `disk2` -> `disk2`.
///
/// Device ids are `disk<N>` optionally followed by `s<M>`. Splits on the first
/// non-digit after the leading digits, which is the partition separator. A bare
/// whole-disk id with no separator maps to itself.
#[must_use]
fn whole_disk(device_id: &str) -> Option<String> {
    let rest = device_id.strip_prefix("disk")?;
    if rest.is_empty() {
        return Some("disk".to_string());
    }
    let Some(end) = rest.find(|c: char| !c.is_ascii_digit()) else {
        // No separator: already a whole disk (`disk2`).
        return Some(device_id.to_string());
    };
    if end == 0 {
        return None;
    }
    let disk = format!("disk{}", &rest[..end]);
    let partition = rest.get(end..).and_then(|tail| tail.strip_prefix('s'))?;
    if partition.is_empty() || !partition.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(disk)
}

/// Run a command through the core runner with the standard UI timeout.
fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let opts = runner::RunOptions {
        timeout: Some(CMD_TIMEOUT),
        ..Default::default()
    };
    let out = runner::run(cmd, args, &opts).map_err(|err| err.to_string())?;
    if !out.success() {
        let detail = out.stderr.trim();
        return Err(if detail.is_empty() {
            format!("{cmd} exited with status {}", out.status)
        } else {
            detail.to_string()
        });
    }
    Ok(out.stdout.trim().to_string())
}

/// Find a volume by device id, or fail with an actionable message.
fn find_volume<'a>(
    volumes: &'a [device::Volume],
    device_id: &str,
) -> Result<&'a device::Volume, String> {
    volumes
        .iter()
        .find(|v| v.device_identifier == device_id)
        .ok_or_else(|| format!("未找到 NTFS 卷 {device_id}"))
}

/// Eject target: prefer the plist's parent disk, else derive from the id.
fn eject_target(vol: &device::Volume) -> String {
    vol.parent_disk
        .clone()
        .or_else(|| whole_disk(&vol.device_identifier))
        .unwrap_or_else(|| vol.device_identifier.clone())
}

/// Escape a shell snippet for embedding in an AppleScript double-quoted string.
fn escape_applescript(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

/// Build the `brew install …` script for every missing dependency.
fn brew_install_script(missing: &[&deps::DepStatus]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for dep in missing {
        let Some(hint) = dep.install_hint.as_deref().map(str::trim) else {
            continue;
        };
        if hint.is_empty() || hint.starts_with('#') {
            continue;
        }
        if !lines.iter().any(|line| line == hint) {
            lines.push(hint.to_string());
        }
    }
    if lines.is_empty() {
        "echo '未识别到可执行的安装命令'".to_string()
    } else {
        let body = lines.join("\n");
        format!("set -e\n{body}")
    }
}

// ---------------------------------------------------------------------------
// Tray menu
// ---------------------------------------------------------------------------

/// Fingerprint of the current volume set + dependency readiness. Only changes
/// when the menu is genuinely worth rebuilding.
#[must_use]
fn menu_signature(volumes: &[device::Volume], ready: bool) -> String {
    let mut sig = String::from(if ready { "r1;" } else { "r0;" });
    for vol in volumes {
        sig.push_str(&vol.device_identifier);
        sig.push(if vol.mounted { '1' } else { '0' });
        sig.push(';');
    }
    sig
}

/// Tray tooltip: what is attached, and whether dependencies are ready.
#[must_use]
fn tray_tooltip(language: &str, volumes: &[device::Volume], ready: bool) -> String {
    let l = labels(language);
    if volumes.is_empty() {
        return if ready {
            format!("ntfs-mac — {}", l.ready)
        } else {
            format!("ntfs-mac — {}", l.missing)
        };
    }
    let mounted = volumes.iter().filter(|v| v.mounted).count();
    let fill = |template: &str, count: usize| template.replace("{n}", &count.to_string());
    let separator = if language == "en" { ", " } else { "，" };
    format!(
        "ntfs-mac — {}{}{}",
        fill(l.volumes_n, volumes.len()),
        separator,
        fill(l.mounted_n, mounted),
    )
}

/// Assemble the flat menu-bar menu.
///
/// Layout mirrors mounty: a disabled group header, one entry per volume with an
/// inline action verb, then refresh / show window / install deps / quit.
fn build_tray_menu(
    app: &AppHandle,
    language: &str,
    volumes: &[device::Volume],
    ready: bool,
) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let l = labels(language);
    let mut b = tauri::menu::MenuBuilder::new(app).id(MENU_ID);

    if volumes.is_empty() {
        b = b.item(&tauri::menu::MenuItem::with_id(
            app,
            ID_EMPTY,
            l.empty,
            false,
            None::<&str>,
        )?);
    } else {
        b = b.item(&tauri::menu::MenuItem::with_id(
            app,
            ID_HEADER,
            l.header,
            false,
            None::<&str>,
        )?);

        // One `弹出移动硬盘` entry per physical drive, not per partition.
        let mut eject_seen: Vec<String> = Vec::new();
        for vol in volumes {
            let label = vol.display_label();
            if vol.mounted {
                let id = format!("{VOL_OPEN_PREFIX}{}", vol.device_identifier);
                b = b.text(id, format!("{} — {}", label, l.open));

                let id = format!("{VOL_UNMOUNT_PREFIX}{}", vol.device_identifier);
                b = b.text(id, format!("{} — {}", label, l.unmount));
            } else {
                let id = format!("{VOL_MOUNT_PREFIX}{}", vol.device_identifier);
                b = b.text(id, format!("{} — {}", label, l.mount));
            }

            let target = eject_target(vol);
            if !eject_seen.contains(&target) {
                eject_seen.push(target);
                let id = format!("{VOL_EJECT_PREFIX}{}", vol.device_identifier);
                b = b.text(id, format!("{} — {}", label, l.eject));
            }
        }
    }

    b = b.separator();

    let status = if ready { l.ready } else { l.missing };
    b = b.item(&tauri::menu::MenuItem::with_id(
        app,
        ID_STATUS,
        status,
        false,
        None::<&str>,
    )?);
    b = b.text(ID_REFRESH, l.refresh);
    b = b.text(ID_SHOW_WINDOW, l.show_window);
    // `安装依赖` is useless once everything is present; grey it out.
    let install =
        tauri::menu::MenuItem::with_id(app, ID_INSTALL_DEPS, l.install_deps, !ready, None::<&str>)?;
    b = b.item(&install);

    b = b.separator();
    b = b.text(ID_QUIT, l.quit);

    b.build()
}

/// Rebuild the tray menu and tooltip from live state, then notify the window.
///
/// Safe to call from any thread: `Menu::new` and `TrayIcon::set_menu` marshal
/// onto the main thread, executing inline when the caller already is.
fn refresh_tray_menu(
    app: &AppHandle,
    volumes: &[device::Volume],
    ready: bool,
) -> tauri::Result<()> {
    let language = app.state::<AppState>().language();
    let menu = build_tray_menu(app, &language, volumes, ready)?;
    let tooltip = tray_tooltip(&language, volumes, ready);

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(tooltip))?;
    } else {
        install_tray(app, &language, volumes, ready)?;
    }

    let _ = app.emit(EVENT_REFRESH, ());
    Ok(())
}

/// Create the tray icon. Called once from `setup`, and again by
/// [`refresh_tray_menu`] if the icon was ever dropped.
fn install_tray(
    app: &AppHandle,
    language: &str,
    volumes: &[device::Volume],
    ready: bool,
) -> tauri::Result<tauri::tray::TrayIcon> {
    let icon = tauri::image::Image::from_bytes(TRAY_ICON_PNG)?;
    let menu = build_tray_menu(app, language, volumes, ready)?;
    let tooltip = tray_tooltip(language, volumes, ready);

    tauri::tray::TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .icon(icon)
        // Pure-black template glyph: macOS tints it for light/dark menu bars.
        .icon_as_template(true)
        .tooltip(tooltip)
        // Left click opens the menu (mounty behaviour); right click too.
        .show_menu_on_left_click(true)
        .on_menu_event(handle_menu_event)
        .build(app)
}

/// List volumes + probe dependencies and refresh. Runs off the calling thread
/// so a slow `diskutil` call never blocks the menu bar.
fn refresh_now(app: &AppHandle) {
    let handle = app.clone();
    std::thread::Builder::new()
        .name("ntfs-mac-refresh".to_string())
        .spawn(move || {
            let result = scan_state();
            let Some((volumes, ready)) = result else {
                return;
            };
            if let Err(err) = refresh_tray_menu(&handle, &volumes, ready) {
                eprintln!("ntfs-mac: tray refresh failed: {err}");
            }
        })
        .ok();
}

/// One read of the live system state. `None` when `diskutil` is unavailable.
fn scan_state() -> Option<(Vec<device::Volume>, bool)> {
    let volumes = device::list_volumes().ok()?;
    let ready = deps::report().ok().is_some_and(|report| report.ready);
    Some((volumes, ready))
}

/// Poll forever: re-scan every [`REFRESH_INTERVAL`], rebuild only on change.
fn start_tray_poller(app_handle: AppHandle) {
    std::thread::Builder::new()
        .name("ntfs-mac-poller".to_string())
        .spawn(move || {
            let mut last: Option<String> = None;
            loop {
                std::thread::sleep(REFRESH_INTERVAL);

                let Some((volumes, ready)) = scan_state() else {
                    continue;
                };
                let sig = menu_signature(&volumes, ready);
                if last.as_deref() == Some(sig.as_str()) {
                    continue;
                }
                last = Some(sig);

                if let Err(err) = refresh_tray_menu(&app_handle, &volumes, ready) {
                    eprintln!("ntfs-mac: poll refresh failed: {err}");
                }
            }
        })
        .ok();
}

// ---------------------------------------------------------------------------
// Menu-event dispatch
// ---------------------------------------------------------------------------

/// Volume-scoped tray action, decoded from the menu-item id.
#[derive(Debug, Clone)]
enum VolumeAction {
    Mount(String),
    Open(String),
    Unmount(String),
    Eject(String),
}

/// Decode a per-volume menu id. Owned `String`s so the result can be moved
/// into a thread without borrowing the event.
#[must_use]
fn volume_action(id: &str) -> Option<VolumeAction> {
    let make = |prefix: &str, f: fn(String) -> VolumeAction| {
        id.strip_prefix(prefix)
            .map(|device_id| f(device_id.to_string()))
    };
    make(VOL_MOUNT_PREFIX, VolumeAction::Mount)
        .or_else(|| make(VOL_OPEN_PREFIX, VolumeAction::Open))
        .or_else(|| make(VOL_UNMOUNT_PREFIX, VolumeAction::Unmount))
        .or_else(|| make(VOL_EJECT_PREFIX, VolumeAction::Eject))
}

/// Handle a menu-bar click. Runs on the main thread; every action that shells
/// out is moved onto a short-lived thread so the menu closes immediately.
fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id.as_ref().to_string();

    match id.as_str() {
        ID_QUIT => {
            app.exit(0);
            return;
        }
        ID_SHOW_WINDOW => {
            show_window(app);
            return;
        }
        ID_REFRESH => {
            refresh_now(app);
            return;
        }
        ID_INSTALL_DEPS => {
            let handle = app.clone();
            std::thread::spawn(move || {
                let note = run_install_dependencies(&handle);
                report(&handle, true, note);
            });
            return;
        }
        _ => {}
    }

    let Some(action) = volume_action(&id) else {
        return;
    };
    let handle = app.clone();
    std::thread::spawn(move || match action {
        VolumeAction::Mount(device_id) => mount_and_refresh(&handle, &device_id, false),
        VolumeAction::Open(device_id) => open_and_refresh(&handle, &device_id),
        VolumeAction::Unmount(device_id) => unmount_and_refresh(&handle, &device_id),
        VolumeAction::Eject(device_id) => eject_and_refresh(&handle, &device_id),
    });
}

/// Bring the main window back; no-op if none exists.
fn show_window(app: &AppHandle) {
    // `Manager::windows` sits behind the `unstable` feature, so enumerate
    // webview windows instead.
    if let Some((_, window)) = app.webview_windows().into_iter().next() {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Push an error (or success note) to the window so it can display it.
fn report(app: &AppHandle, ok: bool, message: String) {
    let _ = app.emit(
        EVENT_ERROR,
        serde_json::json!({ "ok": ok, "message": message }),
    );
}

fn mount_now(app: &AppHandle, device_id: &str, readonly: bool) -> Result<String, String> {
    if !is_device_id(device_id) {
        return Err(format!("非法的设备标识 {device_id}"));
    }
    let state = app.state::<AppState>();
    let config = state.config();
    let volumes = device::list_volumes().map_err(|err| err.to_string())?;
    let vol = find_volume(&volumes, device_id)?;
    let opts = mount::MountOptions {
        mount_point: None,
        readonly,
        extra_options: Vec::new(),
        force_ntfs3g: false,
    };
    mount::mount(vol, &opts, &config).map_err(|err| err.to_string())
}

fn mount_and_refresh(app: &AppHandle, device_id: &str, readonly: bool) {
    match mount_now(app, device_id, readonly) {
        Ok(point) => report(app, true, format!("已挂载到 {point}")),
        Err(msg) => report(app, false, msg),
    }
    refresh_now(app);
}

fn open_and_refresh(app: &AppHandle, device_id: &str) {
    let volumes = match device::list_volumes() {
        Ok(v) => v,
        Err(err) => {
            report(app, false, err.to_string());
            return;
        }
    };
    let vol = match find_volume(&volumes, device_id) {
        Ok(v) => v,
        Err(msg) => {
            report(app, false, msg);
            return;
        }
    };
    let target = vol
        .mount_point
        .clone()
        .unwrap_or_else(|| format!("/dev/{device_id}"));
    let reveal = target.starts_with("/dev/");
    let args: Vec<&str> = if reveal {
        vec!["-R", &target]
    } else {
        vec![&target]
    };
    match run_cmd("open", &args) {
        Ok(_) => report(app, true, format!("已在 Finder 中打开 {target}")),
        Err(msg) => report(app, false, format!("打开失败：{msg}")),
    }
    refresh_now(app);
}

fn unmount_and_refresh(app: &AppHandle, device_id: &str) {
    if !is_device_id(device_id) {
        report(app, false, format!("非法的设备标识 {device_id}"));
        return;
    }
    match mount::unmount(device_id).map_err(|err| err.to_string()) {
        Ok(()) => report(app, true, format!("已卸载 {device_id}")),
        Err(msg) => report(app, false, msg),
    }
    refresh_now(app);
}

fn eject_and_refresh(app: &AppHandle, device_id: &str) {
    if !is_device_id(device_id) {
        report(app, false, format!("非法的设备标识 {device_id}"));
        return;
    }
    let volumes = match device::list_volumes() {
        Ok(v) => v,
        Err(err) => {
            report(app, false, err.to_string());
            return;
        }
    };
    let vol = match find_volume(&volumes, device_id) {
        Ok(v) => v,
        Err(msg) => {
            report(app, false, msg);
            return;
        }
    };
    let target = eject_target(vol);
    let path = format!("/dev/{target}");
    match run_cmd("diskutil", &["eject", &path]) {
        Ok(_) => report(app, true, format!("已弹出 {path}")),
        Err(msg) => report(app, false, format!("弹出失败：{msg}")),
    }
    refresh_now(app);
}

/// Assemble the brew script and hand it to Terminal. Shared by the command and
/// the menu-bar `安装依赖` entry.
fn run_install_dependencies(app: &AppHandle) -> String {
    let report_ = match deps::report() {
        Ok(r) => r,
        Err(err) => return format!("检查依赖失败：{err}"),
    };
    let missing: Vec<&deps::DepStatus> = report_.deps.iter().filter(|d| !d.present).collect();
    if missing.is_empty() {
        return labels(&app.state::<AppState>().language())
            .ready
            .to_string();
    }

    let script = brew_install_script(&missing);
    let escaped = escape_applescript(&script);
    let applescript =
        format!("tell application \"Terminal\"\n  activate\n  do script \"{escaped}\"\nend tell");
    let opts = runner::RunOptions {
        timeout: Some(Duration::from_secs(15)),
        ..Default::default()
    };
    match runner::run("osascript", &[&applescript], &opts) {
        Ok(out) if out.success() => {
            let n = missing.len();
            format!("已在 Terminal 中启动安装命令（{n} 项）")
        }
        Ok(out) => format!("Terminal 启动失败：{}", out.stderr.trim()),
        Err(err) => format!("无法打开 Terminal：{err}"),
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// List every NTFS volume currently visible to macOS.
#[tauri::command]
fn list_volumes() -> tauri::Result<Vec<VolumeInfo>> {
    let volumes = device::list_volumes().map_err(e)?;
    Ok(volumes
        .into_iter()
        .map(|v| VolumeInfo {
            display_label: v.display_label(),
            device_identifier: v.device_identifier,
            volume_name: v.volume_name,
            media_type: v.media_type,
            size_bytes: v.size_bytes,
            size_pretty: v.size_pretty,
            mounted: v.mounted,
            mount_point: v.mount_point,
            parent_disk: v.parent_disk,
            location: v.location,
        })
        .collect())
}

/// Probe every dependency (`ntfs-3g`, FUSE driver, `diskutil`, …).
#[tauri::command]
fn check_deps() -> tauri::Result<DepReportInfo> {
    let report = deps::report().map_err(e)?;
    let deps_info: Vec<DepInfo> = report
        .deps
        .iter()
        .map(|d| DepInfo {
            name: d.name.clone(),
            present: d.present,
            path: d.path.as_ref().map(|p| p.to_string_lossy().to_string()),
            install_hint: d.install_hint.clone(),
        })
        .collect();
    Ok(DepReportInfo {
        deps: deps_info,
        ready: report.ready,
        arch: report.arch,
        macos_version: report.macos_version,
    })
}

/// Mount a volume. `readonly` gives a safer read-only mount.
#[tauri::command]
fn mount_volume(app: AppHandle, device_id: String, readonly: bool) -> tauri::Result<String> {
    mount_now(&app, &device_id, readonly).map_err(e)
}

/// Safely unmount a mounted volume.
#[tauri::command]
fn unmount_volume(device_id: String) -> tauri::Result<()> {
    if !is_device_id(&device_id) {
        return Err(e(format!("非法的设备标识 {device_id}")));
    }
    mount::unmount(&device_id).map_err(e)
}

/// Format a volume to NTFS. Refuses to run while mounted.
#[tauri::command]
fn format_volume(device_id: String, label: Option<String>, quick: bool) -> tauri::Result<()> {
    if !is_device_id(&device_id) {
        return Err(e(format!("非法的设备标识 {device_id}")));
    }
    let config = Config::default();
    let volumes = device::list_volumes().map_err(e)?;
    let vol = find_volume(&volumes, &device_id).map_err(e)?;
    if vol.mounted {
        return Err(e("卷已挂载，请先卸载"));
    }
    let opts = format::FormatOptions {
        label: label.unwrap_or_default(),
        quick,
        sector_size: 4096,
        cluster_size: 4096,
        extra_args: Vec::new(),
    };
    let token = DestructiveToken::new(&vol.device_identifier, &vol.volume_name);
    format::format(vol, &opts, &token, &config).map_err(e)
}

/// Run `ntfsfix` (or `fsck_ntfs`) against an unmounted volume.
#[tauri::command]
fn fix_volume(device_id: String, use_fsck: bool) -> tauri::Result<()> {
    if !is_device_id(&device_id) {
        return Err(e(format!("非法的设备标识 {device_id}")));
    }
    let volumes = device::list_volumes().map_err(e)?;
    let vol = find_volume(&volumes, &device_id).map_err(e)?;
    let opts = fix::FixOptions {
        tool: if use_fsck {
            fix::FixTool::FsckNtfs
        } else {
            fix::FixTool::Ntfsfix
        },
        drop_dirty_flag: true,
    };
    fix::fix_with_opts(vol, &opts).map_err(e)
}

/// Reveal a volume in Finder — its mount point when mounted, else `/dev/<id>`.
#[tauri::command]
fn open_in_finder(app: AppHandle, device_id: String) -> tauri::Result<String> {
    if !is_device_id(&device_id) {
        return Err(e(format!("非法的设备标识 {device_id}")));
    }
    let volumes = device::list_volumes().map_err(e)?;
    let vol = find_volume(&volumes, &device_id).map_err(e)?;
    let target = vol
        .mount_point
        .clone()
        .unwrap_or_else(|| format!("/dev/{device_id}"));
    let reveal = target.starts_with("/dev/");
    let args: Vec<&str> = if reveal {
        vec!["-R", &target]
    } else {
        vec![&target]
    };
    run_cmd("open", &args).map_err(e)?;
    refresh_now(&app);
    Ok(target)
}

/// Physically eject the whole drive that contains the volume.
#[tauri::command]
fn eject_volume(app: AppHandle, device_id: String) -> tauri::Result<()> {
    if !is_device_id(&device_id) {
        return Err(e(format!("非法的设备标识 {device_id}")));
    }
    let volumes = device::list_volumes().map_err(e)?;
    let vol = find_volume(&volumes, &device_id).map_err(e)?;
    let target = eject_target(vol);
    let path = format!("/dev/{target}");
    run_cmd("diskutil", &["eject", &path]).map_err(e)?;
    refresh_now(&app);
    Ok(())
}

/// Kick off `brew install …` in Terminal for every missing dependency.
#[tauri::command]
fn install_dependencies(app: AppHandle) -> tauri::Result<String> {
    Ok(run_install_dependencies(&app))
}

/// Package metadata for the about block.
#[tauri::command]
fn app_info() -> tauri::Result<AppInfo> {
    Ok(AppInfo {
        name: "ntfs-mac".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: env!("CARGO_PKG_DESCRIPTION").to_string(),
        repository: env!("CARGO_PKG_REPOSITORY").to_string(),
        license: env!("CARGO_PKG_LICENSE").to_string(),
        authors: env!("CARGO_PKG_AUTHORS").to_string(),
    })
}

/// Switch UI language. Updates the window and rebuilds the tray labels.
#[tauri::command]
fn set_language(state: State<AppState>, app: AppHandle, language: String) -> tauri::Result<()> {
    let lang = if language == "en" { "en" } else { "zh" };
    let mut guard = state
        .language
        .lock()
        .map_err(|_| anyhow::anyhow!("状态锁已损坏"))?;
    *guard = lang.to_string();
    drop(guard);
    refresh_now(&app);
    Ok(())
}

/// Force an immediate re-scan and menu rebuild (frontend refresh button).
#[tauri::command]
fn refresh_menu(app: AppHandle) -> tauri::Result<()> {
    refresh_now(&app);
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // A second launch (double-clicking the icon again, or a stale process
        // left over from a previous install) must not open a second window with
        // a second tray icon — bring the existing window forward instead.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_window(app);
        }))
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let language = handle.state::<AppState>().language();
            let (volumes, ready) = scan_state().unwrap_or_default();
            install_tray(&handle, &language, &volumes, ready)?;
            start_tray_poller(handle);
            Ok(())
        })
        // Closing the window must NOT quit — the tray icon is the real host.
        // 退出 happens only through the menu-bar `quit` entry.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_volumes,
            check_deps,
            mount_volume,
            unmount_volume,
            format_volume,
            fix_volume,
            open_in_finder,
            eject_volume,
            install_dependencies,
            app_info,
            set_language,
            refresh_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_volume(id: &str, mounted: bool) -> device::Volume {
        device::Volume {
            device_identifier: id.to_string(),
            volume_name: "Test".to_string(),
            media_type: "com.microsoft.ntfs".to_string(),
            uuid: None,
            size_bytes: 1_000_000_000,
            mounted,
            mount_point: None,
            parent_disk: Some("disk2".to_string()),
            size_pretty: "931.5 MiB".to_string(),
            location: "external".to_string(),
            contents: Some("GUID_partition_scheme".to_string()),
        }
    }

    #[test]
    fn whole_disk_splits_partition_suffix() {
        assert_eq!(whole_disk("disk2s2"), Some("disk2".to_string()));
        assert_eq!(whole_disk("disk10s2"), Some("disk10".to_string()));
        assert_eq!(whole_disk("disk12s15"), Some("disk12".to_string()));
    }

    #[test]
    fn whole_disk_accepts_bare_disk() {
        assert_eq!(whole_disk("disk2"), Some("disk2".to_string()));
        assert_eq!(whole_disk("disk"), Some("disk".to_string()));
    }

    #[test]
    fn whole_disk_rejects_garbage() {
        assert_eq!(whole_disk(""), None);
        assert_eq!(whole_disk("/dev/disk2s2"), None);
        assert_eq!(whole_disk("sdisk2s2"), None);
        assert_eq!(whole_disk("diskXs2"), None);
        assert_eq!(whole_disk("disk2s"), None);
        // Two separators are malformed even though every char is a digit or `s`.
        assert_eq!(whole_disk("disk2s2s3"), None);
        // Trailing junk after the partition number.
        assert_eq!(whole_disk("disk2s2x"), None);
        assert_eq!(whole_disk("disk2s2 "), None);
        // Separator with no partition number digits.
        assert_eq!(whole_disk("disk2sX"), None);
    }

    #[test]
    fn device_id_validation_is_strict() {
        assert!(is_device_id("disk2s2"));
        assert!(is_device_id("disk2"));
        assert!(is_device_id("disk10s2"));
        assert!(is_device_id("disk10"));
        assert!(is_device_id("disk0s1"));
        assert!(!is_device_id(""));
        assert!(!is_device_id("disk"));
        assert!(!is_device_id("/dev/disk2s2"));
        assert!(!is_device_id("disk-a"));
        assert!(!is_device_id("disk2s"));
        assert!(!is_device_id("disk2s2s3"));
        assert!(!is_device_id("Disk2s2"));
        assert!(!is_device_id("disk2sx"));
    }

    #[test]
    fn eject_target_prefers_parent_disk() {
        let vol = fake_volume("disk2s2", false);
        assert_eq!(eject_target(&vol), "disk2");
    }

    #[test]
    fn eject_target_falls_back_to_derivation() {
        let mut vol = fake_volume("disk7s2", false);
        vol.parent_disk = None;
        assert_eq!(eject_target(&vol), "disk7");
    }

    #[test]
    fn eject_target_last_resort_is_self() {
        let mut vol = fake_volume("disk9", false);
        vol.parent_disk = None;
        assert_eq!(eject_target(&vol), "disk9");
    }

    #[test]
    fn labels_default_to_chinese() {
        assert!(labels("zh").empty.starts_with("未"));
        assert!(labels("de").empty.starts_with("未"));
        assert!(labels("").empty.starts_with("未"));
        assert!(labels("en").empty.starts_with("No"));
    }

    #[test]
    fn labels_are_synced_between_languages() {
        // Every English label must be non-empty; a blank would render as a
        // blank menu row, which is exactly the defect this rewrite fixes.
        let en = labels("en");
        let zh = labels("zh");
        assert!(!en.header.is_empty() && !en.empty.is_empty());
        assert!(!en.mount.is_empty() && !en.unmount.is_empty());
        assert!(!en.refresh.is_empty() && !en.quit.is_empty());
        assert!(!zh.header.is_empty() && !zh.empty.is_empty());
        assert!(!zh.mount.is_empty() && !zh.unmount.is_empty());
        assert!(!zh.refresh.is_empty() && !zh.quit.is_empty());
    }

    #[test]
    fn menu_signature_detects_mount_state_and_deps() {
        let a = menu_signature(&[fake_volume("disk2s2", false)], true);
        let b = menu_signature(&[fake_volume("disk2s2", true)], true);
        let c = menu_signature(&[fake_volume("disk2s2", false)], false);
        assert_ne!(a, b, "mount state must change the signature");
        assert_ne!(a, c, "dependency readiness must change the signature");
        assert_eq!(a, menu_signature(&[fake_volume("disk2s2", false)], true));
    }

    #[test]
    fn menu_signature_detects_new_volume() {
        let one = menu_signature(&[fake_volume("disk2s2", false)], true);
        let two = menu_signature(
            &[fake_volume("disk2s2", false), fake_volume("disk3s2", true)],
            true,
        );
        assert_ne!(one, two);
    }

    #[test]
    fn tooltip_reports_mounted_count() {
        let vols = vec![fake_volume("disk2s2", true), fake_volume("disk3s2", false)];
        let zh = tray_tooltip("zh", &vols, true);
        assert!(zh.contains("2 个 NTFS 卷"), "got: {zh}");
        assert!(zh.contains("已挂载 1 个"), "got: {zh}");

        let en = tray_tooltip("en", &vols, true);
        assert!(en.contains("2 volume(s)"), "got: {en}");
        assert!(en.contains("1 mounted"), "got: {en}");
    }

    #[test]
    fn tooltip_degrades_to_empty_state() {
        assert!(tray_tooltip("zh", &[], true).contains("依赖已就绪"));
        assert!(tray_tooltip("zh", &[], false).contains("依赖缺失"));
    }

    #[test]
    fn brew_script_dedups_and_drops_comments() {
        let a = deps::DepStatus {
            name: "ntfs-3g".to_string(),
            present: false,
            path: None,
            install_hint: Some("brew install ntfs-3g".to_string()),
        };
        let b = deps::DepStatus {
            name: "fuse-driver".to_string(),
            present: false,
            path: None,
            install_hint: Some("brew install --cask macfuse".to_string()),
        };
        let c = deps::DepStatus {
            name: "dupe".to_string(),
            present: false,
            path: None,
            install_hint: Some("brew install ntfs-3g".to_string()),
        };
        let script = brew_install_script(&[&a, &b, &c]);
        assert!(script.starts_with("set -e"));
        assert!(script.contains("brew install ntfs-3g"));
        assert!(script.contains("brew install --cask macfuse"));
        assert_eq!(
            script.matches("ntfs-3g").count(),
            1,
            "hints must be deduped"
        );
    }

    #[test]
    fn brew_script_reports_when_nothing_actionable() {
        let a = deps::DepStatus {
            name: "diskutil".to_string(),
            present: false,
            path: None,
            install_hint: None,
        };
        let script = brew_install_script(&[&a]);
        assert!(script.starts_with("echo"), "got: {script}");
    }

    #[test]
    fn escape_applescript_handles_quotes_and_newlines() {
        assert_eq!(escape_applescript("a\"b"), "a\\\"b");
        assert_eq!(escape_applescript("a\\b"), "a\\\\b");
        assert_eq!(escape_applescript("a\nb"), "a\\nb");
        assert_eq!(escape_applescript("中文"), "中文");
    }
}
