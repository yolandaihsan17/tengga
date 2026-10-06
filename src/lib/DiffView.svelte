<script>
  import HunkView from "./HunkView.svelte";
  import FindReplace from "./FindReplace.svelte";
  import { highlightCode } from "./syntax.js";

  export let file;
  export let fileContent = null;
  export let initialContent = null;
  export let isLoadingContent = false;
  export let contentError = null;
  export let baseBranch = null;
  export let onAccept;
  export let onReject;
  export let onSave = null;
  export let onSaveAll = null;
  export let onBufferChange = null;

  let activeView = "diff"; // "diff" | "content"
  let editedContent = "";
  let isDirty = false;
  let isSaving = false;
  let textareaEl = null;
  let gutterEl = null;
  let syntaxEl = null;
  let searchEl = null;
  let prevFile = null;

  // Find & Replace state
  let isFindOpen = false;
  let showReplace = false;
  let findQuery = "";
  let replaceText = "";
  let findCaseSensitive = false;
  let matchIndex = 0;
  let matches = [];
  let findReplaceRef = null;

  function escapeHtml(text) {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function buildSearchHtml(content, matchList, activeIdx) {
    if (!content || !matchList || matchList.length === 0) return "";
    const MAX_HIGHLIGHTED_MATCHES = 1000;
    const count = Math.min(matchList.length, MAX_HIGHLIGHTED_MATCHES);
    let result = "";
    let lastIndex = 0;
    for (let i = 0; i < count; i++) {
      const { start, end } = matchList[i];
      if (start > lastIndex) {
        result += escapeHtml(content.substring(lastIndex, start));
      }
      const matchText = escapeHtml(content.substring(start, end));
      const isActive = i === activeIdx;
      result += `<mark class="search-match ${isActive ? "active" : ""}">${matchText}</mark>`;
      lastIndex = end;
    }
    if (lastIndex < content.length) {
      result += escapeHtml(content.substring(lastIndex));
    }
    if (content.endsWith("\n")) {
      result += " ";
    }
    return result;
  }

  $: hasHunks = file && file.hunks && file.hunks.length > 0;
  $: pathSegments = file ? file.path.split("/") : [];

  // When selected file changes, synchronize content & dirty status
  $: {
    if (file && file.path !== prevFile) {
      prevFile = file.path;
      activeView = hasHunks ? "diff" : "content";
      if (initialContent !== null && initialContent !== undefined) {
        editedContent = initialContent;
        isDirty = editedContent !== (fileContent !== null ? fileContent : "");
      } else {
        editedContent = fileContent !== null ? fileContent : "";
        isDirty = false;
      }
      isFindOpen = false;
      matches = [];
    } else if (!isDirty && fileContent !== null && editedContent !== fileContent) {
      editedContent = fileContent;
    }
  }

  $: lineCount = Math.max(1, (editedContent || "").split("\n").length);
  $: gutterText = Array.from({ length: lineCount }, (_, i) => i + 1).join("\n");
  $: highlightedHtml = highlightCode(editedContent, file ? file.path : "");
  $: searchHtml = isFindOpen && matches.length > 0 ? buildSearchHtml(editedContent, matches, matchIndex) : "";

  function handleKeydown(e) {
    const isSaveKey = e.code === "KeyS" || e.key.toLowerCase() === "s" || e.key === "ß";
    const isModifier = e.metaKey || e.ctrlKey;

    // Cmd + Option + S: Save All
    if (isModifier && e.altKey && isSaveKey) {
      e.preventDefault();
      if (onSaveAll) {
        onSaveAll();
      } else {
        save();
      }
      return;
    }

    // Cmd + S: Save Current
    if (isModifier && !e.altKey && isSaveKey) {
      e.preventDefault();
      save();
      return;
    }

    // Cmd + F: Open Find
    if (isModifier && (e.code === "KeyF" || e.key.toLowerCase() === "f")) {
      e.preventDefault();
      isFindOpen = true;
      showReplace = false;
      const start = textareaEl ? textareaEl.selectionStart : 0;
      const end = textareaEl ? textareaEl.selectionEnd : 0;
      const selText = start !== end && textareaEl ? editedContent.substring(start, end) : "";
      if (findReplaceRef) {
        findReplaceRef.focusFind(selText);
      }
      return;
    }

    // Cmd + H: Open Find & Replace
    if (isModifier && (e.code === "KeyH" || e.key.toLowerCase() === "h")) {
      e.preventDefault();
      isFindOpen = true;
      showReplace = true;
      const start = textareaEl ? textareaEl.selectionStart : 0;
      const end = textareaEl ? textareaEl.selectionEnd : 0;
      const selText = start !== end && textareaEl ? editedContent.substring(start, end) : "";
      if (findReplaceRef) {
        if (selText) findQuery = selText;
        findReplaceRef.focusReplace();
      }
      return;
    }

    // Escape: Close Find if active
    if (e.key === "Escape" && isFindOpen) {
      e.preventDefault();
      handleCloseFind();
      return;
    }

    // Tab key 2 spaces indent
    if (e.key === "Tab") {
      e.preventDefault();
      const start = e.target.selectionStart;
      const end = e.target.selectionEnd;
      const val = editedContent;
      editedContent = val.substring(0, start) + "  " + val.substring(end);
      isDirty = editedContent !== (fileContent !== null ? fileContent : "");
      if (onBufferChange && file) {
        onBufferChange(file.path, editedContent, isDirty);
      }
      setTimeout(() => {
        if (textareaEl) {
          textareaEl.selectionStart = textareaEl.selectionEnd = start + 2;
        }
      }, 0);
    }
  }

  function handleInput(e) {
    editedContent = e.target.value;
    isDirty = editedContent !== (fileContent !== null ? fileContent : "");
    if (onBufferChange && file) {
      onBufferChange(file.path, editedContent, isDirty);
    }
    if (isFindOpen && findQuery) {
      runSearch(findQuery, findCaseSensitive);
    }
  }

  function syncScroll() {
    if (gutterEl && textareaEl) {
      gutterEl.scrollTop = textareaEl.scrollTop;
    }
    if (syntaxEl && textareaEl) {
      syntaxEl.scrollTop = textareaEl.scrollTop;
      syntaxEl.scrollLeft = textareaEl.scrollLeft;
    }
    if (searchEl && textareaEl) {
      searchEl.scrollTop = textareaEl.scrollTop;
      searchEl.scrollLeft = textareaEl.scrollLeft;
    }
  }

  async function save() {
    if (!onSave || !file || isSaving || !isDirty) return;
    isSaving = true;
    try {
      await onSave(file.path, editedContent);
      isDirty = false;
      if (onBufferChange) {
        onBufferChange(file.path, editedContent, false);
      }
    } catch (e) {
      // Error is caught and surfaced in App.svelte
    } finally {
      isSaving = false;
    }
  }

  function runSearch(query, caseSens) {
    findQuery = query;
    findCaseSensitive = caseSens;
    if (!query) {
      matches = [];
      matchIndex = 0;
      return;
    }
    const found = [];
    const source = caseSens ? editedContent : editedContent.toLowerCase();
    const target = caseSens ? query : query.toLowerCase();
    let idx = 0;
    while ((idx = source.indexOf(target, idx)) !== -1) {
      found.push({ start: idx, end: idx + target.length });
      idx += target.length;
    }
    matches = found;
    if (matches.length > 0) {
      matchIndex = Math.min(matchIndex, matches.length - 1);
      scrollToMatch(matchIndex, false);
    } else {
      matchIndex = 0;
    }
  }

  function handleNextMatch() {
    if (matches.length === 0) return;
    matchIndex = (matchIndex + 1) % matches.length;
    scrollToMatch(matchIndex, false);
  }

  function handlePrevMatch() {
    if (matches.length === 0) return;
    matchIndex = (matchIndex - 1 + matches.length) % matches.length;
    scrollToMatch(matchIndex, false);
  }

  function scrollToMatch(idx, focusEditor = false) {
    if (matches.length === 0 || !matches[idx] || !textareaEl) return;
    const { start, end } = matches[idx];
    if (focusEditor) {
      textareaEl.focus();
    }
    textareaEl.setSelectionRange(start, end);

    const beforeMatch = editedContent.substring(0, start);
    const lineIndex = beforeMatch.split("\n").length - 1;
    const lineHeight = 19;
    const targetY = lineIndex * lineHeight;

    const visibleHeight = textareaEl.clientHeight;
    if (targetY < textareaEl.scrollTop || targetY > textareaEl.scrollTop + visibleHeight - 40) {
      textareaEl.scrollTop = Math.max(0, targetY - Math.floor(visibleHeight / 3));
    }
    syncScroll();
  }

  function handleReplace(newText) {
    if (matches.length === 0 || !matches[matchIndex]) return;
    const { start, end } = matches[matchIndex];
    editedContent = editedContent.substring(0, start) + newText + editedContent.substring(end);
    isDirty = editedContent !== (fileContent !== null ? fileContent : "");
    if (onBufferChange && file) {
      onBufferChange(file.path, editedContent, isDirty);
    }
    runSearch(findQuery, findCaseSensitive);
  }

  function handleReplaceAll(newText) {
    if (matches.length === 0 || !findQuery) return;
    const escapeRegex = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(escapeRegex(findQuery), findCaseSensitive ? "g" : "gi");
    editedContent = editedContent.replace(regex, newText);
    isDirty = editedContent !== (fileContent !== null ? fileContent : "");
    if (onBufferChange && file) {
      onBufferChange(file.path, editedContent, isDirty);
    }
    runSearch(findQuery, findCaseSensitive);
  }

  function handleCloseFind() {
    isFindOpen = false;
    matches = [];
    if (textareaEl) textareaEl.focus();
  }
</script>

{#if file}
  <div class="diff-container">
    <!-- File Header Bar -->
    <div class="file-header-bar">
      <!-- Row 1: Full File Path Breadcrumb & Target Comparison Badge -->
      <div class="file-header-top">
        <div class="file-breadcrumb">
          {#each pathSegments as segment, i}
            {#if i > 0}
              <span class="crumb-sep">/</span>
            {/if}
            <span class="crumb" class:is-filename={i === pathSegments.length - 1}>
              {segment}
            </span>
          {/each}
          {#if isDirty}
            <span class="status-circle dirty" title="Unsaved changes (⌘S)">●</span>
          {:else if hasHunks}
            <span class="status-circle modified" title="Modified in git">●</span>
          {/if}
        </div>

        <div class="file-header-top-right">
          {#if !hasHunks && !file.is_binary && lineCount}
            <span class="line-count-meta">{lineCount} lines</span>
          {/if}
        </div>
      </div>

      <!-- Row 2: View Mode Switcher (Diff / Content) & Change Metrics (+add / -del) -->
      {#if hasHunks}
        <div class="file-header-bottom">
          <div class="file-toolbar-left">
            <div class="view-switch">
              <button
                class="switch-btn"
                class:active={activeView === "diff"}
                on:click={() => (activeView = "diff")}
              >
                Diff ({file.hunks.length})
              </button>
              <button
                class="switch-btn"
                class:active={activeView === "content"}
                on:click={() => (activeView = "content")}
              >
                File Content
              </button>
            </div>
          </div>

          <div class="file-toolbar-right">
            <span class="stat-badge add">+{file.additions}</span>
            <span class="stat-badge del">-{file.deletions}</span>
            {#if !file.is_binary && lineCount}
              <span class="line-count-meta">{lineCount} lines</span>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <!-- Main Content Area -->
    <div class="diff-scroll" class:editor-mode={activeView === "content" || !hasHunks}>
      {#if activeView === "diff" && hasHunks}
        <!-- Unified Diff Hunk View -->
        {#each file.hunks as hunk (hunk.id)}
          <HunkView
            {hunk}
            {baseBranch}
            onAccept={(id) => onAccept(file.path, id)}
            onReject={(id) => onReject(file.path, id)}
          />
        {/each}
      {:else if file.is_binary}
        <div class="status-panel">
          <div class="status-title">Binary File</div>
          <div class="status-desc">Direct text editing is not supported for binary assets.</div>
        </div>
      {:else}
        <!-- Full File Content & Inline Editor with Syntax Highlighting -->
        {#if isLoadingContent}
          <div class="loading-state">Loading file content...</div>
        {:else if contentError}
          <div class="status-panel">
            <div class="status-title">Unable to display content</div>
            <div class="status-desc">{contentError}</div>
          </div>
        {:else if fileContent !== null || editedContent !== ""}
          <div class="editor-container">
            <!-- Left: Line Numbers Gutter -->
            <div class="gutter-col" bind:this={gutterEl}>
              <pre class="gutter-text">{gutterText}</pre>
            </div>

            <!-- Right: Code Viewport (Syntax Layer behind, Search Highlights in middle, Textarea in front) -->
            <div class="code-viewport">
              <pre class="syntax-layer" bind:this={syntaxEl}><code>{@html highlightedHtml}</code></pre>
              {#if isFindOpen && matches.length > 0}
                <pre class="search-layer" bind:this={searchEl} aria-hidden="true"><code>{@html searchHtml}</code></pre>
              {/if}
              <textarea
                bind:this={textareaEl}
                class="code-editor"
                value={editedContent}
                on:input={handleInput}
                on:scroll={syncScroll}
                on:keydown={handleKeydown}
                spellcheck="false"
                wrap="off"
                placeholder="Empty file"
              ></textarea>

              <FindReplace
                bind:this={findReplaceRef}
                isOpen={isFindOpen}
                {showReplace}
                query={findQuery}
                {replaceText}
                {matchIndex}
                matchCount={matches.length}
                caseSensitive={findCaseSensitive}
                onSearch={runSearch}
                onNext={handleNextMatch}
                onPrev={handlePrevMatch}
                onReplace={handleReplace}
                onReplaceAll={handleReplaceAll}
                onToggleReplace={() => (showReplace = !showReplace)}
                onClose={handleCloseFind}
              />
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </div>
{:else}
  <div class="placeholder-state">
    <div class="placeholder-title">No file selected</div>
    <div class="placeholder-sub">Select a file from the sidebar to inspect its content or diffs</div>
  </div>
{/if}

<style>
  .diff-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: transparent;
    overflow: hidden;
  }

  .file-header-bar {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 16px 8px;
    background: var(--bg-topbar);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .file-header-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .file-breadcrumb {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 12px;
    overflow-x: auto;
    white-space: nowrap;
    min-width: 0;
    flex: 1;
    scrollbar-width: none;
  }

  .file-breadcrumb::-webkit-scrollbar {
    display: none;
  }

  .file-header-top-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .crumb-sep {
    color: var(--border-default);
    font-size: 11px;
    user-select: none;
  }

  .crumb {
    color: var(--text-muted);
  }

  .crumb.is-filename {
    color: var(--text-primary);
    font-weight: 700;
  }

  .status-circle {
    font-size: 8px;
    line-height: 1;
    margin-left: 6px;
    vertical-align: middle;
    display: inline-block;
    flex-shrink: 0;
  }

  .status-circle.dirty {
    color: var(--accent-emerald);
  }

  .status-circle.modified {
    color: #d97706;
  }

  .line-count-meta {
    font-size: 11px;
    color: var(--text-muted);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    user-select: none;
  }

  .file-header-bottom {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding-top: 1px;
  }

  .file-toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .file-toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .view-switch {
    display: flex;
    background: var(--bg-subtle);
    border: 1px solid var(--border-default);
    border-radius: 9999px;
    padding: 2px;
  }

  .switch-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 11.5px;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: 9999px;
    cursor: pointer;
    font-family: inherit;
    transition: all 0.15s ease;
  }

  .switch-btn:hover {
    color: var(--text-primary);
  }

  .switch-btn.active {
    background: var(--bg-card);
    color: var(--accent-emerald);
    font-weight: 700;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  }

  .stat-badge {
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 9999px;
  }

  .stat-badge.add {
    background: var(--diff-add-bg);
    color: var(--diff-add-sign);
    border: 1px solid var(--accent-emerald-border);
  }

  .stat-badge.del {
    background: var(--diff-del-bg);
    color: var(--diff-del-sign);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .diff-scroll {
    flex: 1;
    padding: 14px 16px;
    overflow-y: auto;
    min-height: 0;
    background: transparent;
  }

  .diff-scroll.editor-mode {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .editor-container {
    display: flex;
    flex: 1;
    min-height: 0;
    background: var(--editor-bg);
    border: 1px solid var(--border-default);
    border-radius: 14px;
    overflow: hidden;
    position: relative;
    box-shadow: var(--shadow-card);
    backdrop-filter: var(--backdrop-filter, blur(16px));
    -webkit-backdrop-filter: var(--backdrop-filter, blur(16px));
  }

  .gutter-col {
    background: var(--editor-gutter-bg);
    border-right: 1px solid var(--border-subtle);
    overflow: hidden;
    user-select: none;
    flex-shrink: 0;
    width: 48px;
    z-index: 5;
  }

  .gutter-text {
    margin: 0;
    padding: 8px 8px;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
    line-height: 19px;
    color: var(--editor-gutter-text);
    text-align: right;
  }

  .code-viewport {
    position: relative;
    flex: 1;
    height: 100%;
    min-width: 0;
    background: transparent;
    overflow: hidden;
  }

  .syntax-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    margin: 0;
    padding: 8px 10px;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
    line-height: 19px;
    border: none;
    overflow: hidden;
    white-space: pre;
    tab-size: 2;
    -moz-tab-size: 2;
    pointer-events: none;
    z-index: 1;
    color: var(--editor-text);
  }

  .syntax-layer code {
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    white-space: pre;
  }

  .search-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    margin: 0;
    padding: 8px 10px;
    background: transparent;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
    line-height: 19px;
    border: none;
    overflow: hidden;
    white-space: pre;
    tab-size: 2;
    -moz-tab-size: 2;
    pointer-events: none;
    z-index: 2;
    color: transparent;
  }

  .search-layer code {
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    white-space: pre;
    color: transparent;
  }

  :global(.search-match) {
    background: rgba(254, 240, 138, 0.5);
    border-radius: 2px;
    color: transparent;
  }

  :global(.search-match.active) {
    background: rgba(250, 204, 21, 0.85);
    outline: 1px solid #eab308;
    border-radius: 2px;
    color: transparent;
  }

  .code-editor {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    margin: 0;
    padding: 8px 10px;
    background: transparent;
    color: transparent;
    caret-color: var(--accent-emerald);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
    line-height: 19px;
    border: none;
    outline: none;
    resize: none;
    overflow: auto;
    white-space: pre;
    tab-size: 2;
    -moz-tab-size: 2;
    z-index: 3;
  }

  .code-editor::selection {
    background: var(--editor-selection);
    color: transparent;
  }

  /* Theme-adaptive Color Palette for Prism */
  :global(.token.keyword) {
    color: var(--accent-emerald);
    font-weight: 600;
  }

  :global(.token.tag) {
    color: #38bdf8;
  }

  :global(.token.tag .token.tag) {
    color: #38bdf8;
  }

  :global(.token.tag .token.punctuation) {
    color: var(--text-muted);
  }

  :global(.token.attr-name) {
    color: #2dd4bf;
  }

  :global(.token.attr-value),
  :global(.token.string),
  :global(.token.char) {
    color: var(--accent-emerald);
  }

  :global(.token.attr-value .token.punctuation) {
    color: var(--accent-emerald);
  }

  :global(.token.comment),
  :global(.token.prolog),
  :global(.token.doctype),
  :global(.token.cdata) {
    color: var(--text-muted);
    font-style: italic;
  }

  :global(.token.number),
  :global(.token.boolean) {
    color: #f59e0b;
  }

  :global(.token.function),
  :global(.token.function-variable) {
    color: #60a5fa;
  }

  :global(.token.class-name) {
    color: #22d3ee;
    font-weight: 600;
  }

  :global(.token.constant) {
    color: #f472b6;
    font-weight: 600;
  }

  :global(.token.property) {
    color: #38bdf8;
  }

  :global(.token.selector) {
    color: #f59e0b;
  }

  :global(.token.operator) {
    color: var(--text-secondary);
  }

  :global(.token.punctuation) {
    color: var(--text-muted);
  }

  :global(.token.regex) {
    color: #f87171;
  }

  :global(.token.variable),
  :global(.token.parameter) {
    color: #38bdf8;
  }

  /* Legacy tok-* support */
  :global(.tok-keyword) { color: var(--accent-emerald); font-weight: 600; }
  :global(.tok-string) { color: var(--accent-emerald); }
  :global(.tok-comment) { color: var(--text-muted); font-style: italic; }
  :global(.tok-number) { color: #f59e0b; }
  :global(.tok-const) { color: #f472b6; font-weight: 600; }
  :global(.tok-fn) { color: #60a5fa; }
  :global(.tok-type) { color: #22d3ee; }
  :global(.tok-ident) { color: #38bdf8; }
  :global(.tok-symbol) { color: var(--text-muted); }

  .loading-state {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }

  .status-panel {
    margin: 40px auto;
    max-width: 380px;
    background: var(--bg-card);
    border: 1px solid var(--border-default);
    border-radius: 16px;
    box-shadow: var(--shadow-popover);
    padding: 24px;
    text-align: center;
  }

  .status-title {
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .status-desc {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .placeholder-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    text-align: center;
    padding: 20px;
  }

  .placeholder-title {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .placeholder-sub {
    font-size: 12px;
    color: var(--text-muted);
    max-width: 300px;
    line-height: 1.5;
  }
</style>
