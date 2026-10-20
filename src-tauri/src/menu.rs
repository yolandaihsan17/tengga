use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub fn build_menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    let handle = app.handle();
    let new_win_item = MenuItem::with_id(handle, "new_window", "New Window", true, Some("CmdOrControl+N"))?;
    let close_win_item = PredefinedMenuItem::close_window(handle, None)?;
    let file_menu = Submenu::with_items(handle, "File", true, &[&new_win_item, &close_win_item])?;

    let edit_menu = Submenu::with_items(
        handle,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(handle, None)?,
            &PredefinedMenuItem::redo(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::cut(handle, None)?,
            &PredefinedMenuItem::copy(handle, None)?,
            &PredefinedMenuItem::paste(handle, None)?,
            &PredefinedMenuItem::select_all(handle, None)?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        handle,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(handle, None)?,
            &PredefinedMenuItem::fullscreen(handle, None)?,
        ],
    )?;

    // Help Menu (Available on macOS, Windows, and Linux)
    let help_about_item = MenuItem::with_id(handle, "about_tengga_help", "About Tengga", true, None::<&str>)?;
    let help_repo_item = MenuItem::with_id(handle, "open_repo", "GitHub Repository", true, None::<&str>)?;
    let help_issue_item = MenuItem::with_id(handle, "report_issue", "Report Issue", true, None::<&str>)?;
    let help_menu = Submenu::with_items(
        handle,
        "Help",
        true,
        &[
            &help_about_item,
            &PredefinedMenuItem::separator(handle)?,
            &help_repo_item,
            &help_issue_item,
        ],
    )?;

    #[cfg(target_os = "macos")]
    let about_item = MenuItem::with_id(handle, "about_tengga", "About Tengga", true, None::<&str>)?;

    #[cfg(target_os = "macos")]
    let app_menu = Submenu::with_items(
        handle,
        "Tengga",
        true,
        &[
            &about_item,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::hide(handle, None)?,
            &PredefinedMenuItem::hide_others(handle, None)?,
            &PredefinedMenuItem::show_all(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::quit(handle, None)?,
        ],
    )?;

    #[cfg(target_os = "macos")]
    let menu = Menu::with_items(handle, &[&app_menu, &file_menu, &edit_menu, &window_menu, &help_menu])?;
    #[cfg(not(target_os = "macos"))]
    let menu = Menu::with_items(handle, &[&file_menu, &edit_menu, &window_menu, &help_menu])?;

    Ok(menu)
}
