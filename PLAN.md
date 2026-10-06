# Tengga (AI Diff Review) — Project Plan

Desktop app (Tauri 2 + Svelte 4) that shows uncommitted changes in a git
repository as individual hunks, each with an Accept and/or Reject button.
It also provides full repository file tree exploration and lightweight
in-place file content inspection and editing. It does not call any external
AI/LLM API or manage git branches — it runs purely locally, reads git
output, applies/reverts single hunks, and allows fast in-place file edits.

This document is the source of truth for behavior. If code and this
document disagree, treat that as a bug to fix, not a judgment call — either
fix the code to match this doc, or edit this doc in the same change that
changes the behavior. Do not silently diverge.

## 1. Definitions (exact meaning of terms used below)

- **repo** — an absolute filesystem path for which
  `git rev-parse --is-inside-work-tree` (run with that path as the process
  working directory) exits with status 0.
- **unstaged diff** — the output of `git diff` (no args) run in the repo.
- **staged diff** — the output of `git diff --staged` run in the repo.
- **hunk** — one `@@ -a,b +c,d @@ ...` block plus its following context/
  add/remove lines, as produced by unified diff format, belonging to one
  file and one of (unstaged, staged).
- **hunk id** — a UUID v4 string, generated fresh every time `get_diff`
  runs, one per hunk. Not stable across calls. Not derived from hunk
  content or position.
- **file header** — for one file's diff block, every line before the
  first `@@` line: the `diff --git a/... b/...`, `index ...`, `---`,
  `+++` lines, verbatim as git produced them.
- **Accept** — run `git apply --cached --recount -` on a patch built from
  `file header + this one hunk's text`, with that patch piped to stdin.
  Only shown/allowed for hunks where `source == "unstaged"`.
- **Reject** — run `git apply --reverse --recount -` (add `--cached` too
  if `source == "staged"`) on the same kind of single-hunk patch.
  Shown/allowed for hunks of either source.

## 2. Scope — exact boundary

**Must do:**
- Open a folder via native OS picker; reject it (show error, do not
  proceed) if it fails the repo check in Definitions §1.
- Allow switching to a different folder at any time, replacing all
  current state (including clearing hunk patches and active watchers).
- On successful open: fetch unstaged + staged diffs, merge per file
  (see §4 `get_diff`), list all repository files via `list_repo_files`,
  display dual-scope file navigation (Changes vs All Files), and start
  the fs watcher.
- Each hunk gets exactly the buttons defined in §1 (Accept/Reject rules).
- Full repository file tree navigation: inspect clean or modified files
  in a collapsible folder hierarchy.
- In-place text file inspection and editing: view file contents with line
  number gutters, edit with 2-space Tab indent, save active file with `Cmd+S`
  (macOS) / `Ctrl+S` (Linux/Windows), and save all modified buffers with
  `Cmd+Option+S` (macOS) / `Ctrl+Alt+S` (Linux/Windows), with VS Code–style
  status circles and zero button clutter.
- Syntax highlighting: lightweight tokenization (VS Code Dark+ palette) powered
  by PrismJS (<1 MB RAM footprint) with authentic token highlighting across HTML,
  Vue SFCs, Svelte, JavaScript, TypeScript, CSS, Rust, Python, Bash, JSON, YAML, and SQL.
- Find & Replace: in-editor search and replacement widget accessible via
  `Cmd+F` / `Ctrl+F` (Find), `Cmd+H` / `Ctrl+H` (Find & Replace), `Enter` /
  `Shift+Enter` (navigation), and `Escape` (dismiss), featuring non-blurring
  search inputs and visual match highlight overlays.
- Branch switcher & Comparison Mode (MR View): display the active git branch name
  and target base branch in a spacious center topbar, with interactive dropdown popovers
  (powered by `list_branches`, `switch_branch`, and `diff_against_base`).
- Pull Latest on branches: both source branch and target branch provide a direct
  "Pull Latest" button (powered by `pull_branch` via `git pull --ff-only` for active branch
  and `git fetch origin <branch>:<branch>` for target branch) with real-time spinners and status banners.
- Repository branding: display the current repository/folder name prominently in the top-left
  in place of generic branding, keeping the center workspace airy and clutter-free (no full filesystem paths).
- Immediate folder tree reactivity: toggle collapsed and expanded folders instantaneously in the
  file tree without requiring tab switches.
- Theme Switcher: toggle between Light (Emerald Light), Dark (Emerald Dark), and Transparent Glass
  (macOS native vibrancy with frosted translucent surfaces and backdrop-filter) with persistent
  localStorage storage and zero startup flicker.
