//! Linux platform for Handmade Ferris
#![feature(const_format_args)]
#![feature(variant_count)]

use core::mem::variant_count;
use std::io::{Read, Write};

mod dl;
use game_state::{BitmapAsset, Button, Game, Memory, GAME_WINDOW_HEIGHT, GAME_WINDOW_WIDTH};
use game_state::{PlayerBitmap, PlayerDirection, MEMORY_LENGTH, STATE_SIZE};
use game_state::{MEMORY_BASE_ADDR, MILLISECONDS_PER_FRAME};

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

/// The state of a looping input
struct LoopState {
    /// The state of the game at the start of the loop
    game_state: game_state::State,

    /// The state of memory to start the loop
    memory: Vec<u8>,

    /// The index for the next input buttons
    input_index: usize,

    /// The button sequence of the loop
    buttons: Vec<[bool; variant_count::<Button>()]>,
}

impl LoopState {
    pub fn next_input(&mut self) -> [bool; variant_count::<Button>()] {
        let index = self.input_index;
        self.input_index = (self.input_index + 1) % self.buttons.len();
        self.buttons[index]
    }

    pub fn write_to_disk(&self, filename: &str) {
        // Open the file to write
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .expect("Failed to open loop state file");

        // Write the raw data slice to the file
        let game_state_bytes = unsafe {
            std::slice::from_raw_parts(
                (&self.game_state as *const game_state::State) as *const u8,
                STATE_SIZE,
            )
        };
        file.write(&game_state_bytes).unwrap();

        // Write the data len to the file
        // [ len u64 ][ memory bytes]
        let data_len = self.memory.len() as u64;
        file.write(&data_len.to_le_bytes()).unwrap();
        assert!(self.memory.len() == MEMORY_LENGTH);
        file.write(self.memory.as_slice()).unwrap();

        // Write the buttons to the file
        // [ len u64 ][ button bytes]
        let num_buttons = self.buttons.len() as u64;
        file.write(&num_buttons.to_le_bytes()).unwrap();

        for buttons in &self.buttons {
            for button in buttons {
                file.write(&[*button as u8]).unwrap();
            }
        }
    }

    // Read a saved loop from disk
    pub fn read_from_disk(&self, filename: &str) -> Self {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(filename)
            .expect("Failed to open loop state file");

        // Read the game state from the file
        let mut game_state_data = [0u8; STATE_SIZE];
        file.read(&mut game_state_data)
            .expect("Failed to read loop game state");

        // Raw data case back to the game state
        let game_state = unsafe { *game_state_data.as_ptr().cast::<game_state::State>() };

        // Read the memory length
        let mut memory_len = [0u8; 8];
        file.read(&mut memory_len)
            .expect("Failed to read size of loop memory");
        let memory_len = u64::from_le_bytes(memory_len);
        assert!(
            memory_len as usize == MEMORY_LENGTH,
            "Looping memory different than game memory"
        );

        // Read the memory bytes from disk
        let mut memory = vec![0u8; memory_len as usize];
        file.read(memory.as_mut_slice())
            .expect("Failed to read loop memory");

        // Read the number of buttons
        let mut num_buttons = [0u8; 8];
        file.read(&mut num_buttons)
            .expect("Failed to read number of loop buttons");
        let num_buttons = u64::from_le_bytes(num_buttons);

        // Read the buttons from the file
        let mut buttons = vec![[false; variant_count::<Button>()]; num_buttons as usize];
        for curr_buttons in buttons.iter_mut() {
            let mut tmp_buttons = [0u8; 6];
            file.read(&mut tmp_buttons)
                .expect("Failed to read loop buttons");

            tmp_buttons
                .iter_mut()
                .enumerate()
                .for_each(|(i, x)| curr_buttons[i] = *x != 0);
        }

        Self {
            game_state,
            memory,
            input_index: 0,
            buttons,
        }
    }
}

enum GameplayState {
    Normal,
    LoopRecording,
    LoopPlayback,
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

    // Add a player
    state.add_player();

    // Current button states for the game
    let mut buttons = [false; variant_count::<Button>()];

    // Persistent memory for the game
    let mut memory = Memory::new();

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

    let mut looping = GameplayState::Normal;

    let mut looping_state = LoopState {
        game_state: state.clone(),
        memory: Vec::new(),
        buttons: Vec::with_capacity(256),
        input_index: 0,
    };

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
                    'p' => {
                        // Play a recording from disk
                        looping_state = looping_state.read_from_disk("loop.hmi");
                        looping = GameplayState::LoopPlayback;
                        None
                    }
                    'l' => {
                        // Normal -> Recording -> Playback -> Normal
                        match looping {
                            GameplayState::Normal => {
                                println!("Loop: recording..");

                                // Initialize the loop state
                                looping_state = LoopState {
                                    game_state: state.clone(),
                                    memory: memory.data_as_vec(),
                                    buttons: Vec::with_capacity(256),
                                    input_index: 0,
                                };

                                // Goto the recording state
                                looping = GameplayState::LoopRecording;
                            }
                            GameplayState::LoopRecording => {
                                // Goto the playback state
                                println!("Loop: playback..");

                                looping_state.write_to_disk("loop.hmi");
                                looping_state = looping_state.read_from_disk("loop.hmi");

                                looping = GameplayState::LoopPlayback;
                            }
                            GameplayState::LoopPlayback => {
                                // Goto the normal state
                                println!("Loop: stop..");
                                looping = GameplayState::Normal;
                                buttons = [false; variant_count::<Button>()];
                            }
                        }

                        None
                    }
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

        match looping {
            GameplayState::LoopRecording => {
                looping_state.buttons.push(buttons.clone());
            }
            GameplayState::LoopPlayback => {
                // If at the beginning of the loop, reset the memory
                if looping_state.input_index == 0 {
                    println!("Loop reset..");

                    // Restore the snapshot memory back to the start of the snapshot
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            looping_state.memory.as_ptr(),
                            MEMORY_BASE_ADDR as *mut u8,
                            MEMORY_LENGTH,
                        );
                    }

                    // Reset the game state
                    state = looping_state.game_state.clone();
                }

                buttons = looping_state.next_input();
            }
            GameplayState::Normal => {
                // Nothing to do, game play as normal
            }
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
