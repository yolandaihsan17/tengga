use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub name: String,
    pub branch: String,
    pub head: String,
    pub is_main: bool,
    pub is_current: bool,
}

pub fn git_common_dir(repo: &Path) -> Result<PathBuf, String> {
    let output = run_git(repo, &["rev-parse", "--git-common-dir"])?;
    let path_str = output.trim();
    let p = Path::new(path_str);
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(repo.join(p))
    }
}

pub fn list_worktrees(repo: &Path) -> Result<Vec<WorktreeInfo>, String> {
    let output = run_git(repo, &["worktree", "list", "--porcelain"])?;
    let canon_current = std::fs::canonicalize(repo).unwrap_or_else(|_| repo.to_path_buf());

    let mut worktrees = Vec::new();
    let mut cur_path: Option<String> = None;
    let mut cur_head: Option<String> = None;
    let mut cur_branch: Option<String> = None;
    let mut is_first = true;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            if let Some(path) = cur_path.take() {
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();
                let canon_wt = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
                let is_current = canon_wt == canon_current;
                worktrees.push(WorktreeInfo {
                    path,
                    name,
                    branch: cur_branch.take().unwrap_or_else(|| "detached".to_string()),
                    head: cur_head.take().unwrap_or_default(),
                    is_main: is_first,
                    is_current,
                });
                is_first = false;
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("worktree ") {
            cur_path = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("HEAD ") {
            cur_head = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            let b = rest.trim();
            let short = b.strip_prefix("refs/heads/").unwrap_or(b);
            cur_branch = Some(short.to_string());
        } else if line == "detached" {
            cur_branch = Some("HEAD (detached)".to_string());
        } else if line == "bare" {
            cur_branch = Some("(bare)".to_string());
        }
    }

    if let Some(path) = cur_path.take() {
        let name = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();
        let canon_wt = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
        let is_current = canon_wt == canon_current;
        worktrees.push(WorktreeInfo {
            path,
            name,
            branch: cur_branch.take().unwrap_or_else(|| "detached".to_string()),
            head: cur_head.take().unwrap_or_default(),
            is_main: is_first,
            is_current,
        });
    }

    Ok(worktrees)
}


pub fn is_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn current_branch(repo: &Path) -> Result<String, String> {
    let output = run_git(repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    Ok(output.trim().to_string())
}

pub fn list_branches(repo: &Path) -> Result<Vec<String>, String> {
    let output = run_git(repo, &["branch", "-a", "--format=%(refname:short)"])?;
    let mut branches: Vec<String> = output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.contains("HEAD") && s != "origin")
        .collect();
    branches.sort();
    branches.dedup();
    Ok(branches)
}

pub fn switch_branch(repo: &Path, branch: &str) -> Result<(), String> {
    if branch.starts_with('-') || branch.is_empty() {
        return Err("Invalid branch name".to_string());
    }
    let output = Command::new("git")
        .args(["checkout", branch])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err.is_empty() {
            Err(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(err)
        }
    }
}

pub fn diff_unstaged(repo: &Path) -> Result<String, String> {
    run_git(repo, &["diff"])
}

pub fn diff_staged(repo: &Path) -> Result<String, String> {
    run_git(repo, &["diff", "--staged"])
}

pub fn diff_against_base(repo: &Path, base: &str) -> Result<String, String> {
    if base.starts_with('-') || base.is_empty() {
        return Err("Invalid branch name".to_string());
    }
    let arg = format!("{}...", base);
    match run_git(repo, &["diff", &arg]) {
        Ok(out) => Ok(out),
        Err(e) if e.contains("no merge base") => {
            // Fallback to two dots: direct diff between base and HEAD
            let two_dot = format!("{}..HEAD", base);
            run_git(repo, &["diff", &two_dot])
        }
        Err(e) => Err(e),
    }
}

pub fn branch_ahead_behind(repo: &Path, base: &str) -> Result<(u32, u32), String> {
    if base.starts_with('-') || base.is_empty() {
        return Err("Invalid branch name".to_string());
    }
    let arg = format!("{}...HEAD", base);
    let output = match run_git(repo, &["rev-list", "--left-right", "--count", &arg]) {
        Ok(out) => out,
        Err(_) => {
            let two_dot = format!("{}..HEAD", base);
            let ahead_out = run_git(repo, &["rev-list", "--count", &two_dot]).unwrap_or_default();
            let ahead = ahead_out.trim().parse::<u32>().unwrap_or(0);
            return Ok((ahead, 0));
        }
    };
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() == 2 {
        let behind = parts[0].parse::<u32>().unwrap_or(0);
        let ahead = parts[1].parse::<u32>().unwrap_or(0);
        Ok((ahead, behind))
    } else {
        Ok((0, 0))
    }
}

pub fn pull_branch(repo: &Path, branch: &str) -> Result<String, String> {
    if branch.starts_with('-') || branch.is_empty() {
        return Err("Invalid branch name".to_string());
    }

    let current = current_branch(repo).unwrap_or_default();

    if current == branch {
        let output = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(repo)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let msg = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if msg.is_empty() {
                format!("Pulled latest for {}", branch)
            } else {
                msg
            })
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if err.is_empty() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                err
            })
        }
    } else {
        let refspec = format!("{}:{}", branch, branch);
        let output = Command::new("git")
            .args(["fetch", "origin", &refspec])
            .current_dir(repo)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if msg.is_empty() {
                format!("Updated '{}' to latest remote", branch)
            } else {
                msg
            })
        } else {
            let fallback = Command::new("git")
                .args(["fetch", "origin", branch])
                .current_dir(repo)
                .output()
                .map_err(|e| e.to_string())?;

            if fallback.status.success() {
                Ok(format!("Fetched remote updates for '{}'", branch))
            } else {
                let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                Err(if err.is_empty() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    err
                })
            }
        }
    }
}