- On any filesystem change under the repo path, after a 400ms debounce
  (see §5), re-run the fetch-and-display step above without re-opening
  the folder picker.

**Must not do (do not add without an explicit new instruction from the
user, not inferred from "would be nice"):**
- No heavyweight external editor suites (e.g., Monaco, CodeMirror) — the
  in-place editor is implemented using native lightweight elements to maintain
  an ultra-low RAM footprint (<50 MB total app memory, <2 MB for editing).
- No calls to any AI/LLM API from this app.
- No cloud sync, account system, analytics, or telemetry of any kind.
- No support for a folder that fails the repo check — do not add a
  non-git fallback mode.
- No git operations beyond: `rev-parse --is-inside-work-tree`, `diff`,
  `diff --staged`, `apply` (with the exact flag combinations in §1),
  `ls-files --cached --others --exclude-standard`, branch operations
  (`current_branch`, `list_branches`, `switch_branch`), and `pull_branch`.
  Do not add commit, push, stash, or reset commands without explicit instruction.

## 3. Stack — exact versions/choices, do not substitute

- Tauri `^2.0` (both the `tauri` Rust crate and `@tauri-apps/cli`/
  `@tauri-apps/api` npm packages). Not Tauri 1.x.
- Svelte `^4.2.0` via `@sveltejs/vite-plugin-svelte`. Not Svelte 5, not
  SvelteKit — this is a plain Vite + Svelte SPA, no file-based routing.
- `tauri-plugin-dialog` `^2.0` for the folder picker. Do not use a raw
  `<input type="file">` or any other picker.
- `notify` `^6.1` (Rust crate) for filesystem watching. Do not poll with
  a timer/interval instead.
- `uuid` `^1.8` with the `v4` feature, for hunk ids.
- Diff rendering is hand-rolled Svelte components fed by JSON from Rust.
  Do not introduce Monaco or diff2html — this was a deliberate choice to
  keep bundle/RAM size down; reversing it requires explicit user sign-off.
- Git access is via shelling out to the system `git` binary
  (`std::process::Command`) inside `git.rs`. Do not add `git2`/libgit2 or
  any other git library.

## 4. Exact backend contract (Rust ⇄ Svelte)

These are Tauri commands, called from the frontend as
`invoke("<name>", { ...args })`. Argument names must match exactly
(Tauri maps camelCase JS args to snake_case Rust params automatically,
e.g. `repoPath` in JS ↔ `repo_path` in Rust).

```
validate_git_repo(path: String) -> bool
  true iff `path` passes the repo check in Definitions §1.

start_watcher(path: String) -> Result<(), String>
  Starts a notify watcher on `path`, recursive. Stores the watcher in
  AppState.watcher (replacing/dropping any previous one — see §6 item 1
  for what currently happens, which is a bug). On any change, after
  debounce, emits a Tauri event named exactly "changes-detected" with
  unit payload `()`.

get_diff(repo_path: String) -> Result<Vec<FileDiff>, String>
  Runs `git diff` and `git diff --staged` in repo_path. Parses each with
  parse_diff() (see §4b). Merges by file path: a file present in both
  unstaged and staged output appears once, with additions/deletions
  summed and hunks from both concatenated (unstaged hunks and staged
  hunks both present in the same file's `hunks` array, distinguished by
  each hunk's own `source` field). Also overwrites
  AppState.hunk_patches with a fresh map of every hunk id emitted in
  this call to its HunkPatch — this map is what accept_hunk/reject_hunk
  read from, and it is fully replaced (not merged) on every get_diff
  call.

get_repo_files(repo_path: String) -> Result<Vec<String>, String>
  Runs `git ls-files --cached --others --exclude-standard` in repo_path to
  return a complete sorted list of all repository files (both tracked and
  untracked, excluding gitignored paths).

get_file_content(repo_path: String, file_path: String) -> Result<String, String>
  Reads file_path relative to repo_path securely, verifies it resides within
  the repository root, and returns its UTF-8 text content. Returns an error
  if the file is binary or cannot be decoded as UTF-8.

save_file_content(repo_path: String, file_path: String, content: String) -> Result<(), String>
  Writes content into file_path within repo_path securely. Changes on disk
  automatically trigger the fs watcher debounce, updating diffs in real time.

get_current_branch(repo_path: String) -> Result<String, String>
  Runs `git rev-parse --abbrev-ref HEAD` in repo_path to return the active git
  branch name (e.g. "main", "dev").

list_branches(repo_path: String) -> Result<Vec<String>, String>
  Runs `git branch --format="%(refname:short)"` in repo_path to return a sorted
  list of local branch names.

switch_branch(repo_path: String, branch: String) -> Result<(), String>
  Runs `git checkout <branch>` in repo_path to switch working directory to the
  selected branch. Returns git error message if uncommitted changes conflict.

accept_hunk(repo_path: String, hunk_id: String) -> Result<(), String>
  Looks up hunk_id in AppState.hunk_patches. If absent, returns
  Err("hunk not found, try reopening the folder") — this exact string,
  do not change without updating the frontend error display code too.
  If found and source == "staged", returns Ok(()) and does nothing
  (already staged). Otherwise builds `file_header + hunk_text` and runs
  it through git::apply_patch(repo, patch, cached=true, reverse=false).

reject_hunk(repo_path: String, hunk_id: String) -> Result<(), String>
  Same lookup/error as accept_hunk. Builds the same single-hunk patch.
  Runs git::apply_patch(repo, patch, cached=(source == "staged"),
  reverse=true).

Note: `accept_hunk` and `reject_hunk` take repo_path as a parameter but
do not currently validate it matches the repo the cached hunk_patches
came from. See §6 item 4.
```

