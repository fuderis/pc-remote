use crate::{ prelude::*, Bind, Action, Emulator, Key };

use std::{ io::{ BufReader, BufRead }, process::Command };

/// The PC remote controller
#[derive(Debug)]
pub struct Controller {
    emulator: Mutex<Emulator>,
    mouse_mode_on: Mutex<bool>,
}

impl Controller {
    /// Creates a new PC remote controller
    pub async fn new() -> Result<Self> {
        Ok(Self {
            emulator: Mutex::new(Emulator::new().await?),
            mouse_mode_on: Mutex::new(false),
        })
    }

    /// Runs COM port listenning
    pub async fn listen(self) -> Result<()> {
        // spawn handler:
        tokio::spawn(async move {
            loop {
                let Config { com_port, baud_rate, .. } = CONFIG.lock().await.clone();
        
                // init COM port:
                let port = serialport::new(&fmt!("COM{}", com_port), baud_rate)
                    .timeout(Duration::from_millis(10))
                    .open()
                    .map_err(|e| { err!("Failed to get COM port: {e}"); e });

                if let Ok(port) = port {
                    if let Err(e) = self.listen_handler(port).await {
                        err!("PC Controller panicked with error: {e}");
                    } else {
                        break;
                    }
                } else {
                    sleep(Duration::from_millis(200)).await;
                }
            }
        });

        Ok(())
    }

    /// COM Port listenning handler
    async fn listen_handler(&self, com_port: Box<dyn serialport::SerialPort>) -> Result<()> {
        let mut com_reader = BufReader::new(com_port);
        let mut line = String::new();

        let mut last_code = str!();
        let mut last_action = Instant::now();
        let mut last_update = Instant::now();

        let action_interval = Duration::from_millis(1000);
        let update_interval = Duration::from_millis(5000);
        let repeat_timeout = Duration::from_millis(25);
        
        info!("Reading remote inputs..");
        
        loop {
            line.clear();
            
            // checking timers:
            if last_action.elapsed() >= action_interval {
                if last_update.elapsed() >= update_interval {
                    self.emulator.lock().await.media.update_info().await
                        .unwrap_or_else(|e| err!("Failed to update media info: {e}"));
                    last_update = Instant::now();
                }
            } else {
                last_update = Instant::now();
            }

            // reading remote code:
            match com_reader.read_line(&mut line) {
                Ok(0) => continue,
                Ok(_) => {
                    let code = line.trim().to_owned();

                    // validate remote code:
                    if code.is_empty(){ continue }
                    else if !code.starts_with("0x") {
                        return Err(Error::InvalidRemoteCode.into());
                    }
                    
                    // repeat last bind:
                    else if code == "0xFFFFFFFF" {
                        if last_action.elapsed() < repeat_timeout { continue; }
                        if last_code.is_empty() { continue }

                        if let Err(e) = self.execute_bind(&last_code, true).await {
                            err!("Error with executing bind: {e}");
                        }
                    }
                    // execute bind:
                    else {
                        if code != last_code {
                            info!("Pressed button '{code}'");
                            last_code = code;

                            emit_event("pressed-code", map!{
                                "code": last_code.clone()
                            });
                        }
                        
                        if let Err(e) = self.execute_bind(&last_code, false).await {
                            err!("Error with executing bind: {e}");
                        }
                    }

                    last_action = Instant::now();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => {
                    err!("Error with reading 'COM{port}' port: {e}", port = CONFIG.lock().await.com_port);
                    return Err(e.into());
                }
            }
        }
    }

    /// Execute remote bind
    async fn execute_bind(&self, code: &str, is_repeating: bool) -> Result<()> {
        for (_id, bind) in &CONFIG.lock().await.binds {
            if code == bind.code {
                self.handle_bind(&bind, is_repeating).await?;
            }
        }
        
        Ok(())
    }

    const MOVE_MOUSE_STEP: [i32; 2]          = [30, 70];
    const SCROLL_PAGE_STEP: [i32; 2]         = [2, 5];
    const CHANGE_VOLUME_UP_STEP: [i32; 2]    = [2, 5];
    const CHANGE_VOLUME_DOWN_STEP: [i32; 2]  = [1, 3];

    /// Handle remote bind
    async fn handle_bind(&self, bind: &Bind, is_repeating: bool) -> Result<()> {
        if is_repeating && !bind.repeat { return Ok(()) }
        
        match &bind.action {
            //               M E D I A:

            Action::MediaSwitchDevice => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }
                
                let media = &mut self.emulator.lock().await.media;
                media.switch_next_audio_device().await?;

                info!("Switched to device '{}'", &media.get_active().unwrap().name);
            }

            Action::MediaPlayPause => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }
                
