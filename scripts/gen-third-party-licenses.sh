#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 kodephp contributors
#
# Regenerate THIRD_PARTY_LICENSES.md from Cargo.lock.
#
# The dependency graph is read with `cargo metadata` (which honours the
# lock file), then reduced to the packages that are actually reachable
# from the workspace members over *runtime* edges — `dev` edges are
# dropped, so the inventory matches what ships inside the binary.
#
# Usage:
#   scripts/gen-third-party-licenses.sh            # write the two output files
#   scripts/gen-third-party-licenses.sh --check    # exit 1 if files are stale
#
# Outputs (both generated from one source of truth to prevent drift):
#   THIRD_PARTY_LICENSES.md                        # repo root, human facing
#   crates/ntfs-mac-core/THIRD_PARTY_LICENSES.md   # embedded into the binary
#
# Requirements: cargo, python3.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mode="${1:-write}"

py="$(command -v python3 || true)"
if [ -z "$py" ]; then
  echo "error: python3 is required" >&2
  exit 1
fi

tmp="$(mktemp -t ntfs-mac-third-party)"
meta="$(mktemp -t ntfs-mac-metadata)"
trap 'rm -f "$tmp" "$meta"' EXIT

# Derive the inventory from the pinned versions in Cargo.lock. `--locked`
# makes cargo fail rather than silently re-resolve, but it *also* fails
# when no lock file exists at all, so only pass it when there is one.
locked=""
if [ -f Cargo.lock ]; then
  locked="--locked"
else
  echo "warning: no Cargo.lock — resolving fresh; the inventory may not match a release" >&2
fi

# shellcheck disable=SC2086
cargo metadata --format-version 1 $locked > "$meta"

"$py" - "$tmp" "$meta" <<'PY'
import json
import re
import sys
import datetime
from collections import defaultdict

out_path = sys.argv[1]

with open(sys.argv[2], "r", encoding="utf-8") as fh:
    meta = json.load(fh)

packages = {p["id"]: p for p in meta["packages"]}
workspace_ids = set(meta["workspace_members"])
resolve = meta.get("resolve") or {}
nodes = {n["id"]: n for n in resolve.get("nodes", [])}

# Shipping artifacts we report on. Keyed by package name so a rename of
# the directory does not silently drop a column.
artifacts = {
    "ntfs-mac-cli": "CLI",
    "ntfs-mac-tauri": "GUI",
}
member_by_name = {packages[i]["name"]: i for i in workspace_ids}
missing = [n for n in artifacts if n not in member_by_name]
if missing:
    sys.exit(f"error: workspace member(s) not found: {', '.join(missing)}")


def closure(root_id):
    """Runtime-reachable non-workspace packages from root_id.

    Edges tagged `dev` are skipped: they never end up in a shipped
    binary. A dependency needed by both a build script and normal code
    appears with two dep_kinds, so the `<= {"dev"}` test is required
    instead of an equality check.
    """
    seen = set()
    stack = [root_id]
    while stack:
        pid = stack.pop()
        for dep in nodes.get(pid, {}).get("deps", []):
            kinds = {k.get("kind") for k in dep.get("dep_kinds", [])}
            if kinds and kinds <= {"dev"}:
                continue
            dep_id = dep["pkg"]
            if dep_id in workspace_ids or dep_id in seen:
                continue
            seen.add(dep_id)
            stack.append(dep_id)
    return seen


per_artifact = {
    artifacts[packages[member_by_name[name]]["name"]]: closure(member_by_name[name])
    for name in artifacts
}

reachable = set().union(*per_artifact.values())

rows = []
for pid in reachable:
    p = packages[pid]
    license_expr = p.get("license") or ("see crate" if p.get("license_file") else "UNKNOWN")
    rows.append(
        {
            "name": p["name"],
            "version": p["version"],
            "license": license_expr,
            "repo": p.get("repository") or "",
            "in": [a for a, ids in per_artifact.items() if pid in ids],
        }
    )

# Group by license expression, then by crate name.
by_license = defaultdict(list)
for r in rows:
    by_license[r["license"]].append(r)

def sort_key(lic):
    # Put the common permissive licenses first, unknown ones last.
    order = ["Apache-2.0", "MIT", "MIT OR Apache-2.0", "Apache-2.0 OR MIT",
             "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unlicense", "Zlib"]
    return (0, order.index(lic)) if lic in order else (1, lic)


