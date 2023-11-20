//! Shared game state information between platforms and game logic

#![feature(const_fn_floating_point_arithmetic)]
#![feature(variant_count)]

use std::mem::variant_count;
use std::ops::AddAssign;

mod rng;
pub use rng::Rng;

use vector::Vector2;

/// Number of COLUMNS in the tile map
pub const TILE_MAP_COLUMNS: usize = 16;

/// Number of rows in the tile map
pub const TILE_MAP_ROWS: usize = 9;

/// The row in the center of the screen
pub const SCREEN_CENTER_ROW: usize = TILE_MAP_ROWS / 2;

/// The column in the center of the screen
pub const SCREEN_CENTER_COLUMN: usize = TILE_MAP_COLUMNS / 2;

/// Width of the game window
pub const GAME_WINDOW_WIDTH: u16 = 960;

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
pub const CHUNK_MASK: u32 = 0xff;

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
pub const PIXELS_PER_METER: PixelsPerMeter =
    PixelsPerMeter::new(TILE_SIDE_IN_PIXELS, TILE_SIDE_IN_METERS);

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
    InvalidRectangle,
}

/// Custom [`Result`] type for the game logic
pub type Result<T> = std::result::Result<T, Error>;

/// A bitmap asset
pub struct BitmapAsset<'a> {
    /// Width of the bitmap in pixels
    pub width: u32,

    /// Height of the bitmap in pixels
    pub height: u32,

    /// The index from 0..4 of the red channel from the pixel streaming data
    pub red_index: u8,

    /// The index from 0..4 of the blue channel from the pixel streaming data
    pub blue_index: u8,

    /// The index from 0..4 of the green channel from the pixel streaming data
    pub green_index: u8,

    /// The index from 0..4 of the alphw channel from the pixel streaming data
    pub alpha_index: u8,

    /// Reference to the pixels
    pub data: &'a [u8],
}

impl<'a> std::fmt::Debug for BitmapAsset<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitmapAsset")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("data", &format!("{:p}", &self.data))
            .finish()
    }
}

impl<'a> BitmapAsset<'a> {
    /// Draw this bitmap at (`pos_x`, `pos_y`) on the screen
    ///
    /// # Panics
    ///
    /// *
    pub fn draw(&self, game: &mut Game, pos: Vector2<f32>) {
        let game_height = f32::from(game.height);

        #[allow(clippy::cast_precision_loss)]
        let width = self.width as f32;

        #[allow(clippy::cast_precision_loss)]
        let height = self.height as f32;

        let bytes_per_color = 4;

        // Because the BMP pixels are in bottom row -> top row order, if the requested width
        // or height is less than the self width or height, start the pixels array from the
        // correct location.
        //
        //                    +----------------------------+
        //                    | Draw  |    BMP self        |
        //                    |       |                    |
        // Requested start  -->*      |                    |
        //                    +-------+                    |
        //                    |                            |
        //                    |                            |
        //                    |                            |
        //                    |*                           |
        //                    +^---------------------------+
        //                     |
        //                    Normal starting pixel
        let mut starting_height = (self.height - height.trunc_as_u32()) as usize;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if height + pos.y > game_height {
            let offscreen = height + pos.y - game_height as f32;
            starting_height += offscreen as usize;
        }

        let mut starting_column = 0;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if pos.x < 0.0 {
            starting_column = pos.x.round().abs().trunc() as usize;
        }

        let starting_index = starting_height * self.width as usize * 4;
        let pixels_start = &self.data[starting_index..];

        let upper_left = Vector2::new(
            pos.x.trunc_as_u32().clamp(0, u32::from(game.width)),
            pos.y.trunc_as_u32().clamp(0, u32::from(game.height)),
        );

        let lower_right = Vector2::new(
            (pos.x + width)
                .trunc_as_u32()
                .clamp(0, u32::from(game.width)),
            (pos.y + height)
                .trunc_as_u32()
                .clamp(0, u32::from(game.height)),
        );

        // Get the color channel indexes for each color
        let blue_index = usize::from(self.blue_index);
        let red_index = usize::from(self.red_index);
        let green_index = usize::from(self.green_index);
        let alpha_index = usize::from(self.alpha_index);

        // Draw the self at the requested location
        for (row_index, row) in (upper_left.y..lower_right.y).rev().enumerate() {
            // In the event the self is larger than the requested draw size, update the
            // pixel pointer to the next row of pixels and ignore the non-drawn pixels
            let this_row = row_index * self.width as usize * bytes_per_color;

            // In the event the image is off the left edge of the screen, the starting column
            // should be the remaining portion of the image not NOT from zero.
            let starting_column = starting_column as usize * bytes_per_color;

            let mut pixels = &pixels_start[this_row + starting_column..];

            for col in upper_left.x..lower_right.x {
                // Sanity check that we have enough pixel data to draw the sprite
                if pixels.len() < 4 {
                    continue;
                }

                let index = row
                    .checked_mul(u32::from(game.width))
                    .and_then(|res| res.checked_add(col))
                    .expect("Overflow");

                let index = usize::try_from(index).unwrap();

                let r = f32::from(pixels[red_index]) / 255.0;
                let g = f32::from(pixels[green_index]) / 255.0;
                let b = f32::from(pixels[blue_index]) / 255.0;
                let a = f32::from(pixels[alpha_index]) / 255.0;

                // Create the curent color from the bitmap stream
                let mut new_color = Color::rgba(r, g, b, a);

                // Get the current background color for this pixel
                let current_color: Color = game.framebuffer[index].into();

                // Blend the new color into the background
                new_color.linear_alpha_blend(current_color);

                // Write the new color into the backgrouund
                game.framebuffer[index] = new_color.as_u32();

                pixels = &pixels[4..];
            }
        }
    }
}

