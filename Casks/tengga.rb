cask "tengga" do
  arch arm: "aarch64", intel: "x64"

  version "0.1.2"
  sha256 arm:   "b0bad3b0a4582d1bb4a570391f79ecf99c83ac3600c770da38b135453c52255e",
         intel: "55acddc23fd54cf4b9f510c0aca012fe3b61f599ed530b35364d241ae512e2f8"

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
