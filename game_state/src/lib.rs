//! Shared game state information between platforms and game logic

/// Errors that can occur in the game logic
#[derive(Debug)]
pub enum Error {
    /// Attempted to draw an invalid rectangle
    InvalidRectangle
}

/// Custom [`Result`] type for the game logic
pub type Result<T> = std::result::Result<T, Error>;

/// Game state
pub struct Game<'a>  {
    /// Framebuffer used for rendering to the window
    pub framebuffer: &'a mut Vec<u32>,

    /// Width of the game window
    pub width: u32,

    /// Height of the game window
    pub height: u32,
    
    /// Potential error when executing the game logic
    pub error: Result<()>
}

