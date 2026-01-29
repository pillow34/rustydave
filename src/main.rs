use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::collections::HashSet;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags, PopKeyboardEnhancementFlags},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor,
    style::{Color, Print, SetForegroundColor, ResetColor},
};

use rustydave::{Tile, LEVEL_WIDTH, LEVEL_HEIGHT, generate_level, Config};


/// Represents the player character, Dave.
struct Player {
    /// Horizontal position.
    x: f32,
    /// Vertical position.
    y: f32,
    /// Horizontal velocity.
    vx: f32,
    /// Vertical velocity.
    vy: f32,
    /// Whether Dave is currently standing on solid ground.
    on_ground: bool,
    /// Whether Dave has collected the trophy for the current level.
    has_trophy: bool,
    /// Timer for coyote time (jumping after leaving a platform).
    coyote_timer: f32,
    /// Timer for jump buffering (inputting jump before landing).
    jump_buffer_timer: f32,
}

/// The main game state and engine.
struct Game {
    /// The 2D grid of tiles for the current level.
    level: [[Tile; LEVEL_WIDTH]; LEVEL_HEIGHT],
    /// The player character.
    player: Player,
    /// Whether the game is currently running.
    running: bool,
    /// Whether the player has won the entire game.
    won: bool,
    /// Whether the player is currently dead.
    is_dead: bool,
    /// Whether the player has completed the current level.
    level_complete: bool,
    /// The current level number.
    current_level: u32,
    /// Status message displayed at the bottom of the screen.
    message: String,
    /// Timer for death animation/delay (seconds).
    death_timer: f32,
    /// Timer for level start delay (seconds).
    start_timer: f32,
    /// Current configuration loaded from config.toml or defaults.
    config: Config,
    /// Current number of lives remaining.
    lives: i32,
    /// Current player score.
    score: i32,
    /// Whether to use ASCII graphics (2-char wide) or older graphics (1-char wide).
    use_ascii: bool,
}

impl Game {
    /// Creates a new game instance, starting at the specified level.
    fn new(start_level: u32, config: Config, use_ascii: bool) -> Self {
        let mut game = Game {
            level: [[Tile::Empty; LEVEL_WIDTH]; LEVEL_HEIGHT],
            player: Player {
                x: 2.0,
                y: 18.0,
                vx: 0.0,
                vy: 0.0,
                on_ground: false,
                has_trophy: false,
                coyote_timer: 0.0,
                jump_buffer_timer: 0.0,
            },
            running: true,
            won: false,
            is_dead: false,
            level_complete: false,
            current_level: start_level,
            message: format!("Level {}: Find the Trophy (*) and then reach the Exit (E)!", start_level),
            death_timer: 0.0,
            start_timer: 0.5,
            config,
            lives: 3,
            score: 0,
            use_ascii,
        };
        game.init_level();
        game
    }

    /// Initializes or re-initializes the level based on `current_level`.
    /// Generates a new procedural layout and positions the player.
    fn init_level(&mut self) {
        let (level, (px, py)) = generate_level(self.current_level);
        self.level = level;
        self.player.x = px;
        self.player.y = py;
    }

    /// Resets the game state for the current level or restarts the game if all lives are lost.
    fn reset(&mut self) {
        if self.lives <= 0 {
            self.lives = 3;
            self.score = 0;
            self.current_level = 1;
        }
        self.player.vx = 0.0;
        self.player.vy = 0.0;
        self.player.on_ground = false;
        self.player.has_trophy = false;
        self.player.coyote_timer = 0.0;
        self.player.jump_buffer_timer = 0.0;
        self.is_dead = false;
        self.won = false;
        self.level_complete = false;
        self.death_timer = 0.0;
        self.start_timer = 0.5;
        self.init_level();
        self.message = format!("Level {}: Find the Trophy (*) and then reach the Exit (E)!", self.current_level);
    }