pub fn list_repo_files(repo: &Path) -> Result<Vec<String>, String> {
    let output = run_git(repo, &["ls-files", "--cached", "--others", "--exclude-standard"])?;
    Ok(output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Applies a single hunk's patch. `cached` stages it, `reverse` reverts it.
/// Combine both for reverting a staged hunk.
pub fn apply_patch(repo: &Path, patch: &str, cached: bool, reverse: bool) -> Result<(), String> {
    let mut args = vec!["apply", "--recount"];
    if cached {
        args.push("--cached");
    }
    if reverse {
        args.push("--reverse");
    }
    args.push("-");

    let mut child = Command::new("git")
        .args(&args)
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    child
        .stdin
        .as_mut()
        .ok_or("failed to open stdin")?
        .write_all(patch.as_bytes())
        .map_err(|e| e.to_string())?;

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn list_untracked_files(repo: &Path) -> Result<Vec<String>, String> {
    let output = run_git(repo, &["ls-files", "--others", "--exclude-standard"])?;
    Ok(output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

pub fn diff_untracked(repo: &Path, file_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff", "--no-index", "--", "/dev/null", file_path])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn stage_file(repo: &Path, file_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", "--", file_path])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn unstage_file(repo: &Path, file_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["restore", "--staged", "--", file_path])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let fallback = Command::new("git")
            .args(["reset", "HEAD", "--", file_path])
            .current_dir(repo)
            .output()
            .map_err(|e| e.to_string())?;
        if fallback.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&fallback.stderr).to_string())
        }
    }
}

pub fn discard_file(repo: &Path, file_path: &str) -> Result<(), String> {
    let full_path = repo.join(file_path);
    let is_tracked = Command::new("git")
        .args(["ls-files", "--error-unmatch", "--", file_path])
        .current_dir(repo)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_tracked {
        if full_path.is_file() {
            std::fs::remove_file(&full_path).map_err(|e| e.to_string())?;
        } else if full_path.is_dir() {
            std::fs::remove_dir_all(&full_path).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    let output = Command::new("git")
        .args(["restore", "--worktree", "--", file_path])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let fb = Command::new("git")
            .args(["checkout", "HEAD", "--", file_path])
            .current_dir(repo)
            .output()
            .map_err(|e| e.to_string())?;
        if fb.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&fb.stderr).to_string())
        }
    }
}

pub fn stage_all(repo: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn unstage_all(repo: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args(["restore", "--staged", "."])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let fb = Command::new("git")
            .args(["reset", "HEAD"])
            .current_dir(repo)
            .output()
            .map_err(|e| e.to_string())?;
        if fb.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&fb.stderr).to_string())
        }
    }
}

pub fn discard_all(repo: &Path) -> Result<(), String> {
    let _ = Command::new("git")
        .args(["restore", "--worktree", "."])
        .current_dir(repo)
        .output();

    let clean = Command::new("git")
        .args(["clean", "-fd"])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;

    if clean.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&clean.stderr).to_string())
    }
}

pub fn push_branch(repo: &Path, branch: &str) -> Result<String, String> {
    if branch.starts_with('-') || branch.is_empty() {
        return Err("Invalid branch name".to_string());
    }

    let output = Command::new("git")
        .args(["push", "origin", branch])
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !stdout.is_empty() {
            Ok(stdout)
        } else if !stderr.is_empty() {
            Ok(stderr)
        } else {
            Ok(format!("Pushed '{}' to origin", branch))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("set-upstream") || stderr.contains("no upstream") {
            let u_out = Command::new("git")
                .args(["push", "--set-upstream", "origin", branch])
                .current_dir(repo)
                .output()
                .map_err(|e| e.to_string())?;
            if u_out.status.success() {
                return Ok(format!("Pushed and set upstream for '{}'", branch));
            }
        }
        Err(if stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            stderr
        })
    }
}

