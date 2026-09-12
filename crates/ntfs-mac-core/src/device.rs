//! NTFS volume discovery via `diskutil list -plist` + `mount`.
//!
//! `diskutil list -plist` emits a nested plist document whose top-level
//! is a **dict** with an `AllDisksAndPartitions` array. Each entry is a
//! whole-disk record containing a nested `Partitions` array. Partitions
//! carry only `Content`, `DeviceIdentifier`, `DiskUUID`, and `Size` —
//! no mount point or volume name. Those are recovered by cross-referencing
//! the `mount` command output, which is cheap (one subprocess) and avoids
//! N+1 `diskutil info` calls.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::runner::{RunOptions, run};

/// A discovered NTFS volume/partition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Volume {
    /// `disk2s2` style identifier.
    pub device_identifier: String,
    /// Volume name; may be empty for unlabelled volumes.
    pub volume_name: String,
    /// Media type: `com.microsoft.ntfs`, `Microsoft Basic Data`, …
    pub media_type: String,
    /// Logical volume UUID (if the plist provided one).
    pub uuid: Option<String>,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Whether the volume is currently mounted.
    pub mounted: bool,
    /// Mount point when mounted (e.g. `/Volumes/MyData`).
    pub mount_point: Option<String>,
    /// Parent disk identifier (e.g. `disk2`).
    pub parent_disk: Option<String>,
    /// Human-readable size (e.g. `931.5 GiB`).
    pub size_pretty: String,
    /// Location: `internal` or `external`.
    pub location: String,
    /// Whole-disk contents (GUID_partition_scheme / Apple_partition_scheme / …).
    pub contents: Option<String>,
}

impl Volume {
    /// Short label for CLI display: prefer volume_name, fall back to
    /// device_identifier.
    #[must_use]
    pub fn display_label(&self) -> String {
        if self.volume_name.trim().is_empty() {
            self.device_identifier.clone()
        } else {
            format!("{} ({})", self.volume_name, self.device_identifier)
        }
    }
}

/// NTFS-related partition Content values accepted by [`is_ntfs_content`].
const NTFS_CONTENTS: &[&str] = &[
    "com.microsoft.ntfs",
    "Microsoft Basic Data",
    "Microsoft Reserved",
    "Microsoft Reserved Code",
];

/// Return `true` when `content` identifies an NTFS-family partition.
#[must_use]
fn is_ntfs_content(content: &str) -> bool {
    NTFS_CONTENTS.contains(&content)
}

/// List all NTFS volumes visible to macOS.
pub fn list_volumes() -> Result<Vec<Volume>> {
    let plist_out = run(
        "diskutil",
        &["list", "-plist"],
        &RunOptions {
            timeout: Some(std::time::Duration::from_secs(15)),
            ..Default::default()
        },
    )?;
    if !plist_out.success() {
        return Err(Error::CommandFailed {
            cmd: "diskutil list -plist".into(),
            status: plist_out.status,
            stderr: plist_out.stderr,
            io: None,
        });
    }

    let mut volumes = parse_diskutil_plist(&plist_out.stdout)?;

    // Enrich with mount info from the `mount` command.
    let mount_out = run(
        "mount",
        &[],
        &RunOptions {
            timeout: Some(std::time::Duration::from_secs(5)),
            ..Default::default()
        },
    );
    if let Ok(out) = mount_out {
        if out.success() {
            let mount_map = parse_mount_output(&out.stdout);
            enrich_with_mounts(&mut volumes, &mount_map);
        }
    }

    Ok(volumes)
}

