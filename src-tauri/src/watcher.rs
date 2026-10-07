use notify::{Event, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::{Component, Path};
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE: Duration = Duration::from_millis(400);

pub fn watch(
    app: AppHandle,
    path: &Path,
    common_git_dir: Option<&Path>,
) -> notify::Result<notify::RecommendedWatcher> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ref event) = res {
            let is_worktree_event = event.paths.iter().any(|p| {
                let s = p.to_string_lossy();
                s.contains("/worktrees") || s.contains("\\worktrees")
            });

            let is_ignored_git_event = event.paths.iter().any(|p| {
                let s = p.to_string_lossy();
                if s.ends_with(".lock") {
                    return true;
                }
                let has_git = p.components().any(|c| match c {
                    Component::Normal(name) => name == OsStr::new(".git"),
                    _ => false,
                });
                if has_git {
                    let is_git_state_change = s.ends_with("/.git/HEAD")
                        || s.ends_with("/.git/index")
                        || s.contains("/.git/refs/")
                        || is_worktree_event;
                    return !is_git_state_change;
                }
                false
            });
            if is_ignored_git_event {
                return;
            }
        }
        let _ = tx.send(res);
    })?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    if let Some(common_dir) = common_git_dir {
        if common_dir.exists() && !common_dir.starts_with(path) {
            let _ = watcher.watch(common_dir, RecursiveMode::Recursive);
        }
    }

    std::thread::spawn(move || {
        loop {
            // block for the first event, then drain anything else that
            // arrives within DEBOUNCE before emitting once
            let first = match rx.recv() {
                Ok(evt) => evt,
                Err(_) => break,
            };

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
