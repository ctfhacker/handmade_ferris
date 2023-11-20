//! Game logic for Handmade Ferris

#![feature(const_fn_floating_point_arithmetic)]
#![feature(stmt_expr_attributes)]

use game_state::{BitmapAsset, Button, Memory, Meters, TILE_MAP_COLUMNS, TILE_MAP_ROWS};
use game_state::{ChunkVector, Error, Game, Result, Rng, State};
use game_state::{Color, PlayerDirection, Truncate, GAME_WINDOW_HEIGHT};
use game_state::{TILE_HALF_HEIGHT, TILE_HALF_WIDTH, TILE_HEIGHT, TILE_WIDTH};

use vector::Vector2;

/// Type of tiles that inhabit the world
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TileType {
    /// Empty tile
    Empty,

    /// Wall tile
    Wall,

    /// Ladder tile
    Ladder,
}

impl From<TileType> for Color {
    fn from(tile: TileType) -> Color {
        match tile {
            TileType::Wall => Color::YELLOW,
            TileType::Empty => Color::GREY,
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
                println!("[{}:{}] {} = {:x?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(gcdbg!($val)),+,)
    };
}

/// Single chunk of tiles. A collection of these [`TileMap`] make up an entire [`World`]
pub struct TileMap<const WIDTH: usize, const HEIGHT: usize> {
    /// Tile map data
    data: [[TileType; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> TileMap<WIDTH, HEIGHT> {
    /// Get the `T` from the given `x` and `y` offset into the tilemap
    ///
    /// # Panics
    ///
    /// * Requested (x, y) is outside the bounds of the [`TileMap`]
    pub fn get_tile_at(&self, pos: Vector2<u16>) -> &TileType {
        // Convert the coords to be standard coords
        // ^ |
        // | |
        // y0|
        //   +-----
        //    x0->
        let x = usize::from(pos.x);
        let y = HEIGHT - 1 - usize::from(pos.y);
        assert!(x < WIDTH, "{:#x} larger than WIDTH: {:#x}", x, WIDTH);
        assert!(y < HEIGHT, "{:#x} larger than HEIGHT: {:#x}", y, HEIGHT);

        unsafe { self.data.get_unchecked(y).get_unchecked(x) }
    }

    /// Draw the [`TileMap`] via the given [`Game`]
    fn draw(&self, game: &mut Game) -> Result<()> {
        let display_lower_left_y = f32::from(game.height - TILE_HEIGHT);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let tile_pos = Vector2::new(x, y).into();

                // Get the current tile color
                let curr_tile = self.get_tile_at(tile_pos);

                // Don't draw empty tiles
                if matches!(curr_tile, TileType::Empty) {
                    continue;
                }

                // Get the color of the current tile
                let color: Color = (*curr_tile).into();

                let pixel_pos = Vector2::new(tile_pos.x * TILE_WIDTH, tile_pos.y * TILE_HEIGHT);

                // Get the upper left pixel of the current tile
                let pixel_pos = Vector2::new(
                    f32::from(pixel_pos.x),
                    display_lower_left_y - f32::from(pixel_pos.y),
                );

                // Draw the tile
                draw_rectangle(
                    game,
                    &color,
                    pixel_pos,
                    f32::from(TILE_WIDTH),
                    f32::from(TILE_HEIGHT),
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
    pub fn set_tile_at(&mut self, x: u16, y: u16, val: TileType) {
        // Convert the coords to be standard coords
        // ^ |
        // | |
        // y0|
        //   +-----
        //    x0->
        let x = usize::from(x);
        let y = HEIGHT - 1 - usize::from(y);
        assert!(x < WIDTH, "{:#x} larger than WIDTH: {:#x}", x, WIDTH);
        assert!(y < HEIGHT, "{:#x} larger than HEIGHT: {:#x}", y, HEIGHT);

        unsafe {
            let ptr = self.data.get_unchecked_mut(y).get_unchecked_mut(x);
            (*ptr) = val;
        }
    }
}

/// World containing many tile maps
#[derive(Debug)]
pub struct World<const WIDTH: usize, const HEIGHT: usize> {
    /// Tile maps in the world
    tile_maps: [*mut TileMap<WIDTH, HEIGHT>; PREALLOC_TILE_MAPS],

    /// (x, y, z) tile_map pairing which index corresponds to the index in `tile_maps`
    /// containg the pointer to the `tile_map`
    tile_map_indexes: [Option<(Vector2<u32>, u8)>; PREALLOC_TILE_MAPS],

    /// Index to the next tile_map slot
    next_tile_map_index: usize,

    /// Number of meters to step per frame
    pub step_per_frame: Meters,
}

impl<const WIDTH: usize, const HEIGHT: usize> World<WIDTH, HEIGHT> {
    /// Initialize the world from the given tilemaps
    pub fn init(&mut self) {
        self.tile_maps = [std::ptr::null_mut(); PREALLOC_TILE_MAPS];
        self.tile_map_indexes = [None; PREALLOC_TILE_MAPS];
        self.next_tile_map_index = 0;
        self.step_per_frame = Meters::new(0.11);
    }

    /// Allocate a new [`TileMap`] at chunk id (`x`, `y`)
    ///
    /// # Panics
    ///
    /// * Out of slots to hold tile maps
    pub fn alloc_tilemap_at(
        &mut self,
        memory: &mut Memory,
        pos: Vector2<u32>,
        z: u8,
    ) -> &mut TileMap<WIDTH, HEIGHT> {
        assert!(
            self.next_tile_map_index < PREALLOC_TILE_MAPS,
            "Out of tile map slot"
        );

        println!("Allocating tile map at ({:?}, {})", pos, z);

        let curr_tile_index = self.next_tile_map_index;

        // Tile map wasn't found, allocate a new one
        let tile_map: *mut TileMap<WIDTH, HEIGHT> = memory.alloc();

        // Set the tile map index for this newly allocated tilemap
        self.tile_map_indexes[curr_tile_index] = Some((pos, z));
        self.tile_maps[curr_tile_index] = tile_map;

        // Bump the tile map index
        self.next_tile_map_index += 1;

        unsafe { &mut *self.tile_maps[curr_tile_index] }
    }

    /// Get the [`TileMap`] at (`x`, `y`) in the World or allocate a new [`TileMap`] if
    /// the requested location is not yet allocated.
    ///
    /// # Panics
    ///
    /// * Sanity check of indexes is out of sync
    pub fn get_tilemap_at(
        &mut self,
        pos: Vector2<u32>,
        z: u8,
        memory: &mut Memory,
        rng: &mut Rng,
    ) -> &mut TileMap<WIDTH, HEIGHT> {
        // Look for the requested (x, y) in the allocated tile maps and return the
        // pointer if found
        for (index, coord) in self.tile_map_indexes[..self.next_tile_map_index]
            .iter()
            .enumerate()
        {
            assert!(coord.is_some(), "next_tile_map_index out of sync");

            if coord.unwrap() == (pos, z) {
                return unsafe { &mut *self.tile_maps[index] };
            }
        }

        // Allocate and initialize a new tile map
        self.init_tile_map(pos, z, memory, rng)
    }

    /// Randomly initialize a tile map
    #[allow(clippy::cast_possible_truncation)]
    fn init_tile_map(
        &mut self,
        chunk: Vector2<u32>,
        z: u8,
        memory: &mut Memory,
        rng: &mut Rng,
    ) -> &mut TileMap<WIDTH, HEIGHT> {
        // If a ladder is drawn, write the ladder in the adjacent floor location
        let mut other_floor = None;

        // No tilemap was found, allocate a new one
        let tile_map = self.alloc_tilemap_at(memory, chunk, z);

        let mut ladder_set = false;

        for y in 0..TILE_MAP_ROWS {
            for x in 0..TILE_MAP_COLUMNS {
                // Draw the floor/ceiling with doors
                if y == 0 || y == TILE_MAP_ROWS - 1 {
                    let mid_point = TILE_MAP_COLUMNS / 2;
                    if (mid_point - 1..=mid_point + 1).contains(&x) {
                        tile_map.set_tile_at(x as u16, y as u16, TileType::Empty);
                    } else {
                        tile_map.set_tile_at(x as u16, y as u16, TileType::Wall);
                    }
                }
                // Draw the walls with doors
                else if x == 0 || x == TILE_MAP_COLUMNS - 1 {
                    let mid_point = TILE_MAP_ROWS / 2;
                    if (mid_point - 1..=mid_point + 1).contains(&y) {
                        tile_map.set_tile_at(x as u16, y as u16, TileType::Empty);
                    } else {
                        tile_map.set_tile_at(x as u16, y as u16, TileType::Wall);
                    }
                }
                // Randomly set values in a room
                else if !ladder_set && rng.next() % 64 == 0 {
                    tile_map.set_tile_at(x as u16, y as u16, TileType::Ladder);

                    // Set that we need to set the ladder position in the adjacent floor
                    other_floor = Some((x as u16, y as u16));

                    // Only generate one ladder per floor
                    ladder_set = true;

                    continue;
                }
                // Randomly set values in a room
                else if rng.next() % 16 == 0 {
                    tile_map.set_tile_at(x as u16, y as u16, TileType::Wall);
                }
            }
        }

        // Get the same corresponding ladder on the other floor
        if let Some((x, y)) = other_floor {
            let other_z = (z + 1) % 2;
            let other_tilemap = self.get_tilemap_at(chunk, other_z, memory, rng);
            other_tilemap.set_tile_at(x, y, TileType::Ladder);
        }

        self.get_tilemap_at(chunk, z, memory, rng)
    }
}

/// Update and render the current game state
///
/// # Panics
///
/// * On 16 bit machines
#[no_mangle]
pub extern "C" fn game_update_and_render(game: &mut Game, state: &mut State) {
    // Initialize the game memory if not already initialized
    if !game.memory.initialized {
        let world: *mut World<TILE_MAP_COLUMNS, TILE_MAP_ROWS> = game.memory.alloc();

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

/// Actual game logic code that can return a [`Result`]
fn _game_update_and_render(game: &mut Game, state: &mut State) -> Result<()> {
    // Draw the background
    game.background.draw(game, Vector2::new(0., 0.));

    // Get the world structure which is always at the beginning of the persistent memory
    let world = unsafe {
        #[allow(clippy::cast_ptr_alignment)]
        &mut *(game
            .memory
            .data
            .as_mut_ptr()
            .cast::<World<TILE_MAP_COLUMNS, TILE_MAP_ROWS>>())
    };

    let old_player = state.player.position;
    let mut new_player = state.player.position;

    let mut movement_delta = Vector2::new(Meters::new(0.), Meters::new(0.));

    for (button_id, is_pressed) in game.buttons.as_ref().iter().enumerate() {
        // Not pressed, ignore the button
        if !is_pressed {
            continue;
        }

        // Get the pressed button
        let button = Button::from_usize(button_id);

        // Based on the button pressed, move the player
        match button {
            Button::Up => {
                movement_delta.y += world.step_per_frame;
                // new_player.tile_rel.y += ;
                state.player.direction = PlayerDirection::Back;
            }
            Button::Down => {
                // new_player.tile_rel.y -= world.step_per_frame;
                movement_delta.y -= world.step_per_frame;
                state.player.direction = PlayerDirection::Front;
            }
            Button::Right => {
                // new_player.tile_rel.x += world.step_per_frame;
                movement_delta.x += world.step_per_frame;
                state.player.direction = PlayerDirection::Right;
            }
            Button::Left => {
                // new_player.tile_rel.x -= world.step_per_frame;
                movement_delta.x -= world.step_per_frame;
                state.player.direction = PlayerDirection::Left;
            }
            Button::DecreaseSpeed => {
                world.step_per_frame -= Meters::new(0.06);
                world.step_per_frame = world.step_per_frame.clamp(0.05, 1.0).into();
            }
            Button::IncreaseSpeed => {
                world.step_per_frame += Meters::new(0.06);
                world.step_per_frame = world.step_per_frame.clamp(0.05, 1.0).into();
            }
            Button::Count => {}
        }
    }

    if movement_delta.x != Meters::new(0.0) && movement_delta.y != Meters::new(0.0) {
        // 1/sqrt(2)
        let one_div_sqrt_2 = Meters::new(std::f32::consts::FRAC_1_SQRT_2);
        movement_delta *= one_div_sqrt_2;
    }

    new_player += movement_delta;

    // Update the player coordinates based on the movement. If the player has stepped
    // beyond the bounds of the current tile, update the position to the new tile.
    new_player.canonicalize();
    // dbg_hex!(new_player);

    let ChunkVector { chunk_id, offset } = new_player.into_chunk();

    let tile_map = world.get_tilemap_at(chunk_id, new_player.z, game.memory, &mut state.rng);

    // Draw the tile map
    tile_map.draw(game)?;

    let display_lower_left_y = f32::from(GAME_WINDOW_HEIGHT);

    let mut tile_center = Vector2::new(
        f32::from(offset.x * TILE_WIDTH + TILE_HALF_WIDTH),
        display_lower_left_y - f32::from(offset.y * TILE_HEIGHT) - f32::from(TILE_HALF_HEIGHT),
    );

    let mut player_bottom_center = Vector2::new(
        tile_center.x + *new_player.tile_rel.x.into_pixels(),
        tile_center.y - *new_player.tile_rel.y.into_pixels(),
    );

    // Check that the potential moved to tile is valid (aka, zero)
    let mut valid = true;

    // Handle the tile type
    let next_tile = tile_map.get_tile_at(offset);

    // Block movement to walls
    if matches!(next_tile, &TileType::Wall) {
        valid = false;
    }

    // Only go up/down a ladder if the player didn't originally come from a ladder
    if matches!(next_tile, &TileType::Ladder)
        && (new_player.tile_map_x != old_player.tile_map_x
            || new_player.tile_map_y != old_player.tile_map_y)
    {
        new_player.z = (new_player.z + 1) % 2;
    }

    // If the move is valid, update the player
    if valid {
        state.player.position = new_player;
    } else {
        // Reset the coordinates for places where the player cannot move
        let ChunkVector {
            chunk_id: _,
            offset,
        } = old_player.into_chunk();

        // Use the original player's position since the new position is invalid
        tile_center.x = f32::from(offset.x * TILE_WIDTH + TILE_HALF_WIDTH);
        tile_center.y =
            display_lower_left_y - f32::from(offset.y * TILE_HEIGHT) - f32::from(TILE_HALF_HEIGHT);

        let mut old_player = old_player;

        // Move the player against the barrier if the player is not already at a
        // tile's limit, but is against a wall
        //
        // Example: Player is moving to the right at a speed that would put
        //          the player beyond a wall. Instead of leaving the player
        //          where they were, move them to the wall
        //
        // Old Position:        New Position:
        // +--------+##         +--------+##
        // |        |##         |        |##
        // | P      |## X       |       P|## NEWPOS
        // |        |##         |        |##
        // +--------+##         +--------+##
        match state.player.direction {
            PlayerDirection::Front => {
                old_player.tile_rel.y = Meters::new(-0.499);
            }
            PlayerDirection::Back => {
                old_player.tile_rel.y = Meters::new(0.499);
            }
            PlayerDirection::Right => {
                old_player.tile_rel.x = Meters::new(0.499);
            }
            PlayerDirection::Left => {
                old_player.tile_rel.x = Meters::new(-0.499);
            }
        }

        // Update the player tile_rel in case the player's relative position changed
        // (like moving them into a barrier)
        state.player.position.tile_rel = old_player.tile_rel;

        // Use the original player's position since the new position is invalid
        player_bottom_center = Vector2::new(
            tile_center.x + *old_player.tile_rel.x.into_pixels(),
            tile_center.y - *old_player.tile_rel.y.into_pixels(),
        );

        // Adjust the player to always hit the edge of a barrier
    }

    let tile_half = Vector2::new(f32::from(TILE_HALF_WIDTH), f32::from(TILE_HALF_HEIGHT));

    // DEBUG player position
    draw_rectangle(
        game,
        &Color::BLACK,
        tile_center - tile_half,
        f32::from(TILE_WIDTH),
        f32::from(TILE_HEIGHT),
    )?;

    // Get the player bitmap for the direction they are currently facing
    let player_asset = game.player_assets[state.player.direction as usize];

    let position = player_bottom_center - player_asset.merge_point;
    player_asset.head.draw(game, position);
    player_asset.torso.draw(game, position);
    player_asset.cape.draw(game, position);

    // DEBUG draw the player bottom center
    draw_rectangle(game, &Color::RED, player_bottom_center - 2.0, 4.0, 4.0)?;

    Ok(())
}

/// Debug function to print a set of gradient squares to the display
fn _test_gradient(game: &mut Game) {
    let height = u32::from(game.height);
    let width = u32::from(game.width);

    for col in 0..height {
        for row in 0..width {
            let index = col * width + row;
            let color = (col % 256) << 8 | (row % 256);
            game.framebuffer[usize::try_from(index).unwrap()] = color;
        }
    }
}

/// Fill a rectangle starting at the pixel (`pos_x`, `pos_y`) with a `width` and `height`
fn draw_rectangle(
    game: &mut Game,
    color: &Color,
    pos: Vector2<f32>,
    width: f32,
    height: f32,
) -> Result<()> {
    let upper_left_x = pos.x;
    let upper_left_y = pos.y;
    let lower_right_x = pos.x + width;
    let lower_right_y = pos.y + height;

    let upper_left_x = upper_left_x.trunc_as_u32().clamp(0, u32::from(game.width));
    let lower_right_x = lower_right_x.trunc_as_u32().clamp(0, u32::from(game.width));
    let upper_left_y = upper_left_y.trunc_as_u32().clamp(0, u32::from(game.height));
    let lower_right_y = lower_right_y
        .trunc_as_u32()
        .clamp(0, u32::from(game.height));

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

/// Draw the given [`BitmapAsset`] at (`pos_x`, `pos_y`) on the screen
fn _draw_asset(game: &mut Game, asset: &BitmapAsset, pos_x: f32, pos_y: f32) -> Result<()> {
    let game_height = f32::from(game.height);

    #[allow(clippy::cast_precision_loss)]
    let width = asset.width as f32;

    #[allow(clippy::cast_precision_loss)]
    let height = asset.height as f32;

    let bytes_per_color = 4;

    // Because the BMP pixels are in bottom row -> top row order, if the requested width
    // or height is less than the asset width or height, start the pixels array from the
    // correct location.
    //
    //                    +----------------------------+
    //                    | Draw  |    BMP Asset       |
    //                    |       |                    |
    // Requested start  -->*      |                    |
    //                    +-------+                    |
    //                    |                            |
    //                    |                            |
    //                    |                            |
    //                    |*                           |
    //                    +^---------------------------+
    //                     |
    //                    Normal starting pixel
    let mut starting_height = (asset.height - height.trunc_as_u32()) as usize;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    if height + pos_y > game_height {
        let offscreen = height + pos_y - game_height as f32;
        starting_height += offscreen as usize;
    }

    let mut starting_column = 0;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    if pos_x < 0.0 {
        starting_column = pos_x.round().abs().trunc() as usize;
    }

    let starting_index = starting_height * asset.width as usize * 4;
    let pixels_start = &asset.data[starting_index..];

    let upper_left_x = pos_x;
    let upper_left_y = pos_y;
    let lower_right_x = pos_x + width;
    let lower_right_y = pos_y + height;

    let upper_left_x = upper_left_x.trunc_as_u32().clamp(0, u32::from(game.width));
    let lower_right_x = lower_right_x.trunc_as_u32().clamp(0, u32::from(game.width));
    let upper_left_y = upper_left_y.trunc_as_u32().clamp(0, u32::from(game.height));
    let lower_right_y = lower_right_y
        .trunc_as_u32()
        .clamp(0, u32::from(game.height));

    // If the upper left corner is not the upper left corner, return;
    if upper_left_x > lower_right_x || upper_left_y > lower_right_y {
        return Err(Error::InvalidRectangle);
    }

    let blue_index = usize::from(asset.blue_index);
    let red_index = usize::from(asset.red_index);
    let green_index = usize::from(asset.green_index);
    let alpha_index = usize::from(asset.alpha_index);

    // Draw the asset at the requested location
    for (row_index, row) in (upper_left_y..lower_right_y).rev().enumerate() {
        // In the event the asset is larger than the requested draw size, update the
        // pixel pointer to the next row of pixels and ignore the non-drawn pixels
        let this_row = row_index * asset.width as usize * bytes_per_color;

        // In the event the image is off the left edge of the screen, the starting column
        // should be the remaining portion of the image not NOT from zero.
        let starting_column = starting_column as usize * bytes_per_color;

        let mut pixels = &pixels_start[this_row + starting_column..];

        for col in upper_left_x..lower_right_x {
            // Sanity check that we have enough pixel data to draw the sprite
            if pixels.len() < 4 {
                continue;
            }

            let index = row * u32::from(game.width) + col;
            let index = usize::try_from(index).unwrap();

            let r = f32::from(pixels[red_index]) / 255.0;
            let g = f32::from(pixels[green_index]) / 255.0;
            let b = f32::from(pixels[blue_index]) / 255.0;
            let a = f32::from(pixels[alpha_index]) / 255.0;

            // Create the curent color from the bitmap stream
            let mut new_color = Color::rgba(r, g, b, a);

            // Get the current background color for this pixel
            let current_color: Color = game.framebuffer[index].into();

            // Blend the new color into the background
            new_color.linear_alpha_blend(current_color);

            // Write the new color into the backgrouund
            game.framebuffer[index] = new_color.as_u32();

            pixels = &pixels[4..];
        }
    }

    // Success!
    Ok(())
}
