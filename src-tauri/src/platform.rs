use tauri::Manager;

#[cfg(target_os = "macos")]
pub fn set_macos_dock_icon(png_bytes: &[u8]) {
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

        let nsdata_class = objc_getClass(c"NSData".as_ptr());
        let sel_data_with_bytes = sel_registerName(c"dataWithBytes:length:".as_ptr());
        let data = msg_send2(nsdata_class, sel_data_with_bytes, png_bytes.as_ptr(), png_bytes.len());
        if data.is_null() { return; }

        let nsimage_class = objc_getClass(c"NSImage".as_ptr());
        let sel_alloc = sel_registerName(c"alloc".as_ptr());
        let sel_init_with_data = sel_registerName(c"initWithData:".as_ptr());
        let img_alloc = msg_send0(nsimage_class, sel_alloc);
        let img = msg_send1(img_alloc, sel_init_with_data, data);
        if img.is_null() { return; }

        let nsapp_class = objc_getClass(c"NSApplication".as_ptr());
        let sel_shared_app = sel_registerName(c"sharedApplication".as_ptr());
        let sel_set_icon = sel_registerName(c"setApplicationIconImage:".as_ptr());
        let app = msg_send0(nsapp_class, sel_shared_app);
        if !app.is_null() {
            msg_send1(app, sel_set_icon, img);
        }

        let nsprocessinfo_class = objc_getClass(c"NSProcessInfo".as_ptr());
        let sel_process_info = sel_registerName(c"processInfo".as_ptr());
        let sel_set_process_name = sel_registerName(c"setProcessName:".as_ptr());
        let nsstring_class = objc_getClass(c"NSString".as_ptr());
        let sel_string_with_utf8 = sel_registerName(c"stringWithUTF8String:".as_ptr());
        let pinfo = msg_send0(nsprocessinfo_class, sel_process_info);
        if !pinfo.is_null() {
            let name_str = msg_send1(nsstring_class, sel_string_with_utf8, c"tengga".as_ptr() as _);
            if !name_str.is_null() {
                msg_send1(pinfo, sel_set_process_name, name_str);
            }
        }

        let nsworkspace_class = objc_getClass(c"NSWorkspace".as_ptr());
        let sel_shared_ws = sel_registerName(c"sharedWorkspace".as_ptr());
        let sel_set_icon_for_file = sel_registerName(c"setIcon:forFile:options:".as_ptr());
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

#[cfg(not(target_os = "macos"))]
pub fn set_macos_dock_icon(_png_bytes: &[u8]) {}

#[cfg(target_os = "macos")]
pub fn ensure_macos_transparency(w: &tauri::WebviewWindow) {
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

            let sel_content_view = sel_registerName(c"contentView".as_ptr());
            let sel_view_with_tag = sel_registerName(c"viewWithTag:".as_ptr());
            let sel_set_alpha = sel_registerName(c"setAlphaValue:".as_ptr());

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

#[cfg(not(target_os = "macos"))]
pub fn ensure_macos_transparency(_w: &tauri::WebviewWindow) {}

pub fn set_window_blur_intensity(app: &tauri::AppHandle, intensity: f64) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        for (_label, w) in app.webview_windows() {
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

                    let sel_content_view = sel_registerName(c"contentView".as_ptr());
                    let sel_view_with_tag = sel_registerName(c"viewWithTag:".as_ptr());
                    let sel_set_alpha = sel_registerName(c"setAlphaValue:".as_ptr());

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
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, intensity);
    }
    Ok(())
}

pub fn toggle_maximize_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(ns_win) = window.ns_window() {
            unsafe {
                type MsgSend1 = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void);
                extern "C" {
                    fn sel_registerName(name: *const std::os::raw::c_char) -> *mut std::ffi::c_void;
                    fn objc_msgSend();
                }
                let msg_send1: MsgSend1 = std::mem::transmute(objc_msgSend as *const ());
                let sel_zoom = sel_registerName(c"zoom:".as_ptr());
                msg_send1(ns_win as _, sel_zoom, std::ptr::null_mut());
                return Ok(());
            }
        }
    }

    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
    } else {
        window.maximize().map_err(|e| e.to_string())?;
    }
    Ok(())
}
