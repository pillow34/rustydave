use std::env;
use std::io::stdout;
use rustydave::{generate_level, Tile, LEVEL_WIDTH, LEVEL_HEIGHT};
use crossterm::style::{Color, SetForegroundColor, ResetColor, Print};
use crossterm::execute;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut level_num = None;
    let mut use_ascii = false;

    for arg in args.iter().skip(1) {
        if arg == "--ascii" {
            use_ascii = true;
        } else if let Ok(n) = arg.parse::<u32>() {
            level_num = Some(n);
        }
    }

    let level_num = match level_num {
        Some(n) => n,
        None => {
            println!("Usage: {} <level_number> [--ascii]", args[0]);
            return Ok(());
        }
    };

    let (level, (px, py)) = generate_level(level_num);

    let mut out = stdout();

    execute!(out, SetForegroundColor(Color::Magenta), Print(format!("--- Level {} ---\n", level_num)), ResetColor)?;

    for y in 0..LEVEL_HEIGHT {
        let mut row = String::new();
        for x in 0..LEVEL_WIDTH {
            if x == px.floor() as usize && y == py.floor() as usize {
                // Print buffered row so far
                print!("{}", row);
                row.clear();
                let sym = if use_ascii { "☺ " } else { "D" };
                execute!(out, SetForegroundColor(Color::Cyan), Print(sym), ResetColor)?;
            } else {
                match level[y][x] {
                    Tile::Empty => row.push_str(if use_ascii { "  " } else { " " }),
                    Tile::Wall => {
                        print!("{}", row);
                        row.clear();
                        let sym = if use_ascii { "██" } else { "#" };
                        execute!(out, SetForegroundColor(Color::Blue), Print(sym), ResetColor)?;
                    }
                    Tile::Trophy => {
                        print!("{}", row);
                        row.clear();
                        let sym = if use_ascii { "★ " } else { "*" };
                        execute!(out, SetForegroundColor(Color::Yellow), Print(sym), ResetColor)?;
                    }
                    Tile::Exit => {
                        print!("{}", row);
                        row.clear();
                        let sym = if use_ascii { "][" } else { "E" };
                        execute!(out, SetForegroundColor(Color::Green), Print(sym), ResetColor)?;
                    }
                    Tile::Hazard => {
                        print!("{}", row);
                        row.clear();
                        let sym = if use_ascii { "▲▲" } else { "^" };
                        execute!(out, SetForegroundColor(Color::Red), Print(sym), ResetColor)?;
                    }
                    Tile::Diamond => {
                        print!("{}", row);
                        row.clear();
                        let sym = if use_ascii { "♦ " } else { "+" };
                        execute!(out, SetForegroundColor(Color::Magenta), Print(sym), ResetColor)?;
                    }
                }
            }
        }
        println!("{}", row);
    }

    Ok(())
}
