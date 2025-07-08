use crate::prelude::*;
use super::Action;

/// The remote bind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bind {
    pub id: String,
    pub code: String,
    pub action: Action,
    pub repeat: bool
}

impl ::std::default::Default for Bind {
    fn default() -> Self {
        Self {
            id: uniq_id(),
            code: str!("FFFFFF"),
            action: Action::MediaPlayPause,
            repeat: false,
        }
    }
}
