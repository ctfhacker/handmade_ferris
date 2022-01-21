//! Shared game state information between platforms and game logic

#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_fn_fn_ptr_basics)]

mod rng;
pub use rng::Rng;

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

/// Half of the width of a tile
#[allow(clippy::cast_possible_truncation)]
pub const TILE_HALF_WIDTH: u16 = GAME_WINDOW_WIDTH / (TILE_MAP_COLUMNS as u16) / 2;

/// Height of a tile
#[allow(clippy::cast_possible_truncation)]
pub const TILE_HEIGHT: u16 = GAME_WINDOW_HEIGHT / (TILE_MAP_ROWS as u16);

/// Half of the height of a tile
#[allow(clippy::cast_possible_truncation)]
pub const TILE_HALF_HEIGHT: u16 = GAME_WINDOW_HEIGHT / (TILE_MAP_ROWS as u16) / 2;

/// Number of bits to shift the absolute tile position to get the chunk value
pub const CHUNK_SHIFT: u32 = 8;

/// Mask used to find the offset in a chunk for an absolute tile position
pub const CHUNK_MASK: u32 = 0xf;

/// Number of tiles per x and y axis in a chunk
pub const CHUNK_DIMENSIONS: u32 = 2_u32.pow(CHUNK_SHIFT);

/// Maximum number of chunks
pub const MAX_NUM_CHUNKS: usize = (u32::MAX >> CHUNK_SHIFT) as usize;

/// Tile size in meters
pub const TILE_SIDE_IN_METERS: Meters = Meters::const_new(1.0);

/// Tile size in meters
pub const TILE_RADIUS_IN_METERS: Meters = Meters::const_new(TILE_SIDE_IN_METERS.0 / 2.);

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

/// Provides the `Round` trait for rounding `f32` to `u32`
pub trait Round {
    /// Truncate the given value 
    fn round_as_i32(self) -> i32;
}

impl Round for f32 {
    #[inline]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn round_as_i32(self) -> i32 {
        self.round() as i32
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

/// A bitmap asset
pub struct BitmapAsset<'a> {
    /// Width of the bitmap in pixels
    pub width:  u32,

    /// Height of the bitmap in pixels
    pub height: u32,

    /// Reference to the pixels
    pub data: &'a [u8],
}

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
    pub buttons: &'a [bool; Button::Count as usize],

    /// Reference to the memory backing the game
    pub memory: &'a mut Memory,

    /// Some offset to try and draw (width 
    pub asset: &'a BitmapAsset<'a>
}

impl From<f32> for Meters {
    fn from(val: f32) -> Meters {
        Meters::new(val)
    }
}

/// Typed `f32` representing number of meters.
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

impl std::ops::Sub<f32> for Meters {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self::Output {
        Meters(self.0 - rhs)
    }
}

impl std::ops::SubAssign<f32> for Meters {
    fn sub_assign(&mut self, rhs: f32) {
        self.0 -= rhs;
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
    pub player: Player,

    /// Random number generator
    pub rng: Rng,
}

impl State {
    /// Get the beginning game state
    pub fn reset() -> Self {
        Self {
            player: Player {
                position: WorldPosition {
                    x: AbsoluteTile::from_chunk_offset(0, 5),
                    y: AbsoluteTile::from_chunk_offset(0, 6),
                    z: 0,
                    tile_rel_x: Meters::new(0.0),
                    tile_rel_y: Meters::new(0.0),
                }
            },
            rng: Rng::new()
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

/// An absolute tile location in the world, constrained to only be [`0`, `MAX`) in value.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AbsoluteTile<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize>(u32);

impl<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize> std::fmt::Debug 
        for AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Chunk { chunk_id, offset } = self.into_chunk();

        f.debug_struct("AbsoluteTile")
         .field("chunk_id", &chunk_id)
         .field("offset", &offset)
         .finish()
    }
}

impl<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize> 
        AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
    /// Get an [`AbsoluteTile`] from the combined `chunk` and `offset`
    pub fn from_chunk_offset(chunk: u32, offset: u32) 
            -> AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
        AbsoluteTile((chunk << CHUNK_SHIFT) | (offset & CHUNK_MASK))
    }

    /// Return the (chunk, offset) for this absolute tile
    pub fn into_chunk(&self) -> Chunk {
        Chunk {
            chunk_id: self.0 >> CHUNK_SHIFT,
            offset:   (self.0 & CHUNK_MASK) as u16
        }
    }

