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

use rustydave::{Tile, LEVEL_WIDTH, LEVEL_HEIGHT, generate_level};

const TARGET_VX: f32 = 30.0;
const ACCEL_GROUND: f32 = 200.0;
const ACCEL_AIR: f32 = 80.0;
const JUMP_VY: f32 = -28.0;
const GRAVITY: f32 = 80.0;
const COYOTE_TIME: f32 = 0.1;
const JUMP_BUFFER_TIME: f32 = 0.1;
const JUMP_RELEASE_GRAVITY_MULT: f32 = 3.0;
const FRICTION: f32 = 400.0;

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
    /// The current level number (1-5).
    current_level: u32,
    /// Status message displayed at the bottom of the screen.
    message: String,
    /// Timer for death animation/delay.
    death_timer: f32,
    /// Timer for level start delay.
    start_timer: f32,
}

impl Game {
    /// Creates a new game instance, starting at the specified level.
    fn new(start_level: u32) -> Self {
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
        };
        game.init_level();
        game
    }

    /// Initializes or re-initializes the level based on `current_level`.
    fn init_level(&mut self) {
        let (level, (px, py)) = generate_level(self.current_level);
        self.level = level;
        self.player.x = px;
        self.player.y = py;
    }

    /// Resets the player state and reloads the current level.
    fn reset(&mut self) {
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

    /// Updates the game state based on elapsed time and keyboard input.
    fn update(&mut self, dt: f32, keys: &HashSet<KeyCode>) {
        if self.is_dead {
            self.death_timer -= dt;
            if self.death_timer <= 0.0 && keys.contains(&KeyCode::Enter) {
                self.reset();
            }
            return;
        }

        if self.level_complete {
            if keys.contains(&KeyCode::Enter) {
                if self.current_level < 5 {
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

        // Horizontal movement
        let mut target_vx = 0.0;
        let mut moving = false;
        if keys.contains(&KeyCode::Left) || keys.contains(&KeyCode::Char('a')) || keys.contains(&KeyCode::Char('A')) {
            target_vx -= TARGET_VX;
            moving = true;
        }
        if keys.contains(&KeyCode::Right) || keys.contains(&KeyCode::Char('d')) || keys.contains(&KeyCode::Char('D')) {
            target_vx += TARGET_VX;
            moving = true;
        }
        
        // Acceleration/Friction
        if moving {
            let accel = if self.player.on_ground { ACCEL_GROUND } else { ACCEL_AIR };
            if self.player.vx < target_vx {
                self.player.vx = (self.player.vx + accel * dt).min(target_vx);
            } else if self.player.vx > target_vx {
                self.player.vx = (self.player.vx - accel * dt).max(target_vx);
            }
        } else {
            let friction = if self.player.on_ground { FRICTION } else { FRICTION * 0.5 };
            if self.player.vx > 0.0 {
                self.player.vx = (self.player.vx - friction * dt).max(0.0);
            } else if self.player.vx < 0.0 {
                self.player.vx = (self.player.vx + friction * dt).min(0.0);
            }
        }

        // Jump input and buffering
        let jump_pressed = keys.contains(&KeyCode::Up) || keys.contains(&KeyCode::Char('w')) || keys.contains(&KeyCode::Char('W')) || keys.contains(&KeyCode::Char(' '));
        if jump_pressed {
            self.player.jump_buffer_timer = JUMP_BUFFER_TIME;
        }

        // Jump logic (Coyote time and Buffer)
        if self.player.jump_buffer_timer > 0.0 && self.player.coyote_timer > 0.0 {
            self.player.vy = JUMP_VY;
            self.player.on_ground = false;
            self.player.coyote_timer = 0.0;
            self.player.jump_buffer_timer = 0.0;
        }

        // Gravity with variable jump height
        let gravity = if self.player.vy < 0.0 && !jump_pressed {
            GRAVITY * JUMP_RELEASE_GRAVITY_MULT
        } else {
            GRAVITY
        };
        self.player.vy += gravity * dt;

        // Vertical movement and collision
        let next_y = self.player.y + self.player.vy * dt;
        if self.is_colliding(self.player.x, next_y) {
            if self.player.vy > 0.0 {
                self.player.on_ground = true;
                self.player.coyote_timer = COYOTE_TIME;
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
                self.player.coyote_timer = COYOTE_TIME;
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
                    self.message = "Got the Trophy! Now reach the Exit (E)!".to_string();
                }
                Tile::Exit => {
                    if self.player.has_trophy {
                        self.level_complete = true;
                        if self.current_level < 5 {
                            self.message = "Level Complete! Press ENTER for next level.".to_string();
                        } else {
                            self.message = "All Levels Complete! Press ENTER to win!".to_string();
                        }
                    } else {
                        self.message = "You need the Trophy (*) first!".to_string();
                    }
                }
                Tile::Hazard => {
                    self.is_dead = true;
                    self.death_timer = 0.5;
                    self.message = "Ouch! You hit a hazard! Press ENTER to restart.".to_string();
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
                    if self.is_dead {
                        buffer.push_str("\x1b[31mX\x1b[0m"); // Red X for dead Dave
                    } else {
                        buffer.push_str("\x1b[36mD\x1b[0m"); // Cyan Dave
                    }
                } else {
                    match self.level[y][x] {
                        Tile::Empty => buffer.push(' '),
                        Tile::Wall => buffer.push_str("\x1b[34m#\x1b[0m"), // Blue Wall
                        Tile::Trophy => buffer.push_str("\x1b[33m*\x1b[0m"), // Yellow Trophy
                        Tile::Exit => buffer.push_str("\x1b[32mE\x1b[0m"), // Green Exit
                        Tile::Hazard => buffer.push_str("\x1b[31m^\x1b[0m"), // Red Hazard
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
            Print(format!("Trophy: {} | Pos: ({:.1}, {:.1})", 
                if self.player.has_trophy { "YES" } else { "NO" },
                self.player.x, self.player.y))
        )?;
        
        stdout.flush()?;
        Ok(())
    }
}

fn parse_start_level(args: &[String]) -> u32 {
    if args.len() > 1 {
        args[1].parse::<u32>().unwrap_or(1).clamp(1, 5)
    } else {
        1
    }
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

    let args: Vec<String> = std::env::args().collect();
    let start_level = parse_start_level(&args);

    let mut game = Game::new(start_level);
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
                
                if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Char('Q') {
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
        let game = Game::new(3);
        assert_eq!(game.current_level, 3);
        assert!(game.message.contains("Level 3"));
    }

    #[test]
    fn test_game_init_level_clamping() {
        // We don't clamp in Game::new, we clamp in main (via parse_start_level).
        // But let's check Game::new handles whatever it's given.
        let game = Game::new(10);
        assert_eq!(game.current_level, 10);
    }

    #[test]
    fn test_parse_start_level() {
        assert_eq!(parse_start_level(&vec!["exe".to_string()]), 1);
        assert_eq!(parse_start_level(&vec!["exe".to_string(), "3".to_string()]), 3);
        assert_eq!(parse_start_level(&vec!["exe".to_string(), "0".to_string()]), 1);
        assert_eq!(parse_start_level(&vec!["exe".to_string(), "10".to_string()]), 5);
        assert_eq!(parse_start_level(&vec!["exe".to_string(), "abc".to_string()]), 1);
    }
}

