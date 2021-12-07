//! Shared game state information between platforms and game logic

#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_fn_fn_ptr_basics)]

/// Number of COLUMNS in the tile map
pub const TILE_MAP_COLUMNS: usize = 16;

/// Number of rows in the tile map
pub const TILE_MAP_ROWS: usize = 9;

/// Width of the game window
pub const GAME_WINDOW_WIDTH:  u16 = 960;

/// Height of the game window
pub const GAME_WINDOW_HEIGHT: u16 = 540;

/// Width of a tile
#[allow(clippy::cast_possible_truncation)]
pub const TILE_WIDTH: u16 = GAME_WINDOW_WIDTH / (TILE_MAP_COLUMNS as u16);

/// Height of a tile
#[allow(clippy::cast_possible_truncation)]
pub const TILE_HEIGHT: u16 = GAME_WINDOW_HEIGHT / (TILE_MAP_ROWS as u16);

/// Number of bits to shift the absolute tile position to get the chunk value
pub const CHUNK_SHIFT: u32 = 8;

/// Mask used to find the offset in a chunk for an absolute tile position
pub const CHUNK_MASK: u32 = 0xf;

/// Number of tiles per x and y axis in a chunk
pub const CHUNK_DIMENSIONS: u32 = 2_u32.pow(CHUNK_SHIFT);

/// Tile size in meters
pub const TILE_SIDE_IN_METERS: Meters = Meters::const_new(1.4);

/// Tile size in pixels
pub const TILE_SIDE_IN_PIXELS: Pixels = Pixels::const_new(60.0);

/// Calculated pixels per meter
pub const PIXELS_PER_METER: PixelsPerMeter 
    = PixelsPerMeter::new(TILE_SIDE_IN_PIXELS, TILE_SIDE_IN_METERS);

/// Provides the `truncate` trait for rounding `f32` to `u32`
pub trait Truncate {
    /// Truncate the given value 
    fn trunc_as_u32(self) -> u32;
}

impl Truncate for f32 {
    #[inline]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn trunc_as_u32(self) -> u32 {
        self.trunc() as u32
    }
}

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
    pub buttons: &'a [bool; Button::Count as usize]
}

/// Typed `f32` representing number of meters
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Meters(f32);

impl Meters {
    /// Create a new meters
    pub const fn const_new(val: f32) -> Meters {
        Meters(val)
    }

    /// Create a new meters
    pub fn new(val: f32) -> Meters {
        Meters(val)
    }

    /// Convert the current [`Meters`] into the number of [`Pixels`]
    pub fn into_pixels(&self) -> Pixels {
        Pixels(self.0 * PIXELS_PER_METER.0)
    }
}

impl std::ops::Deref for Meters {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::AddAssign for Meters {
    fn add_assign(&mut self, other: Self) {
        self.0 += *other;
    }
}

impl std::ops::SubAssign for Meters {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= *other;
    }
}

/// Typed `f32` representing number of pixels
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Pixels(f32);

impl Pixels {
    /// Create a new meters
    pub const fn const_new(val: f32) -> Pixels {
        Pixels(val)
    }

    /// Create a new pixels
    pub fn new(val: f32) -> Pixels {
        Pixels(val)
    }

    /// Convert the current [`Pixels`] into the number of [`Pixels`]
    pub fn into_meters(&self) -> Meters {
        Meters(self.0 * (1.0 / PIXELS_PER_METER.0))
    }
}

impl std::ops::Deref for Pixels {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Typed `f32` representing number of pixels per meter
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct PixelsPerMeter(f32);

impl PixelsPerMeter {
    /// Create a new pixels
    pub const fn new(pixels: Pixels, meter: Meters) -> PixelsPerMeter {
        PixelsPerMeter(pixels.0 / meter.0)
    }
}

/// A player in the game
#[derive(Debug)]
pub struct Player {
    /// World position of the player
    pub position: WorldPosition
}

/// Game state
#[derive(Debug)]
pub struct State {
    /// Player in the game
    pub player: Player
}

impl State {
    /// Get the beginning game state
    pub fn reset() -> Self {
        Self {
            player: Player {
                position: WorldPosition {
                    x: AbsoluteTile::from_chunk_offset(0, 7),
                    y: AbsoluteTile::from_chunk_offset(0, 4),
                    tile_rel_x: Meters::new(0.2),
                    tile_rel_y: Meters::new(0.2),
                }
            }
        }

    }
}

/// Chunk position
#[derive(Copy, Clone, Debug)]
pub struct Chunk {
    /// ID of the chunk
    pub chunk_id: u32,

    /// Offset into the chunk
    pub offset: u16
}

/// An absolute tile location in the world
#[derive(Copy, Clone)]
pub struct AbsoluteTile(u32);

impl std::fmt::Debug for AbsoluteTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Chunk { chunk_id, offset } = self.into_chunk();

