use crate::prelude::*;
use enigo::{ Enigo, Key as EnigoKey, Keyboard as EnigoKeyboard, Direction };

/// The keyboard key
#[derive(Debug, Display, Clone, Serialize, Deserialize)]
pub enum Key {
    // Characters:
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Functional keys:
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Numbers:
    N0, N1, N2, N3, N4, N5, N6, N7, N8, N9,

    // Symbols:
    Plus,
    Minus,
    Equal,
    Multiply,
    Divide,
    
    // Special keys:
    Esc,
    Tab,
    CapsLock,
    Shift,
    Ctrl,
    Alt,
    Win,
    Space,
    Enter,
    Backspace,
    Delete,

    // Arrows:
    Left,
    Right,
    Up,
    Down,

    // Media:
    PlayPause,
    PrevTrack,
    NextTrack,
    Stop,
    VolumeUp,
    VolumeDown,
    Mute,
    // MediaMicroMute,

    // Numbers:
    #[serde(untagged)]
    #[display = "{0}"]
    Num(u32),
    
    // Unicode Symbols:
    #[serde(untagged)]
    #[display = "{0}"]
    Unicode(char),
}

impl ::std::convert::Into<EnigoKey> for Key {
    fn into(self) -> EnigoKey {
        match self {
            // Characters:
            Self::A => EnigoKey::A,
            Self::B => EnigoKey::B,
            Self::C => EnigoKey::C,
            Self::D => EnigoKey::D,
            Self::E => EnigoKey::E,
            Self::F => EnigoKey::F,
            Self::G => EnigoKey::G,
            Self::H => EnigoKey::H,
            Self::I => EnigoKey::I,
            Self::J => EnigoKey::J,
            Self::K => EnigoKey::K,
            Self::L => EnigoKey::L,
            Self::M => EnigoKey::M,
            Self::N => EnigoKey::N,
            Self::O => EnigoKey::O,
            Self::P => EnigoKey::P,
            Self::Q => EnigoKey::Q,
            Self::R => EnigoKey::R,
            Self::S => EnigoKey::S,
            Self::T => EnigoKey::T,
            Self::U => EnigoKey::U,
            Self::V => EnigoKey::V,
            Self::W => EnigoKey::W,
            Self::X => EnigoKey::X,
            Self::Y => EnigoKey::Y,
            Self::Z => EnigoKey::Z,

            // Functional keys:
            Self::F1 => EnigoKey::F1,
            Self::F2 => EnigoKey::F2,
            Self::F3 => EnigoKey::F3,
            Self::F4 => EnigoKey::F4,
            Self::F5 => EnigoKey::F5,
            Self::F6 => EnigoKey::F6,
            Self::F7 => EnigoKey::F7,
            Self::F8 => EnigoKey::F8,
            Self::F9 => EnigoKey::F9,
            Self::F10 => EnigoKey::F10,
            Self::F11 => EnigoKey::F11,
            Self::F12 => EnigoKey::F12,

            // Numbers:
            Self::N0 => EnigoKey::Num0,
            Self::N1 => EnigoKey::Num1,
            Self::N2 => EnigoKey::Num2,
            Self::N3 => EnigoKey::Num3,
            Self::N4 => EnigoKey::Num4,
            Self::N5 => EnigoKey::Num5,
            Self::N6 => EnigoKey::Num6,
            Self::N7 => EnigoKey::Num7,
            Self::N8 => EnigoKey::Num8,
            Self::N9 => EnigoKey::Num9,

            // Symbols:
            Self::Plus => EnigoKey::Unicode('+'),
            Self::Minus => EnigoKey::Unicode('-'),
            Self::Equal => EnigoKey::Unicode('='),
            Self::Multiply => EnigoKey::Unicode('*'),
            Self::Divide => EnigoKey::Unicode('/'),
            
            // Special keys:
            Self::Esc => EnigoKey::Escape,
            Self::Tab => EnigoKey::Tab,
            Self::CapsLock => EnigoKey::CapsLock,
            Self::Shift => EnigoKey::Shift,
            Self::Ctrl => EnigoKey::Control,
            Self::Alt => EnigoKey::Alt,
            Self::Win => EnigoKey::Meta, // usually Windows == Meta
            Self::Space => EnigoKey::Space,
            Self::Enter => EnigoKey::Return,
            Self::Backspace => EnigoKey::Backspace,
            Self::Delete => EnigoKey::Delete,

            // Media:
            Self::PlayPause => EnigoKey::MediaPlayPause,
            Self::PrevTrack => EnigoKey::MediaPrevTrack,
            Self::NextTrack => EnigoKey::MediaNextTrack,
            Self::Stop => EnigoKey::MediaStop,
            Self::VolumeUp => EnigoKey::VolumeUp,
            Self::VolumeDown => EnigoKey::VolumeDown,
            Self::Mute => EnigoKey::VolumeMute,
            // Self::MicroMute => EnigoKey::MicMute,

            // Arrows:
            Self::Left => EnigoKey::LeftArrow,
            Self::Right => EnigoKey::RightArrow,
            Self::Up => EnigoKey::UpArrow,
            Self::Down => EnigoKey::DownArrow,

            // Numbers:
            Self::Num(num) => match num {
                0 => EnigoKey::Num0,
                1 => EnigoKey::Num1,
                2 => EnigoKey::Num2,
                3 => EnigoKey::Num3,
                4 => EnigoKey::Num4,
                5 => EnigoKey::Num5,
                6 => EnigoKey::Num6,
                7 => EnigoKey::Num7,
                8 => EnigoKey::Num8,
                9 => EnigoKey::Num9,
                _ => EnigoKey::Other(num)
            }
            
            // Other:
            Self::Unicode(ch) => EnigoKey::Unicode(ch),
        }
    }
}

/// The keyboard emulator
#[derive(Debug, Clone)]
pub struct Keyboard {
    enigo: Arc<Mutex<Enigo>>,
}

impl Keyboard {
    /// Creates new keyboard emulator
    pub fn new(enigo: Arc<Mutex<Enigo>>) -> Self {
        Self {
            enigo,
        }
    }

    /// Press a keyboard key
    pub async fn press(&self, key: &Key, hold: bool) -> Result<()> {
        self.enigo.lock().await.key(key.clone().into(), if hold { Direction::Press }else{ Direction::Click })?;

        Ok(())
    }

    /// Press an keyboard keys at the same time
    pub async fn press_all(&self, keys: &[Key], hold: bool) -> Result<()> {
        for key in keys {
            self.enigo.lock().await.key(key.clone().into(), if hold { Direction::Press }else{ Direction::Click })?;
        }

        Ok(())
    }

    /// Release a keyboard key (if it's hold)
    pub async fn release(&self, key: &Key) -> Result<()> {
        self.enigo.lock().await.key(key.clone().into(), Direction::Release)?;

        Ok(())
    }

    /// Release an keyboard keys at the same time (if it's hold)
    pub async fn release_all(&self, keys: &[Key]) -> Result<()> {
        for key in keys {
            self.enigo.lock().await.key(key.clone().into(), Direction::Release)?;
        }

        Ok(())
    }
}