    /// Updates the game state based on elapsed time (`dt`) and keyboard input.
    /// Handles physics, movement, collisions, and interactions.
    fn update(&mut self, dt: f32, keys: &HashSet<KeyCode>) {
        let restart_pressed = keys.iter().any(|&k| self.config.key_matches(k, &self.config.keys.restart));

        if self.is_dead {
            self.death_timer -= dt;
            if self.death_timer <= 0.0 && restart_pressed {
                self.reset();
            }
            return;
        }

        if self.level_complete {
            if restart_pressed {
                if self.current_level < self.config.max_level {
                    self.current_level += 1;
                    self.reset();
                } else {
                    self.won = true;
                    self.running = false;
                }
            }
            return;
        }

        if self.start_timer > 0.0 {
            self.start_timer -= dt;
            return;
        }

        // Update timers
        self.player.coyote_timer -= dt;
        self.player.jump_buffer_timer -= dt;

        // Key states from config
        let left_pressed = keys.iter().any(|&k| self.config.key_matches(k, &self.config.keys.left));
        let right_pressed = keys.iter().any(|&k| self.config.key_matches(k, &self.config.keys.right));
        let jump_pressed = keys.iter().any(|&k| self.config.key_matches(k, &self.config.keys.jump));

        // Horizontal movement
        let mut target_vx = 0.0;
        let mut moving = false;
        if left_pressed {
            target_vx -= self.config.physics.target_vx;
            moving = true;
        }
        if right_pressed {
            target_vx += self.config.physics.target_vx;
            moving = true;
        }
        
        // Acceleration/Friction
        if moving {
            let accel = if self.player.on_ground { self.config.physics.accel_ground } else { self.config.physics.accel_air };
            if self.player.vx < target_vx {
                self.player.vx = (self.player.vx + accel * dt).min(target_vx);
            } else if self.player.vx > target_vx {
                self.player.vx = (self.player.vx - accel * dt).max(target_vx);
            }
        } else {
            let friction = if self.player.on_ground { self.config.physics.friction } else { self.config.physics.friction * 0.5 };
            if self.player.vx > 0.0 {
                self.player.vx = (self.player.vx - friction * dt).max(0.0);
            } else if self.player.vx < 0.0 {
                self.player.vx = (self.player.vx + friction * dt).min(0.0);
            }
        }

        // Jump input and buffering
        if jump_pressed {
            self.player.jump_buffer_timer = self.config.physics.jump_buffer_time;
        }

        // Jump logic (Coyote time and Buffer)
        if self.player.jump_buffer_timer > 0.0 && self.player.coyote_timer > 0.0 {
            self.player.vy = self.config.physics.jump_vy;
            self.player.on_ground = false;
            self.player.coyote_timer = 0.0;
            self.player.jump_buffer_timer = 0.0;
        }

        // Gravity with variable jump height
        let gravity = if self.player.vy < 0.0 && !jump_pressed {
            self.config.physics.gravity * self.config.physics.jump_release_gravity_mult
        } else {
            self.config.physics.gravity
        };
        self.player.vy += gravity * dt;

        // Vertical movement and collision
        let next_y = self.player.y + self.player.vy * dt;
        if self.is_colliding(self.player.x, next_y) {
            if self.player.vy > 0.0 {
                self.player.on_ground = true;
                self.player.coyote_timer = self.config.physics.coyote_time;
                self.player.y = next_y.floor() as f32 - 0.01;
            } else {
                self.player.y = next_y.floor() as f32 + 1.0;
            }
            self.player.vy = 0.0;
        } else {
            self.player.y = next_y;
            // Robust on-ground check: are we standing on a wall?
            if self.is_colliding(self.player.x, self.player.y + 0.1) {
                self.player.on_ground = true;
                self.player.coyote_timer = self.config.physics.coyote_time;
            } else {
                self.player.on_ground = false;
            }
        }

        // Horizontal movement and collision
        let next_x = self.player.x + self.player.vx * dt;
        if self.is_colliding(next_x, self.player.y) {
            self.player.vx = 0.0;
            if next_x > self.player.x {
                self.player.x = next_x.floor() as f32 - 0.01;
            } else {
                self.player.x = next_x.floor() as f32 + 1.0;
            }
        } else {
            self.player.x = next_x;
        }

        // Interaction
        let tx = self.player.x.floor() as usize;
        let ty = self.player.y.floor() as usize;
        
        if tx < LEVEL_WIDTH && ty < LEVEL_HEIGHT {
            match self.level[ty][tx] {
                Tile::Trophy => {
                    self.player.has_trophy = true;
                    self.level[ty][tx] = Tile::Empty;
                    self.score += 500;
                    self.message = "Got the Trophy! +500 points. Now reach the Exit (E)!".to_string();
                }
                Tile::Diamond => {
                    self.score += 100;
                    self.level[ty][tx] = Tile::Empty;
                    self.message = "Collected a Diamond! +100 points".to_string();
                }
                Tile::Exit => {
                    if self.player.has_trophy {
                        self.level_complete = true;
                        self.score += 1000;
                        if self.current_level < self.config.max_level {
                            self.message = "Level Complete! +1000 points. Press ENTER for next level.".to_string();
                        } else {
                            self.message = "All Levels Complete! +1000 points. Press ENTER to win!".to_string();
                        }
                    } else {
                        self.message = "You need the Trophy (*) first!".to_string();
                    }
                }
                Tile::Hazard => {
                    self.is_dead = true;
                    self.death_timer = 0.5;
                    self.lives -= 1;
                    if self.lives > 0 {
                        self.message = format!("Ouch! You hit a hazard! Lives left: {}. Press ENTER to restart.", self.lives);
                    } else {
                        self.message = "GAME OVER! You ran out of lives. Press ENTER to restart game.".to_string();
                    }
                }
                _ => {}
            }
        }
    }

