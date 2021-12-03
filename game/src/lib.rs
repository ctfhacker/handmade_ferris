//! Game logic for Handmade Ferris

#![feature(asm)]
use game_state::Game;

/// Update and render the current game state
///
/// # Panics
///
/// * On 16 bit machines
#[no_mangle]
pub extern fn game_update_and_render(game: &mut Game) {
    for col in 0..game.width {
        for row in 0..game.height {
            let index = col * game.width + row;
            let color = col % 256 << 16 | row % 256;
            game.framebuffer[usize::try_from(index).unwrap()] = color as u32;
        }
    }
}
