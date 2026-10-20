<script>
  import FileTreeNode from "./FileTreeNode.svelte";
  import { searchRepoContent } from "./api.js";
  import {
    buildTree,
    computeAllFolderPaths,
    getAllFolderPaths,
    highlightSnippet,
    escapeHtml,
  } from "./treeUtils.js";

  export let repoPath = null;
  export let files = [];       // Modified files from get_diff: { path, additions, deletions, hunks, is_binary }
  export let repoFiles = [];   // All files in repo: ["src/index.js", "README.md", ...]
  export let selectedFile = null;
  export let dirtyFiles = new Set();
  export let branch = null;
  export let baseBranch = null;
  export let comparisonInfo = null;
  export let onSelect;
  export let currentTheme = "dark";
  export let onThemeChange = () => {};
  export let onStage = () => {};
  export let onUnstage = () => {};
  export let onDiscard = () => {};
  export let onStageAll = () => {};
  export let onUnstageAll = () => {};
  export let onDiscardAll = () => {};
  export let onRequestRepoFiles = () => {};
  export let disabled = false;

  let isStagedExpanded = true;
  let isUnstagedExpanded = true;

  let filterScope = "changes";    // "changes" (default, lightweight) | "all" | "search"
  let searchQuery = "";
  let expandedAllFolders = {};
  let expandedChangesFolders = {};
  let prevFiles = null;
  let prevRepoFiles = null;

  // Global Search state across repository
  let globalQuery = "";
  let globalCaseSensitive = false;
  let globalMatchWholeWord = false;
  let globalUseRegex = false;
  let globalFilePattern = "";
  let showPatternInput = false;
  let isSearching = false;
  let searchError = null;
  let searchResults = { matches: [], total_count: 0, truncated: false };
  let expandedSearchFiles = {};
  let searchInputEl = null;

  export function focusSearch(initialQuery = "") {
    filterScope = "search";
    if (initialQuery) {
      globalQuery = initialQuery;
      runGlobalSearch();
    }
    setTimeout(() => {
      if (searchInputEl) {
        searchInputEl.focus();
        searchInputEl.select();
      }
    }, 25);
  }

  function handleScopeChange(newScope) {
    filterScope = newScope;
    if (newScope === "all" && (!repoFiles || repoFiles.length === 0)) {
      onRequestRepoFiles();
    }
    if (newScope === "search") {
      setTimeout(() => {
        if (searchInputEl) {
          searchInputEl.focus();
        }
      }, 25);
    }
  }

  async function runGlobalSearch() {
    if (!repoPath || !globalQuery || !globalQuery.trim()) {
      searchResults = { matches: [], total_count: 0, truncated: false };
      searchError = null;
      isSearching = false;
      return;
    }

    isSearching = true;
    searchError = null;
    try {
      const res = await searchRepoContent(repoPath, globalQuery, {
        caseSensitive: globalCaseSensitive,
        matchWholeWord: globalMatchWholeWord,
        useRegex: globalUseRegex,
        filePattern: globalFilePattern || null,
        maxResults: 500,
      });
      searchResults = res;
      const nextExp = {};
      for (const m of res.matches) {
        nextExp[m.file] = true;
      }
      expandedSearchFiles = nextExp;
    } catch (err) {
      searchError = String(err);
      searchResults = { matches: [], total_count: 0, truncated: false };
    } finally {
      isSearching = false;
    }
  }

  function toggleCaseSensitive() {
    globalCaseSensitive = !globalCaseSensitive;
    if (globalQuery && (searchResults.total_count > 0 || searchResults.matches.length > 0)) {
      runGlobalSearch();
    }
  }

  function toggleMatchWholeWord() {
    globalMatchWholeWord = !globalMatchWholeWord;
    if (globalQuery && (searchResults.total_count > 0 || searchResults.matches.length > 0)) {
      runGlobalSearch();
    }
  }

  function toggleUseRegex() {
    globalUseRegex = !globalUseRegex;
    if (globalQuery && (searchResults.total_count > 0 || searchResults.matches.length > 0)) {
      runGlobalSearch();
    }
  }

  function handleSearchInput(e) {
    globalQuery = e.target.value;
    if (!globalQuery.trim()) {
      searchResults = { matches: [], total_count: 0, truncated: false };
      searchError = null;
    }
  }

  function handlePatternInput(e) {
    globalFilePattern = e.target.value;
  }

  function handleSearchKeydown(e) {
    if (e.key === "Enter") {
      e.preventDefault();
      runGlobalSearch();
    } else if (e.key === "Escape") {
      if (globalQuery) {
        globalQuery = "";
        searchResults = { matches: [], total_count: 0, truncated: false };
        searchError = null;
      }
    }
  }

  function toggleSearchFile(file) {
    expandedSearchFiles[file] = !expandedSearchFiles[file];
    expandedSearchFiles = { ...expandedSearchFiles };
  }

  function collapseAllSearchFiles() {
    expandedSearchFiles = {};
  }

  function expandAllSearchFiles() {
    const next = {};
    for (const group of groupedSearchResults) {
      next[group.file] = true;
    }
    expandedSearchFiles = next;
  }

  function handleSelectSearchMatch(filePath, lineNumber, query) {
    if (onSelect) {
      onSelect(filePath, lineNumber, query);
    }
  }

  $: groupedSearchResults = (() => {
    if (!searchResults.matches || searchResults.matches.length === 0) return [];
    const map = new Map();
    for (const match of searchResults.matches) {
      if (!map.has(match.file)) {
        map.set(match.file, []);
      }
      map.get(match.file).push(match);
    }
    const list = [];
    for (const [file, fileMatches] of map.entries()) {
      const parts = file.split("/");
      const fileName = parts.pop();
      const folderPath = parts.join("/");
      list.push({
        file,
        fileName,
        folderPath,
        matches: fileMatches,
      });
    }
    return list;
  })();

  $: totalMatchedFiles = groupedSearchResults.length;
  $: totalMatchedCount = searchResults.total_count;

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
  $: totalAdditions = files.reduce((acc, f) => acc + (f.additions || 0), 0);
  $: totalDeletions = files.reduce((acc, f) => acc + (f.deletions || 0), 0);

  // In "changes" mode or when comparing against a base branch (MR view),
  // only process modified files (saves heap & CPU on large repos)
  $: sourcePaths = (filterScope === "changes" || baseBranch)
    ? files.map((f) => f.path)
    : Array.from(new Set([...repoFiles, ...files.map((f) => f.path)]));
  $: unifiedPaths = sourcePaths.sort();

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

  // Staged / Unstaged paths separation
  $: stagedPaths = files.filter((f) => f.has_staged).map((f) => f.path);
  $: unstagedPaths = files.filter((f) => f.has_unstaged || f.is_untracked).map((f) => f.path);
  $: allUnstagedPaths = Array.from(new Set([...unstagedPaths, ...(dirtyFiles ? Array.from(dirtyFiles) : [])])).sort();

  $: stagedFilteredPaths = stagedPaths.filter((path) => {
    if (searchQuery && !path.toLowerCase().includes(searchQuery.toLowerCase().trim())) {
      return false;
    }
    return true;
  });

  $: unstagedFilteredPaths = allUnstagedPaths.filter((path) => {
    if (searchQuery && !path.toLowerCase().includes(searchQuery.toLowerCase().trim())) {
      return false;
    }
    return true;
  });

  $: stagedTreeData = (!baseBranch && filterScope === "changes") ? buildTree(stagedFilteredPaths, diffMap, dirtyFiles) : [];
  $: unstagedTreeData = (!baseBranch && filterScope === "changes") ? buildTree(unstagedFilteredPaths, diffMap, dirtyFiles) : [];
  $: treeData = (filterScope === "all" || Boolean(baseBranch)) ? buildTree(filteredPaths, diffMap, dirtyFiles) : [];
