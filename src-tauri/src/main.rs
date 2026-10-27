#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod diff;
mod git;
mod menu;
mod platform;
mod watcher;

use diff::{parse_diff, FileDiff, HunkPatch};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    watchers: Mutex<HashMap<String, notify::RecommendedWatcher>>,
    hunk_patches: Mutex<HashMap<String, HashMap<String, HunkPatch>>>,
}

#[tauri::command]
fn validate_git_repo(path: String) -> bool {
    git::is_repo(&PathBuf::from(path))
}

#[tauri::command]
fn start_watcher(app: tauri::AppHandle, state: State<AppState>, path: String) -> Result<(), String> {
    let repo = PathBuf::from(&path);
    let common_dir = git::git_common_dir(&repo).ok();
    let mut watchers = state.watchers.lock().unwrap();
    // Drop any existing watchers before starting the new one for the active worktree/repository
    watchers.clear();
    let w = watcher::watch(app, &repo, common_dir.as_deref()).map_err(|e| e.to_string())?;
    watchers.insert(path, w);
    Ok(())
}

#[tauri::command]
fn stop_watcher(state: State<AppState>, path: String) -> Result<(), String> {
    let mut watchers = state.watchers.lock().unwrap();
    watchers.remove(&path);
    let mut patches = state.hunk_patches.lock().unwrap();
    patches.remove(&path);
    Ok(())
}

#[tauri::command]
fn stop_all_watchers(state: State<AppState>) -> Result<(), String> {
    let mut watchers = state.watchers.lock().unwrap();
    watchers.clear();
    let mut patches = state.hunk_patches.lock().unwrap();
    patches.clear();
    Ok(())
}