/// Searches the `val` for the least significant set bit (1 bit).
fn bit_scan_forward(val: u64) -> Option<u8> {
    if val == 0 {
        return None;
    }

    let mut res: u64;
    unsafe {
        core::arch::asm!(
            "bsf {}, {}",
            out(reg) res,
            in(reg) val,
        );
    }
    Some(u8::try_from(res).unwrap())
}

impl<'a> BitmapAsset<'a> {
    /// Create a [`BitmapAsset`] from the given bytes
    #[allow(clippy::missing_panics_doc)]
    pub fn from_data(data: &'a [u8]) -> Self {
        assert!(data.len() > 0x16 + 4, "BMP data too small");

        let offset = u32::from_le_bytes(data[0x0a..0x0a + 4].try_into().unwrap()) as usize;
        let width = u32::from_le_bytes(data[0x12..0x12 + 4].try_into().unwrap());
        let height = u32::from_le_bytes(data[0x16..0x16 + 4].try_into().unwrap());
        let r_mask = u32::from_le_bytes(data[0x36..0x36 + 4].try_into().unwrap());
        let g_mask = u32::from_le_bytes(data[0x3a..0x3a + 4].try_into().unwrap());
        let b_mask = u32::from_le_bytes(data[0x3e..0x3e + 4].try_into().unwrap());
        let a_mask = u32::from_le_bytes(data[0x42..0x42 + 4].try_into().unwrap());

        // Get the index value for the color channels specific for this image
        let red_index = bit_scan_forward(r_mask.into()).expect("Empty red mask?") / 8;
        let green_index = bit_scan_forward(g_mask.into()).expect("Empty green mask?") / 8;
        let blue_index = bit_scan_forward(b_mask.into()).expect("Empty blue mask?") / 8;
        let alpha_index = bit_scan_forward(a_mask.into()).expect("Empty alpha mask?") / 8;

        BitmapAsset {
            width,
            height,
            red_index,
            green_index,
            blue_index,
            alpha_index,
            data: &data[offset..],
        }
    }
}

/// Bitmap assets for the player
#[derive(Debug)]
pub struct PlayerBitmap<'a> {
    /// [`BitmapAsset`] of the head of the player
    pub head: BitmapAsset<'a>,

    /// [`BitmapAsset`] of the torso of the player
    pub torso: BitmapAsset<'a>,

    /// [`BitmapAsset`] of the cape of the player
    pub cape: BitmapAsset<'a>,

    /// The coordinate of the merge point from the upper left corner of the image
    pub merge_point: Vector2<f32>,
}

impl<'a> PlayerBitmap<'a> {
    /// Create a [`PlayerBitmap`] from the given assets
    pub fn from(
        head: BitmapAsset<'a>,
        torso: BitmapAsset<'a>,
        cape: BitmapAsset<'a>,
        merge_point: Vector2<f32>,
    ) -> Self {
        Self {
            head,
            torso,
            cape,
            merge_point,
        }
    }
}

