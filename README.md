# Rusty Dave

Rusty Dave is a terminal-based platformer game written in Rust, inspired by classic games like "Dangerous Dave". It features procedural level generation, simple physics, and retro terminal graphics.

## Gameplay

In Rusty Dave, you control Dave (**D**) through a series of levels. Your objective in each level is:
1. Find and collect the **Trophy** (**`*`**).
2. Reach the **Exit** (**`E`**) once you have the trophy.

Be careful! If you touch a **Hazard** (**`^`**), you'll perish (**`X`**) and have to restart the level.

## Controls

- **Move Left:** `Left Arrow` or `A`
- **Move Right:** `Right Arrow` or `D`
- **Jump:** `Up Arrow`, `W`, or `Space`
- **Quit Game:** `Esc` or `Q`
- **Restart / Next Level:** `Enter` (when dead or level complete)

## Features

- **Procedural Levels:** Levels are generated on-the-fly, ensuring a unique experience while remaining solvable.
- **Physics-based Movement:** Dave's movement includes acceleration, friction, and gravity for a smooth platforming feel.
- **Terminal Graphics:** Uses `crossterm` for cross-platform terminal manipulation and colors.
- **Progressive Difficulty:** 5 distinct levels to challenge your skills.

## Level Design Example

Here is an example of a procedurally generated level (Level 1):

```text
############################################################
#                                                          #
#                                                          #
#                                 ^^    ^   ^    ^ *       #
#                         ##################################
#                                                          #
#                                                          #
#       ^^       ^^                                        #
#####################################                      #
#                                                          #
#                                                          #
#                                       ^   ^^       ^     #
#                                 ##########################
#                                                          #
#                                                          #
#                ^^    ^           E                       #
#              ######################                      #
# D                                                        #
##########                                                 #
###################^############^###########################
```

In this layout:
- `D`: Dave (Player)
- `#`: Wall / Platform
- `*`: Trophy
- `E`: Exit
- `^`: Hazard

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)

### Running the Game

Clone the repository and run using Cargo:

```bash
cargo run
```

You can also start at a specific level (1-5) by passing it as an argument:

```bash
cargo run -- 3
```

## Technical Details

- **Language:** Rust 2024 edition.
- **Library:** `crossterm` for terminal handling (raw mode, colors, cursor movement).
- **Architecture:** 
    - `src/main.rs`: Game loop, physics update, and rendering logic.
    - `src/lib.rs`: Tile definitions, level generation, and a simple custom RNG.

## License

This project is open-source. Feel free to explore and modify!
