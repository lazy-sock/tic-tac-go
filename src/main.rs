use std::error::Error;
use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            // grid size (fixed)
            let default_grid_w: u16 = 13; // width of the ASCII grid
            let default_grid_h: u16 = 7;  // number of lines in the ASCII grid

            // ensure grid fits terminal
            let grid_w = if default_grid_w + 2 > size.width { size.width.saturating_sub(2) } else { default_grid_w };
            let grid_h = if default_grid_h + 2 > size.height { size.height.saturating_sub(2) } else { default_grid_h };

            let x = (size.width.saturating_sub(grid_w)) / 2;
            let y = (size.height.saturating_sub(grid_h)) / 2;
            let area = Rect::new(x, y, grid_w, grid_h);

            let lines = vec![
                Spans::from(Span::raw("┌───┬───┬───┐")),
                Spans::from(Span::raw("│   │   │   │")),
                Spans::from(Span::raw("├───┼───┼───┤")),
                Spans::from(Span::raw("│   │   │   │")),
                Spans::from(Span::raw("├───┼───┼───┤")),
                Spans::from(Span::raw("│   │   │   │")),
                Spans::from(Span::raw("└───┴───┴───┘")),
            ];

            let paragraph = Paragraph::new(lines).block(Block::default());
            f.render_widget(paragraph, area);
        })?;

        // Exit on 'q' or Esc
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }
    Ok(())
}
