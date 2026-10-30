use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub kind: String, // "add" | "remove" | "context"
    pub content: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    pub id: String,
    pub header: String,
    pub lines: Vec<Line>,
    pub source: String, // "unstaged" | "staged" | "branch_compare"
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct FileDiff {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<Hunk>,
    pub is_binary: bool,
    pub has_staged: bool,
    pub has_unstaged: bool,
    pub is_untracked: bool,
    #[serde(default)]
    pub is_truncated: bool,
    #[serde(default)]
    pub hunks_truncated: bool,
}

// server-side only: enough info to rebuild a patch for a single hunk
pub struct HunkPatch {
    pub file_header: String, // "diff --git ..." through "+++ b/..."
    pub hunk_text: String,   // "@@ ... @@" line through its content lines
    pub source: String,
}

pub const MAX_FILES_WITH_FULL_HUNKS: usize = 200;
pub const MAX_HUNKS_PER_FILE: usize = 35;
pub const MAX_LINES_PER_HUNK: usize = 600;
pub const MAX_LOCKFILE_LINES_PER_HUNK: usize = 120;

pub fn is_lockfile_or_generated(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with("package-lock.json")
        || lower.ends_with("yarn.lock")
        || lower.ends_with("pnpm-lock.yaml")
        || lower.ends_with("cargo.lock")
        || lower.ends_with("composer.lock")
        || lower.ends_with("poetry.lock")
        || lower.ends_with("gemfile.lock")
        || lower.ends_with(".min.js")
        || lower.ends_with(".min.css")
        || lower.ends_with(".map")
        || lower.ends_with(".bundle.js")
}

/// Parses `git diff` output for one source ("unstaged" or "staged" or "branch_compare") into
/// FileDiffs, capping hunks on very large diffs to avoid IPC and WebKit DOM explosion.
pub fn parse_diff(raw: &str, source: &str) -> (Vec<FileDiff>, Vec<(String, HunkPatch)>) {
    parse_diff_internal(raw, source, Some(MAX_FILES_WITH_FULL_HUNKS))
}

/// Parse diff for a single file on-demand with no file-count limit.
pub fn parse_single_file_diff(raw: &str, source: &str) -> (Vec<FileDiff>, Vec<(String, HunkPatch)>) {
    parse_diff_internal(raw, source, None)
}

fn parse_diff_internal(
    raw: &str,
    source: &str,
    max_files_with_hunks: Option<usize>,
) -> (Vec<FileDiff>, Vec<(String, HunkPatch)>) {
    let mut files = Vec::new();
    let mut patches = Vec::new();

    for file_block in split_file_blocks(raw) {
        let path = match extract_path(file_block) {
            Some(p) => p,
            None => continue,
        };

        let is_binary = file_block.lines().any(|l| l.starts_with("Binary files "));
        if is_binary {
            files.push(FileDiff {
                path,
                additions: 0,
                deletions: 0,
                hunks: Vec::new(),
                is_binary: true,
                has_staged: source == "staged",
                has_unstaged: source == "unstaged",
                is_untracked: false,
                is_truncated: false,
                hunks_truncated: false,
            });
            continue;
        }

        let is_lockfile = is_lockfile_or_generated(&path);
        let line_limit = if is_lockfile {
            MAX_LOCKFILE_LINES_PER_HUNK
        } else {
            MAX_LINES_PER_HUNK
        };

        let should_defer_hunks = match max_files_with_hunks {
            Some(limit) => files.len() >= limit,
            None => false,
        };

        let mut additions = 0u32;
        let mut deletions = 0u32;

        if should_defer_hunks {
            // Count additions/deletions directly without allocating Hunk or Line structs
            for line in file_block.lines() {
                if line.starts_with('+') && !line.starts_with("+++") {
                    additions += 1;
                } else if line.starts_with('-') && !line.starts_with("---") {
                    deletions += 1;
                }
            }

            files.push(FileDiff {
                path,
                additions,
                deletions,
                hunks: Vec::new(),
                is_binary: false,
                has_staged: source == "staged",
                has_unstaged: source == "unstaged",
                is_untracked: false,
                is_truncated: false,
                hunks_truncated: true,
            });
            continue;
        }

        let file_header = extract_header(file_block);
        let mut hunks = Vec::new();
        let mut file_is_truncated = false;

        let hunks_list = split_hunks(file_block);
        for (hunk_idx, hunk_text) in hunks_list.into_iter().enumerate() {
            let header = hunk_text.lines().next().unwrap_or("").to_string();
            let id = format!("{}:{}:{}:{}", path, source, hunk_idx, header);

            if hunk_idx < MAX_HUNKS_PER_FILE {
                let (lines, lines_truncated) =
                    parse_hunk_lines(hunk_text, &mut additions, &mut deletions, line_limit);
                if lines_truncated {
                    file_is_truncated = true;
                }

                hunks.push(Hunk {
                    id: id.clone(),
                    header,
                    lines,
                    source: source.to_string(),
                });

                patches.push((
                    id,
                    HunkPatch {
                        file_header: file_header.clone(),
                        hunk_text: hunk_text.to_string(),
                        source: source.to_string(),
                    },
                ));
            } else {
                // If past MAX_HUNKS_PER_FILE, still count additions and deletions
                file_is_truncated = true;
                for line in hunk_text.lines().skip(1) {
                    if line.starts_with('+') {
                        additions += 1;
                    } else if line.starts_with('-') {
                        deletions += 1;
                    }
                }
            }
        }

        if !hunks.is_empty() {
            files.push(FileDiff {
                path,
                additions,
                deletions,
                hunks,
                is_binary: false,
                has_staged: source == "staged",
                has_unstaged: source == "unstaged",
                is_untracked: false,
                is_truncated: file_is_truncated,
                hunks_truncated: false,
            });
        }
    }

    (files, patches)
}

