//! Game logic for Handmade Ferris

#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_trait_bound)]

/// Required type for the tiles in a [`TileMap`]
pub trait Tile: Copy + Into<Color> {}

/// Type of tiles that inhabit the world
#[derive(Debug, Copy, Clone)]
enum TileType {
    /// Empty tile
    Empty,

    /// Wall tile
    Wall,

    /// Ladder tile
    Ladder
}

impl Tile for TileType {}

impl From<TileType> for Color {
    fn from(tile: TileType) -> Color {
        match tile {
            TileType::Wall   => Color::YELLOW,
            TileType::Empty  => Color::GREY,
            TileType::Ladder => Color::BLUE,
        }
    }
}

/// Number of slots for potential tile maps
const PREALLOC_TILE_MAPS: usize = 16;

/// dbg! macro that prints `{:#x?}`
#[allow(unused_macros)]
macro_rules! dbg_hex {
    () => {
        println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                println!("[{}:{}] {} = {:#x?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(gcdbg!($val)),+,)
    };
}

use game_state::{TILE_MAP_ROWS, TILE_MAP_COLUMNS, Button, Meters, Memory};
use game_state::{TILE_WIDTH, TILE_HEIGHT, TILE_HALF_WIDTH, TILE_HALF_HEIGHT, Chunk};
use game_state::{Truncate, GAME_WINDOW_HEIGHT};
use game_state::{Game, Result, Error, State, Rng};

/// Single chunk of tiles. A collection of these [`TileMap`] make up an entire [`World`]
pub struct TileMap<T: Tile, const WIDTH: usize, const HEIGHT: usize> {
    /// Tile map data
    data: [[T; WIDTH]; HEIGHT],
}

impl<T: Tile, const WIDTH: usize, const HEIGHT: usize> TileMap<T, WIDTH, HEIGHT> {
    /// Get the `T` from the given `x` and `y` offset into the tilemap
    ///
    /// # Panics
    ///
    /// * Requested (x, y) is outside the bounds of the [`TileMap`]
    pub fn get_tile_at(&self, x: u16, y: u16) -> &T {
        // Convert the coords to be standard coords
        // ^ |
        // | |
        // y0|
        //   +-----
        //    x0->
        let x = usize::from(x);
        let y = HEIGHT - 1 - usize::from(y);
        assert!(x < WIDTH,  "{:#x} larger than WIDTH: {:#x}", x, WIDTH);
        assert!(y < HEIGHT, "{:#x} larger than HEIGHT: {:#x}", y, HEIGHT);

        unsafe {
            self.data.get_unchecked(y).get_unchecked(x)
        }
    }

    /// Draw the [`TileMap`] via the given [`Game`]
    fn draw(&self, game: &mut Game) -> Result<()> {
        let display_lower_left_y = f32::from(game.height - TILE_HEIGHT);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let x = u16::try_from(x).unwrap();
                let y = u16::try_from(y).unwrap();

                // Get the current tile color
                let color: Color = (*self.get_tile_at(x, y)).into();

                // Get the upper left pixel of the current tile
                let x = x * TILE_WIDTH;
                let y = y * TILE_HEIGHT;

                // Draw the tile
                draw_rectangle(game, &color, f32::from(x), 
                    display_lower_left_y - f32::from(y), 
                    f32::from(TILE_WIDTH), f32::from(TILE_HEIGHT)
                )?;
            }
        }

        Ok(())
    }

    /// Set the given `T` to (`x`, `y`) in the [`TileMap`]
    ///
    /// # Panics
    ///
    /// * Requested (x, y) is outside the bounds of the [`TileMap`]
    pub fn set_tile_at(&mut self, x: u16, y: u16, val: T) {
        // Convert the coords to be standard coords
        // ^ |
        // | |
        // y0|
        //   +-----
        //    x0->
        let x = usize::from(x);
        let y = HEIGHT - 1 - usize::from(y);
        assert!(x < WIDTH,  "{:#x} larger than WIDTH: {:#x}", x, WIDTH);
        assert!(y < HEIGHT, "{:#x} larger than HEIGHT: {:#x}", y, HEIGHT);



        unsafe {
            let ptr = self.data.get_unchecked_mut(y).get_unchecked_mut(x);
            (*ptr) = val;
        }
    }
}

/// World containing many tile maps
#[derive(Debug)]
pub struct World<T: Tile, const WIDTH: usize, const HEIGHT: usize> {
    /// Tile maps in the world
    tile_maps: [*mut TileMap<T, WIDTH, HEIGHT>; PREALLOC_TILE_MAPS],

    /// (x, y, z) tile_map pairing which index corresponds to the index in `tile_maps`
    /// containg the pointer to the `tile_map`
    tile_map_indexes: [Option<(u32, u32, u8)>; PREALLOC_TILE_MAPS],

