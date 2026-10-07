<script>
  import FileTreeNode from "./FileTreeNode.svelte";

  export let files = [];       // Modified files from get_diff: { path, additions, deletions, hunks, is_binary }
  export let repoFiles = [];   // All files in repo: ["src/index.js", "README.md", ...]
  export let selectedFile = null;
  export let dirtyFiles = new Set();
  export let branch = null;
  export let baseBranch = null;
  export let comparisonInfo = null;
  export let onSelect;
  export let currentTheme = "light";
  export let onThemeChange = () => {};

  let filterScope = "all";    // "all" | "changes"
  let searchQuery = "";
  let expandedAllFolders = {};
  let expandedChangesFolders = {};
  let prevFiles = null;
  let prevRepoFiles = null;

  function computeAllFolderPaths(fileList) {
    const res = {};
    for (const f of fileList) {
      const p = typeof f === "string" ? f : f.path;
      if (!p) continue;
      const parts = p.split("/");
      let acc = "";
      for (let i = 0; i < parts.length - 1; i++) {
        acc = acc ? `${acc}/${parts[i]}` : parts[i];
        res[acc] = true;
      }
    }
    return res;
  }

  // When files change, by default in Changes tab all folders should be open/uncollapsed
  $: if (files !== prevFiles) {
    prevFiles = files;
    expandedChangesFolders = computeAllFolderPaths(files);
  }

  // Reset all folders to collapsed when switching repository
  $: if (repoFiles !== prevRepoFiles) {
    prevRepoFiles = repoFiles;
    expandedAllFolders = {};
    expandedChangesFolders = computeAllFolderPaths(files);
  }

  // Active expanded folders based on current scope
  $: expandedFolders = filterScope === "changes" ? expandedChangesFolders : expandedAllFolders;

  // Build a lookup map of diff info by file path
  $: diffMap = new Map(files.map((f) => [f.path, f]));

  // Combine repoFiles with any modified files (e.g. untracked or deleted) into a unified set of paths
  $: unifiedPaths = Array.from(new Set([...repoFiles, ...files.map((f) => f.path)])).sort();

  // When a file is selected, ensure its ancestor folders are expanded so it is visible in tree
  $: if (selectedFile) {
    expandAncestors(selectedFile);
  }

  function expandAncestors(filePath) {
    if (!filePath) return;
    const parts = filePath.split("/");
    let acc = "";
    let changed = false;
    const targetMap = filterScope === "changes" ? expandedChangesFolders : expandedAllFolders;
    for (let i = 0; i < parts.length - 1; i++) {
      acc = acc ? `${acc}/${parts[i]}` : parts[i];
      if (!targetMap[acc]) {
        targetMap[acc] = true;
        changed = true;
      }
    }
    if (changed) {
      if (filterScope === "changes") {
        expandedChangesFolders = { ...expandedChangesFolders };
      } else {
        expandedAllFolders = { ...expandedAllFolders };
      }
    }
  }

  function toggleFolder(path) {
    if (filterScope === "changes") {
      expandedChangesFolders[path] = !expandedChangesFolders[path];
      expandedChangesFolders = { ...expandedChangesFolders };
    } else {
      expandedAllFolders[path] = !expandedAllFolders[path];
      expandedAllFolders = { ...expandedAllFolders };
    }
  }

  function getAllFolderPaths(nodes) {
    let paths = [];
    for (const n of nodes) {
      if (n.type === "folder") {
        paths.push(n.path);
        if (n.childrenList) {
          paths = paths.concat(getAllFolderPaths(n.childrenList));
        }
      }
    }
    return paths;
  }

  function collapseAll() {
    if (filterScope === "changes") {
      expandedChangesFolders = {};
    } else {
      expandedAllFolders = {};
    }
  }

  function expandAll() {
    const allPaths = getAllFolderPaths(treeData);
    const next = {};
    for (const p of allPaths) {
      next[p] = true;
    }
    if (filterScope === "changes") {
      expandedChangesFolders = next;
    } else {
      expandedAllFolders = next;
    }
  }

  // Filter based on scope (all files vs changes only) and text search
  $: filteredPaths = unifiedPaths.filter((path) => {
    const hasDiff = diffMap.has(path) || (dirtyFiles && dirtyFiles.has(path));
    if (filterScope === "changes" && !hasDiff) return false;
    if (searchQuery && !path.toLowerCase().includes(searchQuery.toLowerCase().trim())) {
      return false;
    }
    return true;
  });

  $: treeData = buildTree(filteredPaths, diffMap, dirtyFiles);

  $: totalAdditions = files.reduce((acc, f) => acc + (f.additions || 0), 0);
  $: totalDeletions = files.reduce((acc, f) => acc + (f.deletions || 0), 0);

  function buildTree(paths, diffs, dirtySet) {
    const root = { name: "", type: "folder", children: {}, path: "", additions: 0, deletions: 0, hasChanges: false };

    for (const path of paths) {
      const parts = path.split("/");
      let current = root;
      let accumulatedPath = "";

      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        accumulatedPath = accumulatedPath ? `${accumulatedPath}/${part}` : part;
        const isFile = i === parts.length - 1;

        if (isFile) {
          const diffInfo = diffs.get(path);
          const isDirty = dirtySet ? dirtySet.has(path) : false;
          const hasChanges = Boolean(diffInfo) || isDirty;
          current.children[part] = {
            name: part,
            type: "file",
            path,
            hasChanges,
            isDirty,
            additions: diffInfo ? diffInfo.additions : 0,
            deletions: diffInfo ? diffInfo.deletions : 0,
            is_binary: diffInfo ? diffInfo.is_binary : false,
          };
        } else {
          if (!current.children[part]) {
            current.children[part] = {
              name: part,
              type: "folder",
              path: accumulatedPath,
              children: {},
              additions: 0,
              deletions: 0,
              hasChanges: false,
            };
          }
          current = current.children[part];
        }
      }
    }

    function sumAndSort(node) {
      if (node.type === "file") return;
      let adds = 0;
      let dels = 0;
      let changed = false;

      const sorted = Object.values(node.children).sort((a, b) => {
        if (a.type !== b.type) return a.type === "folder" ? -1 : 1;
        return a.name.localeCompare(b.name);
      });

      for (const child of sorted) {
        sumAndSort(child);
        adds += child.additions || 0;
        dels += child.deletions || 0;
        if (child.hasChanges) changed = true;
      }

      node.additions = adds;
      node.deletions = dels;
      node.hasChanges = changed;
      node.childrenList = sorted;
    }

    sumAndSort(root);
    return root.childrenList || [];
  }
