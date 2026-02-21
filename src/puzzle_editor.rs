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
    let mut circles: Vec<(usize, usize)> = Vec::new();
    let mut crosses: Vec<(usize, usize)> = Vec::new();

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
            lines.extend(create_matrix(&[(preview.0, preview.1)], &cursor, &circles, &crosses));
            lines.push(Spans::from(Span::raw("")));
            lines.push(Spans::from(Span::raw(
                " Press O or X to draw, Backspace to delete ",
            )));
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
                KeyCode::Char('o') | KeyCode::Char('O') | KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Backspace => {
                    edit_cell(key.code, &cursor, &mut circles, &mut crosses)
                }
                code => move_cursor(&mut cursor, code, preview.0, preview.1),
            }
        }
    }
}

fn edit_cell(key: KeyCode, cursor: &[(usize, usize)], circles: &mut Vec<(usize, usize)>, crosses: &mut Vec<(usize, usize)>) {
    if cursor.is_empty() {
        return;
    }
    let pos = cursor[0];
    match key {
        KeyCode::Char('o') | KeyCode::Char('O') => {
            // remove cross if present, add circle if missing
            if let Some(idx) = crosses.iter().position(|&p| p == pos) {
                crosses.remove(idx);
            }
            if !circles.contains(&pos) {
                circles.push(pos);
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if let Some(idx) = circles.iter().position(|&p| p == pos) {
                circles.remove(idx);
            }
            if !crosses.contains(&pos) {
                crosses.push(pos);
            }
        }
        KeyCode::Backspace => {
            if let Some(idx) = circles.iter().position(|&p| p == pos) {
                circles.remove(idx);
            }
            if let Some(idx) = crosses.iter().position(|&p| p == pos) {
                crosses.remove(idx);
            }
        }
        _ => {}
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

fn create_matrix(size: &[(usize, usize)], cursor: &[(usize, usize)], circles: &[(usize, usize)], crosses: &[(usize, usize)]) -> Vec<Spans<'static>> {
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
            // Content line: draw cells with internal vertical separators; draw objects and cursor
            let mut spans: Vec<Span<'static>> = Vec::new();
            for col in 0..cols {
                // priority: circles > crosses > cursor > empty
                if circles.iter().position(|&(rr, cc)| rr == row && cc == col).is_some() {
                    let style = Style::default().fg(Color::LightBlue);
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled("o".to_string(), style));
                    spans.push(Span::raw(if col + 1 < cols { " │" } else { "  " }));
                    continue;
                }
                if crosses.iter().position(|&(rr, cc)| rr == row && cc == col).is_some() {
                    let style = Style::default().fg(Color::Red);
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled("x".to_string(), style));
                    spans.push(Span::raw(if col + 1 < cols { " │" } else { "  " }));
                    continue;
                }

                if cursor.contains(&(row, col)) {
                    // make cursor yellow so it doesn't look like a circle
                    spans.push(Span::styled(
                        " ● ",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::raw("   "));
                }

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
