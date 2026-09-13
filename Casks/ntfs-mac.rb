cask "ntfs-mac" do
  version "0.1.7"
  sha256 "d5ec1f6d97de58ae841218fef983ccb568eb64fa65ebb5971f2857cc60404376"

  url "https://github.com/kodephp/rust_ntfs/releases/download/v#{version}/ntfs-mac-#{version}.pkg"
  name "ntfs-mac"
  desc "Toolkit to mount, unmount, format, fix and copy NTFS volumes"
  homepage "https://github.com/kodephp/rust_ntfs"

  livecheck do
    url :url
    strategy :github_latest
  end

  # The .pkg installs the GUI app, the CLI and the licence artefacts in one
  # pass. It is built on the release host's own architecture (arm64 today);
  # rebuild from source on an Intel host to ship an x86_64 package.
  depends_on macos: :monterey

  pkg "ntfs-mac-#{version}.pkg"

  # postinstall.sh already chmods the CLI and cleans up releases 0.1.0-0.1.2
  # which scattered files into the filesystem root.
  uninstall pkgutil: "com.kodephp.ntfs-mac",
            delete:  [
              "/Applications/ntfs-mac.app",
              "/usr/local/bin/ntfs-mac",
              "/usr/local/share/ntfs-mac",
            ]

  zap pkgutil: "com.kodephp.ntfs-mac",
      delete:  [
        "/Applications/ntfs-mac.app",
        "/usr/local/bin/ntfs-mac",
        "/usr/local/share/ntfs-mac",
      ]

  caveats <<~EOS
    Read/write access to NTFS volumes needs a FUSE driver, which the installer
    does not bundle:

      brew install ntfs-3g
      brew install --cask macfuse   # Intel Macs
      brew install fuse-t           # Apple Silicon Macs

    After installing a FUSE framework, approve it under
    System Settings -> Privacy & Security, then run:

      sudo /usr/local/share/ntfs-mac/install.sh

    `ntfs-mac license --full` shows the licence, NOTICE and third-party
    attribution shipped with the app.
  EOS
end