fn run_git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_worktrees_current_repo() {
        let repo_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let worktrees = list_worktrees(repo_dir).expect("should list worktrees");
        assert!(!worktrees.is_empty(), "should have at least 1 worktree");
        assert!(worktrees[0].is_main, "first worktree should be main");
        assert!(worktrees[0].is_current, "first worktree should be marked current");
        assert_eq!(worktrees[0].branch, "main");
    }


    #[test]
    fn test_git_common_dir() {
        let repo_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let common = git_common_dir(repo_dir).expect("should get common git dir");
        assert!(common.exists(), "common git dir must exist on disk");
        assert!(common.to_string_lossy().ends_with(".git"));
    }

    #[test]
    fn test_list_worktrees_with_linked_worktree() {
        let temp_dir = std::env::temp_dir().join(format!("tengga_wt_test_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let run = |dir: &Path, args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .expect("git cmd failed")
        };

        run(&temp_dir, &["init", "-b", "main"]);
        run(&temp_dir, &["config", "user.name", "Test User"]);
        run(&temp_dir, &["config", "user.email", "test@example.com"]);
        let file_path = temp_dir.join("hello.txt");
        let _ = std::fs::write(&file_path, "hello world");
        run(&temp_dir, &["add", "hello.txt"]);
        run(&temp_dir, &["commit", "-m", "initial commit"]);

        let wt_path = std::env::temp_dir().join(format!("tengga_wt_linked_{}", uuid::Uuid::new_v4()));
        let out = run(&temp_dir, &["worktree", "add", wt_path.to_str().unwrap(), "-b", "feature-branch"]);
        assert!(out.status.success(), "worktree add should succeed: {}", String::from_utf8_lossy(&out.stderr));

        // Test listing worktrees from main repo
        let list_from_main = list_worktrees(&temp_dir).expect("should list from main");
        assert_eq!(list_from_main.len(), 2);
        assert!(list_from_main[0].is_main);
        assert!(list_from_main[0].is_current);
        assert!(!list_from_main[1].is_main);
        assert!(!list_from_main[1].is_current);
        assert_eq!(list_from_main[1].branch, "feature-branch");

        // Test listing worktrees from linked worktree
        let list_from_wt = list_worktrees(&wt_path).expect("should list from wt");
        assert_eq!(list_from_wt.len(), 2);
        assert!(list_from_wt[0].is_main);
        assert!(!list_from_wt[0].is_current);
        assert!(!list_from_wt[1].is_main);
        assert!(list_from_wt[1].is_current);
        assert_eq!(list_from_wt[1].branch, "feature-branch");

        // Cleanup
        let _ = run(&temp_dir, &["worktree", "remove", wt_path.to_str().unwrap()]);
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::remove_dir_all(&wt_path);
    }

    #[test]
    fn test_staging_and_unstaging_and_discarding() {
        let temp_dir = std::env::temp_dir().join(format!("tengga_stage_test_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let run = |dir: &Path, args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .expect("git cmd failed")
        };

        run(&temp_dir, &["init", "-b", "main"]);
        run(&temp_dir, &["config", "user.name", "Test User"]);
        run(&temp_dir, &["config", "user.email", "test@example.com"]);
        let file_path = temp_dir.join("hello.txt");
        let _ = std::fs::write(&file_path, "hello world\n");
        run(&temp_dir, &["add", "hello.txt"]);
        run(&temp_dir, &["commit", "-m", "initial commit"]);

        // Modify hello.txt and create untracked new.txt
        let _ = std::fs::write(&file_path, "hello world modified\n");
        let untracked_path = temp_dir.join("new.txt");
        let _ = std::fs::write(&untracked_path, "brand new content\n");

        // Check untracked files
        let untracked = list_untracked_files(&temp_dir).expect("should list untracked files");
        assert_eq!(untracked, vec!["new.txt"]);

        // Check untracked diff
        let u_diff = diff_untracked(&temp_dir, "new.txt").expect("should diff untracked file");
        assert!(u_diff.contains("+brand new content"));

        // Stage hello.txt
        stage_file(&temp_dir, "hello.txt").expect("should stage file");
        let staged_diff = diff_staged(&temp_dir).expect("should get staged diff");
        assert!(staged_diff.contains("hello world modified"));

        // Unstage hello.txt
        unstage_file(&temp_dir, "hello.txt").expect("should unstage file");
        let staged_diff_after = diff_staged(&temp_dir).expect("should get staged diff");
        assert!(!staged_diff_after.contains("hello world modified"));

        // Stage all
        stage_all(&temp_dir).expect("should stage all");
        let staged_diff_all = diff_staged(&temp_dir).expect("should get staged diff");
        assert!(staged_diff_all.contains("hello world modified"));
        assert!(staged_diff_all.contains("brand new content"));

        // Unstage all
        unstage_all(&temp_dir).expect("should unstage all");
        let staged_diff_none = diff_staged(&temp_dir).expect("should get staged diff");
        assert!(staged_diff_none.is_empty() || !staged_diff_none.contains("hello world modified"));

        // Discard untracked new.txt
        discard_file(&temp_dir, "new.txt").expect("should discard untracked file");
        assert!(!untracked_path.exists(), "untracked file should be deleted on discard");

        // Discard tracked hello.txt
        discard_file(&temp_dir, "hello.txt").expect("should discard tracked file");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world\n");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