    /// Adjust the chunk ID by `val`
    ///
    /// # Panics
    ///
    /// * If `CHUNK_MASK` doens't fit in a u16
    pub fn adjust(&mut self, val: i32) {
        // Early return for adjusting by 0
        if val == 0 {
            return;
        }

        assert!(val == 1 || val == -1, "Adjusting by larger than 1 square");

        let mut chunk = self.into_chunk();

        match val {
            1 => {
                // If incrementing beyond the MAX_OFFSET, go to the next chunk
                if usize::from(chunk.offset + 1) == MAX_OFFSET {
                    chunk.chunk_id += 1;

                    // Wrap around the world if going beyond the MAX_CHUNK_ID
                    if usize::try_from(chunk.chunk_id).unwrap() == MAX_CHUNK_ID {
                        chunk.chunk_id = 0;
                    }

                    chunk.offset = 0;
                } else {
                    // No new chunk, increment the offset in the current chunk
                    chunk.offset += 1;
                }
            }
            -1 => {
                // If decrementing beyond 0, go to the previous chunk
                if chunk.offset == 0 {
                    // Wrap around the world if stepping beyond 0
                    if chunk.chunk_id == 0 {
                        chunk.chunk_id = u32::try_from(MAX_CHUNK_ID - 1).unwrap();
                    } else {
                        chunk.chunk_id -= 1;
                    }

                    // Set the new offset to the max offset of the new chunk
                    chunk.offset = u16::try_from(MAX_OFFSET - 1).unwrap();
                } else {
                    // No new chunk, decrement the offset in the current chunk
                    chunk.offset -= 1;
                }
            }
            _ => unreachable!()
        }
        
        // Re-write the modified chunk back
        *self = chunk.into();
    }

    /// Increment the chunk ID by `val`
    ///
    /// # Panics
    ///
    /// * If `CHUNK_MASK` doens't fit in a u16
    pub fn increment(&mut self, val: u32) {
        let mut chunk = self.into_chunk();
        if let Some(new_offset) = u32::from(chunk.offset).checked_add(val) {
            if new_offset < u32::try_from(MAX_OFFSET).unwrap() {
                // No overflow occured, still in the same chunk
                chunk.offset = u16::try_from(new_offset).unwrap();
            } else {
                // Tile MAX value Overflow occured, moving to the next chunk
                chunk.offset    = 0;
                chunk.chunk_id += 1;
            }
        } else {
            // u32 Overflow occured, moving to the next chunk
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

        if let Some(new_offset) = u32::from(chunk.offset).checked_sub(val) {
            // No overflow occured, still in the same chunk
            chunk.offset = u16::try_from(new_offset).unwrap();
        } else {
            // u32 Overflow occured, moving to the next chunk
            chunk.offset    = u16::try_from(MAX_OFFSET - 1).unwrap();
            if let Some(chunk_id) = chunk.chunk_id.checked_sub(1) {
                // No underflow, proceed as normal
                chunk.chunk_id = chunk_id;
            } else {
                // Underflow detected, wrap chunk_id around to the max value
                chunk.chunk_id = u32::try_from(MAX_CHUNK_ID - 1).unwrap();
            }
        }

        *self = chunk.into();
    }
}

impl<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize> From<Chunk> 
        for AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
    fn from(chunk: Chunk) -> AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
        AbsoluteTile::<MAX_CHUNK_ID, MAX_OFFSET>(
            (chunk.chunk_id << CHUNK_SHIFT) | (u32::from(chunk.offset) & CHUNK_MASK)
        )
    }
}

/// A tile position in the world
///
/// The [`AbsoluteTile`] contains the `chunk` and specific tile in the chunk itself, while 
/// the `tile_rel_*` contains the relative offset the entity is within 
#[derive(Copy, Clone, Debug)]
pub struct WorldPosition {
    /// The absolute tile value in x
    pub x: AbsoluteTile<MAX_NUM_CHUNKS, TILE_MAP_COLUMNS>,

    /// The absolute tile value in y
    pub y: AbsoluteTile<MAX_NUM_CHUNKS, TILE_MAP_ROWS>,

    /// The floor height in z
    pub z: u8,

    /// x offset in the tile
    pub tile_rel_x: Meters,

    /// y offset in the tile
    pub tile_rel_y: Meters,
}

impl WorldPosition {
    /// Update the tile position if the relative tile position moved to an adjacent tile
    ///
    /// # Panics
    ///
    /// * Fails to pass sanity check for the relative tile position
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn canonicalize(&mut self) {
        self.x.adjust(self.tile_rel_x.round() as i32);
        self.tile_rel_x -= self.tile_rel_x.round();

        self.y.adjust(self.tile_rel_y.round() as i32);
        self.tile_rel_y -= self.tile_rel_y.round();

        if *self.tile_rel_x <= -1.0 || *self.tile_rel_x >= 1.0 {
            dbg!(self);
            panic!("Bad x");
        }

        if *self.tile_rel_y <= -1.0 || *self.tile_rel_y >= 1.0 {
            dbg!(self);
            panic!("Bad y");
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

    /// Decrease player speed
    DecreaseSpeed,

    /// Increase player speed
    IncreaseSpeed,

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
            Button::DecreaseSpeed,
            Button::IncreaseSpeed,
        ];

        VALS[val]
    }
}

/// Memory chunk allocated for the game with a basic bump allocator
pub struct Memory {
    /// Has this memory been initialized by the game yet
    pub initialized: bool,

    /// Data bytes for this memory, allocated by the platform
    pub data: Vec<u8>,