#[tauri::command]
async fn get_worktrees(repo_path: String) -> Result<Vec<git::WorktreeInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::list_worktrees(&PathBuf::from(repo_path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
struct ComparisonInfo {
    ahead: u32,
    behind: u32,
}

#[tauri::command]
fn get_branch_comparison_info(
    repo_path: String,
    base_branch: String,
) -> Result<ComparisonInfo, String> {
    let repo = PathBuf::from(repo_path);
    let (ahead, behind) = git::branch_ahead_behind(&repo, &base_branch)?;
    Ok(ComparisonInfo { ahead, behind })
}

#[tauri::command]
fn get_diff(
    state: State<AppState>,
    repo_path: String,
    base_branch: Option<String>,
) -> Result<Vec<FileDiff>, String> {
    let repo = PathBuf::from(&repo_path);

    let (all_files, all_patches) = match base_branch.as_deref() {
        Some(base) if !base.is_empty() && base != "HEAD" => {
            let raw = git::diff_against_base(&repo, base)?;
            let (f, p) = parse_diff(&raw, "branch_compare");
            (f, p)
        }
        _ => {
            let unstaged_raw = git::diff_unstaged(&repo)?;
            let staged_raw = git::diff_staged(&repo)?;

            let (unstaged_files, unstaged_patches) = parse_diff(&unstaged_raw, "unstaged");
            let (staged_files, staged_patches) = parse_diff(&staged_raw, "staged");

            let mut files: HashMap<String, FileDiff> = HashMap::new();

            // 1. First insert staged files
            for mut f in staged_files {
                f.has_staged = true;
                f.has_unstaged = false;
                f.is_untracked = false;
                files.insert(f.path.clone(), f);
            }

            // 2. Merge unstaged files
            for f in unstaged_files {
                files
                    .entry(f.path.clone())
                    .and_modify(|existing| {
                        existing.additions += f.additions;
                        existing.deletions += f.deletions;
                        existing.hunks.extend(f.hunks.clone());
                        existing.has_unstaged = true;
                        if f.is_binary {
                            existing.is_binary = true;
                        }
                    })
                    .or_insert_with(|| {
                        let mut item = f;
                        item.has_staged = false;
                        item.has_unstaged = true;
                        item.is_untracked = false;
                        item
                    });
            }

            // 3. Detect untracked files
            let mut untracked_patches = Vec::new();
            if let Ok(untracked_list) = git::list_untracked_files(&repo) {
                for (idx, u) in untracked_list.into_iter().enumerate() {
                    if files.contains_key(&u) {
                        continue;
                    }
                    if idx < 5 {
                        if let Ok(raw_u) = git::diff_untracked(&repo, &u) {
                            let (u_files, u_p) = parse_diff(&raw_u, "unstaged");
                            if let Some(mut first) = u_files.into_iter().next() {
                                first.has_staged = false;
                                first.has_unstaged = true;
                                first.is_untracked = true;
                                files.insert(first.path.clone(), first);
                                untracked_patches.extend(u_p);
                                continue;
                            }
                        }
                    }
                    files.insert(
                        u.clone(),
                        FileDiff {
                            path: u,
                            additions: 0,
                            deletions: 0,
                            hunks: Vec::new(),
                            is_binary: false,
                            has_staged: false,
                            has_unstaged: true,
                            is_untracked: true,
                        },
                    );
                }
            }

            let patches: Vec<(String, HunkPatch)> = unstaged_patches
                .into_iter()
                .chain(staged_patches)
                .chain(untracked_patches)
                .collect();
            (files.into_values().collect(), patches)
        }
    };

    let mut patches_by_repo = state.hunk_patches.lock().unwrap();
    let repo_patches = patches_by_repo.entry(repo_path).or_default();
    repo_patches.clear();
    for (id, p) in all_patches {
        repo_patches.insert(id, p);
    }

    Ok(all_files)
}

#[tauri::command]
fn accept_hunk(state: State<AppState>, repo_path: String, hunk_id: String) -> Result<(), String> {
    let patches_by_repo = state.hunk_patches.lock().unwrap();
    let patch = patches_by_repo
        .get(&repo_path)
        .and_then(|m| m.get(&hunk_id))
        .ok_or("hunk not found, try reopening the folder")?;
    if patch.source == "staged" {
        return Ok(()); // already staged, nothing to do
    }
    let full_patch = format!("{}{}", patch.file_header, patch.hunk_text);
    git::apply_patch(&PathBuf::from(repo_path), &full_patch, true, false)
}

#[tauri::command]
fn reject_hunk(state: State<AppState>, repo_path: String, hunk_id: String) -> Result<(), String> {
    let patches_by_repo = state.hunk_patches.lock().unwrap();
    let patch = patches_by_repo
        .get(&repo_path)
        .and_then(|m| m.get(&hunk_id))
        .ok_or("hunk not found, try reopening the folder")?;
    let full_patch = format!("{}{}", patch.file_header, patch.hunk_text);
    let cached = patch.source == "staged";
    git::apply_patch(&PathBuf::from(repo_path), &full_patch, cached, true)
}

#[tauri::command]
fn get_repo_files(repo_path: String) -> Result<Vec<String>, String> {
    git::list_repo_files(&PathBuf::from(repo_path))
}

#[tauri::command]
fn get_file_content(repo_path: String, file_path: String) -> Result<String, String> {
    let repo = PathBuf::from(repo_path);
    let full_path = repo.join(&file_path);

    if !full_path.exists() {
        return Err("File does not exist on disk".to_string());
    }

    let bytes = std::fs::read(&full_path).map_err(|e| e.to_string())?;

    if bytes.iter().take(8000).any(|&b| b == 0) {
        return Err("Binary file cannot be displayed as text".to_string());
    }

    String::from_utf8(bytes).map_err(|_| "File contains invalid UTF-8 characters".to_string())
}

#[tauri::command]
fn save_file_content(repo_path: String, file_path: String, content: String) -> Result<(), String> {
    let repo = PathBuf::from(repo_path);
    let full_path = repo.join(&file_path);

    std::fs::write(&full_path, content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_current_branch(repo_path: String) -> Result<String, String> {
    git::current_branch(&PathBuf::from(repo_path))
}

#[tauri::command]
fn list_branches(repo_path: String) -> Result<Vec<String>, String> {
    git::list_branches(&PathBuf::from(repo_path))
}

#[tauri::command]
async fn switch_branch(repo_path: String, branch: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::switch_branch(&PathBuf::from(repo_path), &branch)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pull_branch(repo_path: String, branch: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::pull_branch(&PathBuf::from(repo_path), &branch)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn push_branch(repo_path: String, branch: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::push_branch(&PathBuf::from(repo_path), &branch)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn search_repo_content(
    repo_path: String,
    query: String,
    case_sensitive: bool,
    match_whole_word: bool,
    use_regex: bool,
    file_pattern: Option<String>,
    max_results: Option<usize>,
) -> Result<git::SearchResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let repo = PathBuf::from(repo_path);
        git::search_in_repo(
            &repo,
            &query,
            case_sensitive,
            match_whole_word,
            use_regex,
            file_pattern.as_deref(),
            max_results.unwrap_or(500),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stage_file(repo_path: String, file_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::stage_file(&PathBuf::from(repo_path), &file_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn unstage_file(repo_path: String, file_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::unstage_file(&PathBuf::from(repo_path), &file_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn discard_file(repo_path: String, file_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::discard_file(&PathBuf::from(repo_path), &file_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stage_all(repo_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::stage_all(&PathBuf::from(repo_path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn unstage_all(repo_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::unstage_all(&PathBuf::from(repo_path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn discard_all(repo_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::discard_all(&PathBuf::from(repo_path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn set_window_blur_intensity(app: tauri::AppHandle, intensity: f64) -> Result<(), String> {
    platform::set_window_blur_intensity(&app, intensity)
}

fn create_window_instance(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
    let count = WINDOW_COUNTER.fetch_add(1, Ordering::SeqCst);
    let label = format!("window-{}", count);

    let mut builder = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Tengga — Diff Review")
    .inner_size(1100.0, 750.0)
    .transparent(true);

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    let icon = tauri::include_image!("icons/icon.png");
    let _ = window.set_icon(icon);

    #[cfg(target_os = "macos")]
    {
        platform::ensure_macos_transparency(&window);
    }

    Ok(window)
}

#[tauri::command]
async fn create_new_window(app: tauri::AppHandle) -> Result<(), String> {
    create_window_instance(&app)?;
    Ok(())
}

#[tauri::command]
fn start_drag_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_maximize_window(window: tauri::WebviewWindow) -> Result<(), String> {
    platform::toggle_maximize_window(&window)
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            if let Some(w) = app.get_webview_window("main") {
                let icon = tauri::include_image!("icons/icon.png");
                let _ = w.set_icon(icon);

                #[cfg(target_os = "macos")]
                {
                    platform::ensure_macos_transparency(&w);
                }
            }
            #[cfg(target_os = "macos")]
            {
                let icon_bytes = include_bytes!("../icons/icon.png");
                platform::set_macos_dock_icon(icon_bytes);
            }

            let menu = menu::build_menu(app)?;
            let _ = app.set_menu(menu);

            use tauri::Emitter;
            app.on_menu_event(|app_handle, event| {
                let id = event.id().as_ref();
                if id == "new_window" {
                    let _ = create_window_instance(app_handle);
                } else if id == "about_tengga" || id == "about_tengga_help" {
                    let _ = app_handle.emit("open-about", ());
                } else if id == "open_repo" {
                    let _ = open_external_url("https://github.com/yolandaihsan17/tengga".to_string());
                } else if id == "report_issue" {
                    let _ = open_external_url("https://github.com/yolandaihsan17/tengga/issues".to_string());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            validate_git_repo,
            start_watcher,
            stop_watcher,
            stop_all_watchers,
            get_diff,
            accept_hunk,
            reject_hunk,
            get_repo_files,
            get_file_content,
            save_file_content,
            get_current_branch,
            list_branches,
            switch_branch,
            get_branch_comparison_info,
            pull_branch,
            push_branch,
            stage_file,
            unstage_file,
            discard_file,
            stage_all,
            unstage_all,
            discard_all,
            get_worktrees,
            set_window_blur_intensity,
            search_repo_content,
            start_drag_window,
            toggle_maximize_window,
            create_new_window,
            open_external_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
