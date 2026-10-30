use notify::{Event, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::{Component, Path};
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE: Duration = Duration::from_millis(600);

fn is_git_state_change(p: &Path) -> bool {
    let s = p.to_string_lossy();
    if s.ends_with(".lock") {
        return false;
    }
    s.ends_with("/.git/HEAD")
        || s.ends_with("\\.git\\HEAD")
        || s.ends_with("/.git/index")
        || s.ends_with("\\.git\\index")
        || s.contains("/.git/refs/")
        || s.contains("\\.git\\refs\\")
        || s.contains("/worktrees")
        || s.contains("\\worktrees")
}

fn is_ignored_dir_name(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".turbo"
            | ".nuxt"
            | ".output"
            | ".svelte-kit"
            | ".cache"
            | ".vite"
            | ".yarn"
            | ".pnpm"
            | "vendor"
            | ".idea"
            | ".vscode"
            | "coverage"
            | ".venv"
            | "venv"
            | "__pycache__"
            | "objects"
            | "logs"
            | "hooks"
            | "lfs"
    )
}

pub fn is_ignored_path(p: &Path) -> bool {
    let s = p.to_string_lossy();
    if s.ends_with(".lock")
        || s.ends_with(".DS_Store")
        || s.ends_with('~')
        || s.ends_with(".tmp")
        || s.ends_with(".swp")
        || s.ends_with(".swo")
    {
        return true;
    }

    let has_git = p.components().any(|c| match c {
        Component::Normal(name) => name == OsStr::new(".git"),
        _ => false,
    });

    if has_git {
        return !is_git_state_change(p);
    }

    for c in p.components() {
        if let Component::Normal(name) = c {
            let name_str = name.to_string_lossy();
            if is_ignored_dir_name(&name_str) {
                return true;
            }
        }
    }

    false
}

pub fn watch(
    app: AppHandle,
    path: &Path,
    common_git_dir: Option<&Path>,
) -> notify::Result<notify::RecommendedWatcher> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ref event) = res {
            if !event.paths.is_empty() && event.paths.iter().all(|p| is_ignored_path(p)) {
                return;
            }
        }
        let _ = tx.send(res);
    })?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    if let Some(common_dir) = common_git_dir {
        if common_dir.exists() && !common_dir.starts_with(path) {
            // Watch common .git directory non-recursively (catches HEAD, index, config)
            let _ = watcher.watch(common_dir, RecursiveMode::NonRecursive);
            let refs_dir = common_dir.join("refs");
            if refs_dir.exists() {
                let _ = watcher.watch(&refs_dir, RecursiveMode::Recursive);
            }
            let worktrees_dir = common_dir.join("worktrees");
            if worktrees_dir.exists() {
                let _ = watcher.watch(&worktrees_dir, RecursiveMode::Recursive);
            }
        }
    }

    std::thread::spawn(move || {
        while let Ok(first) = rx.recv() {
            let mut has_worktree_change = false;
            if let Ok(ref event) = first {
                if event.paths.iter().any(|p| {
                    let s = p.to_string_lossy();
                    s.contains("/worktrees") || s.contains("\\worktrees")
                }) {
                    has_worktree_change = true;
                }
            }

            while let Ok(res) = rx.recv_timeout(DEBOUNCE) {
                if let Ok(ref event) = res {
                    if event.paths.iter().any(|p| {
                        let s = p.to_string_lossy();
                        s.contains("/worktrees") || s.contains("\\worktrees")
                    }) {
                        has_worktree_change = true;
                    }
                }
            }

            if has_worktree_change {
                let _ = app.emit("worktrees-changed", ());
            }

            if app.emit("changes-detected", ()).is_err() {
                break;
            }
        }
    });

    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_ignored_path() {
        assert!(is_ignored_path(&PathBuf::from("/repo/node_modules/foo/index.js")));
        assert!(is_ignored_path(&PathBuf::from("/repo/target/debug/app")));
        assert!(is_ignored_path(&PathBuf::from("/repo/.git/objects/12/3456")));
        assert!(is_ignored_path(&PathBuf::from("/repo/.git/index.lock")));
        assert!(is_ignored_path(&PathBuf::from("/repo/.DS_Store")));
        assert!(is_ignored_path(&PathBuf::from("/repo/src/.App.svelte.swp")));

        assert!(!is_ignored_path(&PathBuf::from("/repo/.git/index")));
        assert!(!is_ignored_path(&PathBuf::from("/repo/.git/HEAD")));
        assert!(!is_ignored_path(&PathBuf::from("/repo/.git/refs/heads/main")));
        assert!(!is_ignored_path(&PathBuf::from("/repo/.git/worktrees/feat/HEAD")));
        assert!(!is_ignored_path(&PathBuf::from("/repo/src/App.svelte")));
        assert!(!is_ignored_path(&PathBuf::from("/repo/README.md")));
    }
}
