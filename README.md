<div align="center">
  <img src="app-icon.png" alt="Tengga Logo" width="96" height="96" style="border-radius: 20px;" />

  # Tengga

  **The lightweight, distraction-free Git diff reviewer.** <br />
  *Review uncommitted changes hunk-by-hunk. Accept, stage, and commit with precision—without Electron overhead.*

  [![Website](https://img.shields.io/badge/website-tengga.oan.workers.dev-10b981?style=flat-square)](https://tengga.oan.workers.dev/)
  [![Release](https://img.shields.io/badge/release-v0.1.0-blue?style=flat-square)](https://github.com/yolandaihsan17/tengga/releases)
  [![Tauri](https://img.shields.io/badge/Tauri-v2.0-orange?style=flat-square&logo=tauri)](https://tauri.app/)
  [![Rust](https://img.shields.io/badge/Rust-2021_Edition-black?style=flat-square&logo=rust)](https://www.rust-lang.org/)
  [![Svelte](https://img.shields.io/badge/Svelte-v4.2-ff3e00?style=flat-square&logo=svelte)](https://svelte.dev/)
  [![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

  [**Website**](https://tengga.oan.workers.dev/) • [**Releases**](https://github.com/yolandaihsan17/tengga/releases) • [**Connect with me**](https://www.linkedin.com/in/yolandaihsan/) • [**Issues**](https://github.com/yolandaihsan17/tengga/issues)
</div>

---

## Why Tengga?

Interactive staging via `git add -p` in the terminal provides precise control, but becomes tedious and slow across complex diffs. Conversely, full-featured Git desktop clients (GitKraken, SourceTree) and IDE diff viewers (VS Code) are resource-heavy, frequently demanding 400MB–800MB of RAM just to inspect modified hunks.

Tengga provides a focused desktop utility built specifically for the review and staging workflow:

- **Minimal Memory Overhead**: Operates under 80 MB RAM in typical usage—a fraction of resource-heavy Electron clients.
- **Fast Cold Launch**: Starts in under 150 ms using native OS webview technologies and an optimized Rust backend.
- **Compact Footprint**: Standalone binary under 15 MB with no bundled Chromium or Node.js runtime.
- **Offline and Local**: Direct local Git execution and filesystem monitoring with zero network transmission.

---

## Features

### Hunk-by-Hunk Control
Review changes isolated into individual hunks. Stage changes directly into the Git index with one click (`git apply --cached`), or revert extraneous debug code without affecting adjacent modifications (`git apply --reverse`).

### Sub-Millisecond Filesystem Watcher
Backed by a debounced native Rust watcher (`notify` crate). The moment a file is saved in your editor (VS Code, Zed, Neovim, Cursor), Tengga automatically re-parses and updates diffs in real time.

### First-Class Git Worktrees
Automatically discovers and lists linked Git worktrees in clean tabs. Switch between parallel branches and worktrees with isolated staging buffers without opening multiple application windows.

### Branch and Merge Comparison
Inspect working changes against target base branches (`main`, `develop`, or custom merge request targets) to catch regressions before pushing upstream.

### In-Place Editing
Switch from Diff view to Content view to fix minor formatting or syntax issues directly in Tengga, supported by search and replace functionality.

### Theme Modes
- **Dark Mode**: High-contrast slate and obsidian layout with emerald accents.
- **Light Mode**: Clean daytime layout optimized for bright environments.
- **Glass Mode**: Native macOS vibrancy and frosted background blur via `window-vibrancy`.

---

## Performance Benchmark

| Metric | Tengga (Tauri + Rust) | VS Code Git Diff | Heavy Electron Git GUI |
| :--- | :--- | :--- | :--- |
| **Memory Footprint** | **< 80 MB RAM** | ~300 MB – 600 MB | ~450 MB – 800 MB |
| **Cold Launch Time** | **< 150 ms** | 1.8 s – 3.2 s | 2.5 s – 4.5 s |
| **Installer Binary Size** | **< 15 MB** | ~100 MB+ | ~120 MB – 180 MB |
| **Dedicated Hunk Staging** | **Single-Click UI** | Nested file tree | Complex full-suite dashboard |
| **Worktree Tab Management**| **Built-in Native Tabs**| Workspace reconfiguration | Clunky multiple windows |

---

## Architecture and Tech Stack

- **Desktop Framework**: [Tauri v2](https://tauri.app/) (Rust backend + native OS webview)
- **Frontend UI**: [Svelte 4](https://svelte.dev/), [Vite 5](https://vitejs.dev/)
- **Syntax Highlighting**: [PrismJS](https://prismjs.com/)
- **Backend Components (Rust)**:
  - `tauri` (with `macos-private-api` and window customization)
  - `notify` (debounced filesystem change events)
  - `window-vibrancy` (macOS vibrancy blur effects)
  - `tauri-plugin-dialog` (native folder picker)
- **Landing Page**: Static site in [`landing/`](./landing), deployed to Cloudflare Edge.

---

## Project Structure

```
tengga/
├── src/                      # Svelte frontend
│   ├── App.svelte            # Root application shell and state management
│   ├── lib/
│   │   ├── api.js            # Tauri invoke wrappers for Rust backend
│   │   ├── DiffView.svelte   # Selected file diff rendering and content editor
│   │   ├── FileList.svelte   # Changed files sidebar and batch staging controls
│   │   ├── FileTreeNode.svelte # Hierarchical tree view node layout
│   │   ├── FindReplace.svelte# Search and replace dialog
│   │   ├── HunkView.svelte   # Individual hunk card with stage/revert actions
│   │   ├── SettingsModal.svelte # Application preferences and themes
│   │   ├── WorktreeTabs.svelte # Tabbed multi-worktree navigation
│   │   └── syntax.js         # Prism syntax highlighting utilities
│
├── src-tauri/                # Rust backend (Tauri v2)
│   ├── Cargo.toml            # Rust dependencies and configuration
│   ├── tauri.conf.json       # Tauri window and bundler settings
│   ├── src/
│   │   ├── main.rs           # Tauri commands and lifecycle
│   │   ├── git.rs            # Git CLI execution and staging logic
│   │   ├── diff.rs           # Unified diff parser
│   │   └── watcher.rs        # Debounced filesystem watcher
│
├── landing/                  # Official marketing landing page
│   ├── index.html            # Static responsive landing page
│   ├── style.css             # Stylesheet and design tokens
│   ├── app.js                # OS detection, dropdown, and theme switcher
│   └── assets/               # Logos and icons
│
├── package.json              # Frontend dependencies and scripts
└── README.md
```

---

## Getting Started

### Prerequisites

Ensure the following tools are installed:
1. **Node.js** (v18+) and `npm`
2. **Rust & Cargo** (latest stable release: [rustup.rs](https://rustup.rs/))
3. **Platform Dependencies** (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/)):
   - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
   - **Linux**: `webkit2gtk-4.1`, `libgtk-3-dev`, `libayatana-appindicator3-dev`
   - **Windows**: Microsoft Visual Studio C++ Build Tools & WebView2

### Local Development

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yolandaihsan17/tengga.git
   cd tengga
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Start development mode:**
   ```bash
   npm run tauri dev
   ```

### Building Distributable Packages

To build production binaries (`.dmg` on macOS, `.msi` / `.exe` on Windows, `.deb` / `.AppImage` on Linux):

```bash
npm run tauri build
```

Installers are generated in `src-tauri/target/release/bundle/`.

---

## Landing Page

The project landing page is located in [`landing/`](./landing) and served via Cloudflare Edge:

* **Live URL:** [https://tengga.oan.workers.dev/](https://tengga.oan.workers.dev/)
* Local preview:
  ```bash
  cd landing
  python3 -m http.server 3000
  ```

---

## License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for details.

---

<div align="center">
  Developed by <a href="https://github.com/yolandaihsan17"><strong>Yolanda Ihsan</strong></a> • <a href="https://www.linkedin.com/in/yolandaihsan/"><strong>Connect with me</strong></a>
</div>
