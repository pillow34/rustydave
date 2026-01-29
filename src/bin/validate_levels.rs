use rustydave::{generate_level, Tile, LEVEL_WIDTH, LEVEL_HEIGHT, Config};
use std::collections::VecDeque;

fn main() {
    let config = Config::load();
    let mut failures = 0;
    let total_levels = config.max_level;
    for seed in 1..=total_levels {
        let (level, (px, py)) = generate_level(seed);
        let mut seed_failed = false;

        // 1. Basic Existence Checks
        let mut trophy_pos = None;
        let mut exit_pos = None;

        for y in 0..LEVEL_HEIGHT {
            for x in 0..LEVEL_WIDTH {
                if level[y][x] == Tile::Trophy {
                    trophy_pos = Some((x, y));
                    if y + 1 >= LEVEL_HEIGHT || level[y + 1][x] != Tile::Wall {
                        println!("Seed {}: Trophy at ({}, {}) has no platform below!", seed, x, y);
                        seed_failed = true;
                    }
                }
                if level[y][x] == Tile::Exit {
                    exit_pos = Some((x, y));
                    if y + 1 >= LEVEL_HEIGHT || level[y + 1][x] != Tile::Wall {
                         println!("Seed {}: Exit at ({}, {}) has no platform below!", seed, x, y);
                         seed_failed = true;
                    }
                }
            }
        }

        if trophy_pos.is_none() {
            println!("Seed {}: No Trophy found!", seed);
            seed_failed = true;
        }
        if exit_pos.is_none() {
            println!("Seed {}: No Exit found!", seed);
            seed_failed = true;
        }

        // 2. Player Start Safety
        let p_tx = px.floor() as usize;
        let p_ty = py.floor() as usize;
        if p_tx >= LEVEL_WIDTH || p_ty >= LEVEL_HEIGHT || level[p_ty][p_tx] == Tile::Wall || level[p_ty][p_tx] == Tile::Hazard {
            println!("Seed {}: Player starts in dangerous location ({}, {})", seed, px, py);
            seed_failed = true;
        }

        // 3. Hazard Rule Checks
        // Rules: 
        // - Single or double hazards only (max 2 consecutive)
        // - Separated by at least 3 blocks
        // - Not more than 4 hazards in any 15-block horizontal range
        for y in 0..LEVEL_HEIGHT {
            let mut x = 0;
            while x < LEVEL_WIDTH {
                if level[y][x] == Tile::Hazard {
                    let mut count = 0;
                    while x < LEVEL_WIDTH && level[y][x] == Tile::Hazard {
                        count += 1;
                        x += 1;
                    }
                    if count > 2 {
                        println!("Seed {}: Too many consecutive hazards at y={}! Found {}", seed, y, count);
                        seed_failed = true;
                    }
                    
                    // Separation check: peek ahead for next hazard
                    let mut space = 0;
                    let sep_start = x;
                    while x < LEVEL_WIDTH && level[y][x] != Tile::Hazard {
                        space += 1;
                        x += 1;
                    }
                    if x < LEVEL_WIDTH && level[y][x] == Tile::Hazard && space < 3 {
                        println!("Seed {}: Hazards too close together at y={}! Space was only {} blocks", seed, y, space);
                        seed_failed = true;
                    }
                    // Backtrack to just after the hazard block to continue scanning from there
                    x = sep_start;
                } else {
                    x += 1;
                }
            }
            
            // Density check: sliding window of 15 tiles
            for start_x in 0..=(LEVEL_WIDTH as i32 - 15).max(0) as usize {
                let mut hazard_count = 0;
                for i in 0..15 {
                    if start_x + i < LEVEL_WIDTH && level[y][start_x + i] == Tile::Hazard {
                        hazard_count += 1;
                    }
                }
                if hazard_count > 4 {
                    println!("Seed {}: Hazard density too high at y={}, x range {}..{}", seed, y, start_x, start_x + 15);
                    seed_failed = true;
                    break;
                }
            }
        }

        // 4. Boundary Check
        for x in 0..LEVEL_WIDTH {
            if level[0][x] != Tile::Wall {
                println!("Seed {}: Top boundary broken at x={}", seed, x);
                seed_failed = true;
            }
            if level[LEVEL_HEIGHT - 1][x] != Tile::Wall && level[LEVEL_HEIGHT - 1][x] != Tile::Hazard {
                println!("Seed {}: Bottom boundary broken at x={}", seed, x);
                seed_failed = true;
            }
        }
        for y in 0..LEVEL_HEIGHT {
            if level[y][0] != Tile::Wall {
                println!("Seed {}: Left boundary broken at y={}", seed, y);
                seed_failed = true;
            }
            if level[y][LEVEL_WIDTH - 1] != Tile::Wall {
                println!("Seed {}: Right boundary broken at y={}", seed, y);
                seed_failed = true;
            }
        }

        // 5. Reachability (BFS)
        if let (Some(t_pos), Some(e_pos)) = (trophy_pos, exit_pos) {
            let start_pos = (p_tx, p_ty);
            let can_reach_trophy = is_reachable(&level, start_pos, t_pos);
            if !can_reach_trophy {
                println!("Seed {}: Trophy is NOT reachable from start!", seed);
                seed_failed = true;
            } else {
                let can_reach_exit = is_reachable(&level, t_pos, e_pos);
                if !can_reach_exit {
                    println!("Seed {}: Exit is NOT reachable from Trophy!", seed);
                    seed_failed = true;
                }
            }
        }

        if seed_failed {
            failures += 1;
        }
    }

    if failures == 0 {
        println!("All {total_levels} levels validated successfully!");
    } else {
        println!("Found {} seeds with validation failures.", failures);
        std::process::exit(1);
    }
}

