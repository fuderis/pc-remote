use crate::prelude::*;
use std::process::Command;
use csv::Reader;

pub type DeviceFilter = fn(&str, &DeviceKind) -> bool;

/// The device kind
#[derive(Debug, Display, Clone, Eq, PartialEq)]
pub enum DeviceKind {
    Audio,
    Micro
}

impl DeviceKind {
    /// check for media device type
    pub fn is_audio(&self) -> bool {
        if let Self::Audio = self { true }else{ false }
    }

    /// check for microphone device type
    pub fn is_micro(&self) -> bool {
        if let Self::Micro = self { true }else{ false }
    }
}

/// The media device
#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub kind: DeviceKind,
    pub is_active: bool,
}

/// The media controller
#[derive(Debug, Clone)]
pub struct Media {
    nircmd_path: PathBuf,
    svv_path: PathBuf,
    svcl_path: PathBuf,

    device_filter: Option<DeviceFilter>,
    devices: Vec<Device>,
    active: Option<Device>,
    volume: i32,
}

impl Media {
    /// Creates a new audio controller
    pub async fn new<P: AsRef<Path>>(bin_path: P, device_filter: Option<DeviceFilter>) -> Result<Self> {        
        let bin_path = bin_path.as_ref();
        
        let mut this = Self {
            nircmd_path: bin_path.join("nircmd/nircmd.exe"),
            svv_path: bin_path.join("svv/SoundVolumeView.exe"),
            svcl_path: bin_path.join("svcl/svcl.exe"),

            device_filter,
            devices: vec![],
            active: None,
            volume: 0,
        };

        if let Err(e) = this.update_info().await {
            match e.downcast_ref::<Error>() {
                Some(e) => {
                    match e {
                        Error::ActiveDeviceNotFound => {
                            let _ = this.switch_next_audio_device().await?;
                        },

                        _ => {}
                    }
                }
                
                _ => return Err(e),
            }
        }
        
        Ok(this)
    }
    
    // ______________________________ UPDATE INFO: ____________________________________

    /// Updates full info
    pub async fn update_info(&mut self) -> Result<()> {
        self.update_devices_info().await?;
        self.update_volume_info().await?;
        
        // info!("Media info updated!");

        Ok(())
    }

    /// Updates media devices list
    pub async fn update_devices_info(&mut self) -> Result<()> {
        let (active, devices) = self.get_all_devices().await?;

        self.devices = devices;
        self.active = active;

        Ok(())
    }

    /// Updates media volume value
    pub async fn update_volume_info(&mut self) -> Result<()> {
        self.volume = self.get_audio_volume().await?;

        Ok(())
    }

    // ______________________________ FAST METHODS: ____________________________________

    /// Returns devices list
    pub fn get_devices(&self) -> &Vec<Device> {
        &self.devices
    }

    /// Returns active audio device (fast)
    pub fn get_active(&self) -> Option<&Device> {
        self.active.as_ref()
    }

    /// Get audio devices list
    pub async fn get_audio_devices(&self) -> Result<Vec<Device>> {
        Ok(self.devices.iter().filter(|device| device.kind.is_audio()).map(|device| device.clone()).collect::<Vec<_>>())
    }

    /// Get microphone devices list
    pub async fn get_micro_devices(&self) -> Result<Vec<Device>> {
        Ok(self.devices.iter().filter(|device| device.kind.is_micro()).map(|device| device.clone()).collect::<Vec<_>>())
    }

    /// Get a current media device
    pub async fn get_active_audio_device(&self) -> Result<Device> {
        for device in self.get_audio_devices().await? {
            if device.is_active {
                return Ok(device);
            }
        }

        Err(Error::ActiveDeviceNotFound.into())
    }

    /// Get a current microphone device
    pub async fn get_active_micro_device(&self) -> Result<Device> {
        for device in self.get_micro_devices().await? {
            if device.is_active {
                return Ok(device);
            }
        }

        Err(Error::ActiveDeviceNotFound.into())
    }

    // ______________________________ SLOW METHODS: ____________________________________

