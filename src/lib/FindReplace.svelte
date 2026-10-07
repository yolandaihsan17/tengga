<script>
  export let isOpen = false;
  export let showReplace = false;
  export let query = "";
  export let replaceText = "";
  export let matchIndex = 0;
  export let matchCount = 0;
  export let caseSensitive = false;

  export let onSearch;
  export let onNext;
  export let onPrev;
  export let onReplace;
  export let onReplaceAll;
  export let onToggleReplace;
  export let onClose;

  let findInputEl;
  let replaceInputEl;

  export function focusFind(initialText = "") {
    if (initialText) {
      query = initialText;
      if (onSearch) onSearch(query, caseSensitive);
    }
    setTimeout(() => {
      if (findInputEl) {
        findInputEl.focus();
        findInputEl.select();
      }
    }, 10);
  }

  export function focusReplace() {
    showReplace = true;
    setTimeout(() => {
      if (replaceInputEl) {
        replaceInputEl.focus();
        replaceInputEl.select();
      }
    }, 10);
  }

  function handleFindInput(e) {
    query = e.target.value;
    if (onSearch) onSearch(query, caseSensitive);
  }

  function handleToggleCase() {
    caseSensitive = !caseSensitive;
    if (onSearch) onSearch(query, caseSensitive);
  }

  function handleFindKeydown(e) {
    if (e.key === "Escape") {
      e.preventDefault();
      if (onClose) onClose();
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) {
        if (onPrev) onPrev();
      } else {
        if (onNext) onNext();
      }
    }
  }

  function handleReplaceKeydown(e) {
    if (e.key === "Escape") {
      e.preventDefault();
      if (onClose) onClose();
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      if (e.metaKey || e.ctrlKey) {
        if (onReplaceAll) onReplaceAll(replaceText);
      } else {
        if (onReplace) onReplace(replaceText);
      }
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="find-replace-card" role="dialog" aria-label="Find and Replace" tabindex="-1" on:keydown|stopPropagation>
    <!-- Row 1: Find -->
    <div class="find-row">
      <button
        class="toggle-replace-btn"
        class:is-expanded={showReplace}
        on:mousedown|preventDefault
        on:click={onToggleReplace}
        title="Toggle Replace"
      >
        {showReplace ? "▾" : "▸"}
      </button>

      <div class="input-container">
        <input
          bind:this={findInputEl}
          type="text"
          class="find-input"
          placeholder="Find"
          value={query}
          on:input={handleFindInput}
          on:keydown={handleFindKeydown}
          spellcheck="false"
        />
        <button
          class="case-btn"
          class:active={caseSensitive}
          on:mousedown|preventDefault
          on:click={handleToggleCase}
          title="Match Case (Aa)"
        >
          Aa
        </button>
      </div>

      <span class="match-count" class:no-match={query && matchCount === 0}>
        {#if !query}
          0 of 0
        {:else if matchCount === 0}
          No results
        {:else}
          {matchIndex + 1} of {matchCount}
        {/if}
      </span>

      <div class="nav-actions">
        <button
          class="icon-btn"
          on:mousedown|preventDefault
          on:click={onPrev}
          disabled={matchCount === 0}
          title="Previous Match (Shift+Enter)"
        >
          ▲
        </button>
        <button
          class="icon-btn"
          on:mousedown|preventDefault
          on:click={onNext}
          disabled={matchCount === 0}
          title="Next Match (Enter)"
        >
          ▼
        </button>
        <button
          class="icon-btn close-btn"
          on:click={onClose}
          title="Close (Escape)"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Row 2: Replace -->
    {#if showReplace}
      <div class="replace-row">
        <div class="indent-spacer"></div>

        <div class="input-container replace-container">
          <input
            bind:this={replaceInputEl}
            type="text"
            class="find-input"
            placeholder="Replace"
            bind:value={replaceText}
            on:keydown={handleReplaceKeydown}
            spellcheck="false"
          />
        </div>

        <div class="replace-actions">
          <button
            class="text-btn"
            on:click={() => onReplace(replaceText)}
            disabled={matchCount === 0}
            title="Replace current match (Enter)"
          >
            Replace
          </button>
          <button
            class="text-btn"
            on:click={() => onReplaceAll(replaceText)}
            disabled={matchCount === 0}
            title="Replace all matches (⌘+Enter)"
          >
            All
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .find-replace-card {
    position: absolute;
    top: 10px;
    right: 20px;
    z-index: 100;
    background: var(--bg-dropdown);
    border: none;
    border-radius: 16px;
    box-shadow: var(--shadow-popover);
    backdrop-filter: var(--backdrop-filter, none);
    -webkit-backdrop-filter: var(--backdrop-filter, none);
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    user-select: none;
    min-width: 320px;
    max-width: 440px;
    font-family: "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .find-row,
  .replace-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .toggle-replace-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    padding: 3px 5px;
    border-radius: 6px;
    line-height: 1;
    transition: all 0.12s ease;
  }

  .toggle-replace-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .indent-spacer {
    width: 14px;
    flex-shrink: 0;
  }

  .input-container {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    background: var(--bg-subtle);
    border: none;
    border-radius: 9999px;
    padding: 2px 8px;
    transition: all 0.15s ease;
  }

  .input-container:focus-within {
    background: var(--bg-card);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .find-input {
    width: 100%;
    background: none;
    border: none;
    outline: none;
    padding: 4px 6px;
    color: var(--text-primary);
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 11.5px;
    box-sizing: border-box;
  }

  .find-input::placeholder {
    color: var(--text-muted);
  }

  .case-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 10px;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    cursor: pointer;
    padding: 2px 6px;
    margin-right: 2px;
    border-radius: 9999px;
    font-weight: 700;
    transition: all 0.12s ease;
  }

  .case-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .case-btn.active {
    background: var(--accent-emerald);
    color: #ffffff;
  }

  .match-count {
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 10.5px;
    color: var(--text-muted);
    min-width: 55px;
    text-align: right;
    flex-shrink: 0;
  }

  .match-count.no-match {
    color: #ef4444;
    font-weight: 600;
  }

  .nav-actions {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;
  }

  .icon-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 9px;
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-radius: 50%;
    line-height: 1;
    transition: all 0.12s ease;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .close-btn {
    font-size: 10px;
    margin-left: 2px;
  }

  .replace-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .text-btn {
    background: var(--bg-subtle);
    border: none;
    color: var(--text-secondary);
    font-size: 11px;
    padding: 3px 10px;
    border-radius: 9999px;
    cursor: pointer;
    font-weight: 600;
    font-family: inherit;
    line-height: 1.2;
    transition: all 0.12s ease;
  }

  .text-btn:hover:not(:disabled) {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
  }

  .text-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
