use crate::{ prelude::*, Key };

/// The remote bind action
#[derive(Debug, Display, Clone, Serialize, Deserialize)]
pub enum Action {
    MediaSwitchDevice,
    MediaPlayPause,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStop,
    MediaMuteUnmute,
    MediaVolumeUp,
    MediaVolumeDown,

    #[display = "KeyboardPress"]
    KeyboardPress(Vec<Key>),

    MouseOnOff,
    MouseLeft,
    MouseRight,
    MouseUp,
    MouseDown,
    MouseClick,
    MouseScrollUp,
    MouseScrollDown,

    #[display = "BrowserOpen"]
    BrowserOpen(String),
    BrowserOpenNewTab,
    BrowserReopenTab,
    BrowserSwitchTab,
    BrowserCloseTab,
    BrowserHistoryBack,
    BrowserHistoryForward,
    BrowserBookmarkPage,
    BrowserZoomIn,
    BrowserZoomOut,

    WindowsExit,
    WindowsSleep,
    WindowsPowerOff,
    WindowsSwitchTab,
}

impl Action {
    /// Returns all exists actions
    pub fn get_all() -> Vec<Self> {
        vec![
            Self::KeyboardPress(vec![]),
            Self::BrowserOpen(str!()),
            
            Self::MediaSwitchDevice,
            Self::MediaPlayPause,
            Self::MediaNextTrack,
            Self::MediaPrevTrack,
            Self::MediaStop,
            Self::MediaMuteUnmute,
            Self::MediaVolumeUp,
            Self::MediaVolumeDown,

            Self::MouseOnOff,
            Self::MouseLeft,
            Self::MouseRight,
            Self::MouseUp,
            Self::MouseDown,
            Self::MouseClick,
            Self::MouseScrollUp,
            Self::MouseScrollDown,

            Self::BrowserOpenNewTab,
            Self::BrowserReopenTab,
            Self::BrowserSwitchTab,
            Self::BrowserCloseTab,
            Self::BrowserHistoryBack,
            Self::BrowserHistoryForward,
            Self::BrowserBookmarkPage,
            Self::BrowserZoomIn,
            Self::BrowserZoomOut,

            Self::WindowsExit,
            Self::WindowsSleep,
            Self::WindowsPowerOff,
            Self::WindowsSwitchTab,
        ]
    }
}