/// Parse the plist output of `diskutil list -plist`.
///
/// The real top-level is a **dict** containing `AllDisksAndPartitions`
/// (array of whole-disk dicts, each with a nested `Partitions` array).
///
/// This is a pure function so it can be unit-tested without spawning
/// diskutil. Returned volumes have `mounted = false` and `mount_point =
/// None`; call [`enrich_with_mounts`] to fill those in.
pub fn parse_diskutil_plist(text: &str) -> Result<Vec<Volume>> {
    let root = plist::Value::from_reader(Cursor::new(text.as_bytes())).map_err(|e| {
        Error::CommandFailed {
            cmd: "plist::from_reader".into(),
            status: -1,
            stderr: e.to_string(),
            io: None,
        }
    })?;

    let root_dict = match root.as_dictionary() {
        Some(d) => d,
        None => {
            return Err(Error::CommandFailed {
                cmd: "plist::from_reader".into(),
                status: -1,
                stderr: "expected top-level dict from diskutil list -plist".into(),
                io: None,
            });
        }
    };

    let all_parts = match root_dict
        .get("AllDisksAndPartitions")
        .and_then(plist::Value::as_array)
    {
        Some(a) => a,
        None => {
            return Err(Error::CommandFailed {
                cmd: "plist::from_reader".into(),
                status: -1,
                stderr: "missing AllDisksAndPartitions in diskutil plist".into(),
                io: None,
            });
        }
    };

    let mut volumes = Vec::new();
    for disk_val in all_parts {
        let disk_dict = match disk_val.as_dictionary() {
            Some(d) => d,
            None => continue,
        };
        let content = disk_dict
            .get("Content")
            .and_then(plist::Value::as_string)
            .map(str::to_string);
        let parent_disk = disk_dict
            .get("DeviceIdentifier")
            .and_then(plist::Value::as_string)
            .map(str::to_string);
        let os_internal = disk_dict
            .get("OSInternal")
            .and_then(plist::Value::as_boolean)
            .unwrap_or(true);
        let location = if os_internal { "internal" } else { "external" };

        let partitions = match disk_dict.get("Partitions").and_then(plist::Value::as_array) {
            Some(a) => a,
            None => continue,
        };

        for part_val in partitions {
            let part_dict = match part_val.as_dictionary() {
                Some(d) => d,
                None => continue,
            };
            let Some(part_content) = part_dict.get("Content").and_then(plist::Value::as_string)
            else {
                continue;
            };
            if !is_ntfs_content(part_content) {
                continue;
            }

            let Some(device_identifier) = part_dict
                .get("DeviceIdentifier")
                .and_then(plist::Value::as_string)
                .map(str::to_string)
            else {
                continue;
            };

            let uuid = part_dict
                .get("DiskUUID")
                .and_then(plist::Value::as_string)
                .map(str::to_string);

            let volume_name = part_dict
                .get("VolumeName")
                .and_then(plist::Value::as_string)
                .map(str::to_string)
                .unwrap_or_default();

            let size_bytes = part_dict
                .get("Size")
                .and_then(plist::Value::as_signed_integer)
                .map(|i| i as u64)
                .unwrap_or(0);

            volumes.push(Volume {
                device_identifier,
                volume_name,
                media_type: part_content.to_string(),
                uuid,
                size_bytes,
                mounted: false,
                mount_point: None,
                parent_disk: parent_disk.clone(),
                size_pretty: pretty_size(size_bytes),
                location: location.to_string(),
                contents: content.clone(),
            });
        }
    }
    Ok(volumes)
}

/// Parse the output of the `mount` command into a map of
/// device identifier → mount point.
///
/// Each line has the form:
/// `/dev/disk2s2 on /Volumes/MyData (ntfs, local, …)`
#[must_use]
pub fn parse_mount_output(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let Some(space_pos) = line.find(" on ") else {
            continue;
        };
        let dev_part = &line[..space_pos];
        let rest = &line[space_pos + 4..];
        // Dev path looks like `/dev/disk2s2`; strip the prefix.
        let device_id = match dev_part.strip_prefix("/dev/") {
            Some(id) => id.trim().to_string(),
            None => continue,
        };
        if device_id.is_empty() {
            continue;
        }
        // Mount point is the first whitespace-delimited token.
        let mount_point = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if mount_point.is_empty() {
            continue;
        }
        map.insert(device_id, mount_point);
    }
    map
}

/// Fill in `mounted`, `mount_point`, and `volume_name` on each volume
/// using the device→mount-point map produced by [`parse_mount_output`].
pub fn enrich_with_mounts(volumes: &mut [Volume], mount_map: &HashMap<String, String>) {
    for vol in volumes.iter_mut() {
        if let Some(mp) = mount_map.get(&vol.device_identifier) {
            vol.mounted = true;
            vol.mount_point = Some(mp.clone());
            // Derive volume name from mount-point basename when the
            // plist did not provide one (it never does for partitions).
            if vol.volume_name.trim().is_empty() {
                if let Some(name) = Path::new(mp).file_name().and_then(|s| s.to_str()) {
                    if !name.is_empty() {
                        vol.volume_name = name.to_string();
                    }
                }
            }
        }
    }
}

