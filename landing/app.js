// Tengga Landing Page Logic

// 1. OS Detection for Smart Download CTA
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

// 3. Init Event Listeners
document.addEventListener("DOMContentLoaded", () => {
  // Setup Theme
  initTheme();

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
});