    /// Checks if a given coordinate (x, y) collides with a wall.
    fn is_colliding(&self, x: f32, y: f32) -> bool {
        let tx = x.floor() as i32;
        let ty = y.floor() as i32;
        if tx < 0 || tx >= LEVEL_WIDTH as i32 || ty < 0 || ty >= LEVEL_HEIGHT as i32 {
            return true;
        }
        self.level[ty as usize][tx as usize] == Tile::Wall
    }

    /// Renders the current game state to the terminal.
    fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        
        queue!(stdout, SetForegroundColor(Color::Magenta), Print(format!("--- RUSTY DAVE - Level {} ---\r\n", self.current_level)), ResetColor)?;
        
        let mut buffer = String::with_capacity(LEVEL_WIDTH * LEVEL_HEIGHT * 10);
        
        for y in 0..LEVEL_HEIGHT {
            for x in 0..LEVEL_WIDTH {
                if x == self.player.x.floor() as usize && y == self.player.y.floor() as usize {
                    if self.use_ascii {
                        if self.is_dead {
                            buffer.push_str("\x1b[31mX \x1b[0m"); // Red X for dead Dave
                        } else {
                            buffer.push_str("\x1b[36m☺ \x1b[0m"); // Cyan Dave (Smile)
                        }
                    } else {
                        if self.is_dead {
                            buffer.push_str("\x1b[31mX\x1b[0m"); // Red X for dead Dave
                        } else {
                            buffer.push_str("\x1b[36mD\x1b[0m"); // Cyan Dave (Letter D)
                        }
                    }
                } else {
                    match self.level[y][x] {
                        Tile::Empty => buffer.push_str(if self.use_ascii { "  " } else { " " }),
                        Tile::Wall => buffer.push_str(if self.use_ascii { "\x1b[34m██\x1b[0m" } else { "\x1b[34m#\x1b[0m" }),
                        Tile::Trophy => buffer.push_str(if self.use_ascii { "\x1b[33m★ \x1b[0m" } else { "\x1b[33m*\x1b[0m" }),
                        Tile::Exit => buffer.push_str(if self.use_ascii { "\x1b[32m][\x1b[0m" } else { "\x1b[32mE\x1b[0m" }),
                        Tile::Hazard => buffer.push_str(if self.use_ascii { "\x1b[31m▲▲\x1b[0m" } else { "\x1b[31m^\x1b[0m" }),
                        Tile::Diamond => buffer.push_str(if self.use_ascii { "\x1b[35m♦ \x1b[0m" } else { "\x1b[35m+\x1b[0m" }),
                    }
                }
            }
            buffer.push_str("\r\n");
        }
        
        queue!(
            stdout,
            Print(buffer),
            cursor::MoveTo(0, (LEVEL_HEIGHT + 1) as u16),
            Clear(ClearType::CurrentLine),
        )?;

        if self.is_dead {
            queue!(stdout, SetForegroundColor(Color::Red))?;
        } else if self.level_complete {
            queue!(stdout, SetForegroundColor(Color::Green))?;
        }

        queue!(
            stdout,
            Print(&self.message),
            ResetColor,
            cursor::MoveTo(0, (LEVEL_HEIGHT + 2) as u16),
            Clear(ClearType::CurrentLine),
            Print(format!("Score: {:06} | Lives: {} | Trophy: {} | Pos: ({:.1}, {:.1})", 
                self.score,
                self.lives,
                if self.player.has_trophy { "YES" } else { "NO" },
                self.player.x, self.player.y))
        )?;
        
        stdout.flush()?;
        Ok(())
    }
}