/// Game/Memory state
pub struct Game<'a> {
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

    /// Player assets specific to the direction the player is facing
    pub player_assets: [&'a PlayerBitmap<'a>; variant_count::<PlayerDirection>()],

    /// Background asset
    pub background: &'a BitmapAsset<'a>,
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

impl std::ops::MulAssign<f32> for Meters {
    fn mul_assign(&mut self, other: f32) {
        self.0 *= other;
    }
}

impl std::ops::MulAssign for Meters {
    fn mul_assign(&mut self, other: Self) {
        let other: f32 = *other;

        self.0 *= other;
    }
}

impl std::ops::Add<Meters> for Meters {
    type Output = Self;
    fn add(self, rhs: Meters) -> Self::Output {
        Meters(self.0 + rhs.0)
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

impl std::ops::Mul<Meters> for Meters {
    type Output = Self;
    fn mul(self, rhs: Meters) -> Self::Output {
        Meters(self.0 * rhs.0)
    }
}

impl From<Meters> for f32 {
    fn from(val: Meters) -> f32 {
        *val
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
    pub position: WorldPosition,

    /// Direction the player is facing
    pub direction: PlayerDirection,
}

/// Game state
#[derive(Debug)]
pub struct State {
    /// Player in the game
    pub player: Player,

    /// Camera position to known where to draw the current screen
    pub camera: WorldPosition,

    /// Random number generator
    pub rng: Rng,
}

impl State {
    /// Get the beginning game state
    ///
    /// # Panics
    ///
    /// If the screen center row or column doesn't fit in a u32 -- o.0
    pub fn reset() -> Self {
        Self {
            player: Player {
                position: WorldPosition {
                    tile_map_x: AbsoluteTile::from_chunk_offset(0, 5),
                    tile_map_y: AbsoluteTile::from_chunk_offset(0, 6),
                    z: 0,
                    tile_rel: Vector2::new(Meters::new(0.0), Meters::new(0.0)),
                },

                direction: PlayerDirection::Front,
            },
            camera: WorldPosition {
                tile_map_x: AbsoluteTile::from_chunk_offset(
                    0,
                    SCREEN_CENTER_COLUMN.try_into().unwrap(),
                ),
                tile_map_y: AbsoluteTile::from_chunk_offset(
                    0,
                    SCREEN_CENTER_ROW.try_into().unwrap(),
                ),
                z: 0,
                tile_rel: Vector2::new(Meters::new(0.0), Meters::new(0.0)),
            },
            rng: Rng::new(),
        }
    }
}

/// Chunk position
#[derive(Copy, Clone, Debug)]
pub struct Chunk {
    /// ID of the chunk
    pub chunk_id: u32,

    /// Offset into the chunk
    pub offset: u16,
}

/// [`Chunk`] with `chunk_id` and `offset` as `Vector2`
pub struct ChunkVector {
    /// ID of the chunk
    pub chunk_id: Vector2<u32>,

    /// Offset into the chunk
    pub offset: Vector2<u16>,
}

/// An absolute tile location in the world, constrained to only be [`0`, `MAX`) in value.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct AbsoluteTile<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize>(u32);

impl<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize> std::fmt::Debug
    for AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Chunk { chunk_id, offset } = self.into_chunk();

        f.debug_struct("AbsoluteTile")
            .field("chunk_id", &chunk_id)
            .field("offset", &offset)
            .finish()
    }
}

impl<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize> AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
    /// Get an [`AbsoluteTile`] from the combined `chunk` and `offset`
    pub fn from_chunk_offset(chunk: u32, offset: u32) -> AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
        AbsoluteTile((chunk << CHUNK_SHIFT) | (offset & CHUNK_MASK))
    }

    /// Return the (chunk, offset) for this absolute tile
    pub fn into_chunk(&self) -> Chunk {
        Chunk {
            chunk_id: self.0 >> CHUNK_SHIFT,
            offset: (self.0 & CHUNK_MASK) as u16,
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
            _ => unreachable!(),
        }

        // Re-write the modified chunk back
        *self = chunk.into();
    }
}