                let keyboard = &self.emulator.lock().await.keyboard;
                keyboard.press(&Key::PlayPause, false).await?;

                info!("Switched media play/pause");
            }

            Action::MediaNextTrack => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }
                
                let keyboard = &self.emulator.lock().await.keyboard;
                keyboard.press(&Key::NextTrack, false).await?;

                info!("Switched to next track");
            }

            Action::MediaPrevTrack => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }
                
                let keyboard = &self.emulator.lock().await.keyboard;
                keyboard.press(&Key::PrevTrack, false).await?;

                info!("Switched to previous track");
            }

            Action::MediaStop => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }
                
                let keyboard = &self.emulator.lock().await.keyboard;
                keyboard.press(&Key::Stop, false).await?;

                info!("Media playing is stopped");
            }

            Action::MediaMuteUnmute => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }
                
                /* self.press_shortcut(&[Key::Mute]).await?; */
                
                let media = &self.emulator.lock().await.media;
                media.switch_audio_mute().await?;
                media.switch_micro_mute().await?;

                info!("Switched audio & microphone mute/unmute");
            }

            Action::MediaVolumeUp => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }
                
                /* self.press_shortcut(&[Key::VolumeUp]).await?;
                info!("Increasing sound volume.."); */
                
                let step: i32 = Self::CHANGE_VOLUME_UP_STEP[is_repeating as usize];
                let media = &mut self.emulator.lock().await.media;
                let volume = media.increase_audio_volume(step).await?;

                info!("Set audio volume to {volume}%");

            }

            Action::MediaVolumeDown => {
                if *self.mouse_mode_on.lock().await { return Ok(()) }

                /* self.press_shortcut(&[Key::VolumeDown]).await?;
                info!("Decreasing sound volume.."); */
                
                let step: i32 = Self::CHANGE_VOLUME_DOWN_STEP[is_repeating as usize];
                let media = &mut self.emulator.lock().await.media;
                let volume = media.decrease_audio_volume(step).await?;

                info!("Set audio volume to {volume}%");
            }

            //               K E Y B O A R D:

            Action::KeyboardPress(keys) => {
                self.press_shortcut(&keys).await?;

                info!("Pressed keys: {keys:?}");
            }
            
            //               M O U S E:

            Action::MouseOnOff => {
                let mut lock = self.mouse_mode_on.lock().await;
                *lock = !*lock;

                info!("Mouse mode is {}!", if *lock {"enabled"}else{"disabled"} );
            }

            Action::MouseLeft => {
                if !*self.mouse_mode_on.lock().await { return Ok(()) }
                
                let step: i32 = Self::MOVE_MOUSE_STEP[is_repeating as usize];
                let mouse = &self.emulator.lock().await.mouse;
                mouse.move_x(-step).await?;

                info!("Move mouse left by {step}px", );
            }

            Action::MouseRight => {
                if !*self.mouse_mode_on.lock().await { return Ok(()) }
                
                let step: i32 = Self::MOVE_MOUSE_STEP[is_repeating as usize];
                let mouse = &self.emulator.lock().await.mouse;
                mouse.move_x(step).await?;

                info!("Move mouse right by {step}px", );
            }

            Action::MouseUp => {
                if !*self.mouse_mode_on.lock().await { return Ok(()) }
                
                let step: i32 = Self::MOVE_MOUSE_STEP[is_repeating as usize];
                let mouse = &self.emulator.lock().await.mouse;
                mouse.move_y(-step).await?;

                info!("Move mouse up by {step}px", );
            }

            Action::MouseDown => {
                if !*self.mouse_mode_on.lock().await { return Ok(()) }
                
                let step: i32 = Self::MOVE_MOUSE_STEP[is_repeating as usize];
                let mouse = &self.emulator.lock().await.mouse;
                mouse.move_y(step).await?;

                info!("Move mouse down by {step}px", );
            }

            Action::MouseClick => {
                if !*self.mouse_mode_on.lock().await { return Ok(()) }
                
                let mouse = &self.emulator.lock().await.mouse;
                mouse.press_left(false).await?;
                
                info!("Pressed left mouse button");
            }

            Action::MouseScrollUp => {
                if !*self.mouse_mode_on.lock().await { return Ok(()) }
                
                let step: i32 = Self::SCROLL_PAGE_STEP[is_repeating as usize];
                let mouse = &self.emulator.lock().await.mouse;
                mouse.scroll_y(-step).await?;

                info!("Scroll up by {step}px");
            }

            Action::MouseScrollDown => {
                if !*self.mouse_mode_on.lock().await { return Ok(()) }
                
                let step: i32 = Self::SCROLL_PAGE_STEP[is_repeating as usize];
                let mouse = &self.emulator.lock().await.mouse;
                mouse.scroll_y(step).await?;

                info!("Scroll down by {step}px");
            }

            //               W E B - B R O W S E R:

            Action::BrowserOpen(url) => {
                let url = if url.starts_with("https://") || url.starts_with("http://") { url }else{ &fmt!("https://{url}") };
                webbrowser::open(url)?;

                info!("Opening website: '{url}'..");
            }

            Action::BrowserOpenNewTab => {
                self.press_shortcut(&[Key::Ctrl, Key::T]).await?;
                info!("Opening a new browser tab");
            }

            Action::BrowserReopenTab => {
                self.press_shortcut(&[Key::Ctrl, Key::Shift, Key::T]).await?;
                info!("Reopening a closed browser tab");
            }

            Action::BrowserSwitchTab => {
                self.press_shortcut(&[Key::Ctrl, Key::Tab]).await?;
                info!("Switched to a next browser tab");
            }

            Action::BrowserCloseTab => {
                self.press_shortcut(&[Key::Ctrl, Key::W]).await?;
                info!("Closing a browser tab");
            }

            Action::BrowserHistoryBack => {
                self.press_shortcut(&[Key::Alt, Key::Left]).await?;
                info!("Moving in browser history back");
            }

            Action::BrowserHistoryForward => {
                self.press_shortcut(&[Key::Alt, Key::Right]).await?;
                info!("Moving in browser history forward");
            }

            Action::BrowserBookmarkPage => {
                self.press_shortcut(&[Key::Ctrl, Key::D]).await?;
                self.press_shortcut(&[Key::Enter]).await?;
                info!("Bookmarking a browser page");
            }

            Action::BrowserZoomIn => {
                self.press_shortcut(&[Key::Ctrl, Key::Equal]).await?;
                info!("Zoom in a browser page");
            }

            Action::BrowserZoomOut => {
                self.press_shortcut(&[Key::Ctrl, Key::Minus]).await?;
                info!("Zoom out a browser page");
            }

            //               W I N D O W S:

            Action::WindowsExit => {
                let status = Command::new("shutdown")
                    .arg("/l") // /l — выйти из системы (logoff)
                    .status()?;

                if status.success() {
                    info!("User logged off (exit from Windows session)");
                } else {
                    err!("Failed to log off user");
                }
            }

            Action::WindowsSleep => {
                let status = Command::new("rundll32.exe")
                    .arg("powrprof.dll,SetSuspendState")
                    .arg("0")
                    .arg("1")
                    .arg("0")
                    .status()?;

                if status.success() {
                    info!("PC switched to sleep mode");
                } else {
                    err!("Failed switch PC to sleep mode");
                }
            }

            Action::WindowsPowerOff => {
                let status = Command::new("shutdown")
                    .arg("/s") // /s — выключить компьютер
                    .arg("/t")
                    .arg("0")  // /t 0 — задержка 0 секунд
                    .status()?;

                if status.success() {
                    info!("PC powered off");
                } else {
                    err!("Failed to power off PC");
                }
            }

            Action::WindowsSwitchTab => {
                self.press_shortcut(&[Key::Ctrl, Key::Tab]).await?;

                info!("Windows tab switched");
            }
        }
        
        Ok(())
    }

    /// Press a keyboard shortcut
    async fn press_shortcut(&self, keys: &[Key]) -> Result<()> {
        let keyboard = &self.emulator.lock().await.keyboard;
        
        keyboard.press_all(&keys, true).await?;
        sleep(Duration::from_millis(100)).await;
        keyboard.release_all(&keys).await?;

        Ok(())
    }
}