    /// Index to the next tile_map slot
    next_tile_map_index: usize,

    /// Number of meters to step per frame
    pub step_per_frame: Meters
}

impl<T: Tile, const WIDTH: usize, const HEIGHT: usize> World<T, WIDTH, HEIGHT> {
    /// Initialize the world from the given tilemaps
    pub fn init(&mut self) {
        self.tile_maps = [std::ptr::null_mut(); PREALLOC_TILE_MAPS];
        self.tile_map_indexes = [None; PREALLOC_TILE_MAPS];
        self.next_tile_map_index = 0;
        self.step_per_frame = Meters::new(0.1);
    }

    /// Allocate a new [`TileMap`] at chunk id (`x`, `y`)
    ///
    /// # Panics
    ///
    /// * Out of slots to hold tile maps
    pub fn alloc_tilemap_at(&mut self, memory: &mut Memory, x: u32, y: u32, z: u8) 
            -> &mut TileMap<T, WIDTH, HEIGHT>  {
        assert!(self.next_tile_map_index < PREALLOC_TILE_MAPS, "Out of tile map slot");

        println!("Allocating tile map at ({}, {}, {})", x, y, z);

        let curr_tile_index = self.next_tile_map_index;

        // Tile map wasn't found, allocate a new one
        let tile_map: *mut TileMap<T, WIDTH, HEIGHT> = memory.alloc();

        // Set the tile map index for this newly allocated tilemap
        self.tile_map_indexes[curr_tile_index] = Some((x, y, z));
        self.tile_maps[curr_tile_index] = tile_map;

        // Bump the tile map index
        self.next_tile_map_index += 1;

        unsafe {
            &mut *self.tile_maps[curr_tile_index]
        }
    }

    /// Get the [`TileMap`] at (`x`, `y`) in the World or allocate a new [`TileMap`] if
    /// the requested location is not yet allocated.
    ///
    /// # Panics
    ///
    /// * Sanity check of indexes is out of sync
    pub fn get_tilemap_at(&mut self, x: u32, y: u32, z: u8) 
            -> Option<&mut TileMap<T, WIDTH, HEIGHT>> {
        // Look for the requested (x, y) in the allocated tile maps and return the
        // pointer if found
        for (index, coord) in self.tile_map_indexes[..self.next_tile_map_index].iter().enumerate() {
            assert!(coord.is_some(), "next_tile_map_index out of sync");

            if coord.unwrap() == (x, y, z) {
                return unsafe { Some(&mut *self.tile_maps[index]) };
            }
        }

        // No tilemap was found
        None
    }

}

/// Update and render the current game state
///
/// # Panics
///
/// * On 16 bit machines
#[no_mangle]
pub extern fn game_update_and_render(game: &mut Game, state: &mut State) {
    // Initialize the game memory if not already initialized
    if !game.memory.initialized {
        let world: *mut World<TileType, TILE_MAP_COLUMNS, TILE_MAP_ROWS> = game.memory.alloc();

        // Initialize the world
        unsafe { 
            (*world).init();
        }

        // Game world is now initialized
        game.memory.initialized = true;
    }

    // 
    let res = _game_update_and_render(game, state);

    // Update the error code between the game logic library and the platform layer
    game.error = res;
}

/// Randomly initialize a tile map
#[allow(clippy::cast_possible_truncation)]
fn init_tile_map(rng: &mut Rng, 
                 tile_map: &mut TileMap<TileType, TILE_MAP_COLUMNS, TILE_MAP_ROWS>) {
    for y in 0..TILE_MAP_ROWS {
        for x in 0..TILE_MAP_COLUMNS {

            // Draw the floor/ceiling with doors
            if y == 0 || y == TILE_MAP_ROWS - 1 {
                let mid_point = TILE_MAP_COLUMNS / 2;
                if (mid_point-1..=mid_point+1).contains(&x) {
                    tile_map.set_tile_at(x as u16, y as u16, TileType::Empty);
                } else {
                    tile_map.set_tile_at(x as u16, y as u16, TileType::Wall);
                }
                continue;
            }

            // Draw the walls with doors
            if x == 0 || x == TILE_MAP_COLUMNS - 1 {
                let mid_point = TILE_MAP_ROWS / 2;
                if (mid_point-1..=mid_point+1).contains(&y) {
                    tile_map.set_tile_at(x as u16, y as u16, TileType::Empty);
                } else {
                    tile_map.set_tile_at(x as u16, y as u16, TileType::Wall);
                }
                continue;
            }

            // Randomly set values in a room
            if rng.next() % 64 == 0 {
                tile_map.set_tile_at(x as u16, y as u16, TileType::Ladder);
            }

            // Randomly set values in a room
            if rng.next() % 16 == 0 {
                tile_map.set_tile_at(x as u16, y as u16, TileType::Wall);
            }
        }
    }
}

