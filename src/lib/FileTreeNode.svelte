<script>
  export let node;
  export let depth = 0;
  export let selectedFile = null;
  export let expandedFolders = {};
  export let searchQuery = "";
  export let toggleFolder;
  export let onSelect;

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
    <span class="file-label" class:is-modified={node.hasChanges}>{node.name}</span>
    {#if node.hasChanges}
      <span class="change-circle" class:dirty={node.isDirty} title={node.isDirty ? "Unsaved edits" : "Modified in git"}>●</span>
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
    color: var(--text-primary);
  }

  .folder-row:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
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

  .folder-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .folder-title.has-changes {
    font-weight: 700;
    color: var(--text-primary);
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
</style>
