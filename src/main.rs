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
mod puzzle_editor;
mod rules;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Show main menu to select mode (play or create). If user quits, exit gracefully.
    let res = match game::select_mode(&mut terminal) {
        Ok(game::StartupMode::Play(d)) => game::run_app(&mut terminal, d),
        Ok(game::StartupMode::Create) => {
            // show placeholder for create puzzle, then restore and exit
            puzzle_editor::show_create_placeholder(&mut terminal)?;
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            return Ok(());
        }
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
