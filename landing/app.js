// Tengga Landing Page Interactive Logic

// 1. Files & Diffs Data for Interactive Mockup
const MOCK_FILES = {
  "src/lib/auth.ts": {
    path: "src/lib/auth.ts",
    header: "@@ -42,7 +42,9 @@ validateSession()",
    staged: false,
    additions: 3,
    deletions: 1,
    rows: [
      { oldNum: "42", newNum: "42", marker: " ", kind: "ctx", code: "  const session = await getSessionToken();" },
      { oldNum: "43", newNum: "  ", marker: "-", kind: "del", code: "- if (!session) return { valid: false };" },
      { oldNum: "  ", newNum: "43", marker: "+", kind: "add", code: "+ if (!session || session.isExpired()) {" },
      { oldNum: "  ", newNum: "44", marker: "+", kind: "add", code: "+   await purgeStaleTokens();" },
      { oldNum: "  ", newNum: "45", marker: "+", kind: "add", code: "+   return { valid: false, reason: 'expired' };" },
      { oldNum: "  ", newNum: "46", marker: "+", kind: "add", code: "+ }" },
      { oldNum: "44", newNum: "47", marker: " ", kind: "ctx", code: "  return { valid: true, user: session.user };" }
    ]
  },
  "src-tauri/src/watcher.rs": {
    path: "src-tauri/src/watcher.rs",
    header: "@@ -18,6 +18,8 @@ debounced_fs_watcher()",
    staged: true,
    additions: 2,
    deletions: 0,
    rows: [
      { oldNum: "18", newNum: "18", marker: " ", kind: "ctx", code: "  let debounce_duration = Duration::from_millis(150);" },
      { oldNum: "  ", newNum: "19", marker: "+", kind: "add", code: "+ let (tx, rx) = channel();" },
      { oldNum: "  ", newNum: "20", marker: "+", kind: "add", code: "+ let mut watcher = RecommendedWatcher::new(tx)?;" },
      { oldNum: "19", newNum: "21", marker: " ", kind: "ctx", code: "  watcher.watch(&repo_path, RecursiveMode::Recursive)?;" }
    ]
  },
  "Cargo.toml": {
    path: "Cargo.toml",
    header: "@@ -8,3 +8,4 @@ [dependencies]",
    staged: false,
    additions: 1,
    deletions: 1,
    rows: [
      { oldNum: "8",  newNum: "8",  marker: " ", kind: "ctx", code: "  tauri = { version = \"2.0.0\", features = [\"tray-icon\"] }" },
      { oldNum: "9",  newNum: "  ", marker: "-", kind: "del", code: "- notify = \"5.0\"" },
      { oldNum: "  ", newNum: "9",  marker: "+", kind: "add", code: "+ notify-debouncer-mini = \"0.4\"" },
      { oldNum: "10", newNum: "10", marker: " ", kind: "ctx", code: "  serde = { version = \"1.0\", features = [\"derive\"] }" }
    ]
  }
};

let currentSelectedFile = "src/lib/auth.ts";

// 2. OS Detection for Smart Download CTA
function detectUserOS() {
  const ua = navigator.userAgent || "";
  const platform = navigator.platform || "";

  if (/Mac|iPhone|iPad|iPod/i.test(platform) || /Mac/i.test(ua)) {
    return {
      osName: "macOS",
      label: "Download for macOS",
      sublabel: "Apple Silicon & Intel • .dmg",
      url: "https://github.com/yolandaihsan17/tengga/releases/latest"
    };
  } else if (/Win/i.test(platform) || /Windows/i.test(ua)) {
    return {
      osName: "Windows",
      label: "Download for Windows",
      sublabel: "64-bit • .msi / .exe",
      url: "https://github.com/yolandaihsan17/tengga/releases/latest"
    };
  } else if (/Linux/i.test(platform) || /Linux/i.test(ua)) {
    return {
      osName: "Linux",
      label: "Download for Linux",
      sublabel: ".deb & .AppImage",
      url: "https://github.com/yolandaihsan17/tengga/releases/latest"
    };
  }

  return {
    osName: "All Platforms",
    label: "Download Tengga",
    sublabel: "macOS, Windows, Linux",
    url: "https://github.com/yolandaihsan17/tengga/releases/latest"
  };
}

// 3. Theme Toggle Support
function initTheme() {
  const storedTheme = localStorage.getItem("tengga_theme") || "dark";
  setTheme(storedTheme);

  document.querySelectorAll(".theme-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      const theme = btn.getAttribute("data-theme-val");
      setTheme(theme);
    });
  });
}

function setTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem("tengga_theme", theme);

  document.querySelectorAll(".theme-btn").forEach(b => {
    if (b.getAttribute("data-theme-val") === theme) {
      b.classList.add("active");
    } else {
      b.classList.remove("active");
    }
  });
}

