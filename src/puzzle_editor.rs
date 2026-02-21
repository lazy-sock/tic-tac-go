use crate::CrosstermBackend;
use crate::Error;
use crate::Terminal;
use crate::io::Stdout;
use crate::puzzle_editor::event::Event;
use crossterm::event;
use crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use std::time::Duration;

pub fn show_create_placeholder(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let overlay_w = std::cmp::min(60, size.width.saturating_sub(4));
            let overlay_h = 7u16;
            let ox = (size.width.saturating_sub(overlay_w)) / 2;
            let oy = (size.height.saturating_sub(overlay_h)) / 2;
            let area = Rect::new(ox, oy, overlay_w, overlay_h);

            let mut lines: Vec<Spans> = Vec::new();
            lines.push(Spans::from(Span::styled(
                " Create puzzle ",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Spans::from(Span::raw("")));
            lines.push(Spans::from(Span::raw("Feature coming soon.")));
            lines.push(Spans::from(Span::raw("")));
            lines.push(Spans::from(Span::raw("Press q or Esc to return.")));

            let para = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("tic-tac-go"));

            f.render_widget(Clear, area);
            f.render_widget(
                Block::default().style(Style::default().bg(Color::Black)),
                area,
            );
            f.render_widget(para, area);
        })?;

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}

fn create_matrix(size: Vec<(usize, usize)>) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();

    for (rows, cols) in size.into_iter() {
        // handle degenerate sizes
        if rows == 0 || cols == 0 {
            output.push(String::new());
            output.push(String::new());
            output.push(String::new());
            continue;
        }

        // Top border
        let mut top = String::new();
        for _ in 0..cols {
            top.push_str("─── ");
        }
        output.push(top);

        for row in 0..rows {
            // Content line: draw empty cells with internal vertical separators between adjacent cells
            let mut content = String::new();
            for col in 0..cols {
                let next_present = col + 1 < cols; // rectangular preview: all cells present
                if next_present {
                    content.push_str("   │");
                } else {
                    content.push_str("    ");
                }
            }
            output.push(content);

            // Middle border or bottom border
            if row != rows - 1 {
                let mut mid = String::new();
                for _ in 0..cols {
                    mid.push_str("─── ");
                }
                output.push(mid);
            } else {
                let mut bot = String::new();
                for _ in 0..cols {
                    bot.push_str("─── ");
                }
                output.push(bot);
            }
        }

        // blank separator between previews
        output.push(String::new());
    }

    output
}

