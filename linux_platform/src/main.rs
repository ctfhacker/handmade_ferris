//! Linux platform for Handmade Ferris
#![feature(const_format_args)]
#![feature(variant_count)]

use core::mem::variant_count;

mod dl;
use game_state::MILLISECONDS_PER_FRAME;
use game_state::{BitmapAsset, Button, Game, Memory, GAME_WINDOW_HEIGHT, GAME_WINDOW_WIDTH};
use game_state::{PlayerBitmap, PlayerDirection};

use vector::Vector2;

/// Loads the front/left/right/back player assets from `assets/early_data/test/test_hero_`
macro_rules! load_asset {
    ($name:ident) => {
        // Get the path for this asset
        let path = concat!("assets/early_data/test/test_hero_", stringify!($name));
        let cape = format!("{}_cape.bmp", path);
        let torso = format!("{}_torso.bmp", path);
        let head = format!("{}_head.bmp", path);
        dbg!(&path, &cape, &torso, &head);

        // Read each of the pieces of the asset
        let cape = std::fs::read(cape).unwrap();
        let torso = std::fs::read(torso).unwrap();
        let head = std::fs::read(head).unwrap();

        // Create the player bitmap for this asset
        let $name = PlayerBitmap::from(
            BitmapAsset::from_data(&head),
            BitmapAsset::from_data(&torso),
            BitmapAsset::from_data(&cape),
            Vector2::new(73.0, 174.0),
        );
    };
}

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

    // Current button states for the game
    let mut buttons = [false; variant_count::<Button>()];

    // Persistent memory for the game
    let mut memory = Memory::new(2 * 1024 * 1024);

    // Load the player assets
    load_asset!(front);
    load_asset!(left);
    load_asset!(right);
    load_asset!(back);

    // Set the assets into the player assets array
    let mut player_assets = [&front, &front, &front, &front];
    player_assets[PlayerDirection::Front as usize] = &front;
    player_assets[PlayerDirection::Back as usize] = &back;
    player_assets[PlayerDirection::Left as usize] = &left;
    player_assets[PlayerDirection::Right as usize] = &right;

    let background = std::fs::read("assets/early_data/test/test_background.bmp")
        .expect("Failed to read background asset");
    let background = BitmapAsset::from_data(&background);

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
                    _ => None,
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
                    _ => None,
                };

                if let Some(button) = button {
                    buttons[button as usize] = false;
                }
            }
            Some(x11_rs::Event::Unknown(val)) => {
                println!("Unknown event: {}", val);
            }
            Some(x11_rs::Event::Expose) | None => {}
        }

        // Debug print the frames per second
        if frame > 0 && frame % 120 == 0 {
            println!(
                "Frames: {} Frames/sec: {:6.2}",
                frame,
                f64::from(frame) / time_begin.elapsed().as_secs_f64()
            );
        }

        // Prepare the game state for the game logic
        let mut game = Game {
            framebuffer: &mut window.framebuffer,
            width: GAME_WINDOW_WIDTH,
            height: GAME_WINDOW_HEIGHT,
            error: Ok(()),
            buttons: &buttons,
            memory: &mut memory,
            background: &background,
            player_assets,
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
        let remaining = (MILLISECONDS_PER_FRAME as u128).saturating_sub(elapsed);

        // If there is any remaining time needed to pad until the next frame, sleep for
        // that duration
        if remaining > 0 {
            std::thread::sleep(std::time::Duration::from_millis(
                remaining.try_into().unwrap(),
            ));
        }
    }
}