impl<const MAX_CHUNK_ID: usize, const MAX_OFFSET: usize> From<Chunk>
    for AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET>
{
    fn from(chunk: Chunk) -> AbsoluteTile<MAX_CHUNK_ID, MAX_OFFSET> {
        AbsoluteTile::<MAX_CHUNK_ID, MAX_OFFSET>(
            (chunk.chunk_id << CHUNK_SHIFT) | (u32::from(chunk.offset) & CHUNK_MASK),
        )
    }
}

/// A tile position in the world
///
/// The [`AbsoluteTile`] contains the `chunk` and specific tile in the chunk itself, while
/// the `tile_rel_*` contains the relative offset the entity is within
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WorldPosition {
    /// The x coordinate in the world
    pub tile_map_x: AbsoluteTile<MAX_NUM_CHUNKS, TILE_MAP_COLUMNS>,

    /// The y coordinate in the world
    pub tile_map_y: AbsoluteTile<MAX_NUM_CHUNKS, TILE_MAP_ROWS>,

    /// The floor height in z
    pub z: u8,

    /// The relative position in a given tile
    pub tile_rel: Vector2<Meters>,
}

impl AddAssign<Vector2<Meters>> for WorldPosition {
    fn add_assign(&mut self, right: Vector2<Meters>) {
        self.tile_rel += right;
    }
}

impl WorldPosition {
    /// Update the tile position if the relative tile position moved to an adjacent tile
    ///
    /// # Panics
    ///
    /// * Fails to pass sanity check for the relative tile position
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn canonicalize(&mut self) {
        assert!(
            self.tile_rel.x >= Meters::const_new(-1.5) && self.tile_rel.x <= Meters::const_new(1.5)
        );
        assert!(
            self.tile_rel.y >= Meters::const_new(-1.5) && self.tile_rel.y <= Meters::const_new(1.5)
        );

        self.tile_map_x.adjust(self.tile_rel.x.round() as i32);
        self.tile_rel.x -= self.tile_rel.x.round();

        self.tile_map_y.adjust(self.tile_rel.y.round() as i32);
        self.tile_rel.y -= self.tile_rel.y.round();

        if *self.tile_rel.x <= -1.0 || *self.tile_rel.x >= 1.0 {
            dbg!(self);
            panic!("Bad x");
        }

        if *self.tile_rel.y <= -1.0 || *self.tile_rel.y >= 1.0 {
            dbg!(self);
            panic!("Bad y");
        }
    }

    /// Return the `Vector2` of the (x, y) chunk coordinates
    pub fn into_chunk(&self) -> ChunkVector {
        let Chunk {
            chunk_id: chunk_id_x,
            offset: x_offset,
        } = self.tile_map_x.into_chunk();
        let Chunk {
            chunk_id: chunk_id_y,
            offset: y_offset,
        } = self.tile_map_y.into_chunk();

        let chunk_id = Vector2::new(chunk_id_x, chunk_id_y);
        let offset = Vector2::new(x_offset, y_offset);

        ChunkVector { chunk_id, offset }
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
    pub next_allocation: usize,
}

impl Memory {
    /// Allocate a new chunk of memory
    pub fn new(size: usize) -> Self {
        Self {
            initialized: false,
            data: Vec::with_capacity(size),
            data_len: size,
            next_allocation: 0,
        }
    }