/// A simple BFS to check reachability in the level.
/// Accounts for horizontal movement, falling, and jumping.
fn is_reachable(level: &[[Tile; LEVEL_WIDTH]; LEVEL_HEIGHT], start: (usize, usize), target: (usize, usize)) -> bool {
    let mut visited = [[false; LEVEL_WIDTH]; LEVEL_HEIGHT];
    let mut queue = VecDeque::new();

    queue.push_back(start);
    visited[start.1][start.0] = true;

    while let Some((cx, cy)) = queue.pop_front() {
        if (cx, cy) == target {
            return true;
        }

        // Potential next positions
        let mut neighbors = Vec::new();

        let is_safe = |nx: usize, ny: usize| {
            nx < LEVEL_WIDTH && ny < LEVEL_HEIGHT && 
            level[ny][nx] != Tile::Wall && 
            level[ny][nx] != Tile::Hazard
        };

        let on_ground = cy + 1 < LEVEL_HEIGHT && level[cy + 1][cx] == Tile::Wall;

        // 1. Walk left/right
        if cx > 0 && is_safe(cx - 1, cy) {
            neighbors.push((cx - 1, cy));
        }
        if cx + 1 < LEVEL_WIDTH && is_safe(cx + 1, cy) {
            neighbors.push((cx + 1, cy));
        }

        // 2. Fall down
        if !on_ground {
            if cy + 1 < LEVEL_HEIGHT && is_safe(cx, cy + 1) {
                neighbors.push((cx, cy + 1));
            }
            // Optional: air control / diagonal falling
            if cx > 0 && cy + 1 < LEVEL_HEIGHT && is_safe(cx - 1, cy + 1) {
                neighbors.push((cx - 1, cy + 1));
            }
            if cx + 1 < LEVEL_WIDTH && cy + 1 < LEVEL_HEIGHT && is_safe(cx + 1, cy + 1) {
                neighbors.push((cx + 1, cy + 1));
            }
        }

        // 3. Jump (if on ground)
        if on_ground {
            // Dave can jump ~4 tiles high and ~20 tiles horizontally.
            // We'll use a slightly conservative box to simulate reachable area.
            for dy in 1..=4 {
                if cy >= dy {
                    let ny = cy - dy;
                    // Horizontal range depends on height
                    // At peak (dy=4), horizontal offset can be ~10
                    // We'll just allow a generous range and assume Dave can make the arc.
                    let h_range = match dy {
                        1 => 5,
                        2 => 8,
                        3 => 10,
                        4 => 12,
                        _ => 0,
                    };
                    for dx in -(h_range as i32)..=(h_range as i32) {
                        let nx = cx as i32 + dx;
                        if nx >= 0 && nx < LEVEL_WIDTH as i32 {
                            let nx = nx as usize;
                            if is_safe(nx, ny) {
                                neighbors.push((nx, ny));
                            }
                        }
                    }
                }
            }
        }

        for (nx, ny) in neighbors {
            if !visited[ny][nx] {
                visited[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    false
}
