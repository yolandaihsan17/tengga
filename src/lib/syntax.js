/**
 * Ultra-lightweight syntax highlighter for Tengga powered by PrismJS.
 * Memory footprint: ~1 MB RAM (vs Monaco's 100+ MB).
 * Color theme: Authentic VS Code Dark+
 */
import Prism from "prismjs";

// Ensure globalThis.Prism exists for components that look for it
if (typeof globalThis !== "undefined") {
  globalThis.Prism = Prism;
}

// Common language grammars
import "prismjs/components/prism-javascript.js";
import "prismjs/components/prism-typescript.js";
import "prismjs/components/prism-jsx.js";
import "prismjs/components/prism-tsx.js";
import "prismjs/components/prism-css.js";
import "prismjs/components/prism-json.js";
import "prismjs/components/prism-rust.js";
import "prismjs/components/prism-python.js";
import "prismjs/components/prism-bash.js";
import "prismjs/components/prism-yaml.js";
import "prismjs/components/prism-markdown.js";
import "prismjs/components/prism-sql.js";

function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

export function getLanguage(filePath) {
  if (!filePath) return null;
  const ext = filePath.split(".").pop().toLowerCase();

  // Markup languages (handles HTML, XML, SVG, Vue SFC, Svelte with embedded script & style)
  if (["vue", "svelte", "html", "htm", "xml", "svg"].includes(ext)) return "markup";
  if (["js", "mjs", "cjs"].includes(ext)) return "javascript";
  if (["jsx"].includes(ext)) return "jsx";
  if (["ts"].includes(ext)) return "typescript";
  if (["tsx"].includes(ext)) return "tsx";
  if (["css", "scss", "less"].includes(ext)) return "css";
  if (["rs"].includes(ext)) return "rust";
  if (["py"].includes(ext)) return "python";
  if (["json"].includes(ext)) return "json";
  if (["sh", "bash", "zsh"].includes(ext)) return "bash";
  if (["yml", "yaml"].includes(ext)) return "yaml";
  if (["md", "markdown"].includes(ext)) return "markdown";
  if (["sql"].includes(ext)) return "sql";

  return null;
}

const ABSOLUTE_MAX_HIGHLIGHT_LINES = 1800;

/**
 * Tokenize code into syntax-highlighted HTML spans according to VS Code Dark+ theme.
 * Supports progressive chunking for large files (maxLines), with a hard safety limit
 * of 1,800 lines to protect WebKit DOM memory from ballooning.
 */
export function highlightCode(code, filePath = "", maxLines = null) {
  if (!code) return "";
  const lang = getLanguage(filePath);
  if (!lang || !Prism.languages[lang]) {
    let fallback = escapeHtml(code);
    if (code.endsWith("\n")) {
      fallback += " ";
    }
    return fallback;
  }

  const effectiveMaxLines = maxLines
    ? Math.min(maxLines, ABSOLUTE_MAX_HIGHLIGHT_LINES)
    : ABSOLUTE_MAX_HIGHLIGHT_LINES;

  const lines = code.split("\n");
  if (lines.length > effectiveMaxLines) {
    const head = lines.slice(0, effectiveMaxLines).join("\n");
    const tail = lines.slice(effectiveMaxLines).join("\n");
    try {
      let headHtml = Prism.highlight(head, Prism.languages[lang], lang);
      let tailHtml = escapeHtml(tail);
      let res = headHtml + "\n" + tailHtml;
      if (code.endsWith("\n")) {
        res += " ";
      }
      return res;
    } catch (e) {
      // Fallback
    }
  }

  try {
    let html = Prism.highlight(code, Prism.languages[lang], lang);
    if (code.endsWith("\n")) {
      html += " ";
    }
    return html;
  } catch (e) {
    console.warn("Syntax highlight fallback for:", filePath, e);
  }

  let fallback = escapeHtml(code);
  if (code.endsWith("\n")) {
    fallback += " ";
  }
  return fallback;
}
