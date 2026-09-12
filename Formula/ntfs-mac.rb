# typed:false
# Homebrew formula for ntfs-mac.
# https://formulae.brew.sh/formula/ntfs-mac

class NtfsMac < Formula
  desc "NTFS toolkit for macOS: mount, unmount, format, fix, copy — with GUI"
  homepage "https://github.com/kodephp/ntfs-mac"
  version "0.1.0"

  # SHA256 of the .pkg at release. Update on each release.
  url "https://github.com/kodephp/ntfs-mac/releases/download/v0.1.0/ntfs-mac-0.1.0.pkg"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"

  license "Apache-2.0"

  # macOS 12 (Monterey) or later. The kernel extension requirement
  # differs between Intel and Apple Silicon.
  depends_on :macos => :monterey

  # Runtime dependencies. `ntfs-3g` is the read/write FUSE driver;
  # `macfuse` (Intel) or `fuse-t` (Apple Silicon) is the FUSE framework.
  depends_on "ntfs-3g"
  depends_on "brew-install" do |f|
    f.desc "Install macFUSE or FUSE-T"
    f.when_on_arm do
      f.install "fuse-t"
    end
    f.when_on_intel do
      f.install "macfuse"
    end
  end

  # macOS requires FUSE kernel extension to be approved in
  # System Settings → Privacy & Security before first mount.
  def post_install
    if MacOS.version <= :ventura
      system "security", "authorizationdb", "read", "system.preferences.kernel_extension_allowlist"
    end
  end

  def install
    # pkgutil expands the .pkg payload to a temp directory.
    payload = staging_path/"payload"
    system "pkgutil", "--expand-full", Formula.cachable_download_url(self).to_s, payload
    # Copy the .app to the install prefix.
    cp_r payload/"Applications/ntfs-mac.app", HOMEBREW_PREFIX/Applications
    # Copy the CLI binary.
    bin.install payload/"usr/local/bin/ntfs-mac" => "ntfs-mac"
    # Copy the helper script.
    (share/"ntfs-mac").install payload/"scripts/install.sh"
  end

  def test
    system bin/"ntfs-mac", "--version"
    system bin/"ntfs-mac", "doctor"
  end
end
