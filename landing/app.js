// Tengga Landing Page Logic - Slack-style Layout

// 1. OS Detection for Smart Download CTA
function detectUserOS() {
  const ua = navigator.userAgent || "";
  const platform = navigator.platform || "";

  if (/Mac|iPhone|iPad|iPod/i.test(platform) || /Mac/i.test(ua)) {
    return {
      osName: "macOS",
      label: "Download for macOS",
      sublabel: "Apple Silicon & Intel • .dmg (5.8 MB)",
      url: "./downloads/Tengga.dmg?v=0.1.0",
      download: "Tengga.dmg"
    };
  } else if (/Win/i.test(platform) || /Windows/i.test(ua)) {
    return {
      osName: "Windows",
      label: "Download for Windows",
      sublabel: "64-bit • Coming Soon",
      url: "https://github.com/yolandaihsan17/tengga"
    };
  } else if (/Linux/i.test(platform) || /Linux/i.test(ua)) {
    return {
      osName: "Linux",
      label: "Download for Linux",
      sublabel: "Debian, Ubuntu • Coming Soon",
      url: "https://github.com/yolandaihsan17/tengga"
    };
  }

  return {
    osName: "All Platforms",
    label: "Download Tengga for macOS",
    sublabel: "macOS .dmg (5.8 MB)",
    url: "./downloads/Tengga.dmg?v=0.1.0",
    download: "Tengga.dmg"
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

// 4. Init Event Listeners
document.addEventListener("DOMContentLoaded", () => {
  initTheme();
  initHeroPreviewInteractive();

  // Setup OS Download CTA
  const detected = detectUserOS();
  const ctaBtn = document.getElementById("hero-download-btn");
  const ctaSub = document.getElementById("hero-download-sub");

  if (ctaBtn) {
    ctaBtn.innerHTML = `
      <span class="material-symbols-rounded">download</span>
      <span>${detected.label}</span>
    `;
    ctaBtn.onclick = () => {
      if (detected.download) {
        const a = document.createElement("a");
        a.href = detected.url;
        a.download = detected.download;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
      } else {
        window.open(detected.url, "_blank");
      }
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
});
