# Rusty Dave

Rusty Dave is a terminal-based platformer game written in Rust, inspired by classic games like "Dangerous Dave". It features procedural level generation, simple physics, and retro terminal graphics.

## Gameplay

In Rusty Dave, you control Dave through a series of levels. Your objective in each level is:
1. Find and collect the **Trophy**.
2. Reach the **Exit** once you have the trophy.
3. Collect **Diamonds** along the way for extra points!

Be careful! If you touch a **Hazard**, you'll lose a life. You start with 3 lives. If you lose all lives, it's Game Over!

### Graphics Modes

The game supports two graphics modes:

1.  **Older Graphics (Default):** Uses single-character tiles for a classic retro feel.
2.  **ASCII Graphics:** Uses 2-character wide tiles for a more detailed "graphical" look (enable with `--ascii`).

#### Legend

| Element | Older Graphics | ASCII Graphics |
| :--- | :---: | :---: |
| Dave | `D` | `☺ ` |
| Wall | `#` | `██` |
| Trophy | `*` | `★ ` |
| Exit | `E` | `][` |
| Hazard | `^` | `▲▲` |
| Diamond | `+` | `♦ ` |

## Controls

- **Move Left:** `Left Arrow` or `A`
- **Move Right:** `Right Arrow` or `D`
- **Jump:** `Up Arrow`, `W`, or `Space`
- **Quit Game:** `Esc` or `Q`
- **Restart / Next Level:** `Enter` (when dead or level complete)

## Features

- **Procedural Levels:** Levels are generated on-the-fly, ensuring a unique experience while remaining solvable. Now supports multiple archetypes (Zig-zag and Islands).
- **Physics-based Movement:** Dave's movement includes acceleration, friction, and gravity for a smooth platforming feel.
- **Terminal Graphics:** Uses `crossterm` for cross-platform terminal manipulation and colors.
- **Progressive Difficulty:** 10 distinct levels to challenge your skills.
- **Lives & Score System:** Collect diamonds for points and manage your limited lives.
- **External Configuration:** Customize physics and keybindings via `config.toml`. You can change gravity, speed, jump height, and rebind keys without recompiling.
- **Reachability Validation:** Includes a sophisticated level validator that uses pathfinding to ensure every generated level is solvable.

## Configuration (config.toml)

You can customize the game by editing `config.toml`. If the file is missing, the game will use default values.

```toml
max_level = 10

[physics]
target_vx = 30.0
accel_ground = 200.0
accel_air = 80.0
jump_vy = -28.0
gravity = 80.0
coyote_time = 0.1
jump_buffer_time = 0.1
jump_release_gravity_mult = 3.0
friction = 400.0

[keys]
left = ["Left", "a", "A"]
right = ["Right", "d", "D"]
jump = ["Up", "w", "W", "Space"]
quit = ["Esc", "q", "Q"]
restart = ["Enter"]
```

## Level Design Example

Here is an example of a procedurally generated level (Level 1) using ASCII Graphics (`--ascii`):

```text
████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████
██                                                                                                                    ██
██                                                                                                                    ██
██                                                                  ▲▲▲▲        ▲▲      ▲▲        ▲▲  ★               ██
██                                                  ████████████████████████████████████████████████████████████████████
██                                                                                                                    ██
██                                                                                                                    ██
██              ▲▲▲▲              ▲▲▲▲                                                                                ██
████████████████████████████████████████████████████████████                ██                                        ██
██                                                                                                                    ██
██                                                                                                                    ██
██                                                                              ▲▲      ▲▲▲▲              ▲▲          ██
██                                                  ████████████████████████████████████████████████████████████████████
██                                                                                                                    ██
██                                                                                                                    ██
██                                ▲▲▲▲        ▲▲                    ][                                                ██
██                            ████████████████████████████████████████████                                            ██
██  ☺                                                                                                                 ██
████████████████████                                                                                                  ██
██████████████████████████████████████▲▲████████████████████████████████████▲▲██████████████████████████████████████████
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)

### Running the Game

Clone the repository and run using Cargo:

```bash
# Runs with Older Graphics (Default)
cargo run

# Runs with ASCII Graphics
cargo run -- --ascii
```

You can also start at a specific level (up to the `max_level` defined in `config.toml`) and optionally specify the graphics mode:

```bash
# Start at Level 3 with Older Graphics
cargo run -- 3

# Start at Level 3 with ASCII Graphics
cargo run -- 3 --ascii
```

### Level Validation

To verify the solvability of all levels defined by `max_level` in your configuration, run:

```bash
cargo run --bin validate_levels
```

This tool uses Breadth-First Search (BFS) to simulate player movement and ensure the Trophy and Exit are reachable in every level.

## Technical Details

- **Language:** Rust 2024 edition.
- **Library:** `crossterm` for terminal handling (raw mode, colors, cursor movement).
- **Architecture:** 
    - `src/main.rs`: Game loop, physics update, and rendering logic.
    - `src/lib.rs`: Tile definitions, level generation, and a simple custom RNG.

## License

This project is open-source. Feel free to explore and modify!
