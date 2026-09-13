// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors

//! Licence metadata for the ntfs-mac distribution.
//!
//! The shipped binary has to be able to answer licence questions wherever
//! it was installed, so as much as possible is compile-time data:
//!
//! * the SPDX identifier, copyright line and repository come from
//!   `Cargo.toml`, and a unit test keeps [`SPDX_ID`] pinned to the manifest;
//! * the generated third-party attribution inventory is embedded with
//!   [`include_str!`] into [`THIRD_PARTY_INVENTORY`], and is therefore
//!   always available even when the binary is copied out of its install
//!   layout;
//! * the full Apache-2.0 text and `NOTICE` are *not* embedded — they are
//!   located at runtime by [`license_file`] / [`notice_file`], which fall
//!   back to the source tree so `--full` also works from a checkout.
//!
//! Regenerate the embedded inventory with
//! `scripts/gen-third-party-licenses.sh`; CI fails if it is stale.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// SPDX identifier of the project licence.
pub const SPDX_ID: &str = "Apache-2.0";

/// Copyright line. Kept identical to the one in the repository `NOTICE`.
pub const COPYRIGHT: &str = "Copyright 2026 kodephp contributors";

/// Canonical location of the full licence text.
pub const LICENSE_URL: &str = "https://www.apache.org/licenses/LICENSE-2.0";

/// Upstream repository, taken from the crate manifest at compile time.
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The generated third-party attribution inventory.
///
/// Produced by `scripts/gen-third-party-licenses.sh` from `Cargo.lock` and
/// embedded so `ntfs-mac license --third-party` works offline.
pub const THIRD_PARTY_INVENTORY: &str = include_str!("../THIRD_PARTY_LICENSES.md");

/// What Apache-2.0 grants, requires and disclaims.
///
/// A summary only — the authoritative wording is in `LICENSE`, which
/// [`license_file`] locates. Kept as data so the CLI and the GUI can agree.
pub const APACHE_CLAUSES: &[(&str, &str)] = &[
    (
        "permissions",
        "commercial use, modification, distribution, patent use, private use",
    ),
    ("conditions", "license and copyright notice, state changes"),
    ("limitations", "liability, trademark use, warranty"),
];

/// Directories searched, in order, for `LICENSE` and `NOTICE`.
///
/// The source tree comes first so a `cargo run` checkout resolves without
/// any installation, followed by the layouts produced by
/// `scripts/build-macos.sh` and the Homebrew formula.
const RESOURCE_DIRS: &[&str] = &[
    // <repo root> — the crate lives at <repo>/crates/ntfs-mac-core.
    concat!(env!("CARGO_MANIFEST_DIR"), "/../.."),
    // Installer payload (see scripts/build-macos.sh).
    "/usr/local/share/ntfs-mac",
    // Homebrew on Apple Silicon.
    "/opt/homebrew/share/ntfs-mac",
    // The .app bundle, when the CLI is run from inside it.
    "/Applications/ntfs-mac.app/Contents/Resources",
    // Last resort: the current working directory.
    ".",
];

/// Locate a licence resource such as `LICENSE` or `NOTICE` on disk.
///
/// Returns the first candidate that exists as a regular file, or `None`
/// when the binary has been moved away from every known layout. Callers
/// decide how to present a miss; the embedded
/// [`THIRD_PARTY_INVENTORY`] never goes missing.
pub fn resource_path(file: &str) -> Option<PathBuf> {
    RESOURCE_DIRS
        .iter()
        .map(|dir| Path::new(dir).join(file))
        .find(|candidate| candidate.is_file())
}

/// Path to the full Apache-2.0 text, if it can be found.
pub fn license_file() -> Option<PathBuf> {
    resource_path("LICENSE")
}

/// Path to the repository `NOTICE` file, if it can be found.
///
/// Apache-2.0 §4(d) requires redistributions to carry the `NOTICE` text,
/// so the installer ships it next to `LICENSE`.
pub fn notice_file() -> Option<PathBuf> {
    resource_path("NOTICE")
}

/// Render a located path for display, collapsing `.` and `..` segments.
///
/// Deliberately lexical: `Path::canonicalize` would also resolve symlinks,
/// turning a familiar `/Applications/ntfs-mac.app/...` into
/// `/System/Volumes/Data/Applications/...`, which is accurate but useless
/// to a reader.
pub fn display_path(path: &Path) -> String {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // `pop` is a no-op at the root, which is the behaviour we
                // want for `/..`.
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out.display().to_string()
}

/// Crate counts parsed out of the embedded inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThirdPartyCounts {
    /// Distinct crates reachable from the shipped artifacts.
    pub total: usize,
    /// Crates linked into the `ntfs-mac` CLI binary.
    pub cli: usize,
    /// Crates linked into the `ntfs-mac.app` GUI.
    pub gui: usize,
}