    /// Allocate `T` in the allocated game memory
    ///
    /// # Panics
    ///
    /// * Out of allocated memory
    pub fn alloc<T: Sized>(&mut self) -> *mut T {
        assert!(
            self.next_allocation + std::mem::size_of::<T>() < self.data_len,
            "Out of game memory"
        );

        // Get the resulting address
        let result = unsafe { self.data.as_mut_ptr().add(self.next_allocation) };

        // Bump the allocation to fit the requested type
        self.next_allocation += std::mem::size_of::<T>();

        // 64 bit align the next allocation
        self.next_allocation = (self.next_allocation + 0xf) & !0xf;

        // Return the pointer to the allocation
        result.cast::<T>()
    }
}

/// Color represented by red, green, blue pigments with alpha channel
#[derive(Debug, Copy, Clone)]
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
macro_rules! make_color {
    ($color:ident) => {
        /// Red color bounded to the percentage of 0.0 to 1.0
        #[derive(Debug, Copy, Clone)]
        struct $color(f32);

        impl $color {
            /// Create a new [`Red`] percentage, checked at compile time that it is
            /// within the bounds [0.0..1.0]
            const fn new(val: f32) -> $color {
                assert!(
                    0.0 <= val && val <= 1.0,
                    concat!(stringify!($color), " value out of bounds [0.0..1.0]")
                );
                $color(val)
            }
        }

        impl std::ops::Deref for $color {
            type Target = f32;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $color {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

make_color!(Red);
make_color!(Blue);
make_color!(Green);
make_color!(Alpha);

impl Color {
    /// The color red
    #[allow(dead_code)]
    pub const RED: Color = Color {
        red: Red::new(1.),
        green: Green::new(0.),
        blue: Blue::new(0.),
        alpha: Alpha::new(0.),
    };

    /// The color blue
    #[allow(dead_code)]
    pub const BLUE: Color = Color {
        red: Red::new(0.),
        green: Green::new(0.),
        blue: Blue::new(1.),
        alpha: Alpha::new(0.),
    };

    /// The color green
    #[allow(dead_code)]
    pub const GREEN: Color = Color {
        red: Red::new(0.),
        green: Green::new(1.),
        blue: Blue::new(0.),
        alpha: Alpha::new(0.),
    };

    /// The color yellow
    #[allow(dead_code)]
    pub const YELLOW: Color = Color {
        red: Red::new(1.),
        green: Green::new(1.),
        blue: Blue::new(0.),
        alpha: Alpha::new(0.),
    };

    /// The color white
    #[allow(dead_code)]
    pub const WHITE: Color = Color {
        red: Red::new(1.),
        green: Green::new(1.),
        blue: Blue::new(1.),
        alpha: Alpha::new(0.),
    };

    /// The color white
    #[allow(dead_code)]
    pub const BLACK: Color = Color {
        red: Red::new(0.),
        green: Green::new(0.),
        blue: Blue::new(0.),
        alpha: Alpha::new(0.),
    };
    /// The color white
    #[allow(dead_code)]
    pub const GREY: Color = Color {
        red: Red::new(0.5),
        green: Green::new(0.5),
        blue: Blue::new(0.5),
        alpha: Alpha::new(0.),
    };

    /// Create a [`Color`] from `red`, `green`, and `blue` percentages
    #[allow(dead_code)]
    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red: Red::new(red),
            green: Green::new(green),
            blue: Blue::new(blue),
            alpha: Alpha::new(0.0),
        }
    }

    /// Create a [`Color`] from `red`, `green`, and `blue` percentages
    #[allow(dead_code)]
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: Red::new(red),
            green: Green::new(green),
            blue: Blue::new(blue),
            alpha: Alpha::new(alpha),
        }
    }

    /// `u32` representation of the [`Color`]
    #[inline]
    pub fn as_u32(&self) -> u32 {
        (*self.alpha * 255.).trunc_as_u32() << 24
            | (*self.red * 255.).trunc_as_u32() << 16
            | (*self.green * 255.).trunc_as_u32() << 8
            | (*self.blue * 255.).trunc_as_u32()
    }

    /// Linear blend the (red, green, and blue) channels with the `background` [`Color`]
    pub fn linear_alpha_blend(&mut self, background: Color) {
        let alpha = self.alpha.0;

        self.red.0 = alpha * self.red.0 + background.red.0 * (1.0 - alpha);
        self.blue.0 = alpha * self.blue.0 + background.blue.0 * (1.0 - alpha);
        self.green.0 = alpha * self.green.0 + background.green.0 * (1.0 - alpha);
    }
}

impl From<u32> for Color {
    #[allow(clippy::cast_possible_truncation)]
    fn from(val: u32) -> Color {
        let alpha = Alpha::new(f32::from((val >> 24) as u8) / 255.0);
        let red = Red::new(f32::from((val >> 16) as u8) / 255.0);
        let green = Green::new(f32::from((val >> 8) as u8) / 255.0);
        let blue = Blue::new(f32::from(val as u8) / 255.0);

        Color {
            red,
            green,
            blue,
            alpha,
        }
    }
}

/// The direction the player is currently facing
#[derive(Debug, Copy, Clone)]
pub enum PlayerDirection {
    /// Player is facing front
    Front,

    /// Player is facing back
    Back,

    /// Player is facing left
    Left,

    /// Player is facing right
    Right,
}
