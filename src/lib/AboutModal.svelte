<script>
  import { openExternalUrl } from "./api.js";

  export let onClose = () => {};

  function handleKeydown(e) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function handleOpenLink(url) {
    openExternalUrl(url);
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Backdrop -->
<!-- svelte-ignore a11y-no-static-element-interactions a11y-click-events-have-key-events -->
<div class="about-backdrop" on:click={onClose}>
  <!-- Modal Dialog -->
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div class="about-dialog" role="dialog" aria-modal="true" aria-label="About Tengga" on:click|stopPropagation>
    <!-- Header -->
    <div class="about-header">
      <span class="about-header-title">About Tengga</span>
      <button class="close-btn" on:click={onClose} title="Close (Esc)" aria-label="Close">
        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
        </svg>
      </button>
    </div>

    <!-- Body -->
    <div class="about-body">
      <!-- App Identity Hero -->
      <div class="app-identity">
        <img src="/app-icon.png" alt="Tengga Icon" class="app-avatar" />
        <div class="app-title-group">
          <div class="title-row">
            <h2 class="app-name">Tengga</h2>
            <span class="version-badge">v0.1.0</span>
          </div>
          <p class="app-tagline">The Lightweight Git Diff Reviewer</p>
        </div>
      </div>

      <p class="app-description">
        A fast, distraction-free Git review desktop application built with Tauri, Rust, and Svelte.
        Stage hunks surgically, inspect uncommitted changes, and multitask across Git worktrees without Electron bloat.
      </p>

      <!-- Tech Badges -->
      <div class="tech-pills">
        <span class="tech-pill">Tauri 2.0</span>
        <span class="tech-pill">Rust</span>
        <span class="tech-pill">Svelte</span>
        <span class="tech-pill">Native WebView</span>
      </div>

      <!-- Author Card -->
      <div class="author-card">
        <div class="author-header">
          <div class="author-meta">
            <span class="author-label">AUTHOR & CREATOR</span>
            <span class="author-name">Yolanda Ihsan</span>
          </div>
        </div>

        <div class="author-links">
          <a
            href="https://github.com/yolandaihsan17"
            target="_blank"
            rel="noopener noreferrer"
            class="author-link"
            title="GitHub Profile"
            on:click|preventDefault={() => handleOpenLink("https://github.com/yolandaihsan17")}
          >
            <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor">
              <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
            </svg>
            <span>GitHub</span>
          </a>

          <a
            href="https://www.linkedin.com/in/yolandaihsan/"
            target="_blank"
            rel="noopener noreferrer"
            class="author-link"
            title="LinkedIn Profile"
            on:click|preventDefault={() => handleOpenLink("https://www.linkedin.com/in/yolandaihsan/")}
          >
            <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor">
              <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
            </svg>
            <span>LinkedIn</span>
          </a>

          <a
            href="https://github.com/yolandaihsan17/tengga"
            target="_blank"
            rel="noopener noreferrer"
            class="author-link"
            title="Project Repository"
            on:click|preventDefault={() => handleOpenLink("https://github.com/yolandaihsan17/tengga")}
          >
            <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
              <path fill-rule="evenodd" d="M14 4.5V14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2h5.5L14 4.5zm-3 0A1.5 1.5 0 0 1 9.5 3V1H4a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V4.5h-2z"/>
              <path d="M8.646 6.646a.5.5 0 0 1 .708 0l2 2a.5.5 0 0 1 0 .708l-2 2a.5.5 0 0 1-.708-.708L10.293 9 8.646 7.354a.5.5 0 0 1 0-.708zm-1.292 0a.5.5 0 0 0-.708 0l-2 2a.5.5 0 0 0 0 .708l2 2a.5.5 0 0 0 .708-.708L5.707 9l1.647-1.646a.5.5 0 0 0 0-.708z"/>
            </svg>
            <span>Repository</span>
          </a>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="about-footer">
      <div class="copyright-note">
        Released under the MIT License • Copyright &copy; 2026 Yolanda Ihsan
      </div>
      <button class="btn-close-about" on:click={onClose}>Done</button>
    </div>
  </div>
</div>

<style>
  .about-backdrop {
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

  .about-dialog {
    width: 480px;
    max-width: 95vw;
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
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

  .about-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    background: var(--bg-topbar);
    border-bottom: 1px solid var(--border-subtle);
  }

  .about-header-title {
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .close-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 6px;
    border-radius: 6px;
    transition: all 0.12s ease;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .about-body {
    padding: 22px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .app-identity {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .app-avatar {
    width: 56px;
    height: 56px;
    border-radius: 14px;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.25), 0 0 0 1px rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
  }

  .app-title-group {
    display: flex;
    flex-direction: column;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .app-name {
    margin: 0;
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.4px;
    color: var(--text-primary);
  }

  .version-badge {
    font-size: 11px;
    font-family: "JetBrains Mono", monospace;
    font-weight: 600;
    color: var(--accent-emerald);
    background: var(--accent-emerald-soft);
    padding: 2px 8px;
    border-radius: 9999px;
    border: 1px solid rgba(16, 185, 129, 0.25);
  }

  .app-tagline {
    margin: 3px 0 0;
    font-size: 13px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .app-description {
    margin: 0;
    font-size: 13px;
    line-height: 1.55;
    color: var(--text-secondary);
  }

  .tech-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tech-pill {
    font-size: 11.5px;
    font-family: "JetBrains Mono", monospace;
    color: var(--text-muted);
    background: var(--bg-subtle);
    padding: 3px 10px;
    border-radius: 6px;
    border: 1px solid var(--border-subtle);
  }

  .author-card {
    background: var(--bg-subtle);
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .author-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .author-label {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.6px;
    color: var(--accent-emerald);
  }

  .author-name {
    font-size: 15px;
    font-weight: 800;
    color: var(--text-primary);
  }

  .author-links {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .author-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    padding: 6px 12px;
    border-radius: 8px;
    text-decoration: none;
    transition: all 0.12s ease;
  }

  .author-link:hover {
    color: var(--accent-emerald);
    border-color: var(--accent-emerald);
    background: var(--accent-emerald-soft);
    transform: translateY(-1px);
  }

  .about-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 18px;
    background: var(--bg-topbar);
    border-top: 1px solid var(--border-subtle);
  }

  .copyright-note {
    font-size: 11px;
    color: var(--text-muted);
  }

  .btn-close-about {
    background: var(--accent-emerald);
    color: #ffffff;
    border: none;
    font-size: 12.5px;
    font-weight: 700;
    padding: 6px 16px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-close-about:hover {
    opacity: 0.9;
  }
</style>