/// Parse the `- Crates: <total> total, <cli> CLI, <gui> GUI` line written
/// by `scripts/gen-third-party-licenses.sh`.
///
/// Returns `None` if the line is missing or malformed. A unit test runs
/// against the real embedded inventory so the format cannot drift
/// unnoticed.
pub fn third_party_counts() -> Option<ThirdPartyCounts> {
    let line = THIRD_PARTY_INVENTORY
        .lines()
        .find(|line| line.starts_with("- Crates:"))?;

    let mut numbers = line
        .split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<usize>().ok());

    Some(ThirdPartyCounts {
        total: numbers.next()?,
        cli: numbers.next()?,
        gui: numbers.next()?,
    })
}

/// Machine-readable description of the distribution's licence state.
///
/// Built by [`summary`] and serialised by `ntfs-mac license --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSummary {
    /// Project name.
    pub project: String,
    /// Project version.
    pub version: String,
    /// SPDX identifier of the project licence.
    pub spdx: String,
    /// Copyright line.
    pub copyright: String,
    /// Upstream repository.
    pub repository: String,
    /// Canonical licence text URL.
    pub license_url: String,
    /// Where the full licence text was found, if anywhere.
    pub license_file: Option<String>,
    /// Where `NOTICE` was found, if anywhere.
    pub notice_file: Option<String>,
    /// Embedded third-party crate counts, if they could be parsed.
    pub third_party: Option<ThirdPartyCounts>,
}

/// Gather the current licence state, probing the filesystem for the two
/// resources that are not embedded.
pub fn summary() -> LicenseSummary {
    LicenseSummary {
        project: crate::NAME.to_string(),
        version: crate::VERSION.to_string(),
        spdx: SPDX_ID.to_string(),
        copyright: COPYRIGHT.to_string(),
        repository: REPOSITORY.to_string(),
        license_url: LICENSE_URL.to_string(),
        license_file: license_file().map(|p| display_path(&p)),
        notice_file: notice_file().map(|p| display_path(&p)),
        third_party: third_party_counts(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spdx_id_matches_the_cargo_manifest() {
        // Guards against the constant and `Cargo.toml` drifting apart.
        assert_eq!(SPDX_ID, env!("CARGO_PKG_LICENSE"));
    }

    #[test]
    fn inventory_is_embedded() {
        assert!(!THIRD_PARTY_INVENTORY.is_empty());
        assert!(THIRD_PARTY_INVENTORY.contains("Apache License, Version 2.0"));
        assert!(THIRD_PARTY_INVENTORY.contains("# Third-party licenses"));
    }

    #[test]
    fn third_party_counts_parse() {
        let counts = third_party_counts().expect("inventory should carry a `- Crates:` line");
        assert!(counts.total > 0, "total should be non-zero: {counts:?}");
        assert!(counts.cli > 0, "cli should be non-zero: {counts:?}");
        assert!(counts.gui > 0, "gui should be non-zero: {counts:?}");
        // The GUI links the CLI crate, so the two sets overlap and the
        // total is not their sum; every count must still be a subset.
        assert!(counts.total >= counts.cli, "{counts:?}");
        assert!(counts.total >= counts.gui, "{counts:?}");
    }

    #[test]
    fn resource_lookup_resolves_the_source_tree() {
        // `Cargo.toml` is present at the repo root in any checkout, so it
        // proves the `CARGO_MANIFEST_DIR/../..` candidate works without
        // depending on a licence-specific file.
        let found = resource_path("Cargo.toml").expect("repo root Cargo.toml should be found");
        assert_eq!(found.file_name().unwrap(), "Cargo.toml");
    }

    #[test]
    fn license_and_notice_are_findable_from_the_source_tree() {
        let license = license_file().expect("LICENSE should be locatable from the source tree");
        assert_eq!(license.file_name().unwrap(), "LICENSE");

        let notice = notice_file().expect("NOTICE should be locatable from the source tree");
        assert_eq!(notice.file_name().unwrap(), "NOTICE");
    }

    #[test]
    fn summary_is_self_consistent() {
        let summary = summary();
        assert_eq!(summary.project, crate::NAME);
        assert_eq!(summary.version, crate::VERSION);
        assert_eq!(summary.spdx, SPDX_ID);
        assert_eq!(summary.repository, REPOSITORY);
        assert!(summary.license_file.is_some());
        assert!(summary.third_party.is_some());
    }

    #[test]
    fn apache_clauses_cover_the_three_groups() {
        let kinds: Vec<&str> = APACHE_CLAUSES.iter().map(|(kind, _)| *kind).collect();
        assert_eq!(kinds, vec!["permissions", "conditions", "limitations"]);
    }

    #[test]
    fn display_path_collapses_dot_segments() {
        assert_eq!(display_path(Path::new("/a/b/../c")), "/a/c");
        assert_eq!(display_path(Path::new("/a/./b/../../c")), "/c");
        assert_eq!(display_path(Path::new("/a/b/")), "/a/b");
        assert_eq!(display_path(Path::new("relative/./x")), "relative/x");
    }

    #[test]
    fn summary_paths_are_normalized() {
        let summary = summary();
        let license = summary.license_file.expect("LICENSE should resolve");
        assert!(
            !license.contains("/../"),
            "path should be normalized, got {license}"
        );
        assert!(license.ends_with("/LICENSE"), "got {license}");
    }
}
