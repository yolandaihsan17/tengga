<script>
  import { onMount, tick } from "svelte";

  export let files = []; // Changed files with diff info: [{ path, status, ... }]
  export let repoFiles = []; // All repo file paths: string[]
  export let onSelect = () => {};
  export let onClose = () => {};

  let query = "";
  let selectedIndex = 0;
  let inputEl = null;
  let listEl = null;

  // Build map of changed files for quick status lookup
  $: changedFilesMap = new Map((files || []).map((f) => [f.path, f]));

  // Combine and deduplicate files
  $: allPaths = (() => {
    const set = new Set();
    const result = [];

    // Prioritize changed files first in the base list
    for (const f of files || []) {
      if (f.path && !set.has(f.path)) {
        set.add(f.path);
        result.push(f.path);
      }
    }

    // Add remaining repository files
    for (const p of repoFiles || []) {
      if (p && !set.has(p)) {
        set.add(p);
        result.push(p);
      }
    }

    return result;
  })();

  function parsePath(fullPath) {
    const lastSlash = fullPath.lastIndexOf("/");
    const fileName = lastSlash !== -1 ? fullPath.substring(lastSlash + 1) : fullPath;
    const folderPath = lastSlash !== -1 ? fullPath.substring(0, lastSlash) : "";
    return { path: fullPath, fileName, folderPath };
  }

  // Filter and rank files based on query
  $: filteredFiles = (() => {
    const q = query.trim().toLowerCase();

    if (!q) {
      // Default: show changed files first, followed by other repo files
      return allPaths.slice(0, 60).map((path) => {
        const parsed = parsePath(path);
        const changed = changedFilesMap.get(path);
        return {
          ...parsed,
          status: changed ? (changed.status || "modified") : null,
          isChanged: Boolean(changed),
        };
      });
    }

    const scored = [];

    for (const path of allPaths) {
      const parsed = parsePath(path);
      const nameLower = parsed.fileName.toLowerCase();
      const pathLower = path.toLowerCase();
      const changed = changedFilesMap.get(path);

      let score = -1;

      if (nameLower === q) {
        score = 1000;
      } else if (nameLower.startsWith(q)) {
        score = 500 + (100 - parsed.fileName.length);
      } else if (nameLower.includes(q)) {
        score = 300 + (100 - parsed.fileName.length);
      } else if (pathLower.includes(q)) {
        score = 150;
      } else {
        // Subsequence match
        let qIdx = 0;
        for (let i = 0; i < pathLower.length && qIdx < q.length; i++) {
          if (pathLower[i] === q[qIdx]) {
            qIdx++;
          }
        }
        if (qIdx === q.length) {
          score = 50;
        }
      }

      if (score > 0) {
        if (changed) {
          score += 20; // slight boost for uncommitted files
        }
        scored.push({
          ...parsed,
          status: changed ? (changed.status || "modified") : null,
          isChanged: Boolean(changed),
          score,
        });
      }
    }

    scored.sort((a, b) => b.score - a.score);
    return scored.slice(0, 60);
  })();

  $: if (query !== undefined) {
    selectedIndex = 0;
  }

  onMount(() => {
    if (inputEl) {
      inputEl.focus();
    }
  });

  async function scrollToSelected() {
    await tick();
    if (!listEl) return;
    const item = listEl.children[selectedIndex];
    if (item && item.scrollIntoView) {
      item.scrollIntoView({ block: "nearest" });
    }
  }

  function handleKeydown(e) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (filteredFiles.length > 0) {
        selectedIndex = (selectedIndex + 1) % filteredFiles.length;
        scrollToSelected();
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (filteredFiles.length > 0) {
        selectedIndex = (selectedIndex - 1 + filteredFiles.length) % filteredFiles.length;
        scrollToSelected();
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filteredFiles.length > 0 && filteredFiles[selectedIndex]) {
        selectFile(filteredFiles[selectedIndex].path);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  function selectFile(filePath) {
    onSelect(filePath);
    onClose();
  }

  function escapeHtml(text) {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  function highlightMatches(text, searchStr) {
    if (!searchStr) return escapeHtml(text);
    const escapedText = escapeHtml(text);
    const escapedSearch = escapeHtml(searchStr).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(`(${escapedSearch})`, "gi");
    return escapedText.replace(regex, '<span class="match-highlight">$1</span>');
  }

  function getStatusLabel(status) {
    if (!status) return "";
    switch (status.toLowerCase()) {
      case "modified":
        return "M";
      case "added":
        return "A";
      case "deleted":
        return "D";
      case "untracked":
        return "U";
      case "renamed":
        return "R";
      default:
        return "M";
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="quick-open-backdrop" on:click={onClose}>
  <!-- Modal Card -->
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div
    class="quick-open-palette"
    role="dialog"
    aria-modal="true"
    aria-label="Quick Open File Palette"
    on:click|stopPropagation
  >
    <!-- Search Bar -->
    <div class="palette-input-wrapper">
      <svg class="search-icon" viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
        <path d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0z"/>
      </svg>
      <input
        bind:this={inputEl}
        type="text"
        placeholder="Type the name of a file to open (Cmd+P)..."
        bind:value={query}
        class="palette-input"
        spellcheck="false"
      />
      {#if query}
        <button
          class="clear-query-btn"
          on:click={() => {
            query = "";
            inputEl.focus();
          }}
          title="Clear search"
          aria-label="Clear search"
        >
          <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
            <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
          </svg>
        </button>
      {/if}
      <span class="results-badge">
        {filteredFiles.length} file{filteredFiles.length === 1 ? "" : "s"}
      </span>
    </div>

    <!-- Results List -->
    <div class="palette-results-list" bind:this={listEl}>
      {#if filteredFiles.length === 0}
        <div class="no-results-state">
          No matching files found for "{query}"
        </div>
      {:else}
        {#each filteredFiles as file, index (file.path)}
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
          <div
            class="palette-item"
            class:is-selected={index === selectedIndex}
            on:mouseenter={() => (selectedIndex = index)}
            on:click={() => selectFile(file.path)}
          >
            <!-- File Icon -->
            <span class="file-icon" class:is-changed={file.isChanged}>
              <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
                <path d="M3.75 1.5a1.75 1.75 0 0 0-1.75 1.75v9.5c0 .966.784 1.75 1.75 1.75h8.5A1.75 1.75 0 0 0 14 12.75v-6.5a.75.75 0 0 0-.22-.53l-4.5-4.5A.75.75 0 0 0 8.75 1H3.75zM8.5 2.5v3.25c0 .414.336.75.75.75H12.5v6.25a.25.25 0 0 1-.25.25H3.75a.25.25 0 0 1-.25-.25v-9.5a.25.25 0 0 1 .25-.25h4.75z"/>
              </svg>
            </span>

            <!-- File Name -->
            <span class="file-name">
              {@html highlightMatches(file.fileName, query)}
            </span>

            <!-- Folder Path -->
            {#if file.folderPath}
              <span class="folder-path">{file.folderPath}</span>
            {/if}

            <!-- Status Indicator -->
            {#if file.status}
              <span class="status-badge status-{file.status.toLowerCase()}">
                {getStatusLabel(file.status)}
              </span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    <!-- Footer Controls Hint -->
    <div class="palette-footer">
      <div class="footer-hint">
        <span class="key-hint">↑↓</span> navigate
        <span class="hint-sep">·</span>
        <span class="key-hint">↵</span> open
        <span class="hint-sep">·</span>
        <span class="key-hint">esc</span> close
      </div>
    </div>
  </div>
</div>

<style>
  .quick-open-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 50px;
    animation: fadeIn 0.12s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .quick-open-palette {
    width: 600px;
    max-width: 90vw;
    background: var(--bg-card);
    border-radius: 10px;
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.45), 0 0 0 1px var(--border-subtle, rgba(255, 255, 255, 0.08));
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: scaleDown 0.12s ease-out;
    color: var(--text-primary);
  }

  @keyframes scaleDown {
    from {
      transform: translateY(-8px) scale(0.98);
      opacity: 0.8;
    }
    to {
      transform: translateY(0) scale(1);
      opacity: 1;
    }
  }

  .palette-input-wrapper {
    display: flex;
    align-items: center;
    padding: 10px 14px;
    gap: 10px;
    background: var(--bg-card);
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.06));
  }

  .search-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .palette-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    font-size: 13.5px;
    font-family: inherit;
    color: var(--text-primary);
    padding: 2px 0;
  }

  .palette-input::placeholder {
    color: var(--text-muted);
  }

  .clear-query-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    line-height: 1;
  }

  .clear-query-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .results-badge {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--bg-subtle);
    padding: 2px 8px;
    border-radius: 9999px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .palette-results-list {
    max-height: 380px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 4px 0;
  }

  .palette-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 14px;
    cursor: pointer;
    user-select: none;
    font-size: 12.5px;
    transition: background 0.08s ease;
    border-left: 2px solid transparent;
  }

  .palette-item:hover,
  .palette-item.is-selected {
    background: var(--bg-hover);
    border-left-color: var(--accent-emerald);
  }

  .file-icon {
    display: inline-flex;
    align-items: center;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .file-icon.is-changed {
    color: var(--accent-emerald);
  }

  .file-name {
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.match-highlight) {
    color: var(--accent-emerald);
    font-weight: 800;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .folder-path {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
    margin-left: 4px;
    flex: 1;
  }

  .status-badge {
    font-size: 10px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 3px;
    line-height: 1.2;
    flex-shrink: 0;
    font-family: "JetBrains Mono", monospace;
  }

  .status-modified {
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.15);
  }

  .status-added,
  .status-untracked {
    color: #10b981;
    background: rgba(16, 185, 129, 0.15);
  }

  .status-deleted {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.15);
  }

  .no-results-state {
    padding: 30px 20px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12.5px;
  }

  .palette-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 6px 14px;
    background: var(--bg-topbar);
    border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.04));
    font-size: 11px;
    color: var(--text-muted);
  }

  .footer-hint {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .key-hint {
    background: var(--bg-subtle);
    padding: 1px 5px;
    border-radius: 3px;
    font-family: "JetBrains Mono", monospace;
    font-size: 10px;
    color: var(--text-secondary);
  }

  .hint-sep {
    opacity: 0.5;
  }
</style>
