//! Shared library for Rusty Dave game logic.
//! Contains level generation, tile definitions, and random number generation.

use std::fs;
use std::io;
use serde::{Deserialize, Serialize};
use crossterm::event::KeyCode;

/// Width of the game level in tiles.
pub const LEVEL_WIDTH: usize = 60;
/// Height of the game level in tiles.
pub const LEVEL_HEIGHT: usize = 20;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhysicsConfig {
    pub target_vx: f32,
    pub accel_ground: f32,
    pub accel_air: f32,
    pub jump_vy: f32,
    pub gravity: f32,
    pub coyote_time: f32,
    pub jump_buffer_time: f32,
    pub jump_release_gravity_mult: f32,
    pub friction: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeysConfig {
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub jump: Vec<String>,
    pub quit: Vec<String>,
    pub restart: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_max_level")]
    pub max_level: u32,
    pub physics: PhysicsConfig,
    pub keys: KeysConfig,
}

fn default_max_level() -> u32 { 10 }

impl Default for Config {
    fn default() -> Self {
        Config {
            max_level: 10,
            physics: PhysicsConfig {
                target_vx: 30.0,
                accel_ground: 200.0,
                accel_air: 80.0,
                jump_vy: -28.0,
                gravity: 80.0,
                coyote_time: 0.1,
                jump_buffer_time: 0.1,
                jump_release_gravity_mult: 3.0,
                friction: 400.0,
            },
            keys: KeysConfig {
                left: vec!["Left".to_string(), "a".to_string(), "A".to_string()],
                right: vec!["Right".to_string(), "d".to_string(), "D".to_string()],
                jump: vec!["Up".to_string(), "w".to_string(), "W".to_string(), "Space".to_string()],
                quit: vec!["Esc".to_string(), "q".to_string(), "Q".to_string()],
                restart: vec!["Enter".to_string()],
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        fs::read_to_string("config.toml")
            .and_then(|content| toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
            .unwrap_or_else(|_| Config::default())
    }

    pub fn key_matches(&self, code: KeyCode, key_list: &[String]) -> bool {
        for k in key_list {
            let matches = match k.as_str() {
                "Left" => code == KeyCode::Left,
                "Right" => code == KeyCode::Right,
                "Up" => code == KeyCode::Up,
                "Down" => code == KeyCode::Down,
                "Enter" => code == KeyCode::Enter,
                "Esc" => code == KeyCode::Esc,
                "Space" => code == KeyCode::Char(' '),
                s if s.len() == 1 => {
                    let c = s.chars().next().unwrap();
                    code == KeyCode::Char(c) || 
                    code == KeyCode::Char(c.to_lowercase().next().unwrap()) || 
                    code == KeyCode::Char(c.to_uppercase().next().unwrap())
                },
                _ => false,
            };
            if matches { return true; }
        }
        false
    }
}

/// Represents the different types of tiles in the game.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    /// Empty space that Dave can move through.
    Empty,
    /// Solid wall that Dave cannot move through.
    Wall,
    /// The trophy that Dave must collect to exit.
    Trophy,
    /// The exit door that Dave must reach after collecting the trophy.
    Exit,
    /// Deadly hazards (like fire or spikes) that kill Dave on contact.
    Hazard,
    /// Collectible diamonds for points.
    Diamond,
}

/// A simple, deterministic random number generator for level generation.
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    /// Creates a new `SimpleRng` with the given seed.
    pub fn new(seed: u32) -> Self {
        let mut state = seed as u64 + 0x9E3779B97F4A7C15;
        // Basic mixing
        state = (state ^ (state >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        state = (state ^ (state >> 27)).wrapping_mul(0x94D049BB133111EB);
        state = state ^ (state >> 31);
        SimpleRng { state }
    }

    /// Generates the next random 32-bit unsigned integer.
    pub fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    /// Generates a random 32-bit unsigned integer in the range [min, max).
    pub fn range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max { return min; }
        min + (self.next() % (max - min))
    }
}

/// Generates a level grid and starting player position based on the level number.
///
/// # Arguments
/// * `level_num` - The level number, used as a seed for procedural generation.
///
/// # Returns
/// A tuple containing:
/// * The 2D grid of `Tile` elements.
/// * The starting (x, y) coordinates for the player.
pub fn generate_level(level_num: u32) -> ([[Tile; LEVEL_WIDTH]; LEVEL_HEIGHT], (f32, f32)) {
    let mut level = [[Tile::Empty; LEVEL_WIDTH]; LEVEL_HEIGHT];
    
    // Boundaries
    for x in 0..LEVEL_WIDTH {
        level[0][x] = Tile::Wall;
        level[LEVEL_HEIGHT - 1][x] = Tile::Wall;
    }
    for y in 0..LEVEL_HEIGHT {
        level[y][0] = Tile::Wall;
        level[y][LEVEL_WIDTH - 1] = Tile::Wall;
    }

    let mut rng = SimpleRng::new(level_num);
    
    let player_x = 2.0;
    let player_y = 17.99; // Start on top of the base platform

    // Base platform for player
    for x in 1..10 { level[18][x] = Tile::Wall; }

    let heights = [16, 12, 8, 4];
    let mut w1 = 0;
    let mut w1_start = 15;
    let mut w2 = 0;
    let mut w3 = 0;
    let mut w4 = 0;

    if level_num % 2 != 0 {
        // Archetype 1: Zig-zag (Classic)
        // H16: Left to Rightish
        w1_start = 15;
        w1 = rng.range(35, 55) as usize;
        for x in w1_start..w1 { level[16][x] = Tile::Wall; }

        // H12: Right to Leftish
        w2 = rng.range(25, 45) as usize;
        for x in w2..59 { level[12][x] = Tile::Wall; }

        // H8: Left to Rightish
        w3 = rng.range(35, 55) as usize;
        for x in 1..w3 { level[8][x] = Tile::Wall; }

        // H4: Right to Leftish
        w4 = rng.range(25, 45) as usize;
        for x in w4..59 { level[4][x] = Tile::Wall; }
    } else {
        // Archetype 2: Floating Islands
        for &h in &heights {
            let num_islands = rng.range(2, 4);
            for i in 0..num_islands {
                let start = rng.range(5 + i * 15, 15 + i * 15) as usize;
                let len = rng.range(5, 12) as usize;
                for x in start..(start + len).min(59) {
                    level[h][x] = Tile::Wall;
                }
                // Record some values for Trophy/Exit logic below if needed
                if h == 16 && i == 0 { w1 = start + len; w1_start = start; }
                if h == 12 && i == 0 { w2 = start; }
                if h == 8 && i == 0 { w3 = start + len; }
                if h == 4 && i == 0 { w4 = start; }
            }
        }
        // Ensure some reasonable values for subsequent logic
        if w1 == 0 { w1 = 40; }
        if w2 == 0 { w2 = 20; }
        if w3 == 0 { w3 = 40; }
        if w4 == 0 { w4 = 20; }
    }

    // Trophy: on the top platform
    let mut trophy_candidates = Vec::new();
    for x in 1..LEVEL_WIDTH - 1 {
        if level[4][x] == Tile::Wall {
            trophy_candidates.push(x);
        }
    }
    let trophy_x = if !trophy_candidates.is_empty() {
        let idx = rng.range(0, trophy_candidates.len() as u32) as usize;
        trophy_candidates[idx]
    } else {
        // Fallback for safety
        rng.range(w4 as u32 + 2, 58) as usize
    };
    level[3][trophy_x] = Tile::Trophy;

    // Exit: far right of H16 or Ground
    let (exit_x, exit_y) = if rng.range(0, 2) == 0 {
        level[18][55] = Tile::Exit;
        (55, 18)
    } else {
        let ex = (w1 - 2).max(w1_start).min(58);
        level[15][ex] = Tile::Exit;
        (ex, 15)
    };

    // Diamonds placement
    for _ in 0..8 {
        let h = heights[rng.range(0, heights.len() as u32) as usize];
        let dx = rng.range(2, 58) as usize;
        if level[h][dx] == Tile::Wall && level[h-1][dx] == Tile::Empty {
            level[h-1][dx] = Tile::Diamond;
        }
    }

    // Hazards on floor
    let floor_chance = if level_num == 1 { 10 } else { 30 };
    let mut last_floor_hazard_end: i32 = -10;
    for x in 15..50usize {
        // Keep some columns safe on the floor to allow traversal/recovery
        let is_valid = x % 10 != 0;
        if !is_valid || (x as i32) - last_floor_hazard_end < 4 {
            continue;
        }

        if rng.range(0, 100) < floor_chance {
            let size = if rng.range(0, 2) == 0 { 1 } else { 2 };
            let mut actual_size: usize = 0;
            for k in 0..size {
                let cur_x = x + k;
                if cur_x < 50 && cur_x % 10 != 0 {
                    actual_size += 1;
                } else {
                    break;
                }
            }

            if actual_size > 0 {
                // Rule: Not more than 4 in 15 block range
                let mut violation = false;
                for window_start in (x + actual_size).saturating_sub(15)..=x {
                    let mut count = 0;
                    for i in 0..15 {
                        let check_x = window_start + i;
                        if check_x < LEVEL_WIDTH && ((check_x >= x && check_x < x + actual_size) || level[LEVEL_HEIGHT - 1][check_x] == Tile::Hazard) {
                            count += 1;
                        }
                    }
                    if count > 4 {
                        violation = true;
                        break;
                    }
                }

                if !violation {
                    for k in 0..actual_size {
                        level[LEVEL_HEIGHT - 1][x + k] = Tile::Hazard;
                    }
                    last_floor_hazard_end = (x + actual_size - 1) as i32;
                }
            }
        }
    }

    // Hazards on platforms (placed on top of the walls)
    let heights = [16, 12, 8, 4];
    for &h in &heights {
        let mut last_hazard_end: i32 = -10;
        for x in 5..55usize {
            let is_critical = |cx: usize| {
                (h == 16 && (cx >= w2.max(w1_start).saturating_sub(2) && cx <= w2.max(w1_start) + 2)) || // H16 -> H12 jump point
                (h == 12 && (cx >= w2.max(w1_start).saturating_sub(2) && cx <= w2.max(w1_start) + 2)) || // H16 -> H12 landing
                (h == 12 && (cx >= w3.saturating_sub(2) && cx <= w3 + 2)) || // H12 -> H8 jump point
                (h == 8 && (cx >= w3.saturating_sub(2) && cx <= w3 + 2)) ||  // H12 -> H8 landing
                (h == 8 && (cx >= w4.saturating_sub(2) && cx <= w4 + 2)) ||  // H8 -> H4 jump point
                (h == 4 && (cx >= w4.saturating_sub(2) && cx <= w4 + 2)) ||  // H8 -> H4 landing
                (h == 4 && (cx >= trophy_x.saturating_sub(1) && cx <= trophy_x + 1)) ||
                (h == exit_y + 1 && (cx >= exit_x.saturating_sub(1) && cx <= exit_x + 1))
            };

            let check_valid = |cx: usize| {
                cx >= 5 && cx < 55 && 
                !is_critical(cx) && 
                level[h][cx] == Tile::Wall && 
                level[h][cx-1] == Tile::Wall && 
                level[h][cx+1] == Tile::Wall
            };

            if !check_valid(x) || (x as i32) - last_hazard_end < 4 {
                continue;
            }

            if rng.range(0, 100) < 15 {
                let size = if rng.range(0, 2) == 0 { 1 } else { 2 };
                let mut actual_size: usize = 0;
                for k in 0..size {
                    if check_valid(x + k) {
                        actual_size += 1;
                    } else {
                        break;
                    }
                }

                if actual_size > 0 {
                    let mut violation = false;
                    for window_start in (x + actual_size).saturating_sub(15)..=x {
                        let mut count = 0;
                        for i in 0..15 {
                            let check_x = window_start + i;
                            if check_x < LEVEL_WIDTH && ((check_x >= x && check_x < x + actual_size) || level[h-1][check_x] == Tile::Hazard) {
                                count += 1;
                            }
                        }
                        if count > 4 {
                            violation = true;
                            break;
                        }
                    }

                    if !violation {
                        for k in 0..actual_size {
                            level[h-1][x + k] = Tile::Hazard;
                        }
                        last_hazard_end = (x + actual_size - 1) as i32;
                    }
                }
            }
        }
    }
    
    (level, (player_x, player_y))
}
