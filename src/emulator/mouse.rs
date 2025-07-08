use crate::prelude::*;
use enigo::{ Axis, Button, Coordinate, Direction, Enigo, Mouse as EnigoMouse, };

#[derive(Debug, Clone)]
pub struct Mouse {
    enigo: Arc<Mutex<Enigo>>,
}

impl Mouse {
    /// Creates a new mouse emulator
    pub fn new(enigo: Arc<Mutex<Enigo>>) -> Self {
        Self {
            enigo,
        }
    }

    /// Returns current mouse coordinates
    pub async fn get_coords(&self) -> Result<(i32, i32)> {
        let enigo = self.enigo.lock().await;
        enigo.location().map_err(From::from)
    }

    /// Returns screen resolution (width, height)
    pub async fn get_display_size(&self) -> Result<(i32, i32)> {
        let enigo = self.enigo.lock().await;
        enigo.main_display().map_err(From::from)
    }

    /// Move mouse horizontally (relative)
    pub async fn move_x(&self, dx: i32) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.move_mouse(dx, 0, Coordinate::Rel).map_err(From::from)
    }

    /// Move mouse vertically (relative)
    pub async fn move_y(&self, dy: i32) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.move_mouse(0, dy, Coordinate::Rel).map_err(From::from)
    }

    /// Move mouse to center
    pub async fn move_center(&self) -> Result<()> {
        let (width, height) = self.get_display_size().await?;
        let center_x = width / 2;
        let center_y = height / 2;
        
        let mut enigo = self.enigo.lock().await;
        enigo.move_mouse(center_x, center_y, Coordinate::Abs).map_err(From::from)
    }

    /// Press left mouse button
    pub async fn press_left(&self, hold: bool) -> Result<()> {
        let mut enigo = self.enigo.lock().await;

        enigo.button(Button::Left, Direction::Press)?;

        if !hold {
            enigo.button(Button::Left, Direction::Release)?;
        }

        Ok(())
    }

    /// Release left mouse button
    pub async fn release_left(&self) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.button(Button::Left, Direction::Release).map_err(From::from)
    }

    /// Press right mouse button
    pub async fn press_right(&self, hold: bool) -> Result<()> {
        let mut enigo = self.enigo.lock().await;

        enigo.button(Button::Right, Direction::Press)?;

        if !hold {
            enigo.button(Button::Right, Direction::Release)?;
        }

        Ok(())
    }

    /// Release right mouse button
    pub async fn release_right(&self) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.button(Button::Right, Direction::Release).map_err(From::from)
    }

    /// Scroll horizontally
    pub async fn scroll_x(&self, delta: i32) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.scroll(delta, Axis::Horizontal).map_err(From::from)
    }

    /// Scroll vertically
    pub async fn scroll_y(&self, delta: i32) -> Result<()> {
        let mut enigo = self.enigo.lock().await;
        enigo.scroll(delta, Axis::Vertical).map_err(From::from)
    }
}
