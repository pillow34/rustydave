use std::env;
use std::io::stdout;
use rustydave::{generate_level, Tile, LEVEL_WIDTH, LEVEL_HEIGHT};
use crossterm::style::{Color, SetForegroundColor, ResetColor, Print};
use crossterm::execute;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <level_number>", args[0]);
        return Ok(());
    }

    let level_num: u32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid level number: {}", args[1]);
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
                execute!(out, SetForegroundColor(Color::Cyan), Print("D"), ResetColor)?;
            } else {
                match level[y][x] {
                    Tile::Empty => row.push(' '),
                    Tile::Wall => {
                        print!("{}", row);
                        row.clear();
                        execute!(out, SetForegroundColor(Color::Blue), Print("#"), ResetColor)?;
                    }
                    Tile::Trophy => {
                        print!("{}", row);
                        row.clear();
                        execute!(out, SetForegroundColor(Color::Yellow), Print("*"), ResetColor)?;
                    }
                    Tile::Exit => {
                        print!("{}", row);
                        row.clear();
                        execute!(out, SetForegroundColor(Color::Green), Print("E"), ResetColor)?;
                    }
                    Tile::Hazard => {
                        print!("{}", row);
                        row.clear();
                        execute!(out, SetForegroundColor(Color::Red), Print("^"), ResetColor)?;
                    }
                    Tile::Diamond => {
                        print!("{}", row);
                        row.clear();
                        execute!(out, SetForegroundColor(Color::Magenta), Print("+"), ResetColor)?;
                    }
                }
            }
        }
        println!("{}", row);
    }

    Ok(())
}
