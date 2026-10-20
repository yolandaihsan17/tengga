/**
 * Sorts Git branches by relevance based on search query and common base branches.
 * Prioritizes release, main, dev branches when no query is typed.
 */
export function sortBranchesWithRelevance(branches, query, current) {
  const q = (query || "").toLowerCase().trim();
  const commonBases = [
    "release",
    "origin/release",
    "main",
    "origin/main",
    "master",
    "origin/master",
    "dev",
    "origin/dev",
  ];

  const candidates = branches.filter((b) => b !== current && (!q || b.toLowerCase().includes(q)));

  return candidates.sort((a, b) => {
    const aLower = a.toLowerCase();
    const bLower = b.toLowerCase();

    if (q) {
      // 1. Exact match with query or origin/query
      const aExact = aLower === q || aLower === `origin/${q}`;
      const bExact = bLower === q || bLower === `origin/${q}`;
      if (aExact && !bExact) return -1;
      if (!aExact && bExact) return 1;

      // 2. Starts with query or origin/query
      const aStarts = aLower.startsWith(q) || aLower.startsWith(`origin/${q}`);
      const bStarts = bLower.startsWith(q) || bLower.startsWith(`origin/${q}`);
      if (aStarts && !bStarts) return -1;
      if (!aStarts && bStarts) return 1;

      // 3. Ends with /query (e.g. branch ending with /release)
      const aEnds = aLower.endsWith(`/${q}`);
      const bEnds = bLower.endsWith(`/${q}`);
      if (aEnds && !bEnds) return -1;
      if (!aEnds && bEnds) return 1;
    } else {
      // If no query, prioritize common base branches
      const aCommon = commonBases.indexOf(aLower);
      const bCommon = commonBases.indexOf(bLower);
      if (aCommon !== -1 && bCommon !== -1) return aCommon - bCommon;
      if (aCommon !== -1) return -1;
      if (bCommon !== -1) return 1;
    }

    return aLower.localeCompare(bLower);
  });
}
