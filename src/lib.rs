pub mod error;       pub use error::{ StdResult, Result, Error };
pub mod logger;      pub use logger::Logger;
pub mod config;      pub use config::Config;
pub mod tray;        pub use tray::Tray;
pub mod prelude;     use prelude::*;

pub mod binds;       pub use binds::{ Bind, Action };
pub mod emulator;    pub use emulator::{ Emulator, Keyboard, Key, Mouse, Media, Device, DeviceKind };
pub mod controller;  pub use controller::Controller;
pub mod elementor;

pub static LOGGER: Lazy<Logger> = Lazy::new(|| Logger::new("/logs", 20));
pub static CONFIG: Lazy<Arc<Mutex<Config>>> = Lazy::new(|| Config::new("/config.json").unwrap_or_default());

pub static APP_HANDLE: Lazy<Arc<StdMutex<Option<tauri::AppHandle>>>> = Lazy::new(|| Arc::new(StdMutex::new(None)));
pub static SYSTEM_TRAY: Lazy<Arc<StdMutex<Option<Tray>>>> = Lazy::new(|| Arc::new(StdMutex::new(None)));

/// Send event to frontend
pub fn emit_event(event: &str, payload: impl serde::Serialize + Clone) {
    let guard = APP_HANDLE.lock().unwrap();

    if let Some(app) = guard.as_ref() {
        app.emit(event, payload).ok();
    }
}

/// Generates an unique ID
pub fn uniq_id() -> String {
    use std::time::{ SystemTime, UNIX_EPOCH };
    
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let random: u16 = rand::random();
    format!("{}{:04x}", millis, random)
}