        f.debug_struct("AbsoluteTile")
         .field("chunk_id", &chunk_id)
         .field("offset", &offset)
         .finish()
    }
}

impl AbsoluteTile {
    /// Get an [`AbsoluteTile`] from the combined `chunk` and `offset`
    pub fn from_chunk_offset(chunk: u32, offset: u32) -> AbsoluteTile {
        AbsoluteTile((chunk << CHUNK_SHIFT) | (offset & CHUNK_MASK))
    }

    /// Return the (chunk, offset) for this absolute tile
    pub fn into_chunk(&self) -> Chunk {
        Chunk {
            chunk_id: self.0 >> CHUNK_SHIFT,
            offset:   (self.0 & CHUNK_MASK) as u16
        }
    }

    /// Increment the chunk ID by `val`
    ///
    /// # Panics
    ///
    /// * If `CHUNK_MASK` doens't fit in a u16
    pub fn increment(&mut self, val: u32) {
        let mut chunk = self.into_chunk();
        let new_offset = u32::from(chunk.offset).wrapping_add(val);
        if new_offset & CHUNK_MASK == new_offset {
            // No overflow occured, still in the same chunk
            chunk.offset = u16::try_from(new_offset).unwrap();
        } else {
            // Overflow occured, moving to the next chunk
            chunk.offset    = 0;
            chunk.chunk_id += 1;
        }
        *self = chunk.into();
    }

    /// Decrement the chunk ID by `val`
    ///
    /// # Panics
    ///
    /// * If `CHUNK_MASK` doens't fit in a u16
    pub fn decrement(&mut self, val: u32) {
        let mut chunk = self.into_chunk();
        let new_offset = u32::from(chunk.offset).wrapping_sub(val);
        if new_offset & CHUNK_MASK == new_offset {
            // No overflow occured, still in the same chunk
            chunk.offset = u16::try_from(new_offset).unwrap();
        } else {
            // Overflow occured, moving to the next chunk
            chunk.offset    = CHUNK_MASK.try_into().unwrap();
            chunk.chunk_id -= 1;
        }
        *self = chunk.into();
    }
}

impl From<Chunk> for AbsoluteTile {
    fn from(chunk: Chunk) -> AbsoluteTile {
        AbsoluteTile(
            (chunk.chunk_id << CHUNK_SHIFT) | (u32::from(chunk.offset) & CHUNK_MASK)
        )
    }
}

/// A tile position in the world
#[derive(Copy, Clone, Debug)]
pub struct WorldPosition {
    /// The absolute tile value
    pub x: AbsoluteTile,

    /// The absolute tile value
    pub y: AbsoluteTile,

    /// x offset in the tile
    pub tile_rel_x: Meters,

    /// y offset in the tile
    pub tile_rel_y: Meters,
}

impl WorldPosition {
    /// Update the tile position if the relative tile position moved to an adjacent tile
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn canonicalize(&mut self) {
        if self.tile_rel_x > TILE_SIDE_IN_METERS {
            let inc_by = self.tile_rel_x.div_euclid(*TILE_SIDE_IN_METERS);
            self.x.increment(inc_by.trunc_as_u32());

            self.tile_rel_x = Meters(self.tile_rel_x.rem_euclid(*TILE_SIDE_IN_METERS));
        }

        if self.tile_rel_x < TILE_SIDE_IN_METERS {
            let dec_by = self.tile_rel_x.div_euclid(*TILE_SIDE_IN_METERS);
            self.x.decrement(dec_by.abs().trunc_as_u32());
            self.tile_rel_x = Meters(self.tile_rel_x.rem_euclid(*TILE_SIDE_IN_METERS));
        }

        if self.tile_rel_y > TILE_SIDE_IN_METERS {
            let inc_by = self.tile_rel_y.div_euclid(*TILE_SIDE_IN_METERS);
            self.y.decrement(inc_by.trunc_as_u32());

            self.tile_rel_y = Meters(self.tile_rel_y.rem_euclid(*TILE_SIDE_IN_METERS));
        }

        if self.tile_rel_y < TILE_SIDE_IN_METERS {
            let dec_by = self.tile_rel_y.div_euclid(*TILE_SIDE_IN_METERS);
            self.y.increment(dec_by.abs().trunc_as_u32());
            self.tile_rel_y = Meters(self.tile_rel_y.rem_euclid(*TILE_SIDE_IN_METERS));
        }
    }
}

/// Direction to move the player
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Button {
    /// Move up
    Up = 0,

    /// Move down
    Down,

    /// Move left
    Left,

    /// Move right
    Right,

    /// Total number of button attributes
    Count,
    // Nothing should be added under this value
}

impl Button {
    /// Get a [`Button`] from a `usize`
    pub const fn from_usize(val: usize) -> Self {
        /// All values for the buttons
        const VALS: [Button; Button::Count as usize] = [
            Button::Up,
            Button::Down,
            Button::Left,
            Button::Right,
        ];

        VALS[val]
    }
}
