cask "tengga" do
  arch arm: "aarch64", intel: "x64"

  version "0.1.1"
  sha256 arm:   "e4a46dbdfc826296a14e60d021b8378f625e51a5b1dbae640e28008b198fb14d",
         intel: "93b410d18f85b08f88c7d0feaaac3a06f99924d43e5079ae64091f5be684beee"

  url "https://github.com/yolandaihsan17/tengga/releases/download/v#{version}/Tengga_#{version}_#{arch}.dmg"
  name "Tengga"
  desc "Lightweight Git Diff Reviewer built with Tauri, Rust, and Svelte"
  homepage "https://github.com/yolandaihsan17/tengga"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "Tengga.app"

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-dr", "com.apple.quarantine", "#{appdir}/Tengga.app"],
                   must_succeed: false
  end

  zap trash: [
    "~/Library/Application Support/com.tengga.desktop",
    "~/Library/Caches/com.tengga.desktop",
    "~/Library/Saved Application State/com.tengga.desktop.savedState",
    "~/Library/WebKit/com.tengga.desktop",
  ]
end
