use crate::prelude::*;
use super::{ Media, Keyboard, Mouse };

use enigo::{ Enigo, Settings };

/// The device emulators controller
#[derive(Debug, Clone)]
pub struct Emulator {
    #[allow(dead_code)]
    enigo: Arc<Mutex<Enigo>>,
    
    pub media: Media,
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl Emulator {
    /// Creates a new device emulators controller
    pub async fn new() -> Result<Self> {
        let settings = Settings::default();
        let enigo = Arc::new(Mutex::new(Enigo::new(&settings)?));
        
        Ok(Self {
            media: Media::new(
                path!("/bin"),
                if CONFIG.lock().await.steel_series_filter {
                    Some(|name, kind| !name.contains("SteelSeries Sonar") || kind.is_micro())
                }else{
                    None
                }
            ).await?,
            keyboard: Keyboard::new(enigo.clone()),
            mouse: Mouse::new(enigo.clone()),

            enigo,
        })
    }
}
