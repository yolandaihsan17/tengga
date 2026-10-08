<script>
  export let hunk;
  export let onAccept;
  export let onReject;

  $: linesWithNumbers = computeLines(hunk);

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
    {#each linesWithNumbers as line}
      <div class="diff-row {line.kind}">
        <span class="gutter old-num">{line.oldNum}</span>
        <span class="gutter new-num">{line.newNum}</span>
        <span class="gutter marker">{line.symbol}</span>
        <span class="line-code">{line.content}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .hunk-card {
    border: none;
    border-radius: 14px;
    margin-bottom: 16px;
    background: var(--bg-card);
    backdrop-filter: var(--backdrop-filter, blur(16px));
    -webkit-backdrop-filter: var(--backdrop-filter, blur(16px));
    box-shadow: var(--shadow-card);
    overflow: hidden;
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
</style>