def version_key(v):
    """Total order for version strings.

    `reachable` is a set, so ties in the crate-name comparison would
    otherwise be laid out in a different order on every run and the
    `--check` freshness test would flap. Comparing name *and* version
    removes the tie; numeric parts compare as numbers so 0.9 < 0.10.
    """
    parts = re.split(r"[.\-+]", v)
    return tuple((0, int(p)) if p.isdigit() else (1, p) for p in parts)


def crate_key(r):
    return (r["name"].lower(), version_key(r["version"]))

total = len(rows)
generated = datetime.date.today().isoformat()
cli_count = len(per_artifact.get("CLI", ()))
gui_count = len(per_artifact.get("GUI", ()))

lines = []
lines.append("<!-- SPDX-License-Identifier: Apache-2.0 -->")
lines.append("")
lines.append("# Third-party licenses")
lines.append("")
lines.append("ntfs-mac itself is licensed under the Apache License, Version 2.0, but")
lines.append("each shipped artifact statically links the Rust crates listed below.")
lines.append("This file is the attribution inventory for those crates.")
lines.append("")
lines.append(f"- Generated: {generated}")
# Keep this line machine-readable: `ntfs_mac_core::license::third_party_counts`
# parses the integers out of it and a unit test guards the format.
lines.append(f"- Crates: {total} total, {cli_count} CLI, {gui_count} GUI")
lines.append("- Source: `cargo metadata` (honouring `Cargo.lock`) reduced to packages")
lines.append("  reachable over *runtime* edges; `dev`-only edges are excluded, so the")
lines.append("  list matches what ships rather than the whole test matrix")
lines.append("")
lines.append("The `CLI` and `GUI` columns mark which artifact links each crate. Crates")
lines.append("marked in neither column are build-time or transitive dependencies of one")
lines.append("of the two and are listed for completeness.")
lines.append("")
lines.append("Regenerate with:")
lines.append("")
lines.append("```sh")
lines.append("scripts/gen-third-party-licenses.sh")
lines.append("```")
lines.append("")
lines.append("Do not edit by hand — the file is generated. CI verifies it is current")
lines.append("with `scripts/gen-third-party-licenses.sh --check`.")
lines.append("")
lines.append("## Summary by license")
lines.append("")
lines.append("| License | Crates |")
lines.append("| --- | ---: |")
for lic in sorted(by_license, key=sort_key):
    lines.append(f"| {lic} | {len(by_license[lic])} |")
lines.append("")
lines.append("## Crates")
lines.append("")

for lic in sorted(by_license, key=sort_key):
    lines.append(f"### {lic}")
    lines.append("")
    lines.append("| Crate | Version | CLI | GUI | Repository |")
    lines.append("| --- | --- | :---: | :---: | --- |")
    for r in sorted(by_license[lic], key=crate_key):
        repo = r["repo"]
        link = f"[{repo}]({repo})" if repo else "—"
        cli = "✓" if "CLI" in r["in"] else ""
        gui = "✓" if "GUI" in r["in"] else ""
        lines.append(f"| {r['name']} | {r['version']} | {cli} | {gui} | {link} |")
    lines.append("")

lines.append(f"Total: {total} crates.")
lines.append("")

with open(out_path, "w", encoding="utf-8") as fh:
    fh.write("\n".join(lines))

print(f"generated {total} crates ({cli_count} CLI, {gui_count} GUI) "
      f"across {len(by_license)} license expressions")
PY

targets=(
  "THIRD_PARTY_LICENSES.md"
  "crates/ntfs-mac-core/THIRD_PARTY_LICENSES.md"
)

if [ "$mode" = "--check" ]; then
  stale=0
  for t in "${targets[@]}"; do
    if [ ! -f "$t" ]; then
      echo "missing: $t" >&2
      stale=1
      continue
    fi
    # The `- Generated: <date>` line records the day the file was produced
    # and is expected to differ from a fresh run, so it is excluded from
    # the comparison to keep this check reproducible day to day.
    if ! diff -q <(grep -v '^- Generated:' "$tmp") \
                <(grep -v '^- Generated:' "$t") >/dev/null 2>&1; then
      echo "stale: $t" >&2
      stale=1
    fi
  done
  if [ "$stale" -ne 0 ]; then
    echo "run scripts/gen-third-party-licenses.sh to refresh" >&2
    exit 1
  fi
  echo "ok: THIRD_PARTY_LICENSES.md is up to date"
  exit 0
fi

for t in "${targets[@]}"; do
  mkdir -p "$(dirname "$t")"
  cp "$tmp" "$t"
  echo "wrote $t"
done
