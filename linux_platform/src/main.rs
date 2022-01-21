//! Linux platform for Handmade Ferris
#![feature(asm)]

mod dl;
use game_state::{GAME_WINDOW_WIDTH, GAME_WINDOW_HEIGHT, Game, Button, Memory, BitmapAsset};

/// Target FPS for the game
const TARGET_FRAMES_PER_SECOND: f32 = 30.0;

/// Number of microseconds available per frame
///
/// Acutally do want this to truncate
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const MILLISECONDS_PER_FRAME: u128 = (1.0 / TARGET_FRAMES_PER_SECOND * 1000.) as u128;

fn main() {
    let mut window = x11_rs::SimpleWindow::build()
        .x(0)
        .y(0)
        .width(u32::from(GAME_WINDOW_WIDTH))
        .height(u32::from(GAME_WINDOW_HEIGHT))
        .border_width(1)
        .border(0)
        .background(1)
        .finish()
        .expect("Failed to create X11 simple window");

    window.create_image();

    window.put_image();

    // Load the game logic library
    let mut game_code = dl::get_game_funcs();
    let mut game_update_and_render;

    let time_begin = std::time::Instant::now();

    // Get the reset game state
    let mut state = game_state::State::reset();

    let mut buttons = [false; Button::Count as usize];

    let mut memory = Memory::new(2 * 1024 * 1024);

    let data = std::fs::read("test2.bmp").unwrap();
    let offset = u32::from_le_bytes(data[0x0a..0x0a + 4].try_into().unwrap()) as usize;
    let width  = u32::from_le_bytes(data[0x12..0x12 + 4].try_into().unwrap());
    let height = u32::from_le_bytes(data[0x16..0x16 + 4].try_into().unwrap());
    println!("Offset: {:#x}", offset);
    let asset = BitmapAsset { width, height, data: &data[offset..] };

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
            Some(x11_rs::Event::KeyPress(key)) => {
                let button = match key {
                    'w' => Some(Button::Up),
                    'a' => Some(Button::Left),
                    's' => Some(Button::Down),
                    'd' => Some(Button::Right),
                    'n' => Some(Button::DecreaseSpeed),
                    'm' => Some(Button::IncreaseSpeed),
                      _ => None
                };
    
                if let Some(button) = button {
                    buttons[button as usize] = true;
                }
            }
            Some(x11_rs::Event::KeyRelease(key)) => {
                let button = match key {
                    'w' => Some(Button::Up),
                    'a' => Some(Button::Left),
                    's' => Some(Button::Down),
                    'd' => Some(Button::Right),
                    'n' => Some(Button::DecreaseSpeed),
                    'm' => Some(Button::IncreaseSpeed),
                      _ => None
                };
    
                if let Some(button) = button {
                    buttons[button as usize] = false;
                }
            }
            Some(x11_rs::Event::Unknown(val)) => {
                println!("Unknown event: {}", val);
            }
            Some(x11_rs::Event::Expose) | None => {
            }
        }

        // Debug print the frames per second
        if frame > 0 && frame % 30 == 0 {
            println!("Frames: {} Frames/sec: {:6.2}", frame, 
                f64::from(frame) / time_begin.elapsed().as_secs_f64());
        }

        // Prepare the game state for the game logic
        let mut game = Game {
            framebuffer: &mut window.framebuffer,
            width:       GAME_WINDOW_WIDTH,
            height:      GAME_WINDOW_HEIGHT,
            error:       Ok(()),
            buttons:     &buttons,
            memory:      &mut memory,
            asset:       &asset,
        };

        // Call the event code
        game_update_and_render(&mut game, &mut state);

        if let Err(e) = game.error {
            println!("ERR: {:?}", e);
            panic!();
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
