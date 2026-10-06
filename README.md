# AI Diff Review

Lightweight desktop app (Tauri + Svelte) for reviewing uncommitted git changes
hunk-by-hunk, with per-hunk accept (stage) / reject (revert).

## What's implemented

- Folder picker, validated as a git repo
- File watcher (debounced) that triggers a re-diff on any change
- `git diff` + `git diff --staged` fetched and merged per file
- Diff parsing into structured hunks (Rust side)
- Svelte UI: file list sidebar, hunk view with syntax-free line rendering
- Accept → `git apply --cached` on that hunk (stages it)
- Reject → `git apply --reverse` (and `--cached` if already staged) on that hunk

## What's not done yet (per the build-phase plan)

- Syntax highlighting on lines (currently plain monospace with add/remove coloring)
- Binary file handling, merge conflict states, detached HEAD edge cases
- Loading/empty state polish
- App icons (referenced in `tauri.conf.json` under `bundle.icon`, not yet generated)
- Packaging/signing config for distribution

## Setup

Requires: Node.js, Rust + Cargo, and the Tauri system dependencies for your OS
(see https://tauri.app/start/prerequisites/ — on Linux this means
`webkit2gtk`, `libgtk-3-dev`, etc.)

```bash
npm install
npm run tauri dev
```

To build installers for the current platform:

```bash
npm run tauri build
```

## Project layout

```
src/                  Svelte frontend
  App.svelte          top-level layout, state, wiring
  lib/api.js           invoke() wrappers for Rust commands
  lib/FileList.svelte  changed-files sidebar
  lib/DiffView.svelte  hunks for the selected file
  lib/HunkView.svelte  single hunk + accept/reject buttons

src-tauri/            Rust backend
  src/main.rs          Tauri commands, app state
  src/git.rs           git diff / apply wrappers
  src/diff.rs          unified diff parsing into hunks
  src/watcher.rs        debounced fs watcher -> "changes-detected" event
```
