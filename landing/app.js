// Tengga Landing Page Logic - Slack-style Layout

// 1. OS Detection for Smart Download CTA
function detectUserOS() {
  const ua = navigator.userAgent || "";
  const platform = navigator.platform || "";

  if (/Mac|iPhone|iPad|iPod/i.test(platform) || /Mac/i.test(ua)) {
    return {
      osName: "macOS",
      label: "Download for macOS",
      sublabel: "Apple Silicon & Intel • Universal .dmg",
      url: "https://github.com/yolandaihsan17/tengga/releases/download/v0.1.2/Tengga_0.1.2_universal.dmg",
      download: "Tengga_0.1.2_universal.dmg"
    };
  } else if (/Win/i.test(platform) || /Windows/i.test(ua)) {
    return {
      osName: "Windows",
      label: "Download for Windows",
      sublabel: "64-bit • .exe Installer",
      url: "https://github.com/yolandaihsan17/tengga/releases/download/v0.1.2/Tengga_0.1.2_x64-setup.exe",
      download: "Tengga_0.1.2_x64-setup.exe"
    };
  } else if (/Linux/i.test(platform) || /Linux/i.test(ua)) {
    return {
      osName: "Linux",
      label: "Download for Linux",
      sublabel: "Debian, Ubuntu • .deb Package",
      url: "https://github.com/yolandaihsan17/tengga/releases/download/v0.1.2/Tengga_0.1.2_amd64.deb",
      download: "Tengga_0.1.2_amd64.deb"
    };
  }

  return {
    osName: "All Platforms",
    label: "Download for macOS",
    sublabel: "Apple Silicon & Intel • Universal .dmg",
    url: "https://github.com/yolandaihsan17/tengga/releases/download/v0.1.2/Tengga_0.1.2_universal.dmg",
    download: "Tengga_0.1.2_universal.dmg"
  };
}

// 2. Theme Toggle Support (Dark, Light, Glass)
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

// 3. Interactive Hero Preview Card Toggle
function initHeroPreviewInteractive() {
  const stageBtn = document.getElementById("hero-preview-stage-btn");
  const badge = document.getElementById("hero-preview-badge");
  const card = document.getElementById("hero-preview-card");

  if (!stageBtn || !badge) return;

  let isStaged = false;
  stageBtn.addEventListener("click", () => {
    isStaged = !isStaged;
    if (isStaged) {
      badge.textContent = "STAGED";
      badge.classList.remove("unstaged");
      badge.classList.add("staged");
      stageBtn.textContent = "Unstage";
      stageBtn.classList.remove("btn-accept");
      stageBtn.classList.add("btn-reject");
      if (card) card.classList.add("is-staged");
    } else {
      badge.textContent = "UNSTAGED";
      badge.classList.remove("staged");
      badge.classList.add("unstaged");
      stageBtn.textContent = "Stage";
      stageBtn.classList.add("btn-accept");
      stageBtn.classList.remove("btn-reject");
      if (card) card.classList.remove("is-staged");
    }
  });

  const revertBtn = document.getElementById("hero-preview-revert-btn");
  if (revertBtn) {
    revertBtn.addEventListener("click", () => {
      const originalText = revertBtn.textContent;
      revertBtn.textContent = "Reverted!";
      setTimeout(() => {
        revertBtn.textContent = originalText;
      }, 1200);
    });
  }
}

// 4. Copy-to-clipboard for Homebrew terminal snippets
function initBrewCopy() {
  document.querySelectorAll(".btn-copy-brew").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      const codeEl = btn.closest(".hero-brew-box, .os-card")?.querySelector("code");
      const textToCopy = codeEl ? codeEl.textContent.trim() : "brew install --cask yolandaihsan17/tengga/tengga";
      
      navigator.clipboard.writeText(textToCopy).then(() => {
        btn.classList.add("copied");
        setTimeout(() => {
          btn.classList.remove("copied");
        }, 2000);
      });
    });
  });
}

// 5. Init Event Listeners
document.addEventListener("DOMContentLoaded", () => {
  initTheme();
  initHeroPreviewInteractive();
  initBrewCopy();

  // Setup OS-specific Hero Actions
  const detected = detectUserOS();
  const macFlow = document.getElementById("hero-mac-flow");
  const stdFlow = document.getElementById("hero-standard-flow");
  const ctaBtn = document.getElementById("hero-download-btn");
  const ctaSub = document.getElementById("hero-download-sub");

  if (detected.osName === "Windows" || detected.osName === "Linux") {
    if (macFlow) macFlow.style.display = "none";
    if (stdFlow) stdFlow.style.display = "block";
    if (ctaBtn) {
      ctaBtn.innerHTML = `
        <span class="material-symbols-rounded">download</span>
        <span>${detected.label}</span>
      `;
      ctaBtn.onclick = () => {
        window.open(detected.url, "_blank");
      };
    }
    if (ctaSub) {
      ctaSub.textContent = detected.sublabel;
    }
  } else {
    // macOS (default)
    if (macFlow) macFlow.style.display = "flex";
    if (stdFlow) stdFlow.style.display = "none";
  }

  // Setup dropdown menus
  document.querySelectorAll("[id^='btn-dropdown-toggle']").forEach((toggleBtn) => {
    toggleBtn.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      const parent = toggleBtn.closest(".hero-other-platforms, .download-split-wrapper");
      const menu = parent ? parent.querySelector(".download-dropdown-menu") : null;
      if (menu) {
        menu.classList.toggle("show");
      }
    });
  });

  // Close open dropdowns on click outside
  document.addEventListener("click", () => {
    document.querySelectorAll(".download-dropdown-menu.show").forEach((m) => m.classList.remove("show"));
  });
});
