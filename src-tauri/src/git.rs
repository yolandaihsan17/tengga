use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

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
