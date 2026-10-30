<script>
  export let hunk;
  export let filePath = "";
  export let onAccept;
  export let onReject;
  export let reviewNotes = [];
  export let onAddNote = () => {};
  export let onDeleteNote = () => {};

  $: linesWithNumbers = computeLines(hunk);

  const INITIAL_LINE_LIMIT = 150;
  let visibleLineCount = INITIAL_LINE_LIMIT;
  let prevHunkId = null;

  $: if (hunk && hunk.id !== prevHunkId) {
    prevHunkId = hunk.id;
    visibleLineCount = INITIAL_LINE_LIMIT;
  }

  $: totalHunkLines = linesWithNumbers.length;
  $: isHunkCapped = totalHunkLines > visibleLineCount;
  $: displayedLines = isHunkCapped ? linesWithNumbers.slice(0, visibleLineCount) : linesWithNumbers;

  function showMoreLines() {
    visibleLineCount += 200;
  }

  function showAllLines() {
    visibleLineCount = totalHunkLines;
  }

  $: notesByLine = (() => {
    const map = new Map();
    if (!reviewNotes || reviewNotes.length === 0) return map;
    for (const n of reviewNotes) {
      if (n.filePath === filePath && n.hunkId === hunk?.id) {
        const key = String(n.lineNum);
        if (!map.has(key)) map.set(key, []);
        map.get(key).push(n);
      }
    }
    return map;
  })();

  let isCopiedAI = false;
  let activeNoteLineKey = null;
  let noteDraft = "";

  function handleCopyHunkForAI() {
    if (!hunk) return;
    const pathStr = filePath || "file";
    let formatted = `### Diff in \`${pathStr}\` (${hunk.header})\n\`\`\`diff\n${hunk.header}\n`;
    for (const l of hunk.lines) {
      const prefix = l.kind === "add" ? "+" : l.kind === "remove" ? "-" : " ";
      formatted += `${prefix}${l.content}\n`;
    }
    formatted += "```\n";
    navigator.clipboard.writeText(formatted);
    isCopiedAI = true;
    setTimeout(() => {
      isCopiedAI = false;
    }, 2000);
  }

  function getLineKey(line) {
    return `${hunk.id}_${line.oldNum || 0}_${line.newNum || 0}`;
  }

  function getNotesForLine(line) {
    const lineNum = String(line.newNum || line.oldNum);
    return notesByLine.get(lineNum) || [];
  }

  function toggleNoteInput(line) {
    const key = getLineKey(line);
    if (activeNoteLineKey === key) {
      activeNoteLineKey = null;
      noteDraft = "";
    } else {
      activeNoteLineKey = key;
      noteDraft = "";
    }
  }

  function handleSaveNote(line) {
    if (!noteDraft.trim()) return;
    const lineNum = line.newNum || line.oldNum;
    onAddNote({
      filePath,
      hunkId: hunk.id,
      hunkHeader: hunk.header,
      lineNum,
      lineContent: line.content,
      text: noteDraft.trim(),
    });
    noteDraft = "";
    activeNoteLineKey = null;
  }

  function handleNoteKeydown(e, line) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleSaveNote(line);
    } else if (e.key === "Escape") {
      activeNoteLineKey = null;
      noteDraft = "";
    }
  }

  function computeLines(h) {
    if (!h || !h.lines) return [];
    let oldLine = 1;
    let newLine = 1;
    const match = h.header.match(/@@\s+-(\d+)(?:,\d+)?\s+\+(\d+)(?:,\d+)?\s+@@/);
    if (match) {
      oldLine = parseInt(match[1], 10);
      newLine = parseInt(match[2], 10);
    }

    return h.lines.map((line) => {
      let o = "";
      let n = "";
      let symbol = " ";
      if (line.kind === "add") {
        n = newLine++;
        symbol = "+";
      } else if (line.kind === "remove") {
        o = oldLine++;
        symbol = "-";
      } else {
        o = oldLine++;
        n = newLine++;
        symbol = " ";
      }
      return {
        kind: line.kind,
        content: line.content,
        oldNum: o,
        newNum: n,
        symbol,
      };
    });
  }
</script>

