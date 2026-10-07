<script>
  export let node;
  export let depth = 0;
  export let selectedFile = null;
  export let expandedFolders = {};
  export let searchQuery = "";
  export let toggleFolder;
  export let onSelect;
  export let stageMode = null; // "staged" | "unstaged" | null
  export let disabled = false;
  export let onStage = () => {};
  export let onUnstage = () => {};
  export let onDiscard = () => {};

  $: expanded = Boolean((searchQuery && searchQuery.trim()) || expandedFolders[node.path]);
</script>

{#if node.type === "folder"}
  <div class="folder-group">
    <button
      class="row folder-row"
      style="padding-left: {8 + depth * 14}px;"
      on:click={() => toggleFolder(node.path)}
      title={node.path}
    >
      <span class="chevron" class:expanded>
        <svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="6 4 10 8 6 12"></polyline>
        </svg>
      </span>
      <span class="folder-icon">
        {#if expanded}
          <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
            <path d="M.54 3.87.5 3a2 2 0 0 1 2-2h3.672a2 2 0 0 1 1.414.586l.828.828A2 2 0 0 0 9.828 3h3.982a2 2 0 0 1 1.992 2.181L15.5 8H3.04a2 2 0 0 0-1.92 1.455L.23 12.3A2 2 0 0 1 0 11.5v-7a2 2 0 0 1 .54-.63z"/>
            <path d="M3.23 9.4A1 1 0 0 1 4.19 9h11.43a.5.5 0 0 1 .48.64l-1.2 4.2a1 1 0 0 1-.96.76H2.38a.5.5 0 0 1-.48-.64l1.33-4.56z"/>
          </svg>
        {:else}
          <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
            <path d="M1.75 1A1.75 1.75 0 0 0 0 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0 0 16 13.25v-8.5A1.75 1.75 0 0 0 14.25 3H7.5a.25.25 0 0 1-.2-.1l-.9-1.2C6.07 1.26 5.55 1 5 1H1.75z"/>
          </svg>
        {/if}
      </span>
      <span class="folder-title" class:has-changes={node.hasChanges}>{node.name}</span>
      {#if node.hasChanges}
        <span class="change-circle folder-dot" title="Contains changes">●</span>
      {/if}
    </button>

    {#if expanded && node.childrenList}
      <div class="folder-children">
        {#each node.childrenList as child (child.path)}
          <svelte:self
            node={child}
            depth={depth + 1}
            {selectedFile}
            {expandedFolders}
            {searchQuery}
            {toggleFolder}
            {onSelect}
            {stageMode}
            {disabled}
            {onStage}
            {onUnstage}
            {onDiscard}
          />
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <button
    class="row file-row"
    style="padding-left: {26 + depth * 14}px;"
    class:active={node.path === selectedFile}
    class:modified={node.hasChanges}
    class:is-dirty={node.isDirty}
    on:click={() => onSelect(node.path)}
    title={node.path}
  >
    <span class="file-icon">
      <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
        <path d="M3.75 1.5a1.75 1.75 0 0 0-1.75 1.75v9.5c0 .966.784 1.75 1.75 1.75h8.5A1.75 1.75 0 0 0 14 12.75v-6.5a.75.75 0 0 0-.22-.53l-4.5-4.5A.75.75 0 0 0 8.75 1H3.75zM8.5 2.5v3.25c0 .414.336.75.75.75H12.5v6.25a.25.25 0 0 1-.25.25H3.75a.25.25 0 0 1-.25-.25v-9.5a.25.25 0 0 1 .25-.25h4.75z"/>
      </svg>
    </span>
    <span class="file-label" class:is-modified={node.hasChanges}>{node.name}</span>
    {#if node.isUntracked}
      <span class="status-badge untracked" title="Untracked file">U</span>
    {:else if node.hasChanges}
      <span class="change-circle" class:dirty={node.isDirty} title={node.isDirty ? "Unsaved edits" : "Modified in git"}>●</span>
    {/if}

    {#if stageMode === "staged"}
      <div class="file-hover-actions">
        <button
          class="file-action-icon-btn unstage-btn"
          disabled={disabled}
          on:click|stopPropagation={() => onUnstage(node.path)}
          title="Unstage Changes (-)"
        >
          <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor">
            <path d="M3.75 7.25h8.5a.75.75 0 0 1 0 1.5h-8.5a.75.75 0 0 1 0-1.5z"/>
          </svg>
        </button>
      </div>
    {:else if stageMode === "unstaged"}
      <div class="file-hover-actions">
        <button
          class="file-action-icon-btn stage-btn"
          disabled={disabled}
          on:click|stopPropagation={() => onStage(node.path)}
          title="Stage Changes (+)"
        >
          <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor">
            <path d="M7.25 3.75a.75.75 0 0 1 1.5 0v3.5h3.5a.75.75 0 0 1 0 1.5h-3.5v3.5a.75.75 0 0 1-1.5 0v-3.5h-3.5a.75.75 0 0 1 0-1.5h3.5v-3.5z"/>
          </svg>
        </button>
        <button
          class="file-action-icon-btn discard-btn"
          disabled={disabled}
          on:click|stopPropagation={() => onDiscard(node.path)}
          title="Discard Changes"
        >
          <svg viewBox="0 0 16 16" width="10" height="10" fill="currentColor">
            <path fill-rule="evenodd" d="M1.25 8A6.75 6.75 0 1 1 8 14.75a.75.75 0 0 1 0-1.5 5.25 5.25 0 1 0-4.66-2.85l1.44-.36a.75.75 0 1 1 .36 1.45l-3 1a.75.75 0 0 1-.95-.55l-1-3a.75.75 0 1 1 1.42-.48l.45 1.34A6.71 6.71 0 0 1 1.25 8z"/>
          </svg>
        </button>
      </div>
    {/if}
  </button>
{/if}

<style>
  .folder-group {
    display: flex;
    flex-direction: column;
  }

  .folder-children {
    display: flex;
    flex-direction: column;
  }

  .row {
    display: flex;
    align-items: center;
    width: calc(100% - 8px);
    margin: 1px 4px;
    background: transparent;
    border: none;
    border-radius: 8px;
    padding-top: 6px;
    padding-bottom: 6px;
    padding-right: 8px;
    cursor: pointer;
    font-size: 11.5px;
    text-align: left;
    box-sizing: border-box;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    transition: background 0.1s ease, color 0.1s ease;
  }

  .folder-row {
    color: var(--folder-text, var(--text-primary));
  }

  .folder-row:hover {
    background: var(--bg-hover);
  }

  .folder-row:hover .folder-title {
    color: var(--folder-hover, var(--text-primary));
  }

  .chevron {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    flex-shrink: 0;
    margin-right: 4px;
    user-select: none;
    transition: transform 0.15s ease;
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .folder-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--folder-icon, #f59e0b);
    flex-shrink: 0;
    margin-right: 6px;
    transition: transform 0.12s ease;
  }

  .folder-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--folder-text, var(--text-primary));
    line-height: 1.3;
    transition: color 0.12s ease;
  }

  .folder-title.has-changes {
    font-weight: 700;
  }

  .file-row {
    color: var(--text-primary);
    font-weight: 450;
    padding-top: 6px;
    padding-bottom: 6px;
    padding-right: 8px;
  }

  .file-row:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .file-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    flex-shrink: 0;
    margin-right: 6px;
    opacity: 0.75;
    transition: color 0.12s ease, opacity 0.12s ease;
  }

  .file-row:hover .file-icon {
    color: var(--text-secondary);
    opacity: 1;
  }

  .file-row.modified .file-icon,
  .file-row.is-dirty .file-icon {
    color: var(--accent-emerald);
    opacity: 1;
  }

  .file-row.modified,
  .file-row.is-dirty {
    color: var(--accent-emerald);
  }

  .file-row.modified .file-label,
  .file-row.is-dirty .file-label,
  .file-label.is-modified {
    font-weight: 600 !important;
    color: var(--accent-emerald) !important;
  }

  .file-row.active {
    background: var(--accent-emerald-soft);
    color: var(--text-primary) !important;
  }

  .file-row.active .file-label {
    font-weight: 700 !important;
    color: var(--text-primary) !important;
  }

  .file-row.active.modified .file-label,
  .file-row.active.is-dirty .file-label {
    font-weight: 700 !important;
    color: var(--accent-emerald-dark) !important;
  }

  .file-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 11.5px;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .change-circle {
    font-size: 8px;
    line-height: 1;
    color: #d97706;
    margin-left: 6px;
    flex-shrink: 0;
    display: inline-block;
  }

  .change-circle.dirty {
    color: var(--accent-emerald);
  }

  .change-circle.folder-dot {
    font-size: 6px;
    color: #d97706;
  }

  .status-badge.untracked {
    font-size: 10px;
    font-weight: 700;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
    color: var(--accent-emerald);
    background: var(--accent-emerald-soft);
    padding: 0 4px;
    border-radius: 3px;
    margin-left: 6px;
    flex-shrink: 0;
  }

  .file-hover-actions {
    display: none;
    align-items: center;
    gap: 2px;
    margin-left: auto;
    padding-left: 4px;
    flex-shrink: 0;
  }

  .file-row:hover .file-hover-actions {
    display: flex;
  }

  .file-action-icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.1s ease;
    padding: 0;
  }

  .file-action-icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .file-action-icon-btn.stage-btn:hover {
    color: var(--accent-emerald);
  }

  .file-action-icon-btn.discard-btn:hover {
    color: #ef4444;
  }
</style>