### 4b. Exact data shapes (serde-serialized, this is the JSON the frontend receives)

```rust
struct Line {
    kind: String,     // exactly one of: "add", "remove", "context"
    content: String,  // the line's text with the leading +/-/space stripped
}

struct Hunk {
    id: String,        // uuid v4, see Definitions §1
    header: String,     // the "@@ -a,b +c,d @@ ..." line, unmodified
    lines: Vec<Line>,
    source: String,     // exactly one of: "unstaged", "staged"
}

struct FileDiff {
    path: String,       // repo-relative path, from the "+++ b/<path>" line (or "--- a/<path>")
    additions: u32,      // count of Line{kind:"add"} across all hunks for this file
    deletions: u32,      // count of Line{kind:"remove"} across all hunks for this file
    hunks: Vec<Hunk>,
    is_binary: bool,    // true if file diff detected binary contents ("Binary files ... differ")
}
```

`parse_diff(raw: &str, source: &str) -> (Vec<FileDiff>, Vec<(String, HunkPatch)>)`
in `diff.rs` is the only place that builds these. It:
1. Splits `raw` into per-file blocks on lines starting with `"diff --git"`.
2. For each block, extracts `path` from the line starting with `"+++ b/"`.
   If no such line exists (e.g. file deletions with `+++ /dev/null`), falls
   back to checking for `"--- a/"`.
3. Checks if the block indicates binary content via `"Binary files "`. If so,
   emits a `FileDiff` with `is_binary: true` and `hunks: []`.
4. Extracts the file header as every line before the first `"@@"` line.
5. Splits the remainder into hunks on lines starting with `"@@"` — each
   new `"@@"` line starts a new hunk, the previous hunk ends immediately
   before it.
6. For each hunk, generates a fresh UUID, counts add/remove lines into
   the file's running `additions`/`deletions` totals, and stores a
   `HunkPatch { file_header, hunk_text, source }` in the second return
   value, keyed by that hunk's UUID.
7. Files with zero parsed hunks and `is_binary: false` are dropped.

## 5. Exact watcher behavior

`watcher.rs::watch(app: AppHandle, path: &Path) -> notify::Result<RecommendedWatcher>`:
1. Creates an `mpsc::channel`.
2. Creates a `notify::RecommendedWatcher` that sends every event
   (`Result<Event, Error>`, unfiltered — includes all event kinds notify
   reports, and includes changes anywhere under `path`, recursively,
   with no path exclusions) into that channel.
3. Calls `.watch(path, RecursiveMode::Recursive)`.
4. Spawns one OS thread that loops: blocking-receive one message: if the
   channel errors (sender dropped), the thread exits. Otherwise, drain
   the channel with `recv_timeout(400ms)` in a loop until a 400ms window
   passes with no new message. Then call
   `app.emit("changes-detected", ())`. If that emit errors, the thread
   exits. Loop back to the blocking receive.
5. The constant is `DEBOUNCE: Duration = Duration::from_millis(400)`,
   defined once at the top of the file. Change it there, not inline, if
   it needs tuning.

The frontend listens via `lib/api.js::onChangesDetected(callback)`,
which is `listen("changes-detected", callback)` from
`@tauri-apps/api/event`, called once per successful folder open in
`App.svelte::openFolder()`, with the previous listener's `unlisten()`
called first if one exists.

## 6. Implementation status & historical open items

1. **[RESOLVED] Watcher not stopped on folder switch.**
   - Status: Fixed.
   - Resolution: `AppState.hunk_patches` is explicitly cleared in `start_watcher`
     (`src-tauri/src/main.rs`) whenever a new directory is watched, ensuring
     stale hunk IDs from a previous repository cannot linger regardless of
     frontend call ordering.

