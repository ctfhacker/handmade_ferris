//! Shared game state information between platforms and game logic

/// Game state
pub struct Game<'a>  {
    /// Framebuffer used for rendering to the window
    pub framebuffer: &'a mut Vec<u32>,

    /// Width of the game window
    pub width: u32,

    /// Height of the game window
    pub height: u32
}
