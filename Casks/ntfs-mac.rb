cask "ntfs-mac" do
  version "0.1.4"
  sha256 "dbaf0826ea04ecf4b2414d2a4f3651e7e8329c6315e97ec7f03c4818234ce556"

  url "https://github.com/kodephp/rust_ntfs/releases/download/v#{version}/ntfs-mac-#{version}.pkg"
  name "ntfs-mac"
  desc "Toolkit to mount, unmount, format, fix and copy NTFS volumes"
  homepage "https://github.com/kodephp/rust_ntfs"

  livecheck do
    url :url
    strategy :github_latest
  end

  # The .pkg ships a universal binary (Apple Silicon + Intel) and installs
  # the GUI app, the CLI and the licence artefacts in one pass.
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
