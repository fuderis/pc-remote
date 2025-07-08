use crate::prelude::*;
use tauri::Manager;

/// The system tray-icon
pub struct Tray {
    tray: Option<tauri::tray::TrayIcon>,
}

impl Tray {
    /// Create a new tray-icon
    pub fn new() -> Self {
        use tauri::tray;

        let tray = tray::TrayIconBuilder::new()
            .icon(tauri::image::Image::from_path(path!("/icon.ico")).unwrap())
            .on_tray_icon_event(|tray, event| {
                if let tray::TrayIconEvent::Click { button, button_state, .. } = event {
                    if button == tray::MouseButton::Left && button_state == tray::MouseButtonState::Up {
                        let app_handle = tray.app_handle();
                        
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap() {
                                window.hide().unwrap();
                            } else {
                                window.show().unwrap();
                                window.unminimize().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                    }
                }
            })
            .build(&APP_HANDLE.lock().unwrap().clone().unwrap())
            .expect("Failed to create tray icon");

        Self {
            tray: Some(tray),
        }
    }

    /// Remove tray icon
    pub fn remove(mut self) {
        let app_handle = APP_HANDLE.lock().unwrap().clone().unwrap();
        
        if let Some(tray) = self.tray.take() {
            app_handle.remove_tray_by_id(tray.id());
        }
    }
}
