use std::{error::Error, io::Stdout, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders},
};

pub fn show_browser(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("browser").borders(Borders::ALL);
            let area = Rect::new(0, 0, size.width, size.height);
            f.render_widget(block, area);
        })?;

        if event::poll(Duration::from_millis(150))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                _ => {}
            }
        }
    }
}
