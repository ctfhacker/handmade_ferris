//! Game logic for Handmade Ferris

#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_trait_bound)]

/// Required type for the tiles in a [`TileMap`]
trait Tile: Copy + Into<Color> {}

impl Tile for u8 {}

use game_state::{TILE_MAP_ROWS, TILE_MAP_COLUMNS, Button, Meters};
use game_state::{TILE_WIDTH, TILE_HEIGHT, Chunk, Truncate};
use game_state::{Game, Result, Error, State};

/// Generic tile map data
struct TileMap<T: Tile, const WIDTH: usize, const HEIGHT: usize> 
{
    /// Tile map data
    data: [[T; WIDTH]; HEIGHT],
}

impl<T: Tile, const WIDTH: usize, const HEIGHT: usize> TileMap<T, WIDTH, HEIGHT> 
{
    /// Create a new [`TileMap`]
    const fn new(data: [[T; WIDTH]; HEIGHT]) -> Self {
        Self { data }
    }

    /// Get the `T` from the given `x` and `y` offset into the tilemap
    pub fn get_tile_at(&self, x: u16, y: u16) -> &T {
        // Convert the coords to be standard coords
        // ^ |
        // | |
        // y0|
        //   +-----
        //    x0->
        let x = usize::from(x);
        let y = HEIGHT - 1 - usize::from(y);
        assert!(x < WIDTH,  "{} larger than WIDTH: {}", x, WIDTH);
        assert!(y < HEIGHT, "{} larger than HEIGHT: {}", y, HEIGHT);

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
}

/// World containing many tile maps
struct World<T: Tile, const WIDTH: usize, const HEIGHT: usize> 
{
    /// Tile maps in the world
    tile_maps: [TileMap<T, WIDTH, HEIGHT>; 2],

    /// Number of meters to step per frame
    step_per_frame: Meters
}

impl<T: Tile, const WIDTH: usize, const HEIGHT: usize> World<T, WIDTH, HEIGHT> {
    /// Creatd a new world from the given game and 
    pub fn new(tile_maps: [TileMap<T, WIDTH, HEIGHT>; 2]) -> Self {

        Self {
            tile_maps,
            step_per_frame: Meters::new(0.3)
        }
    }

    /// Get the [`TileMap`] at (`x`, `y`) in the World
    pub fn get_tilemap_at(&self, x: usize, y: usize) -> &TileMap<T, WIDTH, HEIGHT> {
        let index = y * HEIGHT + x;
        assert!(index < self.tile_maps.len());

        unsafe {
            self.tile_maps.get_unchecked(index)
        }
    }
}

/// Update and render the current game state
///
/// # Panics
///
/// * On 16 bit machines
#[no_mangle]
pub extern fn game_update_and_render(game: &mut Game, state: &mut State) {
    let res = _game_update_and_render(game, state);

    // Update the error code between the game logic library and the platform layer
    game.error = res;
}

/// Actual game logic code that can return a [`Result`]
fn _game_update_and_render(game: &mut Game, state: &mut State) -> Result<()> {
    // Blanket fill the screen with green to ensure all the screen is actually being drawn
    fill_screen(game, &Color::GREEN);

    let tile_map0 = TileMap::<u8, TILE_MAP_COLUMNS, TILE_MAP_ROWS>::new([
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 2],
        [2, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        [2, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0],
        [2, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0],
        [2, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
    ]);

    let tile_map1 = TileMap::<u8, TILE_MAP_COLUMNS, TILE_MAP_ROWS>::new([
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
    ]);


    let world = World::new([tile_map0, tile_map1]);

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
            Button::Up    => new_player.tile_rel_y -= world.step_per_frame,
            Button::Down  => new_player.tile_rel_y += world.step_per_frame,
            Button::Right => new_player.tile_rel_x += world.step_per_frame,
            Button::Left  => new_player.tile_rel_x -= world.step_per_frame,
            Button::Count => {}
        }
    }

    // Update the player coordinates based on the movement
    new_player.canonicalize();

    dbg!(new_player);

    let Chunk { chunk_id: tile_map_x, offset: x_offset } = new_player.x.into_chunk();
    let Chunk { chunk_id: tile_map_y, offset: y_offset } = new_player.y.into_chunk();

    let tile_map = world.get_tilemap_at(
        usize::try_from(tile_map_x).unwrap(),
        usize::try_from(tile_map_y).unwrap());

    // Draw the tile map
    tile_map.draw(game)?;

    let display_lower_left_y = f32::from(game.height - TILE_HEIGHT);

    let tile_upper_left_x = x_offset * TILE_WIDTH;
    let tile_upper_left_y = display_lower_left_y - f32::from(y_offset * TILE_HEIGHT);

    // Draw the player
    let player_height = f32::from(TILE_HEIGHT) * 0.75; 
    let player_width  = f32::from(TILE_WIDTH)  * 0.75; 
    let player_upper_left_x  = *new_player.tile_rel_x.into_pixels() 
        + f32::from(tile_upper_left_x);
    let player_upper_left_y  = *new_player.tile_rel_y.into_pixels() + tile_upper_left_y;

    // Check that the potential moved to tile is valid (aka, zero)
    let mut valid = true; 
    if !matches!(tile_map.get_tile_at(x_offset, y_offset), 0) {
        valid = false; 
    }

    // If the move is valid, update the player
    if valid { 
        state.player.position = new_player;
    }

    draw_rectangle(game, &Color::BLACK, f32::from(tile_upper_left_x), tile_upper_left_y,
        f32::from(TILE_WIDTH), f32::from(TILE_HEIGHT))?;

    // println!("Player: ({}, {})", new_player_x.floor(), new_player_y.floor());
    draw_rectangle(game, &Color::BLUE, player_upper_left_x, player_upper_left_y,
        player_width, player_height)?;

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
struct Color {
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