    /// Size of the data allocation
    pub data_len: usize,

    /// Offset to the next allocation in the memory region
    pub next_allocation: usize
}

impl Memory {
    /// Allocate a new chunk of memory
    pub fn new(size: usize) -> Self {
        Self {
            initialized:     false,
            data:            Vec::with_capacity(size),
            data_len:        size,
            next_allocation: 0

        }
    }

    /// Allocate `T` in the allocated game memory
    ///
    /// # Panics
    ///
    /// * Out of allocated memory
    pub fn alloc<T: Sized>(&mut self) -> *mut T {
        assert!(self.next_allocation + std::mem::size_of::<T>() < self.data_len, 
            "Out of game memory");

        // Get the resulting address
        let result = unsafe { 
            self.data.as_mut_ptr().add(self.next_allocation)
        };

        // Bump the allocation to fit the requested type
        self.next_allocation += std::mem::size_of::<T>();

        // 64 bit align the next allocation
        self.next_allocation = (self.next_allocation + 0xf) & !0xf;

        // Return the pointer to the allocation
        result.cast::<T>()
    }
}


/// Color represented by red, green, blue pigments with alpha channel
#[derive(Debug)]
pub struct Color {
    /// Percentage of red color pigment from 0.0 .. 1.0
    red: Red,

    /// Percentage of green color pigment from 0.0 .. 1.0
    green: Green,

    /// Percentage of blue color pigment from 0.0 .. 1.0
    blue: Blue,

    /// Percentage of alpha from 0.0 .. 1.0
    alpha: Alpha,

}

impl From<u8> for Color {
    fn from(val: u8) -> Color {
        match val {
            1 => Color::YELLOW,
            2 => Color::WHITE,
            _ => Color::GREY,
        }
    }
}

/// Creates the bounded color values to percentages of [0.0..1.0]
macro_rules! make_color { ($color:ident) => {
        /// Red color bounded to the percentage of 0.0 to 1.0
        #[derive(Debug)]
        struct $color(f32);

        impl $color {
            /// Create a new [`Red`] percentage, checked at compile time that it is
            /// within the bounds [0.0..1.0]
            const fn new(val: f32) -> $color { 
                assert!(0.0 <= val && val <= 1.0,
                concat!(stringify!($color), " value out of bounds [0.0..1.0]"));
                $color(val) 
            } 
        }

        impl std::ops::Deref for $color {
            type Target = f32;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}

make_color!(Red);
make_color!(Blue);
make_color!(Green);
make_color!(Alpha);

impl Color {
    /// The color red
    #[allow(dead_code)]
    pub const RED:   Color = Color { 
        red: Red::new(1.), green: Green::new(0.), blue: Blue::new(0.), alpha: Alpha::new(0.)
    };

    /// The color blue
    #[allow(dead_code)]
    pub const BLUE:  Color = Color { 
        red: Red::new(0.), green: Green::new(0.), blue: Blue::new(1.), alpha: Alpha::new(0.)
    };

    /// The color green
    #[allow(dead_code)]
    pub const GREEN: Color = Color { 
        red: Red::new(0.), green: Green::new(1.), blue: Blue::new(0.), alpha: Alpha::new(0.)
    };

    /// The color yellow
    #[allow(dead_code)]
    pub const YELLOW: Color = Color { 
        red: Red::new(1.), green: Green::new(1.), blue: Blue::new(0.), alpha: Alpha::new(0.)
    };

    /// The color white
    #[allow(dead_code)]
    pub const WHITE: Color = Color { 
        red: Red::new(1.), green: Green::new(1.), blue: Blue::new(1.), alpha: Alpha::new(0.)
    };

    /// The color white
    #[allow(dead_code)]
    pub const BLACK: Color = Color { 
        red: Red::new(0.), green: Green::new(0.), blue: Blue::new(0.), alpha: Alpha::new(0.)
    }; 
    /// The color white
    #[allow(dead_code)]
    pub const GREY: Color = Color { 
        red: Red::new(0.5), green: Green::new(0.5), blue: Blue::new(0.5), alpha: Alpha::new(0.)
    };

    /// Create a [`Color`] from `red`, `green`, and `blue` percentages
    #[allow(dead_code)]
    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red:   Red::new(red),
            green: Green::new(green),
            blue:  Blue::new(blue),
            alpha: Alpha::new(0.0)
        }
    }

    /// Create a [`Color`] from `red`, `green`, and `blue` percentages
    #[allow(dead_code)]
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red:   Red::new(red),
            green: Green::new(green),
            blue:  Blue::new(blue),
            alpha: Alpha::new(alpha)
        }
    }

    /// `u32` representation of the [`Color`]
    #[inline]
    pub fn as_u32(&self) -> u32 {
        (*self.alpha * 255.).trunc_as_u32() << 24 | 
        (*self.red   * 255.).trunc_as_u32() << 16 | 
        (*self.green * 255.).trunc_as_u32() <<  8 |
        (*self.blue  * 255.).trunc_as_u32()
    }
}