</script>

<div class="sidebar-wrapper">
  <!-- Top Scope Switcher -->
  <div class="sidebar-header">
    <div class="scope-row">
      <div class="segmented-control">
        <button
          class="seg-btn"
          class:active={filterScope === "changes"}
          on:click={() => handleScopeChange("changes")}
        >
          Changes ({files.length})
        </button>
        <button
          class="seg-btn"
          class:active={filterScope === "all"}
          on:click={() => handleScopeChange("all")}
        >
          Files
        </button>
        <button
          class="seg-btn"
          class:active={filterScope === "search"}
          on:click={() => handleScopeChange("search")}
        >
          Search
        </button>
      </div>

      {#if filterScope === "search"}
        <div class="tree-controls">
          <button
            class="tree-control-btn"
            on:click={expandAllSearchFiles}
            title="Expand All Files"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
              <path d="M1 2.5A1.5 1.5 0 0 1 2.5 1h3.086a1.5 1.5 0 0 1 1.06.44l1.354 1.353a.5.5 0 0 0 .354.147h5.146A1.5 1.5 0 0 1 15 4.44v1.56h-1.5V4.44a.25.25 0 0 0-.25-.25H8.354a2 2 0 0 1-1.415-.586L5.586 2.25H2.5a.25.25 0 0 0-.25.25V5.5H1v-3z"/>
              <path d="M8 8a.75.75 0 0 1 .75.75v2h2a.75.75 0 0 1 0 1.5h-2v2a.75.75 0 0 1-1.5 0v-2h-2a.75.75 0 0 1 0-1.5h2v-2A.75.75 0 0 1 8 8z"/>
            </svg>
          </button>
          <button
            class="tree-control-btn"
            on:click={collapseAllSearchFiles}
            title="Collapse All Files"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
              <path d="M1 2.5A1.5 1.5 0 0 1 2.5 1h3.086a1.5 1.5 0 0 1 1.06.44l1.354 1.353a.5.5 0 0 0 .354.147h5.146A1.5 1.5 0 0 1 15 4.44v1.56h-1.5V4.44a.25.25 0 0 0-.25-.25H8.354a2 2 0 0 1-1.415-.586L5.586 2.25H2.5a.25.25 0 0 0-.25.25V5.5H1v-3z"/>
              <path d="M5.5 10.25a.75.75 0 0 1 .75-.75h5a.75.75 0 0 1 0 1.5h-5a.75.75 0 0 1-.75-.75z"/>
            </svg>
          </button>
        </div>
      {:else}
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
      {/if}
    </div>

    {#if filterScope === "search"}
      <div class="global-search-inputs">
        <div class="search-input-box">
          <input
            bind:this={searchInputEl}
            type="text"
            placeholder="Search in repository (press Enter)..."
            value={globalQuery}
            on:input={handleSearchInput}
            on:keydown={handleSearchKeydown}
            class="global-search-field"
            spellcheck="false"
          />
          <div class="search-input-actions">
            <button
              class="search-toggle-btn"
              class:active={globalCaseSensitive}
              on:click={toggleCaseSensitive}
              title="Match Case (Aa)"
            >
              Aa
            </button>
            <button
              class="search-toggle-btn"
              class:active={globalMatchWholeWord}
              on:click={toggleMatchWholeWord}
              title="Match Whole Word (\b)"
            >
              \b
            </button>
            <button
              class="search-toggle-btn"
              class:active={globalUseRegex}
              on:click={toggleUseRegex}
              title="Use Regular Expression (.*)"
            >
              .*
            </button>
            {#if globalQuery}
              <button
                class="search-clear-btn"
                on:click={() => {
                  globalQuery = "";
                  searchResults = { matches: [], total_count: 0, truncated: false };
                  searchError = null;
                }}
                title="Clear Search"
              >
                ✕
              </button>
            {/if}
          </div>
        </div>

        <!-- Collapsible files to include filter -->
        <div class="pattern-toggle-row">
          <button
            class="pattern-toggle-btn"
            on:click={() => (showPatternInput = !showPatternInput)}
          >
            <span class="pattern-chevron" class:expanded={showPatternInput}>▸</span>
            <span>files to include</span>
          </button>
          {#if globalFilePattern}
            <span class="active-filter-indicator">•</span>
          {/if}
        </div>

        {#if showPatternInput}
          <div class="pattern-input-row">
            <input
              type="text"
              placeholder="e.g. *.svelte, src/**"
              value={globalFilePattern}
              on:input={handlePatternInput}
              on:keydown={handleSearchKeydown}
              class="pattern-input"
              spellcheck="false"
            />
            {#if globalFilePattern}
              <button
                class="search-clear-btn"
                on:click={() => {
                  globalFilePattern = "";
                  if (globalQuery && searchResults.total_count > 0) {
                    runGlobalSearch();
                  }
                }}
                title="Clear Filter"
              >
                ✕
              </button>
            {/if}
          </div>
        {/if}
      </div>
    {:else}
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
    {/if}
  </div>

  <!-- Main Scrollable Directory Structure -->
  <div class="file-tree-container">
    {#if filterScope === "search"}
      <div class="search-panel-body">
        <div class="search-status-bar">
          {#if isSearching}
            <span class="search-status-text">Searching in repository...</span>
          {:else if searchError}
            <span class="search-status-error">{searchError}</span>
          {:else if !globalQuery.trim()}
            <span class="search-status-hint">Type a word or phrase to search files</span>
          {:else if totalMatchedCount === 0}
            <span class="search-status-empty">No results found for "{globalQuery}"</span>
          {:else}
            <span class="search-status-count">
              {totalMatchedCount} result{totalMatchedCount === 1 ? "" : "s"} in {totalMatchedFiles} file{totalMatchedFiles === 1 ? "" : "s"}
              {#if searchResults.truncated}
                <span class="truncated-tag">(first 500 results)</span>
              {/if}
            </span>
          {/if}
        </div>

        {#if groupedSearchResults.length > 0}
          <div class="search-groups-list">
            {#each groupedSearchResults as group (group.file)}
              {@const isExpanded = expandedSearchFiles[group.file] !== false}
              <div class="search-file-group">
                <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
                <div
                  class="search-file-header"
                  class:is-active={selectedFile === group.file}
                  on:click={() => toggleSearchFile(group.file)}
                >
                  <span class="search-chevron" class:expanded={isExpanded}>▸</span>
                  <svg viewBox="0 0 16 16" width="13" height="13" class="search-file-icon" fill="currentColor">
                    <path d="M2 1.75C2 .784 2.784 0 3.75 0h6.586c.464 0 .909.184 1.237.513l2.914 2.914c.329.328.513.773.513 1.237v9.586A1.75 1.75 0 0 1 13.25 16h-9.5A1.75 1.75 0 0 1 2 14.25V1.75z"/>
                  </svg>
                  <span class="search-file-name">{group.fileName}</span>
                  {#if group.folderPath}
                    <span class="search-file-folder">{group.folderPath}</span>
                  {/if}
                  <span class="search-file-badge">{group.matches.length}</span>
                </div>

                {#if isExpanded}
                  <div class="search-matches-list">
                    {#each group.matches as match (match.line + "_" + match.content)}
                      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
                      <div
                        class="search-match-item"
                        on:click={() => handleSelectSearchMatch(group.file, match.line, globalQuery)}
                        title={`Go to ${group.file}:${match.line}`}
                      >
                        <span class="search-match-line">L{match.line}</span>
                        <span class="search-match-snippet">{@html highlightSnippet(match.content, globalQuery, globalCaseSensitive, globalUseRegex)}</span>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else if filterScope === "changes" && !baseBranch}
      <!-- Staged Changes Section -->
      {#if stagedFilteredPaths.length > 0}
        <div class="git-staging-section">
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
          <div
            class="git-section-header"
            on:click={() => (isStagedExpanded = !isStagedExpanded)}
          >
            <div class="section-header-left">
              <span class="section-chevron" class:expanded={isStagedExpanded}>▸</span>
              <span class="section-title">STAGED CHANGES</span>
              <span class="section-badge">{stagedFilteredPaths.length}</span>
            </div>
            <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
            <div class="section-header-actions" on:click|stopPropagation>
              <button
                class="section-action-btn"
                disabled={disabled}
                on:click={onUnstageAll}
                title="Unstage All Changes (-)"
              >
                <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                  <path d="M3.75 7.25h8.5a.75.75 0 0 1 0 1.5h-8.5a.75.75 0 0 1 0-1.5z"/>
                </svg>
              </button>
            </div>
          </div>

          {#if isStagedExpanded}
            <div class="tree-root">
              {#each stagedTreeData as node (node.path)}
                <FileTreeNode
                  {node}
                  depth={0}
                  {selectedFile}
                  {expandedFolders}
                  {searchQuery}
                  {toggleFolder}
                  {onSelect}
                  stageMode="staged"
                  {disabled}
                  {onUnstage}
                />
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Unstaged Changes Section -->
      <div class="git-staging-section">
        <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
        <div
          class="git-section-header"
          on:click={() => (isUnstagedExpanded = !isUnstagedExpanded)}
        >
          <div class="section-header-left">
            <span class="section-chevron" class:expanded={isUnstagedExpanded}>▸</span>
            <span class="section-title">CHANGES</span>
            <span class="section-badge">{unstagedFilteredPaths.length}</span>
          </div>
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
          <div class="section-header-actions" on:click|stopPropagation>
            {#if unstagedFilteredPaths.length > 0}
              <button
                class="section-action-btn stage-all-btn"
                disabled={disabled}
                on:click={onStageAll}
                title="Stage All Changes (+)"
              >
                <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                  <path d="M7.25 3.75a.75.75 0 0 1 1.5 0v3.5h3.5a.75.75 0 0 1 0 1.5h-3.5v3.5a.75.75 0 0 1-1.5 0v-3.5h-3.5a.75.75 0 0 1 0-1.5h3.5v-3.5z"/>
                </svg>
              </button>
              <button
                class="section-action-btn discard-all-btn"
                disabled={disabled}
                on:click={onDiscardAll}
                title="Discard All Changes"
              >
                <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor">
                  <path fill-rule="evenodd" d="M1.25 8A6.75 6.75 0 1 1 8 14.75a.75.75 0 0 1 0-1.5 5.25 5.25 0 1 0-4.66-2.85l1.44-.36a.75.75 0 1 1 .36 1.45l-3 1a.75.75 0 0 1-.95-.55l-1-3a.75.75 0 1 1 1.42-.48l.45 1.34A6.71 6.71 0 0 1 1.25 8z"/>
                </svg>
              </button>
            {/if}
          </div>
        </div>

        {#if isUnstagedExpanded}
          <div class="tree-root">
            {#each unstagedTreeData as node (node.path)}
              <FileTreeNode
                {node}
                depth={0}
                {selectedFile}
                {expandedFolders}
                {searchQuery}
                {toggleFolder}
                {onSelect}
                stageMode="unstaged"
                {disabled}
                {onStage}
                {onDiscard}
              />
            {/each}
          </div>
        {/if}
      </div>
    {:else}
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
            {disabled}
          />
        {/each}
      </div>
    {/if}

    {#if filterScope !== "search" && filteredPaths.length === 0}
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
    border-bottom: none;
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
    border: none;
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
    border: none;
    border-radius: 50%;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.12s ease;
    padding: 0;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03);
  }

  .tree-control-btn:hover {
    background: var(--bg-hover);
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

  .git-staging-section {
    margin-bottom: 8px;
  }

  .git-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    min-height: 28px;
    max-height: 28px;
    box-sizing: border-box;
    padding: 0 6px;
    border-radius: 4px;
    cursor: pointer;
    user-select: none;
    transition: background 0.12s ease;
  }

  .git-section-header:hover {
    background: var(--bg-hover);
  }

  .section-header-left {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
  }

  .section-chevron {
    font-size: 10px;
    color: var(--text-muted);
    transition: transform 0.15s ease;
    display: inline-block;
    line-height: 1;
  }

  .section-chevron.expanded {
    transform: rotate(90deg);
  }

  .section-title {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--text-secondary);
  }

  .section-badge {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    background: var(--bg-subtle);
    border-radius: 10px;
    padding: 1px 6px;
    min-width: 14px;
    text-align: center;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, monospace;
  }

  .section-header-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .git-section-header:hover .section-header-actions {
    opacity: 1;
  }

  .section-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.12s ease;
    padding: 0;
  }

  .section-action-btn:hover {
    background: var(--bg-card);
    color: var(--text-primary);
  }

  .section-action-btn.stage-all-btn:hover {
    color: var(--accent-emerald);
  }

  .section-action-btn.discard-all-btn:hover {
    color: var(--diff-del-sign);
  }

  .empty-state {
    padding: 30px 14px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }

  .sidebar-footer {
    padding: 6px 10px;
    border-top: none;
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
    border: none;
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
    color: var(--text-dim);
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
    border: none;
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

  /* Global Search Styles */
  .global-search-inputs {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }

  .search-input-box {
    position: relative;
    display: flex;
    align-items: center;
    background: var(--bg-subtle);
    border-radius: 4px;
    border: none;
    transition: all 0.15s ease;
  }

  .search-input-box:focus-within {
    background: var(--bg-card);
    box-shadow: 0 0 0 1px var(--accent-glow);
  }

  .global-search-field {
    width: 100%;
    background: transparent;
    border: none;
    padding: 6px 95px 6px 10px;
    font-size: 11.5px;
    color: var(--text-primary);
    outline: none;
    box-sizing: border-box;
    font-family: inherit;
  }

  .global-search-field::placeholder {
    color: var(--text-muted);
  }

  .search-input-actions {
    position: absolute;
    right: 4px;
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .search-toggle-btn {
    background: none;
    border: none;
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-muted);
    padding: 2px 4px;
    border-radius: 3px;
    cursor: pointer;
    line-height: 1;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, monospace;
    transition: all 0.12s ease;
  }

  .search-toggle-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .search-toggle-btn.active {
    background: var(--bg-hover);
    color: var(--accent-emerald);
    font-weight: 700;
  }

  .search-clear-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 10px;
    cursor: pointer;
    padding: 2px 4px;
    line-height: 1;
  }

  .search-clear-btn:hover {
    color: var(--text-primary);
  }

  .pattern-toggle-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .pattern-toggle-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: none;
    border: none;
    padding: 0;
    font-size: 10.5px;
    color: var(--text-muted);
    cursor: pointer;
    font-family: inherit;
  }

  .pattern-toggle-btn:hover {
    color: var(--text-secondary);
  }

  .pattern-chevron {
    font-size: 9px;
    display: inline-block;
    transition: transform 0.15s ease;
  }

  .pattern-chevron.expanded {
    transform: rotate(90deg);
  }

  .active-filter-indicator {
    color: var(--accent-emerald);
    font-size: 10px;
  }

  .pattern-input-row {
    position: relative;
    display: flex;
    align-items: center;
  }

  .pattern-input {
    width: 100%;
    background: var(--bg-subtle);
    border-radius: 4px;
    padding: 4px 24px 4px 8px;
    border: none;
    font-size: 11px;
    color: var(--text-primary);
    outline: none;
    box-sizing: border-box;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", monospace;
  }

  .pattern-input:focus {
    background: var(--bg-card);
    box-shadow: 0 0 0 1px var(--accent-glow);
  }

  .search-panel-body {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .search-status-bar {
    padding: 8px 12px 6px;
    font-size: 11px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.04));
    flex-shrink: 0;
  }

  .search-status-error {
    color: #ef4444;
  }

  .search-status-count {
    font-weight: 500;
    color: var(--text-secondary);
  }

  .truncated-tag {
    font-size: 10px;
    color: var(--text-muted);
    margin-left: 4px;
  }

  .search-groups-list {
    display: flex;
    flex-direction: column;
    padding: 4px 0;
  }

  .search-file-group {
    display: flex;
    flex-direction: column;
  }

  .search-file-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    cursor: pointer;
    user-select: none;
    font-size: 11.5px;
    transition: background 0.1s ease;
  }

  .search-file-header:hover {
    background: var(--bg-hover);
  }

  .search-file-header.is-active {
    background: var(--bg-hover);
  }

  .search-chevron {
    font-size: 9px;
    color: var(--text-muted);
    display: inline-block;
    transition: transform 0.15s ease;
    width: 10px;
  }

  .search-chevron.expanded {
    transform: rotate(90deg);
  }

  .search-file-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .search-file-name {
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .search-file-folder {
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
    flex: 1;
  }

  .search-file-badge {
    background: var(--bg-subtle);
    border-radius: 9999px;
    padding: 1px 6px;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-left: auto;
    flex-shrink: 0;
  }

  .search-matches-list {
    display: flex;
    flex-direction: column;
    padding-left: 18px;
  }

  .search-match-item {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 3px 12px 3px 10px;
    cursor: pointer;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", monospace;
    font-size: 11px;
    line-height: 1.4;
    transition: background 0.1s ease;
  }

  .search-match-item:hover {
    background: var(--bg-hover);
  }

  .search-match-line {
    font-size: 10px;
    color: var(--text-muted);
    flex-shrink: 0;
    min-width: 32px;
  }

  .search-match-snippet {
    color: var(--text-secondary);
    white-space: pre;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :global(.search-snippet-match) {
    background: rgba(250, 204, 21, 0.4);
    color: inherit;
    border-radius: 2px;
    padding: 0 1px;
  }
</style>