    /// Gets all devices list
    pub async fn get_all_devices(&self) -> Result<(Option<Device>, Vec<Device>)> {
        let output = Command::new(&self.svv_path)
            .arg("/scomma")
            .output()
            .map_err(|_| Error::FailedReadDevicesList)?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout_clean = stdout.trim_start_matches('\u{feff}');
        
        let mut reader = Reader::from_reader(stdout_clean.as_bytes());
        let mut devices = vec![];
        let mut active = None;
        
        for result in reader.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    err!("CSV parsing error: {}", e);
                    continue;
                }
            };

            if record.get(1) != Some("Device") { continue }
            
            let kind = match &record.get(2).map(|s| s.to_string()).unwrap_or_default()[..] {
                "Render" => DeviceKind::Audio,
                "Capture" => DeviceKind::Micro,
                _ => continue
            };
            
            let is_active = record.get(7) == Some("Active") && (record.get(5) == Some("Render") || record.get(5) == Some("Capture"));

            if let Some(name) = record.get(0) {
                // filter device by name:
                if let Some(filter) = &self.device_filter {
                    if !(filter)(&name, &kind) { continue }
                }
                
                // skip empty names
                if name.is_empty() { continue; }
                
                let device = Device {
                    name: name.to_string(),
                    kind: kind.clone(),
                    is_active,
                };
                
                if is_active {
                    if let DeviceKind::Audio = kind {
                        active = Some(device.clone());
                    }
                }
                
                devices.push(device);
            }
        }

        if devices.is_empty() {
            return Err(Error::FoundNoDevices.into());
        }

        Ok((active, devices))
    }

    /// Set active audio device
    pub async fn set_audio_device(&mut self, name: &str) -> Result<Device> {
        for device in &self.get_audio_devices().await? {
            if device.name == name {
                let status = Command::new(&self.svv_path)
                    .arg("/SetDefault")
                    .arg(name)
                    .arg("all")  // all = Console, Multimedia, Communications
                    .status()?;
                
                if status.success() {
                    self.update_info().await?;
                    
                    return Ok(self.active.clone().unwrap());
                } else {
                    return Err(Error::FailedSwitchToDevice(device.name.clone()).into())
                }
            }
        }

        Err(Error::DeviceNotFound(name.to_owned()).into())
    }

    /// Switch to next audio device
    pub async fn switch_next_audio_device(&mut self) -> Result<Device> {
        let devices = self.get_audio_devices().await?;
        if devices.is_empty() {
            return Err(Error::FoundNoDevices.into());
        }

        let current_index = devices.iter().position(|d| d.is_active).unwrap_or(0);
        let next_index = (current_index + 1) % devices.len();

        self.set_audio_device(&devices[next_index].name).await
    }

    /// Switch to previous audio device
    pub async fn switch_prev_audio_device(&mut self) -> Result<Device> {
        let devices = self.get_audio_devices().await?;
        if devices.is_empty() {
            return Err(Error::FoundNoDevices.into());
        }

        let current_index = devices.iter().position(|d| d.is_active).unwrap_or(0);
        let prev_index = if current_index == 0 {
            devices.len() - 1
        } else {
            current_index - 1
        };

        self.set_audio_device(&devices[prev_index].name).await
    }

    /// Set active microphone device
    pub async fn set_micro_device(&mut self, name: &str) -> Result<Device> {
        for device in &self.get_micro_devices().await? {
            if device.name == name {
                let status = Command::new(&self.svv_path)
                    .arg("/SetDefault")
                    .arg(name)
                    .arg("all")  // all = Console, Multimedia, Communications
                    .status()?;
                
                if status.success() {
                    self.update_info().await?;
                    
                    return Ok(self.active.clone().unwrap());
                } else {
                    return Err(Error::FailedSwitchToDevice(device.name.clone()).into())
                }
            }
        }

        Err(Error::DeviceNotFound(name.to_owned()).into())
    }

    /// Switch to next microphone device
    pub async fn switch_next_micro_device(&mut self) -> Result<Device> {
        let devices = self.get_micro_devices().await?;
        if devices.is_empty() {
            return Err(Error::FoundNoDevices.into());
        }

        let current_index = devices.iter().position(|d| d.is_active).unwrap_or(0);
        let next_index = (current_index + 1) % devices.len();

        self.set_micro_device(&devices[next_index].name).await
    }

    /// Switch to previous microphone device
    pub async fn switch_prev_micro_device(&mut self) -> Result<Device> {
        let devices = self.get_micro_devices().await?;
        if devices.is_empty() {
            return Err(Error::FoundNoDevices.into());
        }

        let current_index = devices.iter().position(|d| d.is_active).unwrap_or(0);
        let prev_index = if current_index == 0 {
            devices.len() - 1
        } else {
            current_index - 1
        };

        self.set_micro_device(&devices[prev_index].name).await
    }

    /// Get current audio volume (0-100)
    pub async fn get_audio_volume(&self) -> Result<i32> {
        let device = self.active.as_ref().ok_or(Error::ActiveDeviceNotFound)?;

        let status = Command::new(&self.svcl_path)
            .arg("/GetPercent")
            .arg(&device.name)
            .status()?;

        let code = status.code().unwrap_or(0);
        
        let volume = code / 10;

        Ok(volume)
    }

    /// Set audio volume (0-100)
    pub async fn set_audio_volume(&mut self, volume: i32) -> Result<i32> {
        let sys_volume = (volume * 65535) / 100;
        
        let status = Command::new(&self.nircmd_path)
            .arg("setsysvolume")  // Исправленная команда!
            .arg(sys_volume.to_string())
            .status()?;
        
        if status.success() {
            self.volume = volume;
        } else {
            return Err(Error::FailedSetVolume.into());
        }
        
        Ok(self.volume)
    }

    /// Increase audio volume by delta
    pub async fn increase_audio_volume(&mut self, delta: i32) -> Result<i32> {
        let new_volume = (self.volume + delta).min(100);
        self.set_audio_volume(new_volume).await
    }

    /// Decrease audio volume by delta
    pub async fn decrease_audio_volume(&mut self, delta: i32) -> Result<i32> {
        let new_volume = self.volume.saturating_sub(delta);
        self.set_audio_volume(new_volume).await
    }

    /// Toggle audio mute/unmute
    pub async fn switch_audio_mute(&self) -> Result<()> {
        let device = self.active.as_ref().ok_or(Error::ActiveDeviceNotFound)?;

        let status = Command::new(&self.svv_path)
            .arg("/Switch")
            .arg(&device.name)
            .status()?;
        
        if status.success() {
            info!("Media device '{}' muted/unmuted", device.name);
        } else {
            err!("Failed to toggle mute");
        }
        Ok(())
    }

    /// Toggle microphone mute/unmute
    pub async fn switch_micro_mute(&self) -> Result<()> {
        let device = self.get_active_micro_device().await?;

        let status = Command::new(&self.svv_path)
            .arg("/Switch")
            .arg(&device.name)
            .status()?;
        
        if status.success() {
            info!("Media device '{}' muted/unmuted", device.name);
        } else {
            err!("Failed to toggle microphone mute");
        }
        Ok(())
    }

    /// Check if audio device is muted
    pub async fn audio_is_muted(&self) -> Result<bool> {
        let device = self.active.as_ref().ok_or(Error::ActiveDeviceNotFound)?;

        let status = Command::new(&self.svcl_path)
            .arg("/GetMute")
            .arg(&device.name)
            .status()?;
        
        // Exit code: 1 = muted, 0 = not muted
        let is_muted = status.code().map(|code| code == 1).unwrap_or(false);
        Ok(is_muted)
    }

    /// Check if microphone is muted
    pub async fn micro_is_muted(&self) -> Result<bool> {
        let device = self.get_active_micro_device().await?;

        let status = Command::new(&self.svcl_path)
            .arg("/GetMute")
            .arg(&device.name)
            .status()?;
        
        // Exit code: 1 = muted, 0 = not muted
        let is_muted = status.code().map(|code| code == 1).unwrap_or(false);
        Ok(is_muted)
    }
}
