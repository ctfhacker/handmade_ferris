//! Game logic for Handmade Ferris

#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_fn_fn_ptr_basics)]

/// Provides the `truncate` trait for rounding `f32` to `u32`
trait Truncate {
    /// Truncate the given value 
    fn truncate(self) -> u32;
}

impl Truncate for f32 {
    #[inline]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn truncate(self) -> u32 {
        self as u32
    }
}

use game_state::{Game, Result, Error, State};

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
    let tile_map: [[u8; 16]; 9] = [
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 2],
        [2, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 2],
        [2, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 2],
        [2, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 2],
        [2, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
    ];
    
    for button in game.buttons_pressed {
        match *button {
            'w' => state.player_y -= 1.,
            's' => state.player_y += 1.,
            'a' => state.player_x -= 1.,
            'd' => state.player_x += 1.,
              _ => {}
        }
    }

    // Draw the tile map
    for (row_index, row) in tile_map.iter().enumerate() {
        for (col_index, col) in row.iter().enumerate() {
            let color = match col {
                1 => Color::YELLOW,
                2 => Color::RED,
                _ => Color::GREY,
            };

            let x = u16::try_from(col_index).unwrap() * game_state::TILE_WIDTH;
            let y = u16::try_from(row_index).unwrap() * game_state::TILE_HEIGHT;

            draw_rectangle(game, &color, f32::from(x), f32::from(y), 
                f32::from(game_state::TILE_WIDTH), f32::from(game_state::TILE_HEIGHT))?;
        }
    }

    // Draw the player
    let player_height = f32::from(game_state::TILE_HEIGHT) * 0.75;
    let player_width  = f32::from(game_state::TILE_WIDTH)  * 0.75;
    let x = state.player_x - (0.5 * player_width);
    let y = state.player_y - player_height;

    draw_rectangle(game, &Color::BLUE, f32::from(x), f32::from(y), 
        f32::from(player_width), f32::from(player_height))?;
 
    Ok(())
}

/// Debug function to print a set of gradient squares to the display
fn test_gradient(game: &mut Game) {
    let height = u32::from(game.height);
    let width  = u32::from(game.width);

    for col in 0..u32::from(game.height) {
        for row in 0..width {
            let index = col * width + row;
            let color = col % 256 << 8 | row % 256;
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

/// Creates the bounded color values to percentages of [0.0..1.0]
macro_rules! make_color {
    ($color:ident) => {
        /// Red color bounded to the percentage of 0.0 to 1.0
        struct $color(f32);

        impl $color {
            /// Create a new [`Red`] percentage, checked at compile time that it is within the
            /// bounds [0.0..1.0]
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

impl Color {
    /// The color red
    const RED:   Color = Color { 
        red: Red::new(1.), green: Green::new(0.), blue: Blue::new(0.) 
    };

    /// The color blue
    const BLUE:  Color = Color { 
        red: Red::new(0.), green: Green::new(0.), blue: Blue::new(1.) 
    };

    /// The color green
    const GREEN: Color = Color { 
        red: Red::new(0.), green: Green::new(1.), blue: Blue::new(0.) 
    };

    /// The color yellow
    const YELLOW: Color = Color { 
        red: Red::new(1.), green: Green::new(1.), blue: Blue::new(0.) 
    };

    /// The color white
    const WHITE: Color = Color { 
        red: Red::new(1.), green: Green::new(1.), blue: Blue::new(1.) 
    };

    /// The color white
    const BLACK: Color = Color { 
        red: Red::new(0.), green: Green::new(0.), blue: Blue::new(0.) 
    }; 
    /// The color white
    const GREY: Color = Color { 
        red: Red::new(0.5), green: Green::new(0.5), blue: Blue::new(0.5) 
    };

    /// Create a [`Color`] from `red`, `green`, and `blue` percentages
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
        (*self.red   * 255.).truncate()   << 16 | 
        (*self.green * 255.).truncate() <<  8 |
        (*self.blue  * 255.).truncate()  <<  0
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
    let lower_right_y = pos_y + width;

    let upper_left_x   = upper_left_x.truncate().clamp(0,  u32::from(game.width));
    let lower_right_x  = lower_right_x.truncate().clamp(0, u32::from(game.width));
    let upper_left_y   = upper_left_y.truncate().clamp(0,  u32::from(game.height));
    let lower_right_y  = lower_right_y.truncate().clamp(0, u32::from(game.height));

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