/// Actual game logic code that can return a [`Result`]
fn _game_update_and_render(game: &mut Game, state: &mut State) -> Result<()> {
    // Blanket fill the screen with green to ensure all the screen is actually being drawn
    fill_screen(game, &Color::GREEN);

    // Get the world structure which is always at the beginning of the persistent memory
    let world = unsafe {
        #[allow(clippy::cast_ptr_alignment)]
        &mut *(game.memory.data.as_mut_ptr().cast::<World<TileType, TILE_MAP_COLUMNS, TILE_MAP_ROWS>>())
    };

    let mut new_player = state.player.position;

    for (button_id, is_pressed) in game.buttons.as_ref().iter().enumerate() {
        // Not pressed, ignore the button
        if !is_pressed {
            continue;
        }

        // Get the pressed button
        let button = Button::from_usize(button_id);

        // Based on the button pressed, move the player
        match button {
            Button::Up    => new_player.tile_rel_y += world.step_per_frame,
            Button::Down  => new_player.tile_rel_y -= world.step_per_frame,
            Button::Right => new_player.tile_rel_x += world.step_per_frame,
            Button::Left  => new_player.tile_rel_x -= world.step_per_frame,
            Button::DecreaseSpeed => {
                world.step_per_frame -= Meters::new(0.05);
                world.step_per_frame = world.step_per_frame.clamp(0.05, 1.0).into();
            }
            Button::IncreaseSpeed => {
                world.step_per_frame += Meters::new(0.05);
                world.step_per_frame = world.step_per_frame.clamp(0.05, 1.0).into();
            }
            Button::Count => {}
        }
    }

    // Update the player coordinates based on the movement. If the player has stepped
    // beyond the bounds of the current tile, update the position to the new tile.
    new_player.canonicalize();
    // dbg_hex!(new_player);

    let Chunk { chunk_id: tile_map_x, offset: x_offset } = new_player.x.into_chunk();
    let Chunk { chunk_id: tile_map_y, offset: y_offset } = new_player.y.into_chunk();

    let mut tile_map = world.get_tilemap_at(tile_map_x, tile_map_y, new_player.z);

    // If the requested tile map isn't allocated, allocate and init a new one
    if tile_map.is_none() {
        let new_map = world.alloc_tilemap_at(game.memory, tile_map_x, tile_map_y, 
            new_player.z);

        init_tile_map(&mut state.rng, new_map);

        // Set the newly created map to be drawn
        tile_map = Some(new_map);
    }

    assert!(tile_map.is_some(), "Failed to create a new tile map: ({:#x}, {:#x})", 
        tile_map_x, tile_map_y);

    // Always confirmed to be some, safe unwrap
    let tile_map = tile_map.unwrap();

    // Draw the tile map
    tile_map.draw(game)?;

    let display_lower_left_y = f32::from(GAME_WINDOW_HEIGHT);

    let tile_center_x = f32::from(x_offset * TILE_WIDTH + TILE_HALF_WIDTH);
    let tile_center_y = display_lower_left_y 
        - f32::from(y_offset * TILE_HEIGHT) 
        - f32::from(TILE_HALF_HEIGHT);

    // Draw the player. 
    let player_height = f32::from(TILE_HEIGHT) * 0.75; 
    let player_width  = f32::from(TILE_WIDTH)  * 0.75; 

    let player_bottom_center_x = tile_center_x + *new_player.tile_rel_x.into_pixels();
    let player_bottom_center_y = tile_center_y - *new_player.tile_rel_y.into_pixels() ;

    let player_x  = player_bottom_center_x - player_width / 2.;
    let player_y  = player_bottom_center_y - player_height;

    // Check that the potential moved to tile is valid (aka, zero)
    let mut valid = true; 

    // Handle the tile type
    
    let next_tile = tile_map.get_tile_at(x_offset, y_offset);

    // Block movement to walls
    if matches!(next_tile, &TileType::Wall) {
        valid = false; 
    }

    if matches!(next_tile, &TileType::Ladder) {
        new_player.z = (new_player.z + 1) % 2;
    }

    // If the move is valid, update the player
    if valid { 
        state.player.position = new_player;
    }

    // Debug draw the tile the player is currently standing on
    if tile_center_y < 0.0 {
        dbg!(display_lower_left_y);
        dbg!(y_offset);
        dbg!(y_offset * TILE_HEIGHT);
        dbg!(TILE_HALF_HEIGHT);
        dbg!(y_offset * TILE_HEIGHT - TILE_HALF_HEIGHT);
        dbg!(tile_center_x, tile_center_y);
    }
    draw_rectangle(game, &Color::BLACK, 
        tile_center_x - f32::from(TILE_HALF_WIDTH), 
        tile_center_y - f32::from(TILE_HALF_HEIGHT),
        f32::from(TILE_WIDTH), f32::from(TILE_HEIGHT))?;

    draw_rectangle(game, &Color::GREEN, player_x, player_y,
        player_width, player_height).unwrap();

    draw_rectangle(game, &Color::RED, 
        player_bottom_center_x - 2.0, 
        player_bottom_center_y - 2.0,
        4.0, 4.0).unwrap();

    Ok(()) 
}

