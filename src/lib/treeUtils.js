/**
 * Tree construction and path manipulation utilities for the file explorer sidebar.
 */

export function escapeHtml(text) {
  return (text || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

export function highlightSnippet(content, query, caseSens, isRegex) {
  if (!content) return "";
  const trimmed = content.trimEnd();
  if (!query || !query.trim()) return escapeHtml(trimmed);

  try {
    let regex;
    if (isRegex) {
      regex = new RegExp(`(${query})`, caseSens ? "g" : "gi");
    } else {
      const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      regex = new RegExp(`(${escaped})`, caseSens ? "g" : "gi");
    }
    const parts = trimmed.split(regex);
    return parts
      .map((part) => {
        if (!part) return "";
        const isMatch = caseSens ? part === query : part.toLowerCase() === query.toLowerCase();
        if (isMatch) {
          return `<mark class="search-snippet-match">${escapeHtml(part)}</mark>`;
        }
        return escapeHtml(part);
      })
      .join("");
  } catch {
    return escapeHtml(trimmed);
  }
}

export function computeAllFolderPaths(fileList) {
  const res = {};
  for (const f of fileList) {
    const p = typeof f === "string" ? f : f.path;
    if (!p) continue;
    const parts = p.split("/");
    let acc = "";
    for (let i = 0; i < parts.length - 1; i++) {
      acc = acc ? `${acc}/${parts[i]}` : parts[i];
      res[acc] = true;
    }
  }
  return res;
}

export function getAllFolderPaths(nodes) {
  let paths = [];
  for (const n of nodes) {
    if (n.type === "folder") {
      paths.push(n.path);
      if (n.childrenList) {
        paths = paths.concat(getAllFolderPaths(n.childrenList));
      }
    }
  }
  return paths;
}

export function buildTree(paths, diffs, dirtySet) {
  const root = {
    name: "",
    type: "folder",
    children: {},
    path: "",
    additions: 0,
    deletions: 0,
    hasChanges: false,
  };

  for (const path of paths) {
    const parts = path.split("/");
    let current = root;
    let accumulatedPath = "";

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${part}` : part;
      const isFile = i === parts.length - 1;

      if (isFile) {
        const diffInfo = diffs.get(path);
        const isDirty = dirtySet ? dirtySet.has(path) : false;
        const hasChanges = Boolean(diffInfo) || isDirty;
        current.children[part] = {
          name: part,
          type: "file",
          path,
          hasChanges,
          isDirty,
          isUntracked: diffInfo ? Boolean(diffInfo.is_untracked) : false,
          additions: diffInfo ? diffInfo.additions : 0,
          deletions: diffInfo ? diffInfo.deletions : 0,
          is_binary: diffInfo ? diffInfo.is_binary : false,
        };
      } else {
        if (!current.children[part]) {
          current.children[part] = {
            name: part,
            type: "folder",
            path: accumulatedPath,
            children: {},
            additions: 0,
            deletions: 0,
            hasChanges: false,
          };
        }
        current = current.children[part];
      }
    }
  }

  function sumAndSort(node) {
    if (node.type === "file") return;
    let adds = 0;
    let dels = 0;
    let changed = false;

    const sorted = Object.values(node.children).sort((a, b) => {
      if (a.type !== b.type) return a.type === "folder" ? -1 : 1;
      return a.name.localeCompare(b.name);
    });

    for (const child of sorted) {
      sumAndSort(child);
      adds += child.additions || 0;
      dels += child.deletions || 0;
      if (child.hasChanges) changed = true;
    }

    node.additions = adds;
    node.deletions = dels;
    node.hasChanges = changed;
    node.childrenList = sorted;
  }

  sumAndSort(root);
  return root.childrenList || [];
}
