use rand::seq::SliceRandom;
use rand::thread_rng;
use std::env;
use std::error::Error;
use std::io::{self};

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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Show start screen to select difficulty. If the user quits from the start screen, exit gracefully.
    let chosen_difficulty = match game::select_difficulty(&mut terminal) {
        Ok(d) => d,
        Err(_) => {
            // Restore terminal and exit without running the game
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            return Ok(());
        }
    };

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
