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
    let preview = (5usize, 5usize);
    let mut cursor = vec![(0usize, 0usize)];

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let overlay_w = std::cmp::min(60, size.width.saturating_sub(4));

            let mut lines: Vec<Spans> = Vec::new();
            lines.push(Spans::from(Span::styled(
                " Create puzzle ",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Spans::from(Span::raw("")));
            lines.extend(create_matrix(&[(preview.0, preview.1)], &cursor));
            lines.push(Spans::from(Span::raw("")));
            lines.push(Spans::from(Span::raw(
                " Use WASD or arrow keys to move the cursor ",
            )));
            lines.push(Spans::from(Span::raw("Press q or Esc to return.")));

            // compute height based on content, cap to terminal size and a reasonable max
            let desired_h = (lines.len() as u16).saturating_add(2);
            let max_h = size.height.saturating_sub(4);
            let overlay_h = std::cmp::min(60u16, std::cmp::min(max_h, desired_h));

            let ox = (size.width.saturating_sub(overlay_w)) / 2;
            let oy = (size.height.saturating_sub(overlay_h)) / 2;
            let area = Rect::new(ox, oy, overlay_w, overlay_h);

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

        if event::poll(Duration::from_millis(150))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                code => move_cursor(&mut cursor, code, preview.0, preview.1),
            }
        }
    }
}

fn move_cursor(cursor: &mut Vec<(usize, usize)>, key: KeyCode, rows: usize, cols: usize) {
    if rows == 0 || cols == 0 {
        return;
    }
    if cursor.is_empty() {
        cursor.push((0, 0));
    }
    if let Some(pos) = cursor.get_mut(0) {
        match key {
            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                pos.0 = pos.0.saturating_sub(1);
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                pos.0 = std::cmp::min(pos.0 + 1, rows.saturating_sub(1));
            }
            KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                pos.1 = pos.1.saturating_sub(1);
            }
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                pos.1 = std::cmp::min(pos.1 + 1, cols.saturating_sub(1));
            }
            _ => {}
        }
    }
}

fn create_matrix(size: &[(usize, usize)], cursor: &[(usize, usize)]) -> Vec<Spans<'static>> {
    let mut output: Vec<Spans<'static>> = Vec::new();

    for (rows, cols) in size.iter().copied() {
        // handle degenerate sizes
        if rows == 0 || cols == 0 {
            output.push(Spans::default());
            output.push(Spans::default());
            output.push(Spans::default());
            continue;
        }

        // Top border
        let mut top = String::new();
        for _ in 0..cols {
            top.push_str("─── ");
        }
        output.push(Spans::from(Span::raw(top)));

        for row in 0..rows {
            // Content line: draw cells with internal vertical separators; highlight cursor cell
            let mut spans: Vec<Span<'static>> = Vec::new();
            for col in 0..cols {
                let cell = if cursor.contains(&(row, col)) {
                    Span::styled(
                        " ● ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::raw("   ")
                };
                spans.push(cell);
                if col + 1 < cols {
                    spans.push(Span::raw("│"));
                } else {
                    spans.push(Span::raw(" "));
                }
            }
            output.push(Spans::from(spans));

            // Border after row
            let mut border = String::new();
            for _ in 0..cols {
                border.push_str("─── ");
            }
            output.push(Spans::from(Span::raw(border)));
        }

        // blank separator between previews
        output.push(Spans::default());
    }

    output
}
