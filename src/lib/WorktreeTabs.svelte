<script>
  export let worktrees = [];
  export let activePath = "";
  export let dirtyWorktrees = new Set();
  export let onSelect = (path) => {};
  export let onClose = (path) => {};

  let isDropdownOpen = false;

  $: activeWorktree = worktrees.find((w) => w.path === activePath);
  $: inactiveWorktrees = worktrees.filter((w) => w.path !== activePath);

  function handleTabClick(path) {
    if (path !== activePath) {
      onSelect(path);
    }
  }

  function handleCloseClick(e, path) {
    e.stopPropagation();
    onClose(path);
  }

  function toggleDropdown() {
    isDropdownOpen = !isDropdownOpen;
  }

  function handleSelectFromDropdown(path) {
    isDropdownOpen = false;
    onSelect(path);
  }
</script>

<div class="worktree-tabs-bar">
  <div class="tabs-scroll-area">
    {#each worktrees as wt, index}
      <div
        class="worktree-tab"
        class:is-active={wt.path === activePath}
        on:click={() => handleTabClick(wt.path)}
        title="{wt.name} ({wt.branch}) — {wt.path}"
        role="button"
        tabindex="0"
        on:keydown={(e) => e.key === "Enter" && handleTabClick(wt.path)}
      >
        <svg class="tab-icon" viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z"/>
        </svg>

        <span class="tab-name">{wt.name}</span>

        {#if wt.branch}
          <span class="tab-branch-pill" class:is-main-branch={wt.branch === "main" || wt.branch === "master"}>
            {wt.branch}
          </span>
        {/if}

        {#if dirtyWorktrees.has(wt.path)}
          <span class="tab-dirty-dot" title="Unsaved changes">●</span>
        {/if}

        {#if index < 9}
          <span class="tab-index-hint" title="Switch with ⌘{index + 1}">⌘{index + 1}</span>
        {/if}

        {#if worktrees.length > 1}
          <button
            class="tab-close-btn"
            on:click={(e) => handleCloseClick(e, wt.path)}
            title="Close tab (does not delete worktree)"
          >
            ✕
          </button>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Worktree switcher dropdown button -->
  <div class="tabs-actions">
    <div class="worktree-menu-container">
      <button
        class="menu-btn"
        on:click|stopPropagation={toggleDropdown}
        title="All Worktrees in Repository"
      >
        <span class="btn-text">Worktrees</span>
        <span class="badge-count">{worktrees.length}</span>
        <span class="chevron">{isDropdownOpen ? "▴" : "▾"}</span>
      </button>

      {#if isDropdownOpen}
        <!-- svelte-ignore a11y-no-static-element-interactions a11y-click-events-have-key-events -->
        <div class="dropdown-panel" on:click|stopPropagation>
          <div class="dropdown-header">
            <span>Repository Worktrees</span>
            <span class="wt-total">{worktrees.length} active</span>
          </div>

          <div class="dropdown-list">
            {#each worktrees as wt}
              <button
                class="dropdown-item"
                class:is-active={wt.path === activePath}
                on:click={() => handleSelectFromDropdown(wt.path)}
              >
                <div class="item-left">
                  <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
                    <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z"/>
                  </svg>
                  <span class="item-name">{wt.name}</span>
                  <span class="item-branch">{wt.branch}</span>
                  {#if wt.is_main}
                    <span class="main-tag">main</span>
                  {/if}
                </div>
                {#if wt.path === activePath}
                  <span class="active-check">✓</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .worktree-tabs-bar {
    position: relative;
    z-index: 90;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 12px;
    background: var(--bg-topbar);
    border-bottom: none;
    user-select: none;
    flex-shrink: 0;
    gap: 8px;
  }

  .tabs-scroll-area {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    flex: 1;
    min-width: 0;
  }

  .tabs-scroll-area::-webkit-scrollbar {
    display: none;
  }

  .worktree-tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    border-radius: 7px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.12s ease;
    flex-shrink: 0;
    position: relative;
  }

  .worktree-tab:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .worktree-tab.is-active {
    color: var(--accent-emerald);
    background: var(--bg-card);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .tab-icon {
    flex-shrink: 0;
    opacity: 0.8;
  }

  .worktree-tab.is-active .tab-icon {
    opacity: 1;
    color: var(--accent-emerald);
  }

  .tab-name {
    max-width: 130px;
    overflow: hidden;
    text-overflow: ellipsis;
    letter-spacing: -0.1px;
  }

  .tab-branch-pill {
    font-size: 10.5px;
    font-weight: 600;
    font-family: "JetBrains Mono", monospace;
    padding: 1px 6px;
    border-radius: 9999px;
    background: var(--bg-subtle);
    color: var(--text-muted);
    letter-spacing: -0.2px;
  }

  .worktree-tab.is-active .tab-branch-pill {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
  }

  .tab-dirty-dot {
    color: #f59e0b;
    font-size: 10px;
    margin-left: -2px;
  }

  .tab-index-hint {
    font-size: 9px;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
    color: var(--text-dim);
    opacity: 0.7;
    margin-left: 2px;
  }

  .tab-close-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 10px;
    cursor: pointer;
    margin-left: 2px;
    padding: 0;
    opacity: 0;
    transition: all 0.1s ease;
  }

  .worktree-tab:hover .tab-close-btn {
    opacity: 1;
  }

  .tab-close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tabs-actions {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .worktree-menu-container {
    position: relative;
  }

  .menu-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 24px;
    padding: 0 8px;
    border-radius: 6px;
    background: var(--bg-subtle);
    border: none;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .menu-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .badge-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 14px;
    height: 14px;
    padding: 0 4px;
    border-radius: 9999px;
    background: var(--bg-card);
    font-size: 9.5px;
    font-weight: 700;
    color: var(--text-muted);
  }

  .chevron {
    font-size: 9px;
    opacity: 0.7;
  }

  /* Dropdown Panel */
  .dropdown-panel {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    width: 280px;
    background: var(--bg-dropdown);
    border-radius: 10px;
    box-shadow: var(--shadow-popover);
    backdrop-filter: var(--backdrop-filter, blur(20px));
    -webkit-backdrop-filter: var(--backdrop-filter, blur(20px));
    overflow: hidden;
    z-index: 1000;
  }

  .dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    background: var(--bg-subtle);
    font-size: 11px;
    font-weight: 700;
    color: var(--text-muted);
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .wt-total {
    font-weight: 600;
    color: var(--accent-emerald);
  }

  .dropdown-list {
    max-height: 220px;
    overflow-y: auto;
    padding: 4px;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 10px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.1s ease;
    text-align: left;
  }

  .dropdown-item:hover {
    background: var(--bg-hover);
  }

  .dropdown-item.is-active {
    color: var(--accent-emerald);
    background: var(--accent-emerald-soft);
  }

  .item-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .item-name {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-branch {
    font-size: 11px;
    color: var(--text-muted);
    font-family: "JetBrains Mono", monospace;
  }

  .main-tag {
    font-size: 9px;
    padding: 1px 4px;
    border-radius: 4px;
    background: var(--bg-subtle);
    color: var(--text-muted);
    text-transform: uppercase;
    font-weight: 700;
  }

  .active-check {
    color: var(--accent-emerald);
    font-weight: 700;
  }
</style>