<div class="hunk-card">
  <div class="hunk-toolbar">
    <div class="hunk-meta">
      <span class="range-pill">{hunk.header}</span>
      {#if hunk.source && hunk.source !== "branch_compare"}
        <span class="source-badge {hunk.source}">{hunk.source}</span>
      {/if}
    </div>
    <div class="hunk-actions">
      <button
        class="btn btn-copy-ai"
        on:click={handleCopyHunkForAI}
        title="Copy hunk formatted in Markdown diff for AI prompt"
      >
        <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor">
          <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25v-7.5z"/>
          <path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25v-7.5zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25h-7.5z"/>
        </svg>
        <span>{isCopiedAI ? "Copied!" : "Copy for AI"}</span>
      </button>
      {#if hunk.source === "unstaged"}
        <button class="btn btn-accept" on:click={() => onAccept(hunk.id)}>
          Stage
        </button>
      {/if}
      {#if hunk.source !== "branch_compare"}
        <button class="btn btn-reject" on:click={() => onReject(hunk.id)}>
          {hunk.source === "staged" ? "Unstage" : "Revert"}
        </button>
      {/if}
    </div>
  </div>

  <div class="diff-table">
    {#each displayedLines as line}
      {@const lineKey = getLineKey(line)}
      {@const lineNotes = getNotesForLine(line)}
      <div class="diff-row {line.kind}">
        <span class="gutter old-num">{line.oldNum}</span>
        <span class="gutter new-num">{line.newNum}</span>
        <span class="gutter marker">{line.symbol}</span>
        <span class="line-code">{line.content}</span>
        <div class="line-actions">
          <button
            class="line-note-trigger"
            class:has-notes={lineNotes.length > 0}
            class:is-active={activeNoteLineKey === lineKey}
            on:click={() => toggleNoteInput(line)}
            title={lineNotes.length > 0 ? `${lineNotes.length} review note(s)` : "Add review note"}
          >
            <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor">
              <path fill-rule="evenodd" d="M1 8c0-3.866 3.134-7 7-7s7 3.134 7 7-3.134 7-7 7a6.96 6.96 0 0 1-3.668-1.042L1.5 15l1.042-2.832A6.96 6.96 0 0 1 1 8zm7-5.5a5.5 5.5 0 1 0 0 11 5.474 5.474 0 0 0 2.964-.868.75.75 0 0 1 .65-.078l1.716.63-.63-1.716a.75.75 0 0 1 .078-.65A5.474 5.474 0 0 0 13.5 8 5.5 5.5 0 0 0 8 2.5z"/>
            </svg>
            {#if lineNotes.length > 0}
              <span class="note-count">{lineNotes.length}</span>
            {/if}
          </button>
        </div>
      </div>

      <!-- Render Existing Notes for this Line -->
      {#if lineNotes.length > 0}
        <div class="line-notes-container">
          {#each lineNotes as note (note.id)}
            <div class="note-card">
              <div class="note-header">
                <span class="note-badge">Line {note.lineNum} Note</span>
                <button
                  class="note-delete-btn"
                  on:click={() => onDeleteNote(note.id)}
                  title="Delete note"
                >
                  <svg viewBox="0 0 16 16" width="10" height="10" fill="currentColor">
                    <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z"/>
                  </svg>
                </button>
              </div>
              <div class="note-body">{note.text}</div>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Render Inline Note Input Form when toggled -->
      {#if activeNoteLineKey === lineKey}
        <div class="inline-note-input-row">
          <div class="note-input-box">
            <!-- svelte-ignore a11y-autofocus -->
            <textarea
              class="note-textarea"
              bind:value={noteDraft}
              on:keydown={(e) => handleNoteKeydown(e, line)}
              placeholder="Add review note or instruction for AI (Cmd+Enter to save)..."
              rows="2"
              autofocus
            ></textarea>
            <div class="note-input-actions">
              <button class="btn-note-cancel" on:click={() => toggleNoteInput(line)}>
                Cancel
              </button>
              <button
                class="btn-note-save"
                disabled={!noteDraft.trim()}
                on:click={() => handleSaveNote(line)}
              >
                Save Note
              </button>
            </div>
          </div>
        </div>
      {/if}
    {/each}
  </div>

  {#if isHunkCapped}
    <div class="hunk-expand-bar">
      <span class="hunk-expand-info">Showing first {visibleLineCount} of {totalHunkLines} lines in this hunk</span>
      <div class="hunk-expand-actions">
        <button class="btn-expand-hunk" on:click={showMoreLines}>
          Show next 200 lines
        </button>
        <button class="btn-expand-hunk" on:click={showAllLines}>
          Show all ({totalHunkLines} lines)
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .hunk-card {
    border: none;
    border-radius: 14px;
    margin-bottom: 16px;
    background: var(--bg-card);
    box-shadow: var(--shadow-card);
    overflow: hidden;
    contain: content;
  }

  .hunk-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--bg-topbar);
    border-bottom: none;
    padding: 8px 12px;
  }

  .hunk-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .range-pill {
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--bg-subtle);
    border: none;
    padding: 2px 8px;
    border-radius: 9999px;
  }

  .source-badge {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 2px 8px;
    border-radius: 9999px;
  }

  .source-badge.unstaged {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
    border: none;
  }

  .source-badge.staged {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
    border: none;
  }

  .hunk-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn {
    font-size: 11px;
    font-weight: 600;
    padding: 3px 12px;
    border-radius: 9999px;
    cursor: pointer;
    font-family: inherit;
    transition: all 0.12s ease;
    border: none;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .btn-copy-ai {
    background: var(--bg-subtle);
    color: var(--text-secondary);
  }

  .btn-copy-ai:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-accept {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
  }

  .btn-accept:hover {
    background: var(--accent-emerald-soft-hover);
    color: var(--accent-emerald-dark);
    box-shadow: 0 2px 6px var(--accent-glow);
  }

  .btn-reject {
    background: rgba(239, 68, 68, 0.12);
    color: #f87171;
  }

  .btn-reject:hover {
    background: rgba(239, 68, 68, 0.22);
    color: #fca5a5;
    box-shadow: 0 2px 6px rgba(239, 68, 68, 0.2);
  }

  .diff-table {
    display: flex;
    flex-direction: column;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 12px;
    line-height: 19px;
    background: transparent;
  }

  .diff-row {
    display: flex;
    align-items: stretch;
    width: 100%;
    min-height: 20px;
    position: relative;
  }

  .gutter {
    flex-shrink: 0;
    user-select: none;
    text-align: right;
    color: var(--text-muted);
    padding: 0 6px;
    box-sizing: border-box;
  }

  .old-num, .new-num {
    width: 42px;
    font-size: 11px;
    background: var(--diff-gutter-bg);
  }

  .old-num {
    border-right: none;
  }

  .new-num {
    border-right: none;
  }

  .marker {
    width: 22px;
    text-align: center;
    font-weight: 700;
  }

  .line-code {
    flex: 1;
    padding: 0 10px;
    white-space: pre;
    overflow-x: auto;
    color: var(--diff-context-text);
  }

  .line-actions {
    display: flex;
    align-items: center;
    padding-right: 8px;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .diff-row:hover .line-actions,
  .line-note-trigger.has-notes,
  .line-note-trigger.is-active {
    opacity: 1;
  }

  .line-note-trigger {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 6px;
    border-radius: 4px;
    border: none;
    background: var(--bg-subtle);
    color: var(--text-dim);
    cursor: pointer;
    font-size: 10px;
    transition: all 0.12s ease;
  }

  .line-note-trigger:hover,
  .line-note-trigger.is-active {
    background: var(--bg-hover);
    color: var(--accent-emerald);
  }

  .line-note-trigger.has-notes {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
    opacity: 1;
  }

  .note-count {
    font-size: 9px;
    font-weight: 700;
  }

  /* Inline Notes Display */
  .line-notes-container {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 16px 8px 106px;
    background: var(--bg-subtle);
    border-left: 3px solid var(--accent-emerald);
  }

  .note-card {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.06));
    border-radius: 6px;
    padding: 6px 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .note-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .note-badge {
    font-size: 10px;
    font-weight: 700;
    color: var(--accent-emerald);
    letter-spacing: 0.3px;
  }

  .note-delete-btn {
    border: none;
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    transition: color 0.12s ease;
  }

  .note-delete-btn:hover {
    color: #ef4444;
  }

  .note-body {
    font-size: 12px;
    color: var(--text-primary);
    line-height: 1.4;
    white-space: pre-wrap;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  /* Inline Note Input Row */
  .inline-note-input-row {
    padding: 6px 16px 8px 106px;
    background: var(--bg-subtle);
    border-left: 3px solid var(--accent-emerald);
  }

  .note-input-box {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--bg-card);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    padding: 8px 10px;
  }

  .note-textarea {
    width: 100%;
    min-height: 48px;
    max-height: 100px;
    padding: 6px 8px;
    border-radius: 4px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    background: var(--bg-subtle);
    color: var(--text-primary);
    font-size: 12px;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    outline: none;
    resize: vertical;
    box-sizing: border-box;
  }

  .note-textarea:focus {
    border-color: var(--accent-emerald);
  }

  .note-input-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

  .btn-note-cancel {
    padding: 3px 8px;
    font-size: 11px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .btn-note-cancel:hover {
    color: var(--text-primary);
  }

  .btn-note-save {
    padding: 3px 10px;
    font-size: 11px;
    font-weight: 600;
    border: none;
    border-radius: 4px;
    background: var(--accent-emerald);
    color: #ffffff;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-note-save:hover:not(:disabled) {
    background: var(--accent-emerald-dark, #059669);
  }

  .btn-note-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .diff-row.add {
    background: var(--diff-add-bg);
  }

  .diff-row.add .new-num {
    color: var(--text-muted);
    background: var(--diff-add-num-bg);
  }

  .diff-row.add .marker {
    color: var(--diff-add-sign);
    background: var(--diff-add-num-bg);
  }

  .diff-row.add .line-code {
    color: var(--diff-add-text);
    font-weight: 450;
  }

  .diff-row.remove {
    background: var(--diff-del-bg);
  }

  .diff-row.remove .old-num {
    color: var(--text-muted);
    background: var(--diff-del-num-bg);
  }

  .diff-row.remove .marker {
    color: var(--diff-del-sign);
    background: var(--diff-del-num-bg);
  }

  .diff-row.remove .line-code {
    color: var(--diff-del-text);
    font-weight: 450;
  }

  .diff-row.context .line-code {
    color: var(--diff-context-text);
  }

  .hunk-expand-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 14px;
    background: var(--bg-topbar);
    border-top: 1px solid var(--border-subtle);
    font-size: 11px;
    color: var(--text-muted);
  }

  .hunk-expand-info {
    font-weight: 500;
  }

  .hunk-expand-actions {
    display: flex;
    gap: 8px;
  }

  .btn-expand-hunk {
    background: var(--bg-hover);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 3px 8px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-expand-hunk:hover {
    background: var(--bg-card);
    color: var(--text-primary);
    border-color: var(--accent);
  }
</style>