fn parse_args(args: &[String], max_level: u32) -> (u32, bool) {
    let mut start_level = 1;
    let mut use_ascii = false;
    for arg in args.iter().skip(1) {
        if arg == "--ascii" {
            use_ascii = true;
        } else if let Ok(level) = arg.parse::<u32>() {
            start_level = level.clamp(1, max_level);
        }
    }
    (start_level, use_ascii)
}

/// Entry point for the Rusty Dave game.
/// Sets up the terminal, runs the game loop, and cleans up on exit.
fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout, 
        EnterAlternateScreen, 
        cursor::Hide, 
        Clear(ClearType::All),
    )?;

    // Try to enable keyboard enhancement for better input handling (e.g. in Windows Terminal or modern Unix terminals)
    // We ignore the error if it's not supported (e.g. in legacy Windows Console)
    let _ = execute!(stdout, PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES));

    let config = Config::load();
    let args: Vec<String> = std::env::args().collect();
    let (start_level, use_ascii) = parse_args(&args, config.max_level);

    let mut game = Game::new(start_level, config, use_ascii);
    let mut last_tick = Instant::now();
    let mut keys = HashSet::new();

    while game.running {
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32().min(0.05);
        last_tick = now;

        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.kind {
                    KeyEventKind::Press | KeyEventKind::Repeat => {
                        keys.insert(key_event.code);
                    }
                    KeyEventKind::Release => {
                        keys.remove(&key_event.code);
                    }
                }
                
                if game.config.key_matches(key_event.code, &game.config.keys.quit) {
                    game.running = false;
                }
            }
        }

        game.update(dt, &keys);
        game.draw(&mut stdout)?;
        
        let elapsed = now.elapsed();
        if elapsed < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - elapsed);
        }
    }

    let _ = execute!(stdout, PopKeyboardEnhancementFlags);
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    if game.won {
        println!("CONGRATULATIONS! You escaped with the trophy!");
    } else {
        println!("GAME OVER: {}", game.message);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_init_level() {
        let game = Game::new(3, Config::default(), false);
        assert_eq!(game.current_level, 3);
        assert!(game.message.contains("Level 3"));
    }

    #[test]
    fn test_game_init_level_clamping() {
        // We don't clamp in Game::new, we clamp in main (via parse_args).
        // But let's check Game::new handles whatever it's given.
        let game = Game::new(10, Config::default(), false);
        assert_eq!(game.current_level, 10);
    }

    #[test]
    fn test_parse_args() {
        let max = 10;
        assert_eq!(parse_args(&vec!["exe".to_string()], max), (1, false));
        assert_eq!(parse_args(&vec!["exe".to_string(), "3".to_string()], max), (3, false));
        assert_eq!(parse_args(&vec!["exe".to_string(), "0".to_string()], max), (1, false));
        assert_eq!(parse_args(&vec!["exe".to_string(), "20".to_string()], max), (max, false));
        assert_eq!(parse_args(&vec!["exe".to_string(), "--ascii".to_string()], max), (1, true));
        assert_eq!(parse_args(&vec!["exe".to_string(), "5".to_string(), "--ascii".to_string()], max), (5, true));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.physics.gravity, 80.0);
        assert!(config.keys.left.contains(&"Left".to_string()));
    }

    #[test]
    fn test_key_matches() {
        let config = Config::default();
        assert!(config.key_matches(KeyCode::Left, &config.keys.left));
        assert!(config.key_matches(KeyCode::Char('a'), &config.keys.left));
        assert!(config.key_matches(KeyCode::Char('A'), &config.keys.left));
        assert!(!config.key_matches(KeyCode::Right, &config.keys.left));
    }

    #[test]
    fn test_diamond_collection() {
        let mut game = Game::new(1, Config::default(), false);
        game.start_timer = 0.0;
        game.score = 0;
        game.level[10][10] = Tile::Diamond;
        game.player.x = 10.0;
        game.player.y = 10.0;
        
        // Mock a small update to trigger interaction
        let keys = HashSet::new();
        game.update(0.01, &keys);
        
        assert_eq!(game.score, 100);
        assert_eq!(game.level[10][10], Tile::Empty);
    }

    #[test]
    fn test_lives_decrement() {
        let mut game = Game::new(1, Config::default(), false);
        game.start_timer = 0.0;
        game.lives = 3;
        game.level[10][10] = Tile::Hazard;
        game.player.x = 10.0;
        game.player.y = 10.0;
        
        let keys = HashSet::new();
        game.update(0.01, &keys);
        
        assert_eq!(game.lives, 2);
        assert!(game.is_dead);
    }
}

