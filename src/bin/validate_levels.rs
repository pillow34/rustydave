use rustydave::{generate_level, Tile, LEVEL_WIDTH, LEVEL_HEIGHT};

fn main() {
    let mut failures = 0;
    let total_levels = 1000;
    for seed in 1..=total_levels {
        let (level, _) = generate_level(seed);
        let mut trophy_found = false;
        let mut exit_found = false;

        for y in 0..LEVEL_HEIGHT {
            for x in 0..LEVEL_WIDTH {
                if level[y][x] == Tile::Trophy {
                    trophy_found = true;
                    // Check if there is a wall below
                    if y + 1 >= LEVEL_HEIGHT || level[y + 1][x] != Tile::Wall {
                        println!("Seed {}: Trophy at ({}, {}) has no platform below!", seed, x, y);
                        failures += 1;
                    }
                }
                if level[y][x] == Tile::Exit {
                    exit_found = true;
                    // Check if there is a wall below (Exit can be on the floor y=18, or on a platform)
                    // Floor is y=18, boundary is y=19. If it's on floor, y+1 is 19 which is Wall.
                    if y + 1 >= LEVEL_HEIGHT || level[y + 1][x] != Tile::Wall {
                         println!("Seed {}: Exit at ({}, {}) has no platform below!", seed, x, y);
                         failures += 1;
                    }
                }
            }
        }

        if !trophy_found {
            println!("Seed {}: No Trophy found!", seed);
            failures += 1;
        }
        if !exit_found {
            println!("Seed {}: No Exit found!", seed);
            failures += 1;
        }
    }

    if failures == 0 {
        println!("All {total_levels} levels validated successfully!");
    } else {
        println!("Found {} validation failures.", failures);
        std::process::exit(1);
    }
}
