<script>
  import { onDestroy, onMount } from "svelte";
  import FileList from "./lib/FileList.svelte";
  import DiffView from "./lib/DiffView.svelte";
  import { pickFolder, getDiff, getBranchComparisonInfo, getRepoFiles, getFileContent, saveFileContent, acceptHunk, rejectHunk, getCurrentBranch, listBranches, switchBranch, pullBranch, onChangesDetected } from "./lib/api.js";

  let currentTheme = typeof window !== "undefined" ? localStorage.getItem("tengga_theme") || "glass" : "glass";

  function setTheme(t) {
    currentTheme = t;
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("data-theme", t);
      try {
        localStorage.setItem("tengga_theme", t);
      } catch (e) {}
    }
  }

  onMount(() => {
    const saved = localStorage.getItem("tengga_theme") || "glass";
    setTheme(saved);
  });

  let repoPath = null;
  let currentBranch = null;
  let availableBranches = [];
  let isBranchDropdownOpen = false;
  let isSwitchingBranch = false;
  let branchFilter = "";

  // Branch Comparison / MR Viewer state
  let baseBranch = null; // null = working tree (HEAD), string = compare branch
  let isCompareDropdownOpen = false;
  let compareFilter = "";
  let comparisonInfo = null; // { ahead: 0, behind: 0 }

  let pullingBranch = null; // branch currently being pulled (source or target)
  let statusMessage = null;
  let statusTimeout = null;

  let files = [];
  let repoFiles = [];
  let selectedFile = null;
  let error = null;
  let unlisten = null;
  let isRefreshing = false;

  let fileContent = null;
  let isLoadingContent = false;
  let contentError = null;

  $: repoName = repoPath ? repoPath.split("/").filter(Boolean).pop() : null;
  $: diffFile = files.find((f) => f.path === selectedFile);
  $: currentFile = diffFile || (selectedFile ? { path: selectedFile, hunks: [], additions: 0, deletions: 0, is_binary: false } : null);

  function sortBranchesWithRelevance(branches, query, current) {
    const q = (query || "").toLowerCase().trim();
    const commonBases = ["release", "origin/release", "main", "origin/main", "master", "origin/master", "dev", "origin/dev"];

    const candidates = branches.filter((b) => b !== current && (!q || b.toLowerCase().includes(q)));

    return candidates.sort((a, b) => {
      const aLower = a.toLowerCase();
      const bLower = b.toLowerCase();

      if (q) {
        // 1. Exact match with query or origin/query
        const aExact = aLower === q || aLower === `origin/${q}`;
        const bExact = bLower === q || bLower === `origin/${q}`;
        if (aExact && !bExact) return -1;
        if (!aExact && bExact) return 1;

        // 2. Starts with query or origin/query
        const aStarts = aLower.startsWith(q) || aLower.startsWith(`origin/${q}`);
        const bStarts = bLower.startsWith(q) || bLower.startsWith(`origin/${q}`);
        if (aStarts && !bStarts) return -1;
        if (!aStarts && bStarts) return 1;

        // 3. Ends with /query (e.g. branch ending with /release)
        const aEnds = aLower.endsWith(`/${q}`);
        const bEnds = bLower.endsWith(`/${q}`);
        if (aEnds && !bEnds) return -1;
        if (!aEnds && bEnds) return 1;
      } else {
        // If no query, prioritize common base branches
        const aCommon = commonBases.indexOf(aLower);
        const bCommon = commonBases.indexOf(bLower);
        if (aCommon !== -1 && bCommon !== -1) return aCommon - bCommon;
        if (aCommon !== -1) return -1;
        if (bCommon !== -1) return 1;
      }

      return aLower.localeCompare(bLower);
    });
  }

  $: filteredBranches = sortBranchesWithRelevance(availableBranches, branchFilter, null);
  $: filteredCompareBranches = sortBranchesWithRelevance(availableBranches, compareFilter, currentBranch);

  $: if (repoPath && selectedFile) {
    loadFileContent(selectedFile);
  } else {
    fileContent = null;
    contentError = null;
  }

  async function loadFileContent(filePath) {
    if (!repoPath || !filePath) return;
    isLoadingContent = true;
    contentError = null;
    try {
      fileContent = await getFileContent(repoPath, filePath);
    } catch (e) {
      contentError = String(e);
      fileContent = null;
    } finally {
      isLoadingContent = false;
    }
  }

  async function refresh() {
    if (!repoPath) return;
    isRefreshing = true;
    try {
      const [diffList, allList, branch, compInfo] = await Promise.all([
        getDiff(repoPath, baseBranch),
        getRepoFiles(repoPath).catch(() => []),
        getCurrentBranch(repoPath).catch(() => null),
        baseBranch ? getBranchComparisonInfo(repoPath, baseBranch).catch(() => null) : Promise.resolve(null),
      ]);
      files = diffList;
      repoFiles = allList;
      currentBranch = branch;
      comparisonInfo = compInfo;
      if (selectedFile) {
        loadFileContent(selectedFile);
      }
    } catch (e) {
      error = String(e);
    } finally {
      isRefreshing = false;
    }
  }

  function setStatus(msg, timeoutMs = 4500) {
    if (statusTimeout) clearTimeout(statusTimeout);
    statusMessage = msg;
    if (timeoutMs > 0) {
      statusTimeout = setTimeout(() => {
        statusMessage = null;
      }, timeoutMs);
    }
  }

  async function handlePullBranch(branchName) {
    if (!repoPath || !branchName || pullingBranch) return;
    pullingBranch = branchName;
    error = null;
    statusMessage = null;
    try {
      const result = await pullBranch(repoPath, branchName);
      setStatus(result || `Successfully updated '${branchName}' from remote`);
      await refresh();
    } catch (e) {
      error = `Pull '${branchName}' failed: ${String(e)}`;
    } finally {
      pullingBranch = null;
    }
  }

  async function toggleBranchDropdown() {
    if (!repoPath) return;
    if (isBranchDropdownOpen) {
      isBranchDropdownOpen = false;
      return;
    }
    isCompareDropdownOpen = false;
    try {
      availableBranches = await listBranches(repoPath);
      branchFilter = "";
      isBranchDropdownOpen = true;
    } catch (e) {
      error = "Failed to list branches: " + String(e);
    }
  }

  async function toggleCompareDropdown() {
    if (!repoPath) return;
    if (isCompareDropdownOpen) {
      isCompareDropdownOpen = false;
      return;
    }
    isBranchDropdownOpen = false;
    try {
      availableBranches = await listBranches(repoPath);
      compareFilter = "";
      isCompareDropdownOpen = true;
    } catch (e) {
      error = "Failed to list branches: " + String(e);
    }
  }

  async function handleSelectCompareBase(targetBranch) {
    baseBranch = targetBranch;
    isCompareDropdownOpen = false;
    await refresh();
  }

  async function handleSelectBranch(branchName) {
    if (!repoPath || isSwitchingBranch || branchName === currentBranch) {
      isBranchDropdownOpen = false;
      return;
    }

    if (unsavedBuffers.size > 0) {
      const proceed = confirm(
        `You have unsaved changes in ${unsavedBuffers.size} file(s). Switching branches might overwrite them. Discard unsaved changes and proceed?`
      );
      if (!proceed) return;
      unsavedBuffers.clear();
    }

    isSwitchingBranch = true;
    error = null;
    try {
      await switchBranch(repoPath, branchName);
      isBranchDropdownOpen = false;
      baseBranch = null;
      comparisonInfo = null;
      await refresh();
    } catch (e) {
      error = `Branch switch failed: ${String(e)}`;
    } finally {
      isSwitchingBranch = false;
    }
  }

  function handleWindowClick() {
    if (isBranchDropdownOpen) {
      isBranchDropdownOpen = false;
    }
    if (isCompareDropdownOpen) {
      isCompareDropdownOpen = false;
    }
  }

  async function openFolder() {
    error = null;
    try {
      const path = await pickFolder();
      if (!path) return;
      repoPath = path;
      selectedFile = null;
      baseBranch = null;
      comparisonInfo = null;
      await refresh();
      if (unlisten) unlisten();
      unlisten = await onChangesDetected(refresh);
    } catch (e) {
      error = String(e);
    }
  }

  async function handleAccept(filePath, hunkId) {
    try {
      await acceptHunk(repoPath, hunkId);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function handleReject(filePath, hunkId) {
    try {
      await rejectHunk(repoPath, hunkId);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  let unsavedBuffers = new Map(); // filePath -> string
  $: dirtyFiles = new Set(unsavedBuffers.keys());

  function handleBufferChange(filePath, newContent, isDirty) {
    if (isDirty) {
      unsavedBuffers.set(filePath, newContent);
    } else {
      unsavedBuffers.delete(filePath);
    }
    unsavedBuffers = new Map(unsavedBuffers);
  }

  async function handleSaveFile(filePath, newContent) {
    if (!repoPath) return;
    try {
      await saveFileContent(repoPath, filePath, newContent);
      unsavedBuffers.delete(filePath);
      unsavedBuffers = new Map(unsavedBuffers);
      if (selectedFile === filePath) {
        fileContent = newContent;
      }
    } catch (e) {
      error = String(e);
      throw e;
    }
  }

  async function handleSaveAll() {
    if (!repoPath || unsavedBuffers.size === 0) return;
    try {
      const entries = Array.from(unsavedBuffers.entries());
      await Promise.all(
        entries.map(([filePath, content]) => saveFileContent(repoPath, filePath, content))
      );
      if (selectedFile && unsavedBuffers.has(selectedFile)) {
        fileContent = unsavedBuffers.get(selectedFile);
      }
      unsavedBuffers.clear();
      unsavedBuffers = new Map();
    } catch (e) {
      error = String(e);
    }
  }

  function handleGlobalKeydown(e) {
    const isSaveKey = e.code === "KeyS" || e.key.toLowerCase() === "s" || e.key === "ß";
    const isModifier = e.metaKey || e.ctrlKey;

    if (isModifier && e.altKey && isSaveKey) {
      e.preventDefault();
      handleSaveAll();
      return;
    }

    if (isModifier && !e.altKey && isSaveKey) {
      if (selectedFile && unsavedBuffers.has(selectedFile)) {
        e.preventDefault();
        handleSaveFile(selectedFile, unsavedBuffers.get(selectedFile));
      }
      return;
    }
  }

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

<svelte:window on:keydown={handleGlobalKeydown} on:click={handleWindowClick} />

<main>
  <!-- Top Navigation Bar -->
  <header class="app-topbar">
    <div class="brand-cluster">
      {#if repoName}
        <div class="repo-brand-title" title={repoPath}>
          <svg class="repo-brand-icon" viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5v-9zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8V1.5z"/>
          </svg>
          <span class="repo-brand-text">{repoName}</span>
        </div>
      {:else}
        <img src="/favicon.png" alt="Tengga" class="app-logo-mini" />
        <span class="app-name">TENGGA</span>
        <span class="app-tagline">Diff Review</span>
      {/if}
    </div>

    <!-- Center branch bar (roomy, no full paths) -->
    <div class="topbar-center">
      {#if repoPath}
        {#if currentBranch}
          <div class="branch-bar">
            <!-- Source Branch Button Group -->
            <div class="branch-btn-group source-group">
              <button
                class="branch-main-btn"
                class:is-active={isBranchDropdownOpen}
                on:click|stopPropagation={toggleBranchDropdown}
                title="Current branch: {currentBranch}. Click to switch branch."
              >
                <svg class="branch-icon" viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
                  <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z"/>
                </svg>
                <span class="branch-text">{currentBranch}</span>
                <span class="branch-chevron">{isBranchDropdownOpen ? "▴" : "▾"}</span>
              </button>

              <button
                class="branch-action-btn pull-btn"
                class:is-loading={pullingBranch === currentBranch}
                disabled={pullingBranch !== null}
                on:click|stopPropagation={() => handlePullBranch(currentBranch)}
                title="Pull latest for '{currentBranch}' from remote (git pull --ff-only)"
              >
                {#if pullingBranch === currentBranch}
                  <span class="pull-spin">↻</span>
                {:else}
                  <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                    <path fill-rule="evenodd" d="M8 2a.75.75 0 0 1 .75.75v8.69l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22V2.75A.75.75 0 0 1 8 2z"/>
                  </svg>
                {/if}
              </button>

              {#if isBranchDropdownOpen}
                <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                <div
                  class="branch-dropdown"
                  role="menu"
                  tabindex="-1"
                  on:click|stopPropagation
                  on:keydown={(e) => e.key === "Escape" && (isBranchDropdownOpen = false)}
                >
                  <div class="branch-dropdown-header">
                    <span>Switch Branch</span>
                    {#if isSwitchingBranch}
                      <span class="branch-switching-spinner">Switching...</span>
                    {:else}
                      <span class="branch-count">{availableBranches.length} branches</span>
                    {/if}
                  </div>

                  {#if availableBranches.length > 5}
                    <div class="branch-search-box">
                      <input
                        type="text"
                        placeholder="Filter branches..."
                        bind:value={branchFilter}
                        class="branch-filter-input"
                        spellcheck="false"
                      />
                    </div>
                  {/if}

                  <div class="branch-list">
                    {#each filteredBranches as b}
                      <button
                        class="branch-item"
                        class:is-current={b === currentBranch}
                        on:click={() => handleSelectBranch(b)}
                        disabled={isSwitchingBranch}
                      >
                        <svg class="branch-item-icon" viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                          <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z"/>
                        </svg>
                        <span class="branch-item-label">{b}</span>
                        {#if b === currentBranch}
                          <span class="branch-check" title="Current branch">✓</span>
                        {/if}
                      </button>
                    {/each}
                    {#if filteredBranches.length === 0}
                      <div class="no-matching-branches">No matching branch</div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>

            <!-- Direction arrow -->
            <span class="compare-arrow" title="Comparing current branch against target base branch">←</span>

            <!-- Target Base Branch Button Group -->
            <div class="branch-btn-group target-group" class:is-mr-mode={baseBranch !== null}>
              <button
                class="branch-main-btn"
                class:is-active={isCompareDropdownOpen}
                on:click|stopPropagation={toggleCompareDropdown}
                title={baseBranch ? `MR Mode: Comparing changes against base branch '${baseBranch}'` : "Compare working tree changes against latest commit (HEAD)"}
              >
                {#if baseBranch}
                  <svg class="compare-icon mr-icon" viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
                    <path d="M5 3.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0zm0 2.122V11a2.5 2.5 0 0 0 2.5 2.5h1.75a.75.75 0 0 0 0-1.5H7.5A1 1 0 0 1 6.5 11V5.372a2.25 2.25 0 1 0-1.5 0zM11 12.75a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0zm0-2.122V5A2.5 2.5 0 0 0 8.5 2.5H6.75a.75.75 0 0 0 0 1.5H8.5a1 1 0 0 1 1 1v5.628a2.25 2.25 0 1 0 1.5 0z"/>
                  </svg>
                  <span class="branch-text">vs {baseBranch}</span>
                  {#if comparisonInfo && comparisonInfo.ahead > 0}
                    <span class="compare-ahead-tag" title="{comparisonInfo.ahead} commit(s) ahead of {baseBranch}">+{comparisonInfo.ahead}</span>
                  {/if}
                {:else}
                  <svg class="compare-icon" viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                    <path d="M1 2.75A.75.75 0 0 1 1.75 2h12.5a.75.75 0 0 1 0 1.5H1.75A.75.75 0 0 1 1 2.75zm0 5A.75.75 0 0 1 1.75 7h12.5a.75.75 0 0 1 0 1.5H1.75A.75.75 0 0 1 1 7.75zM1.75 12h12.5a.75.75 0 0 1 0 1.5H1.75a.75.75 0 0 1 0-1.5z"/>
                  </svg>
                  <span class="branch-text">vs HEAD (latest commit)</span>
                {/if}
                <span class="branch-chevron">{isCompareDropdownOpen ? "▴" : "▾"}</span>
              </button>

              <!-- Pull button on target branch -->
              {#if baseBranch}
                <button
                  class="branch-action-btn pull-btn"
                  class:is-loading={pullingBranch === baseBranch}
                  disabled={pullingBranch !== null}
                  on:click|stopPropagation={() => handlePullBranch(baseBranch)}
                  title="Pull / fetch latest changes for '{baseBranch}' from remote"
                >
                  {#if pullingBranch === baseBranch}
                    <span class="pull-spin">↻</span>
                  {:else}
                    <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                      <path fill-rule="evenodd" d="M8 2a.75.75 0 0 1 .75.75v8.69l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22V2.75A.75.75 0 0 1 8 2z"/>
                    </svg>
                  {/if}
                </button>

                <button
                  class="branch-action-btn reset-btn"
                  on:click|stopPropagation={() => handleSelectCompareBase(null)}
                  title="Reset to uncommitted working tree (HEAD)"
                >✕</button>
              {/if}

              {#if isCompareDropdownOpen}
                <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                <div
                  class="compare-dropdown"
                  role="menu"
                  tabindex="-1"
                  on:click|stopPropagation
                  on:keydown={(e) => e.key === "Escape" && (isCompareDropdownOpen = false)}
                >
                  <div class="compare-dropdown-header">
                    <span class="compare-header-title">Compare Mode / MR View</span>
                  </div>

                  <!-- Default Mode Option: Working Tree (HEAD) -->
                  <div class="compare-options-section">
                    <button
                      class="compare-item mode-option"
                      class:is-current={baseBranch === null}
                      on:click={() => handleSelectCompareBase(null)}
                    >
                      <div class="compare-item-left">
                        <span class="compare-mode-badge default">HEAD</span>
                        <div class="compare-item-desc">
                          <span class="compare-item-title">Latest Commit (HEAD)</span>
                          <span class="compare-item-sub">Compare working tree changes against latest commit</span>
                        </div>
                      </div>
                      {#if baseBranch === null}
                        <span class="branch-check" title="Active">✓</span>
                      {/if}
                    </button>
                  </div>

                  <div class="compare-section-divider">
                    <span>OR COMPARE AGAINST BRANCH (MR VIEW)</span>
                  </div>

                  <div class="branch-search-box">
                    <input
                      type="text"
                      placeholder="Search base branch (e.g. release, main)..."
                      bind:value={compareFilter}
                      class="branch-filter-input"
                      spellcheck="false"
                    />
                  </div>

                  <div class="branch-list">
                    {#each filteredCompareBranches as b}
                      <button
                        class="branch-item"
                        class:is-current={b === baseBranch}
                        on:click={() => handleSelectCompareBase(b)}
                      >
                        <svg class="branch-item-icon mr-branch-icon" viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                          <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z"/>
                        </svg>
                        <span class="branch-item-label">{b}</span>
                        {#if b === baseBranch}
                          <span class="branch-check" title="Current base branch">✓</span>
                        {/if}
                      </button>
                    {/each}
                    {#if filteredCompareBranches.length === 0}
                      <div class="no-matching-branches">No matching branches found</div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>

            <!-- Refresh button -->
            <button
              class="refresh-icon-btn"
              on:click={refresh}
              title="Refresh repository"
              disabled={isRefreshing}
            >
              <span class:spinning={isRefreshing}>↻</span>
            </button>
          </div>
        {/if}
      {:else}
        <span class="no-repo-hint">No repository selected</span>
      {/if}
    </div>

    <!-- Folder Switcher & Theme Selector -->
    <!-- Folder Switcher -->
    <div class="topbar-actions">
      <button class="action-btn" on:click={openFolder}>
        {repoPath ? "Switch Folder" : "Open Folder"}
      </button>
    </div>
  </header>

  <!-- Error & Status Notification Banners -->
  {#if error}
    <div class="banner error-banner">
      <span class="banner-tag error-tag">Error</span>
      <span class="banner-text">{error}</span>
      <button class="dismiss-banner" on:click={() => (error = null)}>✕</button>
    </div>
  {:else if statusMessage}
    <div class="banner success-banner">
      <span class="banner-tag success-tag">✓</span>
      <span class="banner-text">{statusMessage}</span>
      <button class="dismiss-banner" on:click={() => (statusMessage = null)}>✕</button>
    </div>
  {/if}

  <!-- Split Main Workspace -->
  <div class="workspace-body">
    <aside class="sidebar-panel">
      <FileList
        {files}
        {repoFiles}
        {selectedFile}
        {dirtyFiles}
        branch={currentBranch}
        {baseBranch}
        {comparisonInfo}
        {currentTheme}
        onThemeChange={setTheme}
        onSelect={(p) => (selectedFile = p)}
      />
    </aside>
    <section class="diff-panel">
      <DiffView
        file={currentFile}
        {fileContent}
        initialContent={unsavedBuffers.get(selectedFile) ?? fileContent}
        {isLoadingContent}
        {contentError}
        {baseBranch}
        onAccept={handleAccept}
        onReject={handleReject}
        onSave={handleSaveFile}
        onSaveAll={handleSaveAll}
        onBufferChange={handleBufferChange}
      />
    </section>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    background: transparent !important;
    color: var(--text-primary);
    font-family: "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    -webkit-font-smoothing: antialiased;
    overflow: hidden;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: var(--bg-app);
    backdrop-filter: var(--backdrop-filter, none);
    -webkit-backdrop-filter: var(--backdrop-filter, none);
  }

  .app-topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px;
    height: 50px;
    background: var(--bg-topbar);
    border-bottom: 1px solid var(--border-subtle);
    user-select: none;
    flex-shrink: 0;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
    backdrop-filter: var(--backdrop-filter, blur(24px));
    -webkit-backdrop-filter: var(--backdrop-filter, blur(24px));
  }

  .brand-cluster {
    display: flex;
    align-items: center;
    min-width: 140px;
    flex-shrink: 0;
  }

  .repo-brand-title {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--text-primary);
    background: var(--bg-subtle);
    padding: 5px 12px;
    border-radius: 9999px;
    cursor: default;
    border: 1px solid var(--border-default);
    transition: all 0.12s ease;
  }

  .repo-brand-title:hover {
    border-color: var(--border-hover);
    background: var(--bg-hover);
  }

  .repo-brand-icon {
    color: var(--accent-emerald);
    flex-shrink: 0;
  }

  .repo-brand-text {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: -0.1px;
    color: var(--text-primary);
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .app-logo-mini {
    width: 20px;
    height: 20px;
    border-radius: 5px;
    flex-shrink: 0;
    margin-right: 6px;
  }

  .app-name {
    font-size: 14px;
    font-weight: 800;
    letter-spacing: 0.8px;
    color: var(--accent-emerald);
  }

  .app-tagline {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 600;
    margin-left: 6px;
  }

  .topbar-center {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    padding: 0 12px;
    min-width: 0;
  }

  .branch-bar {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .branch-btn-group {
    position: relative;
    display: inline-flex;
    align-items: center;
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: 9999px;
    height: 34px;
    padding: 0 4px 0 2px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.03);
    transition: all 0.15s ease;
  }

  .branch-btn-group.source-group:hover,
  .branch-btn-group.source-group:focus-within {
    border-color: var(--accent-emerald);
    background: var(--accent-emerald-soft);
  }

  .branch-btn-group.target-group.is-mr-mode {
    background: var(--accent-emerald-soft);
    border-color: var(--accent-emerald-border);
  }

  .branch-btn-group.target-group.is-mr-mode:hover,
  .branch-btn-group.target-group.is-mr-mode:focus-within {
    border-color: var(--accent-emerald);
    background: var(--accent-emerald-soft-hover);
  }

  .branch-main-btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 10px 0 8px;
    height: 100%;
    background: transparent;
    border: none;
    color: var(--accent-emerald-dark);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    border-radius: 9999px;
    outline: none;
    transition: background 0.12s, color 0.12s;
  }

  .branch-main-btn:hover {
    background: var(--bg-hover);
  }

  .branch-btn-group.target-group:not(.is-mr-mode) .branch-main-btn {
    color: var(--text-muted);
  }

  .branch-btn-group.target-group.is-mr-mode .branch-main-btn {
    color: var(--accent-emerald-dark);
  }

  .branch-text {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .branch-icon {
    color: var(--accent-emerald);
    flex-shrink: 0;
  }

  .compare-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .compare-icon.mr-icon {
    color: var(--accent-emerald);
  }

  .branch-chevron {
    font-size: 9px;
    color: var(--text-muted);
    margin-left: 2px;
  }

  .branch-action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: 50%;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.12s ease;
    padding: 0;
    outline: none;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
    margin-left: 2px;
  }

  .branch-action-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--accent-emerald);
    border-color: var(--border-hover);
  }

  .target-group.is-mr-mode .branch-action-btn:hover:not(:disabled) {
    color: var(--accent-emerald-dark);
  }

  .branch-action-btn.reset-btn:hover {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.35);
  }

  .branch-action-btn:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .pull-spin {
    display: inline-block;
    animation: spin 0.9s linear infinite;
    font-size: 13px;
    line-height: 1;
  }

  @keyframes spin {
    100% {
      transform: rotate(360deg);
    }
  }

  .compare-arrow {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-muted);
    user-select: none;
    margin: 0 2px;
  }

  .compare-ahead-tag {
    font-size: 10px;
    font-weight: 700;
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald-dark);
    border: 1px solid var(--accent-emerald-border);
    border-radius: 9999px;
    padding: 1px 6px;
    line-height: 1.2;
  }

  .refresh-icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: 50%;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.12s ease;
    margin-left: 4px;
  }

  .refresh-icon-btn:hover:not(:disabled) {
    background: var(--bg-card);
    color: var(--accent-emerald);
    border-color: var(--border-hover);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.05);
  }

  .refresh-icon-btn .spinning {
    display: inline-block;
    animation: spin 0.8s linear infinite;
  }

  .branch-dropdown,
  .compare-dropdown {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    z-index: 1000;
    min-width: 260px;
    max-width: 360px;
    background: var(--bg-dropdown);
    border: 1px solid var(--border-default);
    border-radius: 16px;
    box-shadow: var(--shadow-popover);
    backdrop-filter: var(--backdrop-filter, none);
    -webkit-backdrop-filter: var(--backdrop-filter, none);
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-family: "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .branch-dropdown-header,
  .compare-dropdown-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 8px 6px;
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-subtle);
    letter-spacing: 0.3px;
    text-transform: uppercase;
  }

  .branch-count {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
    text-transform: none;
  }

  .branch-switching-spinner {
    font-size: 11px;
    color: var(--accent-emerald);
    text-transform: none;
  }

  .branch-search-box {
    padding: 2px 0;
  }

  .branch-filter-input {
    width: 100%;
    box-sizing: border-box;
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: 9999px;
    color: var(--text-primary);
    font-size: 12px;
    padding: 6px 12px;
    outline: none;
    font-family: inherit;
    transition: all 0.15s ease;
  }

  .branch-filter-input:focus {
    border-color: var(--accent-emerald);
    background: var(--bg-card);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .branch-list {
    max-height: 240px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 2px 0;
  }

  .branch-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    text-align: left;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .branch-item:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .branch-item.is-current {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
    font-weight: 600;
  }

  .branch-item:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .branch-item-icon {
    color: inherit;
    flex-shrink: 0;
  }

  .branch-item-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .branch-check {
    color: var(--accent-emerald);
    font-size: 12px;
    font-weight: 700;
  }

  .no-matching-branches {
    padding: 12px;
    text-align: center;
    font-size: 12px;
    color: var(--text-muted);
  }

  .compare-options-section {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .compare-item.mode-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s ease;
  }

  .compare-item.mode-option:hover {
    background: var(--bg-hover);
    border-color: var(--border-hover);
  }

  .compare-item.mode-option.is-current {
    background: var(--accent-emerald-soft);
    border-color: var(--accent-emerald-border);
  }

  .compare-item-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .compare-mode-badge {
    font-size: 10px;
    font-weight: 700;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    padding: 2px 6px;
    border-radius: 6px;
  }

  .compare-mode-badge.default {
    background: var(--bg-subtle);
    color: var(--text-secondary);
    border: 1px solid var(--border-default);
  }

  .compare-item.mode-option.is-current .compare-mode-badge.default {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
    border-color: var(--accent-emerald-border);
  }

  .compare-item-desc {
    display: flex;
    flex-direction: column;
  }

  .compare-item-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .compare-item-sub {
    font-size: 11px;
    color: var(--text-muted);
  }

  .compare-section-divider {
    display: flex;
    align-items: center;
    padding: 8px 8px 4px;
    border-top: 1px solid var(--border-subtle);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .mr-branch-icon {
    color: var(--accent-emerald);
  }

  .no-repo-hint {
    color: var(--text-muted);
    font-size: 12.5px;
    font-weight: 500;
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }



  .action-btn {
    background: var(--accent-emerald);
    color: #ffffff;
    border: none;
    border-radius: 9999px;
    padding: 6px 16px;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 2px 6px var(--accent-glow);
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: var(--accent-emerald-hover);
    box-shadow: 0 4px 12px var(--accent-glow);
    transform: translateY(-1px);
  }

  .action-btn:active {
    transform: translateY(0);
  }

  .banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px;
    font-size: 12px;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: #f87171;
  }

  .success-banner {
    background: var(--accent-emerald-soft);
    border-bottom: 1px solid var(--accent-emerald-border);
    color: var(--accent-emerald-dark);
  }

  .banner-tag {
    font-weight: 700;
    text-transform: uppercase;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 9999px;
  }

  .error-tag {
    background: rgba(239, 68, 68, 0.25);
    color: #fca5a5;
  }

  .success-tag {
    background: var(--accent-emerald-soft-hover);
    color: var(--accent-emerald);
  }

  .banner-text {
    flex: 1;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11.5px;
  }

  .dismiss-banner {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    opacity: 0.6;
    font-size: 13px;
    padding: 2px 6px;
    border-radius: 50%;
  }

  .dismiss-banner:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.05);
  }

  .workspace-body {
    display: flex;
    flex: 1;
    overflow: hidden;
    background: transparent;
  }

  .sidebar-panel {
    width: 320px;
    border-right: 1px solid var(--border-subtle);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-sidebar);
    backdrop-filter: var(--backdrop-filter, blur(24px));
    -webkit-backdrop-filter: var(--backdrop-filter, blur(24px));
  }

  .diff-panel {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: var(--bg-pane);
  }
</style>