fn split_file_blocks(raw: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let marker = "\ndiff --git";
    let mut start_idx = 0;

    for (i, _) in raw.match_indices(marker) {
        let block_end = i + 1; // keep newline with preceding block
        let block = &raw[start_idx..block_end];
        if !block.trim().is_empty() {
            blocks.push(block);
        }
        start_idx = block_end;
    }

    if start_idx < raw.len() {
        let rest = &raw[start_idx..];
        if !rest.trim().is_empty() {
            blocks.push(rest);
        }
    }

    blocks
}

fn extract_path(block: &str) -> Option<String> {
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            return Some(rest.to_string());
        }
    }
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("--- a/") {
            return Some(rest.to_string());
        }
    }
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("Binary files ") {
            if let Some(idx) = rest.find(" and b/") {
                let b_part = &rest[idx + 7..];
                if let Some(end) = b_part.strip_suffix(" differ") {
                    return Some(end.to_string());
                }
            } else if let Some(idx) = rest.find("a/") {
                let a_part = &rest[idx + 2..];
                if let Some(end) = a_part.find(" and ") {
                    return Some(a_part[..end].to_string());
                }
            }
        }
    }
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            if let Some(idx) = rest.rfind(" b/") {
                return Some(rest[idx + 3..].to_string());
            }
        }
    }
    None
}

fn extract_header(block: &str) -> String {
    let mut header = String::new();
    for line in block.lines() {
        if line.starts_with("@@") {
            break;
        }
        header.push_str(line);
        header.push('\n');
    }
    header
}

fn split_hunks(block: &str) -> Vec<&str> {
    let mut hunks = Vec::new();
    let mut start_idx: Option<usize> = None;
    let mut line_start = 0;

    for line in block.lines() {
        if line.starts_with("@@") {
            if let Some(s) = start_idx {
                let hunk = &block[s..line_start];
                if !hunk.trim().is_empty() {
                    hunks.push(hunk);
                }
            }
            start_idx = Some(line_start);
        }
        line_start = match block[line_start..].find('\n') {
            Some(nl_idx) => line_start + nl_idx + 1,
            None => block.len(),
        };
    }

    if let Some(s) = start_idx {
        if s < block.len() {
            let hunk = &block[s..];
            if !hunk.trim().is_empty() {
                hunks.push(hunk);
            }
        }
    }

    hunks
}