</script>

<div class="sidebar-wrapper">
  <!-- Top Scope Switcher -->
  <div class="sidebar-header">
    <div class="scope-row">
      <div class="segmented-control">
        <button
          class="seg-btn"
          class:active={filterScope === "all"}
          on:click={() => (filterScope = "all")}
        >
          All Files
        </button>
        <button
          class="seg-btn"
          class:active={filterScope === "changes"}
          on:click={() => (filterScope = "changes")}
        >
          Changes ({files.length})
        </button>
      </div>

      <div class="tree-controls">
        <button
          class="tree-control-btn"
          on:click={expandAll}
          title="Uncollapse all folders (Expand All)"
        >
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <path d="M1 2.5A1.5 1.5 0 0 1 2.5 1h3.086a1.5 1.5 0 0 1 1.06.44l1.354 1.353a.5.5 0 0 0 .354.147h5.146A1.5 1.5 0 0 1 15 4.44v1.56h-1.5V4.44a.25.25 0 0 0-.25-.25H8.354a2 2 0 0 1-1.415-.586L5.586 2.25H2.5a.25.25 0 0 0-.25.25V5.5H1v-3z"/>
            <path d="M8 8a.75.75 0 0 1 .75.75v2h2a.75.75 0 0 1 0 1.5h-2v2a.75.75 0 0 1-1.5 0v-2h-2a.75.75 0 0 1 0-1.5h2v-2A.75.75 0 0 1 8 8z"/>
          </svg>
        </button>
        <button
          class="tree-control-btn"
          on:click={collapseAll}
          title="Collapse all folders"
        >
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <path d="M1 2.5A1.5 1.5 0 0 1 2.5 1h3.086a1.5 1.5 0 0 1 1.06.44l1.354 1.353a.5.5 0 0 0 .354.147h5.146A1.5 1.5 0 0 1 15 4.44v1.56h-1.5V4.44a.25.25 0 0 0-.25-.25H8.354a2 2 0 0 1-1.415-.586L5.586 2.25H2.5a.25.25 0 0 0-.25.25V5.5H1v-3z"/>
            <path d="M5.5 10.25a.75.75 0 0 1 .75-.75h5a.75.75 0 0 1 0 1.5h-5a.75.75 0 0 1-.75-.75z"/>
          </svg>
        </button>
      </div>
    </div>

    {#if baseBranch}
      <div class="mr-mode-banner">
        <div class="mr-mode-info">
          <div class="mr-mode-text">
            <span class="mr-mode-title">comparing to: <strong>{baseBranch}</strong></span>
            {#if comparisonInfo}
              <span class="mr-mode-stats">
                {comparisonInfo.ahead} commit{comparisonInfo.ahead === 1 ? "" : "s"} ahead
                {#if comparisonInfo.behind > 0}
                  · {comparisonInfo.behind} behind
                {/if}
              </span>
            {/if}
          </div>
        </div>
      </div>
    {:else if filterScope === "changes"}
      <div class="head-mode-banner">
        <span class="head-mode-title">Comparing changes to latest commit (HEAD)</span>
      </div>
    {/if}

    <!-- Search / Filter input (clean text only, no icons) -->
    <div class="filter-row">
      <input
        type="text"
        placeholder="Filter files..."
        bind:value={searchQuery}
        class="filter-input"
      />
      {#if searchQuery}
        <button class="clear-btn" on:click={() => (searchQuery = "")}>✕</button>
      {/if}
    </div>
  </div>

  <!-- Main Scrollable Directory Structure -->
  <div class="file-tree-container">
    <div class="tree-root">
      {#each treeData as node (node.path)}
        <FileTreeNode
          {node}
          depth={0}
          {selectedFile}
          {expandedFolders}
          {searchQuery}
          {toggleFolder}
          {onSelect}
        />
      {/each}
    </div>

    {#if filteredPaths.length === 0}
      {#if filterScope === "changes" && !searchQuery}
        <div class="empty-changes-state">
          <div class="empty-icon">✓</div>
          {#if baseBranch}
            <div class="empty-title">No Differences Found</div>
            <div class="empty-desc">Branch is completely up to date with {baseBranch}.</div>
          {:else}
            <div class="empty-title">Clean Working Tree</div>
            <div class="empty-desc">No uncommitted changes since latest commit (HEAD).</div>
          {/if}
        </div>
      {:else}
        <div class="empty-state">
          <span class="empty-title">
            {searchQuery ? `No files match "${searchQuery}"` : "No files found"}
          </span>
        </div>
      {/if}
    {/if}
  </div>

  <!-- Bottom status bar & theme switcher -->
  <div class="sidebar-footer">
    <div class="footer-theme-switcher" role="radiogroup" aria-label="Theme mode">
      <button
        type="button"
        class="theme-btn"
        class:active={currentTheme === "light"}
        on:click={() => onThemeChange("light")}
        title="Light Mode"
        aria-label="Light Mode"
      >
        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="M8 11a3 3 0 1 1 0-6 3 3 0 0 1 0 6zm0 1a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM8 0a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0V.75A.75.75 0 0 1 8 0zm0 13a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0v-1.5A.75.75 0 0 1 8 13zm8-5a.75.75 0 0 1-.75.75h-1.5a.75.75 0 0 1 0-1.5h1.5A.75.75 0 0 1 16 8zM3 8a.75.75 0 0 1-.75.75H.75a.75.75 0 0 1 0-1.5h1.5A.75.75 0 0 1 3 8zm10.657-4.243a.75.75 0 0 1 0 1.061l-1.06 1.06a.75.75 0 0 1-1.061-1.06l1.06-1.061a.75.75 0 0 1 1.061 0zm-9.193 9.192a.75.75 0 0 1 0 1.061l-1.06 1.06a.75.75 0 0 1-1.062-1.06l1.061-1.061a.75.75 0 0 1 1.061 0zm9.193 0a.75.75 0 0 1-1.061 0l-1.06-1.06a.75.75 0 0 1 1.06-1.061l1.061 1.06a.75.75 0 0 1 0 1.061zM4.464 4.818a.75.75 0 0 1-1.06 0l-1.061-1.06a.75.75 0 0 1 1.06-1.061l1.061 1.06a.75.75 0 0 1 0 1.061z"/>
        </svg>
      </button>
      <button
        type="button"
        class="theme-btn"
        class:active={currentTheme === "dark"}
        on:click={() => onThemeChange("dark")}
        title="Dark Mode"
        aria-label="Dark Mode"
      >
        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="M6 .278a.768.768 0 0 1 .08.858 7.208 7.208 0 0 0-.878 3.46c0 4.021 3.278 7.277 7.318 7.277.527 0 1.04-.055 1.533-.16a.787.787 0 0 1 .81.316.733.733 0 0 1-.031.893A8.349 8.349 0 0 1 8.344 16C3.734 16 0 12.286 0 7.71 0 4.266 2.114 1.312 5.124.06A.752.752 0 0 1 6 .278z"/>
        </svg>
      </button>
      <button
        type="button"
        class="theme-btn"
        class:active={currentTheme === "glass"}
        on:click={() => onThemeChange("glass")}
        title="Glass Mode (Translucent)"
        aria-label="Glass Mode"
      >
        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="M2 3.75C2 2.784 2.784 2 3.75 2h8.5c.966 0 1.75.784 1.75 1.75v8.5A1.75 1.75 0 0 1 12.25 14h-8.5A1.75 1.75 0 0 1 2 12.25v-8.5zm1.75-.25a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-8.5a.25.25 0 0 0-.25-.25h-8.5zM4 5.5a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 4 5.5zm0 3a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 4 8.5zm0 3a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 0 1.5h-3.5A.75.75 0 0 1 4 11.5z"/>
        </svg>
      </button>
    </div>

    <span class="summary-text">
      {#if baseBranch}
        <span class="footer-compare-label" title="Comparing {branch} against {baseBranch}">
          {branch} ← {baseBranch}
        </span>
        <span class="dot-sep">·</span>
      {:else if branch}
        <span class="footer-branch" title="Current git branch: {branch}">
          <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor" style="vertical-align: -1px; margin-right: 3px;">
            <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z"/>
          </svg>{branch}
        </span>
        <span class="dot-sep">·</span>
      {/if}
      {files.length} changed
      {#if totalAdditions > 0 || totalDeletions > 0}
        &nbsp;·&nbsp;<span class="add">+{totalAdditions}</span>&nbsp;<span class="del">-{totalDeletions}</span>
      {/if}
    </span>
  </div>
</div>

<style>
  .sidebar-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: transparent;
    user-select: none;
  }

  .sidebar-header {
    padding: 10px 12px;
    background: var(--bg-topbar);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .scope-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .segmented-control {
    display: flex;
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: 9999px;
    padding: 2px;
    flex: 1;
  }

  .tree-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .tree-control-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: 50%;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.12s ease;
    padding: 0;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03);
  }

  .tree-control-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border-hover);
    color: var(--accent-emerald);
  }

  .seg-btn {
    flex: 1;
    text-align: center;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 11.5px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 9999px;
    cursor: pointer;
    font-family: inherit;
    transition: all 0.15s ease;
  }

  .seg-btn:hover {
    color: var(--text-primary);
  }

  .seg-btn.active {
    background: var(--bg-card);
    color: var(--text-primary);
    font-weight: 700;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }

  .filter-row {
    position: relative;
    display: flex;
    align-items: center;
  }

  .filter-input {
    width: 100%;
    background: var(--bg-subtle);
    border-radius: 4px;
    padding: 6px 24px 6px 12px;
    border: none;
    font-size: 11.5px;
    color: var(--text-primary);
    outline: none;
    box-sizing: border-box;
    font-family: inherit;
    transition: all 0.15s ease;
  }

  .filter-input::placeholder {
    color: var(--text-muted);
  }

  .filter-input:focus {
    background: var(--bg-card);
    box-shadow: 0 0 0 1px var(--accent-glow);
  }

  .clear-btn {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }

  .clear-btn:hover {
    color: var(--text-secondary);
  }

  .file-tree-container {
    flex: 1;
    overflow-y: auto;
    padding: 6px 4px;
  }

  .tree-root {
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    padding: 30px 14px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }

  .sidebar-footer {
    padding: 6px 10px;
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-topbar);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 11px;
    color: var(--text-muted);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    user-select: none;
    flex-shrink: 0;
  }

  .footer-theme-switcher {
    display: inline-flex;
    align-items: center;
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: 9999px;
    padding: 2px;
    gap: 1px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03);
    flex-shrink: 0;
  }

  .theme-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 20px;
    border-radius: 9999px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.12s ease;
    padding: 0;
  }

  .theme-btn:hover {
    color: var(--text-primary);
  }

  .theme-btn.active {
    background: var(--bg-card);
    color: var(--accent-emerald);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
  }

  .summary-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: right;
  }

  .summary-text .add { color: var(--diff-add-sign); font-weight: 700; }
  .summary-text .del { color: var(--diff-del-sign); font-weight: 700; }

  .footer-branch {
    color: var(--accent-emerald);
    font-weight: 600;
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    vertical-align: middle;
  }

  .dot-sep {
    color: var(--border-default);
    margin: 0 4px;
  }

  .mr-mode-banner {
    display: flex;
    align-items: center;
    background: var(--accent-emerald-soft);
    border-radius: 12px;
    padding: 6px 10px;
    margin-top: 4px;
  }

  .mr-mode-info {
    display: flex;
    align-items: center;
    gap: 7px;
    overflow: hidden;
    width: 100%;
  }

  .mr-mode-text {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    line-height: 1.35;
    width: 100%;
  }

  .mr-mode-title {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mr-mode-title strong {
    color: var(--accent-emerald);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, monospace;
    font-weight: 700;
  }

  .mr-mode-stats {
    font-size: 10px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .footer-compare-label {
    color: var(--accent-emerald);
    font-weight: 600;
    font-size: 10.5px;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, monospace;
  }

  .head-mode-banner {
    display: flex;
    align-items: center;
    border-radius: 4px;
    padding: 6px 10px;
    margin-top: 4px;
  }

  .head-mode-title {
    font-size: 10px;
    font-weight: 600;
    color: var(--accent-emerald-dark);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .empty-changes-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 16px;
    text-align: center;
    gap: 8px;
  }

  .empty-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--accent-emerald-soft);
    border: 1px solid var(--accent-emerald-border);
    color: var(--accent-emerald);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: 700;
  }

  .empty-title {
    font-size: 12.5px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .empty-desc {
    font-size: 11px;
    line-height: 1.5;
    color: var(--text-muted);
    max-width: 200px;
  }
</style>