/// Format a byte count as `1.2 GiB`.
#[must_use]
pub fn pretty_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    const PB: u64 = TB * 1024;

    let (val, unit) = if bytes >= PB {
        (bytes as f64 / PB as f64, "PiB")
    } else if bytes >= TB {
        (bytes as f64 / TB as f64, "TiB")
    } else if bytes >= GB {
        (bytes as f64 / GB as f64, "GiB")
    } else if bytes >= MB {
        (bytes as f64 / MB as f64, "MiB")
    } else if bytes >= KB {
        (bytes as f64 / KB as f64, "KiB")
    } else {
        return format!("{bytes} B");
    };
    format!("{val:.1} {unit}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real-world-shaped `diskutil list -plist` output: top-level dict
    /// with `AllDisksAndPartitions`, partitions carry only
    /// Content/DeviceIdentifier/DiskUUID/Size.
    const REALISTIC_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AllDisks</key>
	<array>
		<string>disk2</string>
		<string>disk2s1</string>
		<string>disk2s2</string>
		<string>disk2s3</string>
	</array>
	<key>AllDisksAndPartitions</key>
	<array>
		<dict>
			<key>Content</key>
			<string>GUID_partition_scheme</string>
			<key>DeviceIdentifier</key>
			<string>disk2</string>
			<key>OSInternal</key>
			<false/>
			<key>Size</key>
			<integer>1000000000000</integer>
			<key>Partitions</key>
			<array>
				<dict>
					<key>Content</key>
					<string>Microsoft Reserved</string>
					<key>DeviceIdentifier</key>
					<string>disk2s1</string>
					<key>Size</key>
					<integer>104857600</integer>
				</dict>
				<dict>
					<key>Content</key>
					<string>com.microsoft.ntfs</string>
					<key>DeviceIdentifier</key>
					<string>disk2s2</string>
					<key>DiskUUID</key>
					<string>ABC-123</string>
					<key>Size</key>
					<integer>999000000000</integer>
				</dict>
				<dict>
					<key>Content</key>
					<string>Apple_APFS</string>
					<key>DeviceIdentifier</key>
					<string>disk2s3</string>
					<key>Size</key>
					<integer>1000000000</integer>
				</dict>
			</array>
		</dict>
		<dict>
			<key>Content</key>
			<string>GUID_partition_scheme</string>
			<key>DeviceIdentifier</key>
			<string>disk5</string>
			<key>OSInternal</key>
			<true/>
			<key>Size</key>
			<integer>500277792768</integer>
			<key>Partitions</key>
			<array>
				<dict>
					<key>Content</key>
					<string>Apple_APFS</string>
					<key>DeviceIdentifier</key>
					<string>disk5s1</string>
					<key>Size</key>
					<integer>500000000000</integer>
				</dict>
			</array>
		</dict>
	</array>
	<key>VolumesFromDisks</key>
	<array>
		<string>MyData</string>
	</array>
	<key>WholeDisks</key>
	<array>
		<string>disk2</string>
		<string>disk5</string>
	</array>
</dict>
</plist>
"#;

    #[test]
    fn pretty_size_units() {
        assert_eq!(pretty_size(0), "0 B");
        assert_eq!(pretty_size(512), "512 B");
        assert_eq!(pretty_size(2048), "2.0 KiB");
        assert_eq!(pretty_size(3 * 1024 * 1024), "3.0 MiB");
        assert_eq!(pretty_size(2 * 1024usize.pow(3) as u64), "2.0 GiB");
        assert_eq!(pretty_size(1024usize.pow(4) as u64), "1.0 TiB");
    }

    #[test]
    fn parse_synthetic_plist_finds_ntfs_partitions() {
        let vols = parse_diskutil_plist(REALISTIC_PLIST).unwrap();
        // Microsoft Reserved + com.microsoft.ntfs = 2 NTFS volumes.
        assert_eq!(vols.len(), 2);
        let ntfs = vols
            .iter()
            .find(|v| v.media_type == "com.microsoft.ntfs")
            .unwrap();
        assert_eq!(ntfs.device_identifier, "disk2s2");
        assert_eq!(ntfs.uuid.as_deref(), Some("ABC-123"));
        assert_eq!(ntfs.parent_disk.as_deref(), Some("disk2"));
        assert_eq!(ntfs.location, "external");
        assert!(ntfs.size_pretty.contains("GiB"));
        assert_eq!(ntfs.contents.as_deref(), Some("GUID_partition_scheme"));
        // Not yet enriched with mount info.
        assert!(!ntfs.mounted);
        assert!(ntfs.mount_point.is_none());
        assert!(ntfs.volume_name.is_empty());
    }

    #[test]
    fn parse_mount_output_extracts_device_to_mount_point() {
        let mount = r#"/dev/disk3s1s1 on / (apfs, sealed, local, read-only, journaled)
/dev/disk3s6 on /System/Volumes/VM (apfs, local, noexec, journaled, noatime, nobrowse)
/dev/disk2s2 on /Volumes/MyData (ntfs, local, noowners, nobrowse)
devfs on /dev (devfs, local, nobrowse)
map auto_home on /System/Volumes/Data/home (autofs, automounted, nobrowse)
"#;
        let map = parse_mount_output(mount);
        assert_eq!(map.get("disk3s1s1").map(String::as_str), Some("/"));
        assert_eq!(
            map.get("disk2s2").map(String::as_str),
            Some("/Volumes/MyData")
        );
        // Non-/dev entries are ignored.
        assert!(!map.contains_key("devfs"));
        assert!(!map.contains_key("auto_home"));
        assert!(!map.contains_key("map"));
    }

    #[test]
    fn enrich_with_mounts_fills_mounted_and_volume_name() {
        let vols = parse_diskutil_plist(REALISTIC_PLIST).unwrap();
        let mount_map = HashMap::from([
            ("disk2s2".to_string(), "/Volumes/MyData".to_string()),
            ("disk2s1".to_string(), "/Volumes/RECOVERY".to_string()),
        ]);
        let mut vols = vols;
        enrich_with_mounts(&mut vols, &mount_map);

        let ntfs = vols
            .iter()
            .find(|v| v.media_type == "com.microsoft.ntfs")
            .unwrap();
        assert!(ntfs.mounted);
        assert_eq!(ntfs.mount_point.as_deref(), Some("/Volumes/MyData"));
        assert_eq!(ntfs.volume_name, "MyData");

        // The Microsoft Reserved partition was also enriched.
        let reserved = vols
            .iter()
            .find(|v| v.media_type == "Microsoft Reserved")
            .unwrap();
        assert!(reserved.mounted);
        assert_eq!(reserved.volume_name, "RECOVERY");
    }

    #[test]
    fn enrich_without_match_leaves_unmounted() {
        let mut vols = parse_diskutil_plist(REALISTIC_PLIST).unwrap();
        let empty_map = HashMap::new();
        enrich_with_mounts(&mut vols, &empty_map);
        for v in &vols {
            assert!(!v.mounted);
            assert!(v.mount_point.is_none());
            assert!(v.volume_name.is_empty());
        }
    }

    #[test]
    fn parse_plist_rejects_non_dict_root() {
        let text = r#"<plist version="1.0"><array><string>foo</string></array></plist>"#;
        let r = parse_diskutil_plist(text);
        assert!(r.is_err());
    }

    #[test]
    fn parse_plist_rejects_missing_all_disks_and_partitions() {
        let text =
            r#"<plist version="1.0"><dict><key>Foo</key><string>bar</string></dict></plist>"#;
        let r = parse_diskutil_plist(text);
        assert!(r.is_err());
    }

    #[test]
    fn display_label_prefers_volume_name() {
        let v = Volume {
            device_identifier: "disk2s2".into(),
            volume_name: "MyData".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 0,
            mounted: false,
            mount_point: None,
            parent_disk: None,
            size_pretty: "0 B".into(),
            location: "external".into(),
            contents: None,
        };
        assert_eq!(v.display_label(), "MyData (disk2s2)");
    }

    #[test]
    fn display_label_falls_back_to_identifier() {
        let v = Volume {
            device_identifier: "disk2s2".into(),
            volume_name: "".into(),
            media_type: "com.microsoft.ntfs".into(),
            uuid: None,
            size_bytes: 0,
            mounted: false,
            mount_point: None,
            parent_disk: None,
            size_pretty: "0 B".into(),
            location: "external".into(),
            contents: None,
        };
        assert_eq!(v.display_label(), "disk2s2");
    }
}