fn parse_hunk_lines(
    hunk_text: &str,
    additions: &mut u32,
    deletions: &mut u32,
    max_lines: usize,
) -> (Vec<Line>, bool) {
    let mut lines = Vec::new();
    let mut total_lines = 0;
    let mut truncated = false;

    for line in hunk_text.lines().skip(1) {
        let (kind, content) = if let Some(rest) = line.strip_prefix('+') {
            *additions += 1;
            ("add", rest)
        } else if let Some(rest) = line.strip_prefix('-') {
            *deletions += 1;
            ("remove", rest)
        } else {
            ("context", line.strip_prefix(' ').unwrap_or(line))
        };

        total_lines += 1;
        if lines.len() < max_lines {
            lines.push(Line {
                kind: kind.to_string(),
                content: content.to_string(),
            });
        } else {
            truncated = true;
        }
    }

    if truncated {
        let omitted = total_lines - lines.len();
        lines.push(Line {
            kind: "context".to_string(),
            content: format!("... [Diff truncated: {} lines hidden for performance] ...", omitted),
        });
    }

    (lines, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_file_single_hunk() {
        let diff = "\
diff --git a/foo.txt b/foo.txt
index e69de29..499d63f 100644
--- a/foo.txt
+++ b/foo.txt
@@ -1,3 +1,4 @@
 line1
-old_line2
+new_line2
+line2_b
 line3
";
        let (files, patches) = parse_diff(diff, "unstaged");
        assert_eq!(files.len(), 1);
        assert_eq!(patches.len(), 1);

        let file = &files[0];
        assert_eq!(file.path, "foo.txt");
        assert_eq!(file.additions, 2);
        assert_eq!(file.deletions, 1);
        assert!(!file.is_binary);
        assert_eq!(file.hunks.len(), 1);

        let hunk = &file.hunks[0];
        assert_eq!(hunk.header, "@@ -1,3 +1,4 @@");
        assert_eq!(hunk.source, "unstaged");
        assert_eq!(hunk.lines.len(), 5);
        assert_eq!(hunk.lines[0], Line { kind: "context".into(), content: "line1".into() });
        assert_eq!(hunk.lines[1], Line { kind: "remove".into(), content: "old_line2".into() });
        assert_eq!(hunk.lines[2], Line { kind: "add".into(), content: "new_line2".into() });
        assert_eq!(hunk.lines[3], Line { kind: "add".into(), content: "line2_b".into() });
        assert_eq!(hunk.lines[4], Line { kind: "context".into(), content: "line3".into() });

        assert_eq!(patches[0].0, hunk.id);
        assert_eq!(patches[0].1.source, "unstaged");
        assert!(patches[0].1.file_header.contains("--- a/foo.txt\n+++ b/foo.txt\n"));
    }

    #[test]
    fn test_single_file_multiple_hunks() {
        let diff = "\
diff --git a/src/app.js b/src/app.js
index 1111111..2222222 100644
--- a/src/app.js
+++ b/src/app.js
@@ -1,4 +1,4 @@
-const a = 1;
+const a = 2;
 const b = 2;
 const c = 3;
 const d = 4;
@@ -10,4 +10,4 @@
 const w = 7;
 const x = 8;
-const y = 9;
+const y = 10;
 const z = 11;
";
        let (files, patches) = parse_diff(diff, "staged");
        assert_eq!(files.len(), 1);
        assert_eq!(patches.len(), 2);

        let file = &files[0];
        assert_eq!(file.path, "src/app.js");
        assert_eq!(file.additions, 2);
        assert_eq!(file.deletions, 2);
        assert!(!file.is_binary);
        assert_eq!(file.hunks.len(), 2);

        let hunk1 = &file.hunks[0];
        assert_eq!(hunk1.header, "@@ -1,4 +1,4 @@");
        assert_eq!(hunk1.source, "staged");
        assert_eq!(hunk1.lines.len(), 5);
        assert_eq!(hunk1.lines[0], Line { kind: "remove".into(), content: "const a = 1;".into() });
        assert_eq!(hunk1.lines[1], Line { kind: "add".into(), content: "const a = 2;".into() });
        assert_eq!(hunk1.lines[2], Line { kind: "context".into(), content: "const b = 2;".into() });
        assert_eq!(hunk1.lines[3], Line { kind: "context".into(), content: "const c = 3;".into() });
        assert_eq!(hunk1.lines[4], Line { kind: "context".into(), content: "const d = 4;".into() });

        let hunk2 = &file.hunks[1];
        assert_eq!(hunk2.header, "@@ -10,4 +10,4 @@");
        assert_eq!(hunk2.source, "staged");
        assert_eq!(hunk2.lines.len(), 5);
        assert_eq!(hunk2.lines[0], Line { kind: "context".into(), content: "const w = 7;".into() });
        assert_eq!(hunk2.lines[1], Line { kind: "context".into(), content: "const x = 8;".into() });
        assert_eq!(hunk2.lines[2], Line { kind: "remove".into(), content: "const y = 9;".into() });
        assert_eq!(hunk2.lines[3], Line { kind: "add".into(), content: "const y = 10;".into() });
        assert_eq!(hunk2.lines[4], Line { kind: "context".into(), content: "const z = 11;".into() });
    }

    #[test]
    fn test_multiple_files_in_one_diff() {
        let diff = "\
diff --git a/file1.txt b/file1.txt
index 1111111..2222222 100644
--- a/file1.txt
+++ b/file1.txt
@@ -1,2 +1,2 @@
-hello
+world
 context
diff --git a/file2.txt b/file2.txt
index 3333333..4444444 100644
--- a/file2.txt
+++ b/file2.txt
@@ -1,2 +1,3 @@
 start
+inserted
 end
";
        let (files, patches) = parse_diff(diff, "unstaged");
        assert_eq!(files.len(), 2);
        assert_eq!(patches.len(), 2);

        assert_eq!(files[0].path, "file1.txt");
        assert_eq!(files[0].additions, 1);
        assert_eq!(files[0].deletions, 1);
        assert!(!files[0].is_binary);
        assert_eq!(files[0].hunks.len(), 1);
        assert_eq!(files[0].hunks[0].lines.len(), 3);
        assert_eq!(files[0].hunks[0].lines[0], Line { kind: "remove".into(), content: "hello".into() });
        assert_eq!(files[0].hunks[0].lines[1], Line { kind: "add".into(), content: "world".into() });
        assert_eq!(files[0].hunks[0].lines[2], Line { kind: "context".into(), content: "context".into() });

        assert_eq!(files[1].path, "file2.txt");
        assert_eq!(files[1].additions, 1);
        assert_eq!(files[1].deletions, 0);
        assert!(!files[1].is_binary);
        assert_eq!(files[1].hunks.len(), 1);
        assert_eq!(files[1].hunks[0].lines.len(), 3);
        assert_eq!(files[1].hunks[0].lines[0], Line { kind: "context".into(), content: "start".into() });
        assert_eq!(files[1].hunks[0].lines[1], Line { kind: "add".into(), content: "inserted".into() });
        assert_eq!(files[1].hunks[0].lines[2], Line { kind: "context".into(), content: "end".into() });
    }

    #[test]
    fn test_hunk_at_start_of_file() {
        let diff = "\
diff --git a/start.txt b/start.txt
index 1111111..2222222 100644
--- a/start.txt
+++ b/start.txt
@@ -1,2 +1,3 @@
+first line
 line1
 line2
";
        let (files, patches) = parse_diff(diff, "unstaged");
        assert_eq!(files.len(), 1);
        assert_eq!(patches.len(), 1);

        let file = &files[0];
        assert_eq!(file.path, "start.txt");
        assert_eq!(file.additions, 1);
        assert_eq!(file.deletions, 0);
        assert_eq!(file.hunks.len(), 1);
        assert_eq!(file.hunks[0].header, "@@ -1,2 +1,3 @@");
        assert_eq!(file.hunks[0].lines.len(), 3);
        assert_eq!(file.hunks[0].lines[0], Line { kind: "add".into(), content: "first line".into() });
        assert_eq!(file.hunks[0].lines[1], Line { kind: "context".into(), content: "line1".into() });
        assert_eq!(file.hunks[0].lines[2], Line { kind: "context".into(), content: "line2".into() });
    }

    #[test]
    fn test_hunk_at_end_of_file() {
        let diff = "\
diff --git a/end.txt b/end.txt
index 1111111..2222222 100644
--- a/end.txt
+++ b/end.txt
@@ -8,3 +8,4 @@
 line8
 line9
 line10
+line11
";
        let (files, patches) = parse_diff(diff, "unstaged");
        assert_eq!(files.len(), 1);
        assert_eq!(patches.len(), 1);

        let file = &files[0];
        assert_eq!(file.path, "end.txt");
        assert_eq!(file.additions, 1);
        assert_eq!(file.deletions, 0);
        assert_eq!(file.hunks.len(), 1);
        assert_eq!(file.hunks[0].header, "@@ -8,3 +8,4 @@");
        assert_eq!(file.hunks[0].lines.len(), 4);
        assert_eq!(file.hunks[0].lines[0], Line { kind: "context".into(), content: "line8".into() });
        assert_eq!(file.hunks[0].lines[1], Line { kind: "context".into(), content: "line9".into() });
        assert_eq!(file.hunks[0].lines[2], Line { kind: "context".into(), content: "line10".into() });
        assert_eq!(file.hunks[0].lines[3], Line { kind: "add".into(), content: "line11".into() });
    }

    #[test]
    fn test_newly_added_file() {
        let diff = "\
diff --git a/new.txt b/new.txt
new file mode 100644
index 0000000..1111111
--- /dev/null
+++ b/new.txt
@@ -0,0 +1,2 @@
+alpha
+beta
";
        let (files, patches) = parse_diff(diff, "unstaged");
        assert_eq!(files.len(), 1);
        assert_eq!(patches.len(), 1);

        let file = &files[0];
        assert_eq!(file.path, "new.txt");
        assert_eq!(file.additions, 2);
        assert_eq!(file.deletions, 0);
        assert!(!file.is_binary);
        assert_eq!(file.hunks.len(), 1);
        assert_eq!(file.hunks[0].header, "@@ -0,0 +1,2 @@");
        assert_eq!(file.hunks[0].lines.len(), 2);
        assert_eq!(file.hunks[0].lines[0], Line { kind: "add".into(), content: "alpha".into() });
        assert_eq!(file.hunks[0].lines[1], Line { kind: "add".into(), content: "beta".into() });
    }

    #[test]
    fn test_deleted_file() {
        let diff = "\
diff --git a/deleted.txt b/deleted.txt
deleted file mode 100644
index 1111111..0000000
--- a/deleted.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-alpha
-beta
";
        let (files, patches) = parse_diff(diff, "unstaged");
        assert_eq!(files.len(), 1);
        assert_eq!(patches.len(), 1);

        let file = &files[0];
        assert_eq!(file.path, "deleted.txt");
        assert_eq!(file.additions, 0);
        assert_eq!(file.deletions, 2);
        assert!(!file.is_binary);
        assert_eq!(file.hunks.len(), 1);
        assert_eq!(file.hunks[0].header, "@@ -1,2 +0,0 @@");
        assert_eq!(file.hunks[0].lines.len(), 2);
        assert_eq!(file.hunks[0].lines[0], Line { kind: "remove".into(), content: "alpha".into() });
        assert_eq!(file.hunks[0].lines[1], Line { kind: "remove".into(), content: "beta".into() });
    }

    #[test]
    fn test_binary_file() {
        let diff = "\
diff --git a/images/logo.png b/images/logo.png
index eaf36c1..f75c32a 100644
Binary files a/images/logo.png and b/images/logo.png differ
";
        let (files, patches) = parse_diff(diff, "unstaged");
        assert_eq!(files.len(), 1);
        assert_eq!(patches.len(), 0);

        let file = &files[0];
        assert_eq!(file.path, "images/logo.png");
        assert_eq!(file.additions, 0);
        assert_eq!(file.deletions, 0);
        assert!(file.is_binary);
        assert_eq!(file.hunks.len(), 0);
    }

    #[test]
    fn test_lockfile_truncation() {
        let mut diff = String::from("diff --git a/package-lock.json b/package-lock.json\nindex 1111..2222 100644\n--- a/package-lock.json\n+++ b/package-lock.json\n@@ -1,500 +1,500 @@\n");
        for i in 0..300 {
            diff.push_str(&format!("+  \"dep_{}\": \"1.0.0\",\n", i));
        }

        let (files, _) = parse_diff(&diff, "unstaged");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "package-lock.json");
        assert_eq!(files[0].additions, 300); // exact count preserved!
        assert!(files[0].is_truncated);
        // Hunk should have MAX_LOCKFILE_LINES_PER_HUNK (120) + 1 truncated info line = 121 lines
        assert_eq!(files[0].hunks[0].lines.len(), 121);
        assert!(files[0].hunks[0].lines.last().unwrap().content.contains("Diff truncated"));
    }

    #[test]
    fn test_deferred_hunks_on_massive_files() {
        let mut diff = String::new();
        // Generate diff with 205 files
        for f in 0..205 {
            diff.push_str(&format!(
                "diff --git a/file_{}.txt b/file_{}.txt\nindex 11..22 100644\n--- a/file_{}.txt\n+++ b/file_{}.txt\n@@ -1,2 +1,2 @@\n-old\n+new\n",
                f, f, f, f
            ));
        }

        let (files, _) = parse_diff(&diff, "unstaged");
        assert_eq!(files.len(), 205);
        // First 200 files should have hunks
        assert_eq!(files[0].hunks.len(), 1);
        assert!(!files[0].hunks_truncated);
        assert_eq!(files[199].hunks.len(), 1);
        assert!(!files[199].hunks_truncated);

        // Files beyond 200 should have empty hunks and hunks_truncated = true
        assert_eq!(files[200].hunks.len(), 0);
        assert!(files[200].hunks_truncated);
        assert_eq!(files[200].additions, 1);
        assert_eq!(files[200].deletions, 1);

        assert_eq!(files[204].hunks.len(), 0);
        assert!(files[204].hunks_truncated);
    }
}
