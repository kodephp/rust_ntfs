#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

use tauri::State;

use ntfs_mac_core::{Config, DestructiveToken, deps, device, fix, format, mount};

#[derive(Default)]
struct AppState {
    config: Option<Config>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DepInfo {
    name: String,
    present: bool,
    path: Option<String>,
    install_hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DepReportInfo {
    deps: Vec<DepInfo>,
    ready: bool,
    arch: String,
    macos_version: Option<String>,
}

#[tauri::command]
fn list_volumes() -> tauri::Result<Vec<VolumeInfo>> {
    let volumes = device::list_volumes().map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))?;
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

#[tauri::command]
fn check_deps() -> tauri::Result<DepReportInfo> {
    let report = deps::report().map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))?;
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

#[tauri::command]
fn mount_volume(
    state: State<AppState>,
    device_id: String,
    readonly: bool,
) -> tauri::Result<String> {
    let cfg = state.config.clone().unwrap_or_default();
    let volumes = device::list_volumes().map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))?;
    let vol = volumes
        .into_iter()
        .find(|v| v.device_identifier == device_id)
        .ok_or_else(|| tauri::Error::from(anyhow::anyhow!("volume '{}' not found", device_id)))?;
    let opts = mount::MountOptions {
        mount_point: None,
        readonly,
        extra_options: Vec::new(),
        force_ntfs3g: false,
    };
    mount::mount(&vol, &opts, &cfg).map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))
}

#[tauri::command]
fn unmount_volume(device_id: String) -> tauri::Result<()> {
    mount::unmount(&device_id).map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))
}

#[tauri::command]
fn format_volume(device_id: String, label: Option<String>, quick: bool) -> tauri::Result<()> {
    let cfg = Config::default();
    let volumes = device::list_volumes().map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))?;
    let vol = volumes
        .into_iter()
        .find(|v| v.device_identifier == device_id)
        .ok_or_else(|| tauri::Error::from(anyhow::anyhow!("volume '{}' not found", device_id)))?;
    if vol.mounted {
        return Err(tauri::Error::from(anyhow::anyhow!(
            "volume is mounted; unmount first"
        )));
    }
    let label_str = label.unwrap_or_default();
    let opts = format::FormatOptions {
        label: label_str,
        quick,
        sector_size: 4096,
        cluster_size: 4096,
        extra_args: Vec::new(),
    };
    let token = DestructiveToken::new(&vol.device_identifier, &vol.volume_name);
    format::format(&vol, &opts, &token, &cfg).map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))
}

#[tauri::command]
fn fix_volume(device_id: String, use_fsck: bool) -> tauri::Result<()> {
    let volumes = device::list_volumes().map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))?;
    let vol = volumes
        .into_iter()
        .find(|v| v.device_identifier == device_id)
        .ok_or_else(|| tauri::Error::from(anyhow::anyhow!("volume '{}' not found", device_id)))?;
    let opts = fix::FixOptions {
        tool: if use_fsck {
            fix::FixTool::FsckNtfs
        } else {
            fix::FixTool::Ntfsfix
        },
        drop_dirty_flag: true,
    };
    fix::fix_with_opts(&vol, &opts).map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))
}

#[tauri::command]
fn get_config() -> tauri::Result<serde_json::Value> {
    let cfg = Config::load().map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))?;
    serde_json::to_value(&cfg).map_err(|e| tauri::Error::from(anyhow::anyhow!(e)))
}

/// Reveal the sponsorship QR code image in Finder.
///
/// In debug builds the asset lives in the source tree; in release builds
/// it is bundled inside the .app. We try several candidate paths so the
/// button works in both contexts.
#[tauri::command]
fn reveal_sponsor_qr() -> tauri::Result<String> {
    let candidates = [
        // Debug: source tree (relative to crate)
        concat!(env!("CARGO_MANIFEST_DIR"), "/../src/assets/sponsor-qr.svg"),
        // Release: .app bundle resources
        "/Applications/ntfs-mac.app/Contents/Resources/assets/sponsor-qr.svg",
        // Release: local bundle (next to binary)
        "assets/sponsor-qr.svg",
    ];
    for path in &candidates {
        let p = std::path::Path::new(path);
        if p.exists() {
            let path_str = p.to_string_lossy().to_string();
            // `open -R` reveals the file in Finder (macOS).
            let _ = std::process::Command::new("open")
                .args(["-R", &path_str])
                .spawn();
            return Ok(path_str);
        }
    }
    Err(tauri::Error::from(anyhow::anyhow!(
        "sponsor-qr.svg not found. Place your QR code at src/assets/sponsor-qr.svg"
    )))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            list_volumes,
            check_deps,
            mount_volume,
            unmount_volume,
            format_volume,
            fix_volume,
            get_config,
            reveal_sponsor_qr,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
