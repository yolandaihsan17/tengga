use notify::{Event, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::{Component, Path};
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE: Duration = Duration::from_millis(400);

pub fn watch(app: AppHandle, path: &Path) -> notify::Result<notify::RecommendedWatcher> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ref event) = res {
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
                        || s.contains("/.git/refs/");
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

    std::thread::spawn(move || {
        loop {
            // block for the first event, then drain anything else that
            // arrives within DEBOUNCE before emitting once
            if rx.recv().is_err() {
                break;
            }
            while rx.recv_timeout(DEBOUNCE).is_ok() {}
            if app.emit("changes-detected", ()).is_err() {
                break;
            }
        }
    });

    Ok(watcher)
}
