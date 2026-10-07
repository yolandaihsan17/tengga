<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let settings = {
    autoDetectWorktrees: true,
    autoOpenNewWorktreeTab: true,
    autoSwitchToNewTab: true,
    hideTabBarWhenSingle: true,
    autoRefreshOnFocus: true,
    warnUnsavedOnSwitch: true,
    theme: "glass",
    glassBlur: 0.30,
  };

  export let onSave = (newSettings) => {};
  export let onClose = () => {};

  let activeTab = "worktrees"; // "worktrees" | "appearance" | "diff"

  function updateSetting(key, val) {
    settings = { ...settings, [key]: val };
    onSave(settings);
  }

  function handleBlurChange(e) {
    const val = parseFloat(e.target.value);
    updateSetting("glassBlur", val);
  }

  function resetDefaults() {
    settings = {
      autoDetectWorktrees: true,
      autoOpenNewWorktreeTab: true,
      autoSwitchToNewTab: true,
      hideTabBarWhenSingle: true,
      autoRefreshOnFocus: true,
      warnUnsavedOnSwitch: true,
      theme: "glass",
      glassBlur: 0.30,
    };
    onSave(settings);
  }

  function handleKeydown(e) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y-no-static-element-interactions a11y-click-events-have-key-events -->
<div class="settings-backdrop" on:click={onClose}>
  <!-- Modal Dialog -->
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div class="settings-dialog" role="dialog" aria-modal="true" on:click|stopPropagation>
    <!-- Header -->
    <div class="settings-header">
      <div class="header-left">
        <svg class="header-icon" viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
          <path d="M8 4.75a3.25 3.25 0 1 0 0 6.5 3.25 3.25 0 0 0 0-6.5zM6.25 8a1.75 1.75 0 1 1 3.5 0 1.75 1.75 0 0 1-3.5 0z"/>
          <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.901-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319z"/>
        </svg>
        <span class="header-title">Settings</span>
      </div>
      <button class="close-btn" on:click={onClose} title="Close (Esc)">✕</button>
    </div>

    <!-- Navigation Tabs -->
    <div class="settings-tabs">
      <button
        class="tab-btn"
        class:is-active={activeTab === "worktrees"}
        on:click={() => (activeTab = "worktrees")}
      >
        <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
          <path d="M11.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zm-2.25.75a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25zM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5zM3.5 3.25a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0z"/>
        </svg>
        Git Worktrees
      </button>

      <button
        class="tab-btn"
        class:is-active={activeTab === "appearance"}
        on:click={() => (activeTab = "appearance")}
      >
        <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
          <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 13V2a6 6 0 1 1 0 12z"/>
        </svg>
        Appearance
      </button>

      <button
        class="tab-btn"
        class:is-active={activeTab === "diff"}
        on:click={() => (activeTab = "diff")}
      >
        <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
          <path d="M0 2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V2zm4.5 5.5a.75.75 0 0 0 0 1.5h7a.75.75 0 0 0 0-1.5h-7zM8 3.5a.75.75 0 0 0-.75.75v7a.75.75 0 0 0 1.5 0v-7A.75.75 0 0 0 8 3.5z"/>
        </svg>
        Editor & Workflow
      </button>
    </div>

    <!-- Body Content -->
    <div class="settings-content">
      {#if activeTab === "worktrees"}
        <div class="section-group">
          <div class="setting-row">
            <div class="setting-info">
              <label class="setting-label" for="autoDetectWorktrees">Auto-detect Worktrees</label>
              <span class="setting-desc">Automatically discover linked git worktrees belonging to this repository.</span>
            </div>
            <label class="toggle-switch">
              <input
                id="autoDetectWorktrees"
                type="checkbox"
                checked={settings.autoDetectWorktrees}
                on:change={(e) => updateSetting("autoDetectWorktrees", e.target.checked)}
              />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div class="setting-row" class:is-disabled={!settings.autoDetectWorktrees}>
            <div class="setting-info">
              <label class="setting-label" for="autoOpenNewWorktreeTab">Auto-open New Worktrees in Tabs</label>
              <span class="setting-desc">When a new worktree is created (`git worktree add`), automatically open it in a tab.</span>
            </div>
            <label class="toggle-switch">
              <input
                id="autoOpenNewWorktreeTab"
                type="checkbox"
                checked={settings.autoOpenNewWorktreeTab}
                disabled={!settings.autoDetectWorktrees}
                on:change={(e) => updateSetting("autoOpenNewWorktreeTab", e.target.checked)}
              />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div class="setting-row" class:is-disabled={!settings.autoDetectWorktrees || !settings.autoOpenNewWorktreeTab}>
            <div class="setting-info">
              <label class="setting-label" for="autoSwitchToNewTab">Auto-focus Newly Created Tab</label>
              <span class="setting-desc">Immediately switch to the newly created worktree tab when detected.</span>
            </div>
            <label class="toggle-switch">
              <input
                id="autoSwitchToNewTab"
                type="checkbox"
                checked={settings.autoSwitchToNewTab}
                disabled={!settings.autoDetectWorktrees || !settings.autoOpenNewWorktreeTab}
                on:change={(e) => updateSetting("autoSwitchToNewTab", e.target.checked)}
              />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <label class="setting-label" for="hideTabBarWhenSingle">Hide Tab Bar for Single Worktree</label>
              <span class="setting-desc">Keep the interface borderless and compact when there are no extra worktrees.</span>
            </div>
            <label class="toggle-switch">
              <input
                id="hideTabBarWhenSingle"
                type="checkbox"
                checked={settings.hideTabBarWhenSingle}
                on:change={(e) => updateSetting("hideTabBarWhenSingle", e.target.checked)}
              />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      {:else if activeTab === "appearance"}
        <div class="section-group">
          <div class="setting-row vertical">
            <div class="setting-info">
              <span class="setting-label">Application Theme</span>
              <span class="setting-desc">Select your preferred visual style and vibrancy level.</span>
            </div>
            <div class="theme-options">
              <button
                class="theme-card"
                class:is-active={settings.theme === "glass"}
                on:click={() => updateSetting("theme", "glass")}
              >
                <div class="theme-preview glass-preview">
                  <div class="preview-bar"></div>
                  <div class="preview-glow"></div>
                </div>
                <span class="theme-name">Glass</span>
                <span class="theme-sub">macOS Vibrancy</span>
              </button>

              <button
                class="theme-card"
                class:is-active={settings.theme === "dark"}
                on:click={() => updateSetting("theme", "dark")}
              >
                <div class="theme-preview dark-preview">
                  <div class="preview-bar"></div>
                </div>
                <span class="theme-name">Dark</span>
                <span class="theme-sub">Deep Night</span>
              </button>

              <button
                class="theme-card"
                class:is-active={settings.theme === "light"}
                on:click={() => updateSetting("theme", "light")}
              >
                <div class="theme-preview light-preview">
                  <div class="preview-bar"></div>
                </div>
                <span class="theme-name">Light</span>
                <span class="theme-sub">Crisp Clean</span>
              </button>
            </div>
          </div>

          {#if settings.theme === "glass"}
            <div class="setting-row">
              <div class="setting-info">
                <label class="setting-label" for="glassBlur">Glass Background Blur Opacity</label>
                <span class="setting-desc">Adjust the subtle background translucency level.</span>
              </div>
              <div class="slider-cluster">
                <input
                  id="glassBlur"
                  type="range"
                  min="0.10"
                  max="0.70"
                  step="0.05"
                  value={settings.glassBlur}
                  on:input={handleBlurChange}
                  class="blur-slider"
                />
                <span class="slider-value">{Math.round(settings.glassBlur * 100)}%</span>
              </div>
            </div>
          {/if}
        </div>
      {:else if activeTab === "diff"}
        <div class="section-group">
          <div class="setting-row">
            <div class="setting-info">
              <label class="setting-label" for="autoRefreshOnFocus">Auto-refresh on Window Focus</label>
              <span class="setting-desc">Re-check git status and refresh diffs whenever returning to Tengga.</span>
            </div>
            <label class="toggle-switch">
              <input
                id="autoRefreshOnFocus"
                type="checkbox"
                checked={settings.autoRefreshOnFocus}
                on:change={(e) => updateSetting("autoRefreshOnFocus", e.target.checked)}
              />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <label class="setting-label" for="warnUnsavedOnSwitch">Warn on Unsaved Changes</label>
              <span class="setting-desc">Confirm before switching worktrees or branches when you have unsaved editor edits.</span>
            </div>
            <label class="toggle-switch">
              <input
                id="warnUnsavedOnSwitch"
                type="checkbox"
                checked={settings.warnUnsavedOnSwitch}
                on:change={(e) => updateSetting("warnUnsavedOnSwitch", e.target.checked)}
              />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="settings-footer">
      <button class="btn-reset" on:click={resetDefaults}>
        Reset to Defaults
      </button>
      <button class="btn-done" on:click={onClose}>
        Done
      </button>
    </div>
  </div>
</div>

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .settings-dialog {
    width: 540px;
    max-width: 90vw;
    background: var(--bg-card);
    border-radius: 14px;
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: var(--text-primary);
    border: none;
    animation: scaleUp 0.15s ease-out;
  }

  @keyframes scaleUp {
    from { transform: scale(0.97); opacity: 0.8; }
    to { transform: scale(1); opacity: 1; }
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    background: var(--bg-topbar);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .header-icon {
    color: var(--accent-emerald);
  }

  .header-title {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.2px;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 15px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
    transition: all 0.12s ease;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .settings-tabs {
    display: flex;
    gap: 4px;
    padding: 8px 16px;
    background: var(--bg-subtle);
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 600;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .tab-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .tab-btn.is-active {
    color: var(--accent-emerald);
    background: var(--bg-card);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  }

  .settings-content {
    padding: 20px;
    min-height: 260px;
    max-height: 400px;
    overflow-y: auto;
  }

  .section-group {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 4px 0;
  }

  .setting-row.vertical {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .setting-row.is-disabled {
    opacity: 0.45;
    pointer-events: none;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .setting-label {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
  }

  .setting-desc {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.4;
  }

  /* Toggle Switch */
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 42px;
    height: 24px;
    flex-shrink: 0;
    cursor: pointer;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    inset: 0;
    background-color: var(--bg-hover);
    border-radius: 24px;
    transition: 0.2s;
  }

  .toggle-slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background-color: #ffffff;
    border-radius: 50%;
    transition: 0.2s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .toggle-switch input:checked + .toggle-slider {
    background-color: var(--accent-emerald);
  }

  .toggle-switch input:checked + .toggle-slider:before {
    transform: translateX(18px);
  }

  /* Theme options */
  .theme-options {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 8px;
    border-radius: 10px;
    background: var(--bg-subtle);
    border: 2px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .theme-card:hover {
    background: var(--bg-hover);
  }

  .theme-card.is-active {
    border-color: var(--accent-emerald);
    background: var(--accent-emerald-soft);
  }

  .theme-preview {
    width: 100%;
    height: 48px;
    border-radius: 6px;
    margin-bottom: 8px;
    position: relative;
    overflow: hidden;
  }

  .preview-bar {
    height: 10px;
    width: 100%;
    opacity: 0.8;
  }

  .glass-preview {
    background: linear-gradient(135deg, rgba(16, 24, 40, 0.9), rgba(30, 41, 59, 0.8));
    backdrop-filter: blur(10px);
  }
  .glass-preview .preview-bar { background: rgba(16, 185, 129, 0.4); }
  .preview-glow {
    position: absolute;
    bottom: -10px;
    right: -10px;
    width: 30px;
    height: 30px;
    background: rgba(16, 185, 129, 0.35);
    filter: blur(10px);
  }

  .dark-preview {
    background: #0f172a;
  }
  .dark-preview .preview-bar { background: #1e293b; }

  .light-preview {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
  }
  .light-preview .preview-bar { background: #e2e8f0; }

  .theme-name {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .theme-sub {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 2px;
  }

  /* Slider cluster */
  .slider-cluster {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .blur-slider {
    width: 120px;
    accent-color: var(--accent-emerald);
    cursor: pointer;
  }

  .slider-value {
    font-size: 12px;
    font-family: "JetBrains Mono", monospace;
    font-weight: 600;
    color: var(--accent-emerald);
    width: 36px;
    text-align: right;
  }

  /* Footer */
  .settings-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    background: var(--bg-topbar);
  }

  .btn-reset {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 12.5px;
    cursor: pointer;
    padding: 6px 10px;
    border-radius: 6px;
    transition: all 0.12s ease;
  }

  .btn-reset:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .btn-done {
    background: var(--accent-emerald);
    color: #ffffff;
    border: none;
    font-size: 13px;
    font-weight: 700;
    padding: 7px 18px;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-done:hover {
    background: var(--accent-emerald-hover);
  }
</style>
