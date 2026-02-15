use std::error::Error;
use std::io::{self};
use std::env;
use rand::thread_rng;
use rand::seq::SliceRandom;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

mod board;
mod game;
mod generator;
mod movement;
mod rules;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    let chosen_difficulty = {
        // parse CLI args for -d / --difficulty; if absent, pick random
        let mut args = env::args().skip(1);
        let mut parsed: Option<crate::generator::Difficulty> = None;
        while let Some(arg) = args.next() {
            if arg == "-d" || arg == "--difficulty" {
                if let Some(val) = args.next() {
                    parsed = match val.to_lowercase().as_str() {
                        "easy" => Some(crate::generator::Difficulty::Easy),
                        "medium" => Some(crate::generator::Difficulty::Medium),
                        "hard" => Some(crate::generator::Difficulty::Hard),
                        _ => { eprintln!("unknown difficulty: {}", val); None }
                    };
                    break;
                }
            } else if arg.starts_with("--difficulty=") {
                let val = arg.splitn(2, '=').nth(1).unwrap().to_string();
                parsed = match val.to_lowercase().as_str() {
                    "easy" => Some(crate::generator::Difficulty::Easy),
                    "medium" => Some(crate::generator::Difficulty::Medium),
                    "hard" => Some(crate::generator::Difficulty::Hard),
                    _ => { eprintln!("unknown difficulty: {}", val); None }
                };
                break;
            } else if arg.starts_with("-d=") {
                let val = arg.splitn(2, '=').nth(1).unwrap().to_string();
                parsed = match val.to_lowercase().as_str() {
                    "easy" => Some(crate::generator::Difficulty::Easy),
                    "medium" => Some(crate::generator::Difficulty::Medium),
                    "hard" => Some(crate::generator::Difficulty::Hard),
                    _ => { eprintln!("unknown difficulty: {}", val); None }
                };
                break;
            }
        }
        if let Some(d) = parsed {
            d
        } else {
            let mut rng = thread_rng();
            *[crate::generator::Difficulty::Easy, crate::generator::Difficulty::Medium, crate::generator::Difficulty::Hard].choose(&mut rng).unwrap()
        }
    };
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = game::run_app(&mut terminal, chosen_difficulty);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }
    Ok(())
}
