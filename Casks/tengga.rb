cask "tengga" do
  arch arm: "aarch64", intel: "x64"

  version "0.1.1"
  sha256 arm:   :no_check,
         intel: :no_check

  url "https://github.com/yolandaihsan17/tengga/releases/download/v#{version}/Tengga_#{version}_#{arch}.dmg"
  name "Tengga"
  desc "Lightweight Git Diff Reviewer built with Tauri, Rust, and Svelte"
  homepage "https://github.com/yolandaihsan17/tengga"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "Tengga.app"

  zap trash: [
    "~/Library/Application Support/com.tengga.desktop",
    "~/Library/Caches/com.tengga.desktop",
    "~/Library/Saved Application State/com.tengga.desktop.savedState",
    "~/Library/WebKit/com.tengga.desktop",
  ]
end
