#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use app::{ prelude::*, Controller, Bind, elementor };
use tauri::Manager;

/// Get binds list from config
#[tauri::command]
async fn get_binds() -> StdResult<Vec<String>, String> {    
    let mut binds = vec![];
    
    for (id, bind) in &CONFIG.lock().await.binds {
        let bind_html = elementor::generate_bind(&id, &bind).map_err(|e| e.to_string())?;
        binds.push(bind_html);
    }

    Ok(binds)
}

/// Add a new bind
#[tauri::command]
async fn add_bind() -> StdResult<String, String> {    
    let bind = Bind::default();

    // generate bind html:
    let bind_html = elementor::generate_bind(&bind.id, &bind).map_err(|e| e.to_string())?;

    // save bind to config:
    let mut config = CONFIG.lock().await;
    config.binds.insert(bind.id.clone(), bind);
    config.save().map_err(|e| e.to_string())?;

    Ok(bind_html)
}

/// Update bind data
#[tauri::command]
async fn update_bind(data: Bind) -> StdResult<String, String> {    
    let bind = data;
    
    // generate bind html:
    let bind_html = elementor::generate_bind(&bind.id, &bind).map_err(|e| e.to_string())?;
    
    // save bind to config:
    let mut config = CONFIG.lock().await;
    if let Some(bind_mut) = config.binds.get_mut(&bind.id) {
        *bind_mut = bind;
        config.save().map_err(|e| e.to_string())?;
    }

    Ok(bind_html)
}

/// Remove bind
#[tauri::command]
async fn remove_bind(id: String) -> StdResult<(), String> {    
    // remove bind from config:
    let mut config = CONFIG.lock().await;
    let _ = config.binds.remove(&id);
    config.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // init logger:
    LOGGER.init()?;

    // init & start pc remote controller:
    Controller::new().await?
        .listen().await?;
    
    // run ui:
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_binds,
            add_bind,
            update_bind,
            remove_bind,
        ])
        .setup(|app| {
            let app_handle = app.app_handle().clone();
            let window = app_handle.get_webview_window("main").unwrap();

            // init app handler:
            *APP_HANDLE.lock().unwrap() = Some(app_handle.clone());
            
            // init tray-icon:
            *SYSTEM_TRAY.lock().unwrap() = Some(Tray::new());
            
            // window events:
            window.on_window_event(move |event| {
                let window = app_handle.get_webview_window("main").unwrap();
                
                match event {
                    // if window closes:
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        
                        // saving logs:
                        LOGGER.save().unwrap();

                        // removing tray:
                        if let Some(tray) = SYSTEM_TRAY.lock().unwrap().take() {
                            tray.remove();
                        }

                        // closing program:
                        app_handle.exit(0);
                    }

                    // if window minimized:
                    tauri::WindowEvent::Resized(_) => {
                        if window.is_minimized().unwrap_or(false) {
                            window.hide().unwrap();
                        }
                    }

                    _ => {}
                }
            });
            
            Ok(())
        })
        .plugin(tauri_plugin_prevent_default::Builder::new()
            .with_flags(tauri_plugin_prevent_default::Flags::empty())
            .build()
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None
        ))
        .run(tauri::generate_context!())?;

    Ok(())
}