/// Debug function to print a set of gradient squares to the display
fn _test_gradient(game: &mut Game) { 
    let height = u32::from(game.height); 
    let width  = u32::from(game.width);

    for col in 0..height { 
        for row in 0..width { 
            let index = col * width + row; 
            let color = (col % 256) << 8 | (row % 256);
            game.framebuffer[usize::try_from(index).unwrap()] = color;
        }; 
    } 
}

/// Color represented by red, green, and blue pigments
#[derive(Debug)]
pub struct Color {
    /// Percentage of red color pigment from 0.0 .. 1.0
    red: Red,

    /// Percentage of green color pigment from 0.0 .. 1.0
    green: Green,

    /// Percentage of blue color pigment from 0.0 .. 1.0
    blue: Blue,

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
            const fn new(val: f32) -> $color { assert!(0.0 <= val && val <= 1.0,
                concat!(stringify!($color), " value out of bounds [0.0..1.0]"));
            $color(val) } }

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

impl Color {
    /// The color red
    #[allow(dead_code)]
    const RED:   Color = Color { 
        red: Red::new(1.), green: Green::new(0.), blue: Blue::new(0.) 
    };

    /// The color blue
    #[allow(dead_code)]
    const BLUE:  Color = Color { 
        red: Red::new(0.), green: Green::new(0.), blue: Blue::new(1.) 
    };

    /// The color green
    #[allow(dead_code)]
    const GREEN: Color = Color { 
        red: Red::new(0.), green: Green::new(1.), blue: Blue::new(0.) 
    };

    /// The color yellow
    #[allow(dead_code)]
    const YELLOW: Color = Color { 
        red: Red::new(1.), green: Green::new(1.), blue: Blue::new(0.) 
    };

    /// The color white
    #[allow(dead_code)]
    const WHITE: Color = Color { 
        red: Red::new(1.), green: Green::new(1.), blue: Blue::new(1.) 
    };

    /// The color white
    #[allow(dead_code)]
    const BLACK: Color = Color { 
        red: Red::new(0.), green: Green::new(0.), blue: Blue::new(0.) 
    }; 
    /// The color white
    #[allow(dead_code)]
    const GREY: Color = Color { 
        red: Red::new(0.5), green: Green::new(0.5), blue: Blue::new(0.5) 
    };

    /// Create a [`Color`] from `red`, `green`, and `blue` percentages
    #[allow(dead_code)]
    const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red:   Red::new(red),
            green: Green::new(green),
            blue:  Blue::new(blue)
        }
    }

    /// `u32` representation of the [`Color`]
    #[inline]
    fn as_u32(&self) -> u32 {
        (*self.red   * 255.).trunc_as_u32() << 16 | 
        (*self.green * 255.).trunc_as_u32() <<  8 |
        (*self.blue  * 255.).trunc_as_u32()
    }
}

/// Fill the game display with the given [`Color`]
fn fill_screen(game: &mut Game, color: &Color) {
    for col in 0..game.height {
        for row in 0..game.width {
            let index = col * game.width + row;
            game.framebuffer[usize::try_from(index).unwrap()] = color.as_u32();
        }
    }
}

/// Fill a rectangle starting at the pixel (`pos_x`, `pos_y`) with a `width` and `height`
fn draw_rectangle(game: &mut Game, color: &Color, 
        pos_x: f32, pos_y: f32, width: f32, height: f32) -> Result<()> {
    let upper_left_x  = pos_x;
    let upper_left_y  = pos_y;
    let lower_right_x = pos_x + width;
    let lower_right_y = pos_y + height;

    let upper_left_x   = upper_left_x.trunc_as_u32().clamp(0,  u32::from(game.width));
    let lower_right_x  = lower_right_x.trunc_as_u32().clamp(0, u32::from(game.width));
    let upper_left_y   = upper_left_y.trunc_as_u32().clamp(0,  u32::from(game.height));
    let lower_right_y  = lower_right_y.trunc_as_u32().clamp(0, u32::from(game.height));

    // If the upper left corner is not the upper left corner, return;
    if upper_left_x > lower_right_x || upper_left_y > lower_right_y {
        return Err(Error::InvalidRectangle);
    }

    // Draw the valid rectangle
    for col in upper_left_y..lower_right_y {
        for row in upper_left_x..lower_right_x {
            let index = col * u32::from(game.width) + row;
            game.framebuffer[usize::try_from(index).unwrap()] = color.as_u32();
        }
    }

    // Success!
    Ok(())
}