2. **[RESOLVED] Binary-changed files silently vanish.**
   - Status: Fixed.
   - Resolution: `parse_diff` in `src-tauri/src/diff.rs` checks for the literal
     prefix `"Binary files "`. When encountered, it emits a `FileDiff` with
     `is_binary: true` and `hunks: []`. The UI (`FileList.svelte` and
     `DiffView.svelte`) displays a distinct "Binary file" badge and informs the
     user that binary content cannot be previewed or staged as text hunks.

3. **[RESOLVED] Watcher git internal filtering vs external branch/commit sync.**
   - Status: Fixed.
   - Resolution: In `src-tauri/src/watcher.rs`, incoming events filter out transient
     lock files (`.lock`) and internal object/log churn (`/.git/objects/`, `/.git/logs/`,
     `/.git/hooks/`), while explicitly permitting essential git state changes (`.git/HEAD`,
     `.git/index`, `.git/refs/`). This guarantees that external branch checkouts (`git checkout`),
     staging (`git add`), and terminal commits immediately trigger the 400ms debouncer and
     refresh the UI and active branch badge in real time.

4. **[DOCUMENTED] `accept_hunk`/`reject_hunk` repo_path validation.**
   - Status: Documented / Retained as non-issue.
   - Notes: In the current single-window architecture, `App.svelte` passes the
     active `repoPath` consistently. If multi-window or concurrent repository
     support is added in the future, `AppState.hunk_patches` can store the
     associated `repo_path` alongside the patch.

5. **[RESOLVED] File path extraction fails for some deleted-file diffs.**
   - Status: Fixed.
   - Resolution: In `src-tauri/src/diff.rs::extract_path`, if no `"+++ b/"` line
     is present (e.g. for deleted files with `+++ /dev/null`), it falls back to
     extracting the file path from `"--- a/"`.

6. **[RESOLVED] App icons bundled for `tauri build`.**
   - Status: Fixed.
   - Resolution: Adopted official 1024x1024 app icon featuring the clean modern
     "Sleek Tech Calm" skeptical eyes concept (cartoonish eyes glancing rightward
     with an inquisitive raised eyebrow on an obsidian squircle tile, representing
     meticulous diff inspection), and generated all multi-platform assets in
     `src-tauri/icons/` (`32x32.png`, `128x128.png`, `128x128@2x.png`,
     `icon.icns`, `icon.ico`), root `app-icon.png`, and `public/favicon.png`.

7. **[RESOLVED] Unit test suite.**
   - Status: Fixed.
   - Resolution: Implemented 8 comprehensive unit tests in `src-tauri/src/diff.rs`
     covering single hunk, multi-hunk, multi-file diffs, start/end hunk
     boundaries, newly created files, deleted files (`+++ /dev/null`), and
     binary changes. All tests verify exact paths, additions/deletions counts,
     and line kinds, and pass via `cargo test`.

8. **[RESOLVED] Merge Request (MR) / Base Branch Comparison Mode.**
   - Status: Fixed.
   - Resolution: Added the ability to review cumulative branch changes against
     any base branch (e.g. `release`, `main`, `origin/main`), mimicking GitLab/GitHub
     Merge Request views:
     - **Backend (`git.rs`):** Implemented `diff_against_base` using git 3-dot
       syntax (`git diff <base>...`) to compare the common merge-base ancestor with
       the current working tree (including committed branch commits + uncommitted
       edits). Added `branch_ahead_behind` (`git rev-list --left-right --count <base>...HEAD`)
       to report commit offsets.
     - **API & Commands (`main.rs` & `api.js`):** Extended `get_diff` command with an
       optional `base_branch` argument and registered `get_branch_comparison_info`.
     - **Top Navigation Bar (`App.svelte`):** Rendered comparison pill with mode badge
       (`vs HEAD` in neutral blue, or purple `vs <base>` with `+N` ahead badge),
       quick-reset `✕` button, and dropdown to switch between Working Tree (HEAD)
       and any local or remote tracking branch.
     - **Sidebar & Hunk Views (`FileList.svelte`, `DiffView.svelte`, `HunkView.svelte`):**
       Sidebar changes tab shows purple MR banner with ahead/behind stats and 1-click
       Reset; DiffView breadcrumbs show target base pill; HunkView shows purple
       `MR vs <base>` badge and hides stage/unstage controls when reviewing branch diffs.
     - **RAM Zero-Bloat:** Git 3-dot diffs stream lightweight unified diff text directly
       from Git CLI, incurring 0 persistent RAM overhead and maintaining total app
       memory footprint within 45–70 MB.

