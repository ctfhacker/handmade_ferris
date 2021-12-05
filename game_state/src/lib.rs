//! Shared game state information between platforms and game logic

use std::collections::HashSet;

/// Number of COLUMNS in the tile map
pub const TILE_MAP_COLUMNS: u16 = 16;

/// Number of rows in the tile map
pub const TILE_MAP_ROWS: u16 = 9;

/// Width of the game window
pub const GAME_WINDOW_WIDTH:  u16 = 960;

/// Height of the game window
pub const GAME_WINDOW_HEIGHT: u16 = 540;

/// Width of a tile
pub const TILE_WIDTH: u16 = GAME_WINDOW_WIDTH / TILE_MAP_COLUMNS;

/// Height of a tile
pub const TILE_HEIGHT: u16 = GAME_WINDOW_HEIGHT / TILE_MAP_ROWS;

/// Errors that can occur in the game logic
#[derive(Debug)]
pub enum Error {
    /// Attempted to draw an invalid rectangle
    InvalidRectangle
}

/// Custom [`Result`] type for the game logic
pub type Result<T> = std::result::Result<T, Error>;

/// Game/Memory state
pub struct Game<'a>  {
    /// Framebuffer used for rendering to the window
    pub framebuffer: &'a mut Vec<u32>,

    /// Width of the game window
    pub width: u16,

    /// Height of the game window
    pub height: u16,
    
    /// Potential error when executing the game logic
    pub error: Result<()>,

    /// Current buttons pressed
    pub buttons_pressed: &'a HashSet<char>,
}

/// Game state
#[derive(Debug)]
pub struct State {
    /// X position of the player
    pub player_x: f32,

    /// Y position of the player
    pub player_y: f32,
}