// 4. Render Mockup Diff View
function renderMockup() {
  const file = MOCK_FILES[currentSelectedFile];
  if (!file) return;

  // Title
  const titleEl = document.getElementById("mock-diff-filename");
  if (titleEl) titleEl.textContent = file.path;

  // Hunk Meta
  const rangePill = document.getElementById("mock-range-pill");
  if (rangePill) rangePill.textContent = file.header;

  const badge = document.getElementById("mock-source-badge");
  const stageBtn = document.getElementById("mock-btn-stage");
  const hunkCard = document.getElementById("mock-hunk-card");

  if (file.staged) {
    badge.className = "source-badge staged";
    badge.textContent = "staged";
    stageBtn.textContent = "Unstage";
    stageBtn.classList.remove("btn-accept");
    stageBtn.classList.add("btn-reject");
    hunkCard.classList.add("staged");
  } else {
    badge.className = "source-badge unstaged";
    badge.textContent = "unstaged";
    stageBtn.textContent = "Stage";
    stageBtn.classList.add("btn-accept");
    stageBtn.classList.remove("btn-reject");
    hunkCard.classList.remove("staged");
  }

  // Render Rows
  const tableEl = document.getElementById("mock-diff-table");
  if (!tableEl) return;

  tableEl.innerHTML = file.rows.map(r => `
    <div class="diff-row ${r.kind}">
      <span class="gutter old-num">${r.oldNum}</span>
      <span class="gutter new-num">${r.newNum}</span>
      <span class="gutter marker">${r.marker}</span>
      <span class="line-code">${escapeHtml(r.code)}</span>
    </div>
  `).join("");

  // Update active file in sidebar
  document.querySelectorAll(".sidebar-file-item").forEach(item => {
    if (item.getAttribute("data-file") === currentSelectedFile) {
      item.classList.add("active");
    } else {
      item.classList.remove("active");
    }
  });
}

function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

// 5. Toast Feedback
let toastTimer = null;
function showToast(message) {
  const toast = document.getElementById("mock-toast");
  const msgEl = document.getElementById("mock-toast-msg");
  if (!toast || !msgEl) return;

  msgEl.textContent = message;
  toast.classList.add("show");

  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.classList.remove("show");
  }, 2400);
}

// 6. Init Event Listeners
document.addEventListener("DOMContentLoaded", () => {
  // Setup Theme
  initTheme();

  // Setup OS Download CTA
  const detected = detectUserOS();
  const ctaBtn = document.getElementById("hero-download-btn");
  const ctaSub = document.getElementById("hero-download-sub");
  if (ctaBtn) {
    ctaBtn.innerHTML = `
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="7 10 12 15 17 10"></polyline>
        <line x1="12" y1="15" x2="12" y2="3"></line>
      </svg>
      <span>${detected.label}</span>
    `;
    ctaBtn.onclick = () => {
      window.open(detected.url, "_blank");
    };
  }
  if (ctaSub) {
    ctaSub.textContent = detected.sublabel;
  }

  // Dropdown Toggle
  const dropdownToggle = document.getElementById("btn-dropdown-toggle");
  const dropdownMenu = document.getElementById("download-dropdown-menu");
  if (dropdownToggle && dropdownMenu) {
    dropdownToggle.addEventListener("click", (e) => {
      e.stopPropagation();
      dropdownMenu.classList.toggle("show");
    });
    document.addEventListener("click", () => {
      dropdownMenu.classList.remove("show");
    });
  }

  // File clicks in sidebar
  document.querySelectorAll(".sidebar-file-item").forEach(item => {
    item.addEventListener("click", () => {
      currentSelectedFile = item.getAttribute("data-file");
      renderMockup();
    });
  });

  // Stage / Unstage Hunk Action
  const stageBtn = document.getElementById("mock-btn-stage");
  if (stageBtn) {
    stageBtn.addEventListener("click", () => {
      const file = MOCK_FILES[currentSelectedFile];
      if (!file) return;
      file.staged = !file.staged;
      renderMockup();
      showToast(file.staged ? "✓ Hunk staged to git index" : "↺ Hunk unstaged from git index");
    });
  }

  // Revert Action
  const rejectBtn = document.getElementById("mock-btn-reject");
  if (rejectBtn) {
    rejectBtn.addEventListener("click", () => {
      showToast("⤺ Hunk reverted in working directory");
    });
  }

  // Refresh Action
  const refreshBtn = document.getElementById("mock-btn-refresh");
  if (refreshBtn) {
    refreshBtn.addEventListener("click", () => {
      showToast("⚡ Watcher re-synced with git repo (3ms)");
    });
  }

  // Initial render
  renderMockup();
});