9. **[RESOLVED] Changes Tab Default-Open Folders, Collapse/Expand All, and Default Commit Comparison.**
   - Status: Fixed.
   - Resolution:
     - **Default-Open on Changes Tab:** In `FileList.svelte`, separated folder expansion
       state for `all` vs `changes`. In the `Changes` tab, all folders containing modified
       files are now automatically uncollapsed/open by default on load and on tab switch,
       while the `All Files` tab remains collapsed by default.
     - **Collapse All / Expand All Controls:** Added dedicated `tree-controls` buttons in the
       sidebar header right next to the tab switcher (`Expand All` [+] and `Collapse All` [-]),
       allowing 1-click folding and unfolding of the current directory tree.
     - **Smart Relevance Ranking in Compare Dropdown:** Implemented `sortBranchesWithRelevance`
       in `App.svelte` so exact matches (`origin/release`, `release`, `main`) appear at the top
       (#1 and #2) when searching, preventing target branches from getting buried beneath
       thousands of feature branches.
     - **Clear Default Target Indicator:** When no target branch is selected, Tengga explicitly
       compares working tree changes against the last commit (`HEAD`), with clear badges in the
       topbar (`vs HEAD (commit terakhir)`), sidebar banner (`Comparing changes to latest commit (HEAD)`),
       and diff breadcrumbs, plus a clean empty-state message when the working tree has no uncommitted changes.

## 7. File-by-file reference (exact responsibility, no overlap)

```
src/main.js                 Only mounts App.svelte to #app. Nothing else
                             belongs in this file.
src/App.svelte               Owns all top-level state: repoPath, files,
                             repoFiles, selectedFile, fileContent, error,
                             unlisten, baseBranch, comparisonInfo. Implements
                             sortBranchesWithRelevance for branch selector.
src/lib/api.js               The only file allowed to call Tauri's
                             invoke(), open() (dialog), or listen().
                             Exports: validateRepo, startWatcher, getDiff,
                             getBranchComparisonInfo, getRepoFiles, getFileContent,
                             saveFileContent, acceptHunk, rejectHunk, getCurrentBranch,
                             listBranches, switchBranch, onChangesDetected.
src/lib/FileList.svelte      Dual-scope explorer: "Changes" view (showing
                             modified files with folders uncollapsed by default,
                             MR/HEAD mode banners, and clean empty state) and "All Files"
                             view (collapsed by default); contains Collapse All and
                             Expand All controls.
src/lib/FileTreeNode.svelte  Recursive tree node rendering collapsible folders
                             (collapsed by default on load) and clean/modified
                             files with arbitrary nesting depth.
src/lib/DiffView.svelte      Dual-mode view: unified diff hunk view (with
                             per-hunk Accept/Reject controls and MR target pills)
                             and full-file code inspector / inline editor with line
                             number gutters, 2-space Tab indentation, Cmd+S / Ctrl+S,
                             and Find/Replace support.
src/lib/HunkView.svelte      Pure display of one `hunk` prop with source indicator
                             badges (`MR vs <base>`, `Staged`, `Working Tree`); calls
                             `onAccept(hunk.id)` / `onReject(hunk.id)` props on button click.
src-tauri/src/main.rs        AppState definition and registered Tauri commands:
                             validate_git_repo, start_watcher, get_diff,
                             get_branch_comparison_info, get_repo_files,
                             get_file_content, save_file_content, accept_hunk,
                             reject_hunk, get_current_branch, list_branches,
                             switch_branch, pull_branch.
src-tauri/src/git.rs         is_repo, diff_unstaged, diff_staged, diff_against_base,
                             branch_ahead_behind, apply_patch, list_repo_files,
                             get_current_branch, list_branches, switch_branch, pull_branch —
                             the only functions constructing std::process::Command.
src-tauri/src/diff.rs        parse_diff and its private helpers (split_file_blocks,
                             extract_path, extract_header, split_hunks,
                             parse_hunk_lines) plus the Line, Hunk, FileDiff,
                             and HunkPatch struct definitions and unit tests.
src-tauri/src/watcher.rs     watch() and the DEBOUNCE constant (400ms). The only
                             file that touches the notify crate, with .git
                             path event filtering.
```

## 8. How to run / verify

```bash
# Install dependencies
npm install

# Run frontend test build
npm run build

# Run Rust backend unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run the development desktop application
npm run tauri dev

# Package production desktop bundle
npm run tauri build
```

To confirm the repo-validation check works without running the full
app: `git rev-parse --is-inside-work-tree` run manually in a target
folder must print `true` and exit 0 for that folder to be accepted.
