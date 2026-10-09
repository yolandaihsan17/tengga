<div align="center">
  <img src="app-icon.png" alt="Tengga Logo" width="96" height="96" style="border-radius: 20px;" />

  # Tengga

  **The lightweight, distraction-free Git diff reviewer.** <br />
  *Review uncommitted changes hunk-by-hunk. Accept, stage, and commit with pure confidence—without the Electron bloat.*

  [![Website](https://img.shields.io/badge/website-tengga.oan.workers.dev-10b981?style=flat-square)](https://tengga.oan.workers.dev/)
  [![Release](https://img.shields.io/badge/release-v0.1.0-blue?style=flat-square)](https://github.com/yolandaihsan17/tengga/releases)
  [![Tauri](https://img.shields.io/badge/Tauri-v2.0-orange?style=flat-square&logo=tauri)](https://tauri.app/)
  [![Rust](https://img.shields.io/badge/Rust-2021_Edition-black?style=flat-square&logo=rust)](https://www.rust-lang.org/)
  [![Svelte](https://img.shields.io/badge/Svelte-v4.2-ff3e00?style=flat-square&logo=svelte)](https://svelte.dev/)
  [![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

  [**Live Website**](https://tengga.oan.workers.dev/) • [**Download Releases**](https://github.com/yolandaihsan17/tengga/releases) • [**Report Issue**](https://github.com/yolandaihsan17/tengga/issues)
</div>

---

## 💡 Why Tengga?

Running `git add -p` in the terminal is powerful but clunky. Full-suite Git GUI clients (GitKraken, SourceTree) and IDE diff viewers (VS Code) are heavy, often consuming 400MB–800MB of RAM just to inspect a few modified lines.

**Tengga** solves this by providing a hyper-focused desktop tool purpose-built for the staging workflow:
* **~30 MB Idle RAM** — 10x to 20x lighter than Electron-based alternatives.
* **Sub-150ms Cold Launch** — Native OS webview with a high-performance Rust backend.
* **Under 15MB Binary Size** — No bundled Chromium browser or bundled Node.js runtime.
* **100% Offline & Private** — Direct local Git CLI and filesystem operations with zero external network requests.

---

## ✨ Features

- **✂️ Hunk-by-Hunk Control**  
  Review uncommitted changes hunk-by-hunk. Stage clean changes into the Git index with one click (`git apply --cached`), or discard temporary debug logs without undoing your feature work (`git apply --reverse`).

- **⚡ Sub-Millisecond Filesystem Watcher**  
  Debounced native Rust filesystem watcher (`notify` crate). The millisecond you press `Cmd+S` or `Ctrl+S` in VS Code, Zed, Neovim, or Cursor, Tengga automatically updates diffs in real time.

- **🌿 First-Class Git Worktrees**  
  Auto-detects and displays linked Git worktrees in clean, browser-like tabs. Switch between parallel branches and worktrees with independent staging buffers without juggling multiple app windows.

- **🔀 Branch & Merge Request Comparison**  
  Inspect differences between your current branch and the target merge base (`main`, `develop`, or custom base branch) before pushing upstream.

- **✏️ In-Place Quick Editing**  
  Spotted a quick typo, formatting mistake, or debug statement while reviewing? Switch from Diff mode to Edit mode directly inside Tengga with full find-and-replace support.

- **🎨 Thoughtful Aesthetics & Themes**  
  - **Dark Mode**: Sleek obsidian and slate palette with emerald accents.
  - **Light Mode**: Crisp, high-contrast daytime palette.
  - **Glass Mode**: Native macOS vibrancy and frosted glass backdrop blur powered by `window-vibrancy`.

---

## 📊 Performance Comparison

| Metric | Tengga (Tauri + Rust) | VS Code Git Diff | Heavy Electron Git GUI |
| :--- | :--- | :--- | :--- |
| **Idle Memory Footprint** | **~30 MB RAM** | ~300 MB – 600 MB | ~450 MB – 800 MB |
| **Cold Launch Time** | **< 150 ms** | 1.8 s – 3.2 s | 2.5 s – 4.5 s |
| **Installer Binary Size** | **< 15 MB** | ~100 MB+ | ~120 MB – 180 MB |
| **Dedicated Hunk Staging** | **Single-Click UI** | Nested file tree | Complex full-suite dashboard |
| **Worktree Tab Management**| **Built-in Native Tabs**| Clunky workspace switch | Multiple open windows |

---

## 🛠️ Tech Stack

- **Desktop Framework**: [Tauri v2](https://tauri.app/) (Rust backend + native OS webview)
- **Frontend UI**: [Svelte 4](https://svelte.dev/), [Vite 5](https://vitejs.dev/)
- **Syntax Highlighting**: [PrismJS](https://prismjs.com/)
- **Backend Utilities (Rust)**:
  - `tauri` (with `macos-private-api` & window customization)
  - `notify` (debounced cross-platform filesystem watcher)
  - `window-vibrancy` (native macOS glassmorphic blur)
  - `tauri-plugin-dialog` (native folder & repository picker)
- **Landing Page**: Modern static website inspired by Slack's design language, located in [`landing/`](./landing) and deployed via Cloudflare Edge.

---

## 📁 Project Structure

```
tengga/
├── src/                      # Svelte frontend
│   ├── App.svelte            # Main app shell & global state orchestration
│   ├── lib/
│   │   ├── api.js            # Tauri invoke wrappers for Rust commands
│   │   ├── DiffView.svelte   # Selected file diff rendering & edit mode
│   │   ├── FileList.svelte   # Changed files sidebar & stage/unstage triggers
│   │   ├── FileTreeNode.svelte # Tree node layout with hover actions
│   │   ├── FindReplace.svelte# In-place search and replace panel
│   │   ├── HunkView.svelte   # Single hunk card with Stage & Revert buttons
│   │   ├── SettingsModal.svelte # App settings & theme configuration
│   │   ├── WorktreeTabs.svelte # Tabbed multi-worktree switcher
│   │   └── syntax.js         # Prism syntax highlighting helpers
│
├── src-tauri/                # Rust backend (Tauri v2)
│   ├── Cargo.toml            # Rust dependencies & metadata
│   ├── tauri.conf.json       # App window, icons, and bundle configurations
│   ├── src/
│   │   ├── main.rs           # Tauri commands & event loops
│   │   ├── git.rs            # Git CLI execution & staging operations
│   │   ├── diff.rs           # Unified diff parsing into structured hunks
│   │   └── watcher.rs        # Debounced filesystem watcher
│
├── landing/                  # Official marketing landing page
│   ├── index.html            # Slack-inspired responsive layout
│   ├── style.css             # Design tokens & Material Symbols styles
│   ├── app.js                # OS detection, dropdown, and theme switcher
│   └── assets/               # Branding icons and assets
│
├── package.json              # Node dependencies & build scripts
└── README.md
```

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed on your system:
1. **Node.js** (v18 or newer) and `npm`
2. **Rust & Cargo** (latest stable release — [rustup.rs](https://rustup.rs/))
3. **Platform Build Dependencies** (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/)):
   - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
   - **Linux**: `webkit2gtk-4.1`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, etc.
   - **Windows**: Microsoft Visual Studio C++ Build Tools & WebView2

### Local Development

1. **Clone repository:**
   ```bash
   git clone https://github.com/yolandaihsan17/tengga.git
   cd tengga
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode (hot-reload):**
   ```bash
   npm run tauri dev
   ```

### Building Distributable Packages

To build production installers (`.dmg` on macOS, `.msi` / `.exe` on Windows, `.deb` / `.AppImage` on Linux):

```bash
npm run tauri build
```

Built binaries and installers will be generated under `src-tauri/target/release/bundle/`.

---

## 🌐 Official Landing Page

The official marketing landing page is maintained in the [`landing/`](./landing) directory and deployed to Cloudflare Edge:

* **Live URL:** [https://tengga.oan.workers.dev/](https://tengga.oan.workers.dev/)
* To preview the landing page locally:
  ```bash
  cd landing
  python3 -m http.server 3000
  # Open http://localhost:3000 in your browser
  ```

---

## 📄 License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for more information.

---

<div align="center">
  Crafted with care by <a href="https://github.com/yolandaihsan17"><strong>Yolanda Ihsan</strong></a>
</div>
