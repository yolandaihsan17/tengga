#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod diff;
mod git;
mod watcher;

use diff::{parse_diff, FileDiff, HunkPatch};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
    hunk_patches: Mutex<HashMap<String, HunkPatch>>,
}

#[tauri::command]
fn validate_git_repo(path: String) -> bool {
    git::is_repo(&PathBuf::from(path))
}

#[tauri::command]
fn start_watcher(app: tauri::AppHandle, state: State<AppState>, path: String) -> Result<(), String> {
    let w = watcher::watch(app, &PathBuf::from(&path)).map_err(|e| e.to_string())?;
    *state.watcher.lock().unwrap() = Some(w);
    state.hunk_patches.lock().unwrap().clear();
    Ok(())
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
            for f in unstaged_files.into_iter().chain(staged_files.into_iter()) {
                files
                    .entry(f.path.clone())
                    .and_modify(|existing| {
                        existing.additions += f.additions;
                        existing.deletions += f.deletions;
                        existing.hunks.extend(f.hunks.clone());
                        if f.is_binary {
                            existing.is_binary = true;
                        }
                    })
                    .or_insert(f);
            }
            let patches: Vec<(String, HunkPatch)> =
                unstaged_patches.into_iter().chain(staged_patches.into_iter()).collect();
            (files.into_values().collect(), patches)
        }
    };

    let mut patches = state.hunk_patches.lock().unwrap();
    patches.clear();
    for (id, p) in all_patches {
        patches.insert(id, p);
    }

    Ok(all_files)
}

#[tauri::command]
fn accept_hunk(state: State<AppState>, repo_path: String, hunk_id: String) -> Result<(), String> {
    let patches = state.hunk_patches.lock().unwrap();
    let patch = patches.get(&hunk_id).ok_or("hunk not found, try reopening the folder")?;
    if patch.source == "staged" {
        return Ok(()); // already staged, nothing to do
    }
    let full_patch = format!("{}{}", patch.file_header, patch.hunk_text);
    git::apply_patch(&PathBuf::from(repo_path), &full_patch, true, false)
}

