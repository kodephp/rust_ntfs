/// Synthetic plist test fixture for device::parse_diskutil_plist.
///
/// The `plist` crate's `from_reader` expects binary or XML plist format,
/// not JSON. We write a proper XML plist fixture that mimics the real
/// `diskutil list -plist` output structure.

#[cfg(test)]
mod tests {
    use ntfs_mac_core::device;

    /// XML plist mimicking `diskutil list -plist` output with:
    /// - disk0 (internal APFS — should be filtered out)
    /// - disk2s1 (NTFS "MYNTFS" — should be included)
    /// - disk3s1 (HFS+ "MACOS_DATA" — should be filtered out)
    const SAMPLE_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AllDisksAndPartitions</key>
    <array>
        <dict>
            <key>DeviceIdentifier</key>
            <string>disk0</string>
            <key>Whole</key>
            <true/>
            <key>Internal</key>
            <true/>
            <key>Ejectable</key>
            <false/>
            <key>Content</key>
            <string>GUID_partition_scheme</string>
            <key>Size</key>
            <integer>494384795648</integer>
            <key>MediaName</key>
            <string>APPLE SSD</string>
            <key>Partitions</key>
            <array>
                <dict>
                    <key>DeviceIdentifier</key>
                    <string>disk0s1</string>
                    <key>Content</key>
                    <string>Apple_APFS</string>
                    <key>Size</key>
                    <integer>494370152448</integer>
                    <key>Internal</key>
                    <true/>
                    <key>Ejectable</key>
                    <false/>
                    <key>VolumeName</key>
                    <string></string>
                </dict>
            </array>
        </dict>
        <dict>
            <key>DeviceIdentifier</key>
            <string>disk2</string>
            <key>Whole</key>
            <true/>
            <key>Internal</key>
            <false/>
            <key>Ejectable</key>
            <true/>
            <key>Content</key>
            <string>GUID_partition_scheme</string>
            <key>Size</key>
            <integer>488000000000</integer>
            <key>MediaName</key>
            <string>USB DISK</string>
            <key>Partitions</key>
            <array>
                <dict>
                    <key>DeviceIdentifier</key>
                    <string>disk2s1</string>
                    <key>Content</key>
                    <string>Microsoft Basic Data</string>
                    <key>Size</key>
                    <integer>488000000000</integer>
                    <key>Internal</key>
                    <false/>
                    <key>Ejectable</key>
                    <true/>
                    <key>VolumeName</key>
                    <string>MYNTFS</string>
                    <key>DiskUUID</key>
                    <string>11111111-2222-3333-4444-555555555555</string>
                </dict>
            </array>
        </dict>
        <dict>
            <key>DeviceIdentifier</key>
            <string>disk3</string>
            <key>Whole</key>
            <true/>
            <key>Internal</key>
            <false/>
            <key>Ejectable</key>
            <true/>
            <key>Content</key>
            <string>GUID_partition_scheme</string>
            <key>Size</key>
            <integer>240000000000</integer>
            <key>MediaName</key>
            <string>SEAGATE BACKUP</string>
            <key>Partitions</key>
            <array>
                <dict>
                    <key>DeviceIdentifier</key>
                    <string>disk3s1</string>
                    <key>Content</key>
                    <string>com.apple.hfs</string>
                    <key>Size</key>
                    <integer>240000000000</integer>
                    <key>Internal</key>
                    <false/>
                    <key>Ejectable</key>
                    <true/>
                    <key>VolumeName</key>
                    <string>MACOS_DATA</string>
                </dict>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#;

    #[test]
    fn parses_xml_fixture() {
        let volumes = device::parse_diskutil_plist(SAMPLE_PLIST).unwrap();

        // Only NTFS partitions should be returned.
        assert!(
            volumes.iter().any(|v| v.device_identifier == "disk2s1"),
            "should find disk2s1 (NTFS)"
        );
        assert!(
            !volumes.iter().any(|v| v.device_identifier == "disk0s1"),
            "should not include APFS"
        );
        assert!(
            !volumes.iter().any(|v| v.device_identifier == "disk3s1"),
            "should not include HFS+"
        );

        // The NTFS volume should have the correct label.
        let ntfs_vol = volumes
            .iter()
            .find(|v| v.device_identifier == "disk2s1")
            .unwrap();
        assert_eq!(ntfs_vol.volume_name, "MYNTFS");
        assert!(ntfs_vol.size_bytes > 0);
        assert!(!ntfs_vol.location.is_empty());
    }
}
