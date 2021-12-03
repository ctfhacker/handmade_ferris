//! Linux platform for Handmade Ferris
#![feature(asm)]

mod dl;
use game_state::Game;

/// Target FPS for the game
const TARGET_FRAMES_PER_SECOND: f32 = 30.0;

/// Resulting number of 
const MICROSECONDS_PER_FRAME: f32 = 1.0 / TARGET_FRAMES_PER_SECOND * 1000. * 1000.;

/// Width of the game window
const GAME_WINDOW_WIDTH:  u32 = 800;

/// Height of the game window
const GAME_WINDOW_HEIGHT: u32 = 800;

fn main() {
    let mut window = x11_rs::SimpleWindow::build()
        .x(0)
        .y(0)
        .width(GAME_WINDOW_WIDTH)
        .height(GAME_WINDOW_HEIGHT)
        .border_width(1)
        .border(0)
        .background(1)
        .finish()
        .expect("Failed to create X11 simple window");

    window.create_image();

    /*
    for col in 0..800 {
        for row in 0..800 {
            let index = col * 800 + row;
            let color = (col % 256) << 8 | (row % 256);
            window.framebuffer[index] = u32::try_from(color).unwrap();
        }
    }
    */

    window.put_image();

    // Load the game logic library
    let mut game_code = dl::get_game_funcs();
    let mut game_update_and_render;

    // Main event loop
    loop {
        // Begin the timer for this loop iteration
        let start = std::time::Instant::now();

        // Get the next event from X11
        let event = window.next_event();
        println!("Event: {:?}", event);

        // Event handler loop
        match event {
            x11_rs::Event::Expose => window.put_image(),
            x11_rs::Event::KeyPress => {
                println!("Key pressed");

                game_code = game_code.reload();
                game_update_and_render = &game_code.game_update_and_render;

                // Prepare the game state for the game logic
                let mut game_state = Game {
                    framebuffer: &mut window.framebuffer,
                    width: GAME_WINDOW_WIDTH,
                    height: GAME_WINDOW_HEIGHT,
                };

                game_update_and_render(&mut game_state);

                // Place the updated framebuffer into the X11 window
                window.put_image();
            }
            x11_rs::Event::Unknown(val) => {
                println!("Unknown event: {}", val);
            }
        }

        // Wait for the time 
        println!("NS Per frame: {}", start.elapsed().as_micros());
    }
}