#[tauri::command]
fn reject_hunk(state: State<AppState>, repo_path: String, hunk_id: String) -> Result<(), String> {
    let patches = state.hunk_patches.lock().unwrap();
    let patch = patches.get(&hunk_id).ok_or("hunk not found, try reopening the folder")?;
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
fn switch_branch(repo_path: String, branch: String) -> Result<(), String> {
    git::switch_branch(&PathBuf::from(repo_path), &branch)
}

#[tauri::command]
fn pull_branch(repo_path: String, branch: String) -> Result<String, String> {
    git::pull_branch(&PathBuf::from(repo_path), &branch)
}

#[cfg(target_os = "macos")]
fn set_macos_dock_icon(png_bytes: &[u8]) {
    #[link(name = "objc", kind = "dylib")]
    extern "C" {
        fn objc_getClass(name: *const std::os::raw::c_char) -> *mut std::ffi::c_void;
        fn sel_registerName(name: *const std::os::raw::c_char) -> *mut std::ffi::c_void;
        fn objc_msgSend();
    }

    unsafe {
        type MsgSend0 = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        type MsgSend1 = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        type MsgSend2 = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *const u8, usize) -> *mut std::ffi::c_void;

        let msg_send0: MsgSend0 = std::mem::transmute(objc_msgSend as *const ());
        let msg_send1: MsgSend1 = std::mem::transmute(objc_msgSend as *const ());
        let msg_send2: MsgSend2 = std::mem::transmute(objc_msgSend as *const ());

        let nsdata_class = objc_getClass(b"NSData\0".as_ptr() as _);
        let sel_data_with_bytes = sel_registerName(b"dataWithBytes:length:\0".as_ptr() as _);
        let data = msg_send2(nsdata_class, sel_data_with_bytes, png_bytes.as_ptr(), png_bytes.len());
        if data.is_null() { return; }

        let nsimage_class = objc_getClass(b"NSImage\0".as_ptr() as _);
        let sel_alloc = sel_registerName(b"alloc\0".as_ptr() as _);
        let sel_init_with_data = sel_registerName(b"initWithData:\0".as_ptr() as _);
        let img_alloc = msg_send0(nsimage_class, sel_alloc);
        let img = msg_send1(img_alloc, sel_init_with_data, data);
        if img.is_null() { return; }

        let nsapp_class = objc_getClass(b"NSApplication\0".as_ptr() as _);
        let sel_shared_app = sel_registerName(b"sharedApplication\0".as_ptr() as _);
        let sel_set_icon = sel_registerName(b"setApplicationIconImage:\0".as_ptr() as _);
        let app = msg_send0(nsapp_class, sel_shared_app);
        if !app.is_null() {
            msg_send1(app, sel_set_icon, img);
        }

        let nsprocessinfo_class = objc_getClass(b"NSProcessInfo\0".as_ptr() as _);
        let sel_process_info = sel_registerName(b"processInfo\0".as_ptr() as _);
        let sel_set_process_name = sel_registerName(b"setProcessName:\0".as_ptr() as _);
        let nsstring_class = objc_getClass(b"NSString\0".as_ptr() as _);
        let sel_string_with_utf8 = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as _);
        let pinfo = msg_send0(nsprocessinfo_class, sel_process_info);
        if !pinfo.is_null() {
            let name_str = msg_send1(nsstring_class, sel_string_with_utf8, b"tengga\0".as_ptr() as _);
            if !name_str.is_null() {
                msg_send1(pinfo, sel_set_process_name, name_str);
            }
        }

        let nsworkspace_class = objc_getClass(b"NSWorkspace\0".as_ptr() as _);
        let sel_shared_ws = sel_registerName(b"sharedWorkspace\0".as_ptr() as _);
        let sel_set_icon_for_file = sel_registerName(b"setIcon:forFile:options:\0".as_ptr() as _);
        let ws = msg_send0(nsworkspace_class, sel_shared_ws);
        if !ws.is_null() {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(path_str) = exe_path.to_str() {
                    let mut c_path = path_str.as_bytes().to_vec();
                    c_path.push(0);
                    let path_obj = msg_send1(nsstring_class, sel_string_with_utf8, c_path.as_ptr() as _);
                    if !path_obj.is_null() {
                        type MsgSendSetIcon = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void, usize) -> bool;
                        let msg_send_set_icon: MsgSendSetIcon = std::mem::transmute(objc_msgSend as *const ());
                        msg_send_set_icon(ws, sel_set_icon_for_file, img, path_obj, 0);
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn ensure_macos_transparency(w: &tauri::WebviewWindow) {
    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
    let _ = apply_vibrancy(w, NSVisualEffectMaterial::HudWindow, None, None);

    if let Ok(ns_win) = w.ns_window() {
        unsafe {
            type MsgSend0 = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            type MsgSendInt = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, isize) -> *mut std::ffi::c_void;
            type MsgSendFloat = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, f64);

            extern "C" {
                fn sel_registerName(name: *const std::os::raw::c_char) -> *mut std::ffi::c_void;
                fn objc_msgSend();
            }

            let msg_send0: MsgSend0 = std::mem::transmute(objc_msgSend as *const ());
            let msg_send_int: MsgSendInt = std::mem::transmute(objc_msgSend as *const ());
            let msg_send_float: MsgSendFloat = std::mem::transmute(objc_msgSend as *const ());

            let sel_content_view = sel_registerName(b"contentView\0".as_ptr() as _);
            let sel_view_with_tag = sel_registerName(b"viewWithTag:\0".as_ptr() as _);
            let sel_set_alpha = sel_registerName(b"setAlphaValue:\0".as_ptr() as _);

            let content_view = msg_send0(ns_win as _, sel_content_view);
            if !content_view.is_null() {
                // NS_VIEW_TAG_BLUR_VIEW is 91376254 from window_vibrancy
                let blur_view = msg_send_int(content_view, sel_view_with_tag, 91376254);
                if !blur_view.is_null() {
                    // Set blur opacity to 0.30 (soft subtle blur instead of heavy frosted blur)
                    msg_send_float(blur_view, sel_set_alpha, 0.30);
                }
            }
        }
    }
}

#[tauri::command]
fn set_window_blur_intensity(app: tauri::AppHandle, intensity: f64) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(w) = app.get_webview_window("main") {
            if let Ok(ns_win) = w.ns_window() {
                unsafe {
                    type MsgSend0 = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
                    type MsgSendInt = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, isize) -> *mut std::ffi::c_void;
                    type MsgSendFloat = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, f64);

                    extern "C" {
                        fn sel_registerName(name: *const std::os::raw::c_char) -> *mut std::ffi::c_void;
                        fn objc_msgSend();
                    }

                    let msg_send0: MsgSend0 = std::mem::transmute(objc_msgSend as *const ());
                    let msg_send_int: MsgSendInt = std::mem::transmute(objc_msgSend as *const ());
                    let msg_send_float: MsgSendFloat = std::mem::transmute(objc_msgSend as *const ());

                    let sel_content_view = sel_registerName(b"contentView\0".as_ptr() as _);
                    let sel_view_with_tag = sel_registerName(b"viewWithTag:\0".as_ptr() as _);
                    let sel_set_alpha = sel_registerName(b"setAlphaValue:\0".as_ptr() as _);

                    let content_view = msg_send0(ns_win as _, sel_content_view);
                    if !content_view.is_null() {
                        let blur_view = msg_send_int(content_view, sel_view_with_tag, 91376254);
                        if !blur_view.is_null() {
                            let clamped = intensity.clamp(0.0, 1.0);
                            msg_send_float(blur_view, sel_set_alpha, clamped);
                        }
                    }
                }
            }
        }
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
                    ensure_macos_transparency(&w);
                }
            }
            #[cfg(target_os = "macos")]
            {
                let icon_bytes = include_bytes!("../icons/icon.png");
                set_macos_dock_icon(icon_bytes);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            validate_git_repo,
            start_watcher,
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
            set_window_blur_intensity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
