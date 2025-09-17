use macron::{ Display, From, Error };

/// The std result
pub type StdResult<T, E> = std::result::Result<T, E>;
/// The result alias
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// The application error
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[from]
    String(String),

    #[from]
    Logger(log::SetLoggerError),

    #[display = "Invalid remote code format: it should starts with '0x..'"]
    InvalidRemoteCode,

    #[display = "No audio devices set"]
    FoundNoDevices,

    #[display = "Failed to get devices list"]
    FailedReadDevicesList,

    #[display = "Failed to switch to device {0}"]
    FailedSwitchToDevice(String),

    #[display = "Found no audio device named as '{0}'"]
    DeviceNotFound(String),

    #[display = "Found no active audio device"]
    ActiveDeviceNotFound,

    #[display = "Failed to change audio volume"]
    FailedSetVolume,
}
