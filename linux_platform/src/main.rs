//! Linux platform for Handmade Ferris
#![feature(asm)]

mod dl;
use game_state::Game;

/// Target FPS for the game
const TARGET_FRAMES_PER_SECOND: f32 = 30.0;

/// Number of microseconds available per frame
///
/// Acutally do want this to truncate
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const MILLISECONDS_PER_FRAME: u128 = (1.0 / TARGET_FRAMES_PER_SECOND * 1000.) as u128;

/// Width of the game window
const GAME_WINDOW_WIDTH:  u32 = 960;

/// Height of the game window
const GAME_WINDOW_HEIGHT: u32 = 540;

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

    let time_begin = std::time::Instant::now();

    // Main event loop
    for frame in 0.. {
        // Begin the timer for this loop iteration
        let frame_start = std::time::Instant::now();

        game_code = game_code.reload();
        game_update_and_render = &game_code.game_update_and_render;

        // Get the next event from X11
        let event = window.check_event();

        // Event handler loop
        match event {
            Some(x11_rs::Event::Expose) => {
            }
            Some(x11_rs::Event::KeyPress) => {
            }
            Some(x11_rs::Event::Unknown(val)) => {
                println!("Unknown event: {}", val);
            }
            _ => { }
        }

        // Debug print the frames per second
        if frame > 0 && frame % 30 == 0 {
            println!("Frames: {} Frames/sec: {:6.2}", frame, 
                f64::from(frame) / time_begin.elapsed().as_secs_f64());
        }

        // Prepare the game state for the game logic
        let mut game_state = Game {
            framebuffer: &mut window.framebuffer,
            width: GAME_WINDOW_WIDTH,
            height: GAME_WINDOW_HEIGHT,
            error: Ok(())
        };

        // Call the event code
        game_update_and_render(&mut game_state);

        if let Err(e) = game_state.error {
            println!("ERR: {:?}", e);
        }

        // Place the updated framebuffer into the X11 window
        window.put_image();

        // Get the time it took to execute this frame
        let elapsed = frame_start.elapsed().as_millis();

        // Get the number of milliseconds remaining to hit the target frame count,
        // clamping the value to zero
        let remaining = MILLISECONDS_PER_FRAME.saturating_sub(elapsed);

        // If there is any remaining time needed to pad until the next frame, sleep for 
        // that duration
        if remaining > 0 {
            std::thread::sleep(std::time::Duration::from_millis(
                    remaining.try_into().unwrap()));
        }
    }
}
