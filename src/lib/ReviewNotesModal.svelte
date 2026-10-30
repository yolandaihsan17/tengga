<script>
  export let notes = [];
  export let onClose = () => {};
  export let onDeleteNote = (id) => {};
  export let onClearAll = () => {};
  export let onSelectFile = (filePath, lineNum) => {};

  let isCopied = false;

  $: groupedNotes = (() => {
    const map = new Map();
    for (const note of notes) {
      if (!map.has(note.filePath)) {
        map.set(note.filePath, []);
      }
      map.get(note.filePath).push(note);
    }
    return Array.from(map.entries()).map(([filePath, fileNotes]) => ({
      filePath,
      notes: fileNotes.sort((a, b) => Number(a.lineNum || 0) - Number(b.lineNum || 0)),
    }));
  })();

  function handleCopyAllForAI() {
    if (notes.length === 0) return;
    let prompt = "Please address the following code review comments:\n\n";
    for (const group of groupedNotes) {
      prompt += `### File: \`${group.filePath}\`\n`;
      for (const n of group.notes) {
        prompt += `- Line ${n.lineNum}:\n`;
        if (n.lineContent && n.lineContent.trim()) {
          prompt += `  Code snippet: \`${n.lineContent.trim()}\`\n`;
        }
        prompt += `  Instruction: ${n.text}\n\n`;
      }
    }
    navigator.clipboard.writeText(prompt);
    isCopied = true;
    setTimeout(() => {
      isCopied = false;
    }, 2000);
  }

  function handleKeydown(e) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="modal-backdrop" on:click={onClose}>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions a11y-click-events-have-key-events -->
  <div class="modal-dialog" on:click|stopPropagation role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-header">
      <div class="header-left">
        <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor" class="header-icon">
          <path fill-rule="evenodd" d="M1 8c0-3.866 3.134-7 7-7s7 3.134 7 7-3.134 7-7 7a6.96 6.96 0 0 1-3.668-1.042L1.5 15l1.042-2.832A6.96 6.96 0 0 1 1 8zm7-5.5a5.5 5.5 0 1 0 0 11 5.474 5.474 0 0 0 2.964-.868.75.75 0 0 1 .65-.078l1.716.63-.63-1.716a.75.75 0 0 1 .078-.65A5.474 5.474 0 0 0 13.5 8 5.5 5.5 0 0 0 8 2.5z"/>
        </svg>
        <h2 id="modal-title" class="modal-title">Code Review Notes</h2>
        <span class="count-badge">{notes.length}</span>
      </div>
      <button class="close-btn" on:click={onClose} aria-label="Close modal">✕</button>
    </div>

    <!-- Action Bar -->
    <div class="modal-actions-bar">
      <button
        class="action-btn copy-prompt-btn"
        disabled={notes.length === 0}
        on:click={handleCopyAllForAI}
      >
        <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
          <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25v-7.5z"/>
          <path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25v-7.5zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25h-7.5z"/>
        </svg>
        <span>{isCopied ? "Copied Prompt to Clipboard!" : "Copy All Notes for AI Prompt"}</span>
      </button>

      {#if notes.length > 0}
        <button
          class="action-btn clear-btn"
          on:click={() => {
            if (confirm("Are you sure you want to clear all review notes?")) {
              onClearAll();
            }
          }}
        >
          Clear All
        </button>
      {/if}
    </div>

    <!-- Notes List Body -->
    <div class="modal-body">
      {#if notes.length === 0}
        <div class="empty-state">
          <div class="empty-title">No review notes yet</div>
          <div class="empty-desc">
            Hover over any diff line in the reviewer and click the comment icon to add inline review notes. You can then copy all notes into an AI agent prompt with one click.
          </div>
        </div>
      {:else}
        <div class="groups-container">
          {#each groupedNotes as group (group.filePath)}
            <div class="file-group">
              <div class="file-group-header">
                <span class="file-path">{group.filePath}</span>
                <span class="file-notes-count">{group.notes.length} note(s)</span>
              </div>
              <div class="file-notes-list">
                {#each group.notes as note (note.id)}
                  <div class="note-item">
                    <div class="note-item-header">
                      <span class="line-pill">Line {note.lineNum}</span>
                      <div class="note-item-actions">
                        <button
                          class="jump-btn"
                          on:click={() => {
                            onSelectFile(note.filePath, Number(note.lineNum || 1));
                            onClose();
                          }}
                          title="Jump to this file and line"
                        >
                          View
                        </button>
                        <button
                          class="delete-note-btn"
                          on:click={() => onDeleteNote(note.id)}
                          title="Delete note"
                        >
                          ✕
                        </button>
                      </div>
                    </div>
                    {#if note.lineContent && note.lineContent.trim()}
                      <div class="code-context">
                        <code>{note.lineContent.trim()}</code>
                      </div>
                    {/if}
                    <div class="note-text">{note.text}</div>
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <span class="footer-hint">Notes are kept in local storage and can be pasted into Claude Code, Antigravity, or Cursor.</span>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 16px;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal-dialog {
    width: 620px;
    max-width: 95vw;
    max-height: 85vh;
    background: var(--bg-card);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 16px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.45), 0 0 0 1px rgba(255, 255, 255, 0.05);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    animation: scaleUp 0.15s cubic-bezier(0.16, 1, 0.3, 1);
    color: var(--text-primary);
  }

  @keyframes scaleUp {
    from { transform: scale(0.96); opacity: 0.8; }
    to { transform: scale(1); opacity: 1; }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    background: var(--bg-topbar);
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.05));
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .header-icon {
    color: var(--accent-emerald);
  }

  .modal-title {
    font-size: 14px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
  }

  .count-badge {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
    font-size: 11px;
    font-weight: 700;
    padding: 1px 7px;
    border-radius: 9999px;
  }

  .close-btn {
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
    transition: all 0.12s ease;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .modal-actions-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 18px;
    background: var(--bg-subtle);
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.04));
    gap: 10px;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .copy-prompt-btn {
    background: var(--accent-emerald);
    color: #ffffff;
  }

  .copy-prompt-btn:hover:not(:disabled) {
    background: var(--accent-emerald-dark, #059669);
  }

  .copy-prompt-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .clear-btn {
    background: transparent;
    color: var(--text-muted);
  }

  .clear-btn:hover {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }

  .modal-body {
    padding: 16px 18px;
    overflow-y: auto;
    flex: 1;
    min-height: 200px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
  }

  .empty-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .empty-desc {
    font-size: 12px;
    color: var(--text-muted);
    max-width: 380px;
    line-height: 1.5;
  }

  .groups-container {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .file-group {
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.06));
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-topbar);
  }

  .file-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--bg-subtle);
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.04));
  }

  .file-path {
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", monospace;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .file-notes-count {
    font-size: 11px;
    color: var(--text-muted);
  }

  .file-notes-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
  }

  .note-item {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.04));
    border-left: 3px solid var(--accent-emerald);
    border-radius: 6px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .note-item-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .line-pill {
    font-size: 10px;
    font-weight: 700;
    color: var(--accent-emerald);
    background: var(--accent-emerald-soft);
    padding: 1px 6px;
    border-radius: 4px;
  }

  .note-item-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .jump-btn {
    font-size: 10.5px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: 4px;
    border: none;
    background: var(--bg-subtle);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .jump-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .delete-note-btn {
    border: none;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 11px;
    padding: 2px 4px;
  }

  .delete-note-btn:hover {
    color: #ef4444;
  }

  .code-context {
    padding: 4px 8px;
    border-radius: 4px;
    background: var(--bg-subtle);
    font-family: "JetBrains Mono", monospace;
    font-size: 11px;
    color: var(--text-muted);
    overflow-x: auto;
    white-space: pre;
  }

  .note-text {
    font-size: 12px;
    color: var(--text-primary);
    line-height: 1.4;
    white-space: pre-wrap;
  }

  .modal-footer {
    padding: 10px 18px;
    background: var(--bg-topbar);
    border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.05));
  }

  .footer-hint {
    font-size: 11px;
    color: var(--text-dim);
  }
</style>