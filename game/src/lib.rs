//! Game logic for Handmade Ferris

/// Provides the `truncate` trait for rounding `f32` to `u32`
trait Truncate {
    /// Truncate the given value 
    fn truncate(self) -> u32;
}

impl Truncate for f32 {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn truncate(self) -> u32 {
        self.floor() as u32
    }
}

use game_state::{Game, Result, Error};

/// Update and render the current game state
///
/// # Panics
///
/// * On 16 bit machines
#[no_mangle]
pub extern fn game_update_and_render(game: &mut Game) {
    let res = _game_update_and_render(game);

    // Update the error code between the game logic library and the platform layer
    game.error = res;
}

/// Actual game logic code that can return a [`Result`]
fn _game_update_and_render(game: &mut Game) -> Result<()> {
    fill_screen(game, &Color::RED);

    draw_rectangle(game, &Color::BLUE, 5., 5., 100., 100.)?;

    Ok(())
}

/// Debug function to print a set of gradient squares to the display
fn test_gradient(game: &mut Game) {
    for col in 0..game.height {
        for row in 0..game.width {
            let index = col * game.width + row;
            let color = col % 256 << 8 | row % 256;
            game.framebuffer[usize::try_from(index).unwrap()] = color as u32;
        }
    }
}

/// Color represented by red, green, and blue pigments
struct Color {
    /// Percentage of red color pigment from 0.0 .. 1.0
    red: f32,

    /// Percentage of green color pigment from 0.0 .. 1.0
    green: f32,

    /// Percentage of blue color pigment from 0.0 .. 1.0
    blue: f32,

}

impl Color {
    /// The color red
    const RED:   Color = Color { red: 1., green: 0., blue: 0. };

    /// The color blue
    const BLUE:  Color = Color { red: 0., green: 0., blue: 1. };

    /// The color green
    const GREEN: Color = Color { red: 0., green: 1., blue: 0. };

    /// The color white
    const WHITE: Color = Color { red: 1., green: 1., blue: 1. };

    /// The color white
    const BLACK: Color = Color { red: 0., green: 0., blue: 0. };

    /// `u32` representation of the [`Color`]
    fn as_u32(&self) -> u32 {
        (255. * self.red).truncate()   << 16 | 
        (255. * self.green).truncate() <<  8 |
        (255. * self.blue).truncate()  <<  0
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

/// Fill a rectangle starting at the pixel (`upper_left_x`, `upper_left_y`) and ending at
/// pixel (`lower_right_x`, `lower_right_y`)
fn draw_rectangle(game: &mut Game, color: &Color, 
        upper_left_x:  f32, upper_left_y:  f32, lower_right_x: f32, lower_right_y: f32) 
        -> Result<()> {

    let upper_left_x   = upper_left_x.truncate().clamp(0,game.width);
    let lower_right_x  = lower_right_x.truncate().clamp(0, game.width);
    let upper_left_y   = upper_left_y.truncate().clamp(0, game.height);
    let lower_right_y  = lower_right_y.truncate().clamp(0, game.height);

    // If the upper left corner is not the upper left corner, return;
    if upper_left_x > lower_right_x || upper_left_y > lower_right_y {
        return Err(Error::InvalidRectangle);
    }

    // Draw the valid rectangle
    for col in upper_left_y..lower_right_y {
        for row in upper_left_x..lower_right_x {
            let index = col * game.width + row;
            game.framebuffer[usize::try_from(index).unwrap()] = color.as_u32();
        }
    }

    // Success!
    Ok(())
}
