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
    let mut preview = (5usize, 5usize);
    let mut cursor = vec![(0usize, 0usize)];
    let mut circles: Vec<(usize, usize)> = Vec::new();
    let mut crosses: Vec<(usize, usize)> = Vec::new();
    let mut removed: Vec<(usize, usize)> = Vec::new();

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
            lines.extend(create_matrix(
                &[(preview.0, preview.1)],
                &cursor,
                &circles,
                &crosses,
                &removed,
            ));
            lines.push(Spans::from(Span::raw("")));
            lines.push(Spans::from(Span::raw(
                " Press O or X to draw, Backspace to delete ",
            )));
            lines.push(Spans::from(Span::raw(
                " Use WASD or arrow keys to move the cursor ",
            )));
            lines.push(Spans::from(Span::raw(
                " Use + and - to change size of matrix ",
            )));
            lines.push(Spans::from(Span::raw(
                " Backspace on empty cell to delete, Space on empty cell to add. Press R to restore all cells. ",
            )));
            lines.push(Spans::from(Span::raw("Press q or Esc to return.")));
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
                KeyCode::Char('o')
                | KeyCode::Char('O')
                | KeyCode::Char('x')
                | KeyCode::Char('X')
                | KeyCode::Backspace => {
                    edit_cell(key.code, &cursor, &mut circles, &mut crosses, &mut removed)
                }
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    // Increase matrix size (append to bottom/right)
                    increase_preview(&mut preview);
                }
                KeyCode::Char('-') => {
                    // Decrease matrix size and drop any marks that fall outside
                    decrease_preview(&mut preview, &mut circles, &mut crosses, &mut removed);
                    // Ensure cursor remains within bounds
                    if let Some(pos) = cursor.get_mut(0) {
                        pos.0 = std::cmp::min(pos.0, preview.0.saturating_sub(1));
                        pos.1 = std::cmp::min(pos.1, preview.1.saturating_sub(1));
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    // Restore all removed cells
                    removed.clear();
                }
                code => move_cursor(&mut cursor, code, preview.0, preview.1),
            }
        }
    }
}

fn edit_cell(
    key: KeyCode,
    cursor: &[(usize, usize)],
    circles: &mut Vec<(usize, usize)>,
    crosses: &mut Vec<(usize, usize)>,
    removed: &mut Vec<(usize, usize)>,
) {
    if cursor.is_empty() {
        return;
    }
    let pos = cursor[0];
    match key {
        KeyCode::Char('o') | KeyCode::Char('O') => {
            // ignore if cell is removed
            if removed.contains(&pos) {
                return;
            }
            // remove cross if present, add circle if missing
            if let Some(idx) = crosses.iter().position(|&p| p == pos) {
                crosses.remove(idx);
            }
            if !circles.contains(&pos) {
                circles.push(pos);
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if removed.contains(&pos) {
                return;
            }
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
            } else if let Some(idx) = crosses.iter().position(|&p| p == pos) {
                crosses.remove(idx);
            } else if !removed.contains(&pos) {
                // delete the empty cell
                removed.push(pos);
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

fn create_matrix(
    size: &[(usize, usize)],
    cursor: &[(usize, usize)],
    circles: &[(usize, usize)],
    crosses: &[(usize, usize)],
    removed: &[(usize, usize)],
) -> Vec<Spans<'static>> {
    let mut output: Vec<Spans<'static>> = Vec::new();

    for (rows, cols) in size.iter().copied() {
        // handle degenerate sizes
        if rows == 0 || cols == 0 {
            output.push(Spans::default());
            output.push(Spans::default());
            output.push(Spans::default());
            continue;
        }

        // Top border: draw top edges; highlight when cursor is on a cell (including removed cells)
        let mut top_spans: Vec<Span> = Vec::new();
        for col in 0..cols {
            let is_removed = removed.contains(&(0usize, col));
            if !is_removed {
                let filled = circles.iter().any(|&(r, c)| r == 0 && c == col)
                    || crosses.iter().any(|&(r, c)| r == 0 && c == col);
                let highlight = cursor.contains(&(0usize, col)) && (filled);
                if highlight {
                    top_spans.push(Span::styled("─── ", Style::default().fg(Color::Yellow)));
                } else {
                    top_spans.push(Span::raw("─── "));
                }
            } else {
                // removed cell: show blank unless cursor is on it, then highlight top border
                if cursor.contains(&(0usize, col)) {
                    top_spans.push(Span::styled("─── ", Style::default().fg(Color::Yellow)));
                } else {
                    top_spans.push(Span::raw("    "));
                }
            }
        }
        output.push(Spans::from(top_spans));

        for row in 0..rows {
            // Precompute occupancy for the row including removed cells
            let mut circle_here = vec![false; cols];
            let mut cross_here = vec![false; cols];
            let mut removed_here = vec![false; cols];
            for col in 0..cols {
                circle_here[col] = circles.iter().any(|&(r, c)| r == row && c == col);
                cross_here[col] = crosses.iter().any(|&(r, c)| r == row && c == col);
                removed_here[col] = removed.iter().any(|&(r, c)| r == row && c == col);
            }

            // Content line: draw cells and separators with conditional highlighting
            let mut content_spans: Vec<Span> = Vec::new();
            for col in 0..cols {
                // left padding
                content_spans.push(Span::raw(" "));

                // cell contents: circle, cross, cursor (only if empty and cell present), or empty/removed
                if circle_here[col] {
                    content_spans.push(Span::styled(
                        "o".to_string(),
                        Style::default().fg(Color::LightBlue),
                    ));
                } else if cross_here[col] {
                    content_spans.push(Span::styled(
                        "x".to_string(),
                        Style::default().fg(Color::Red),
                    ));
                } else if cursor.contains(&(row, col)) && !removed_here[col] {
                    content_spans.push(Span::styled(
                        "●",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    // empty or removed cell; display blank content
                    content_spans.push(Span::raw(" "));
                }

                // right padding
                content_spans.push(Span::raw(" "));

                // separator between cells (vertical).
                if col + 1 < cols {
                    let left_present = !removed_here[col];
                    let right_present = !removed_here[col + 1];
                    let left_cursor_marker = cursor.contains(&(row, col)) && (circle_here[col] || cross_here[col] || removed_here[col]);
                    let right_cursor_marker = cursor.contains(&(row, col + 1)) && (circle_here[col + 1] || cross_here[col + 1] || removed_here[col + 1]);

                    if left_present && right_present {
                        if left_cursor_marker || right_cursor_marker {
                            content_spans.push(Span::styled("│", Style::default().fg(Color::Yellow)));
                        } else {
                            content_spans.push(Span::raw("│"));
                        }
                    } else {
                        // draw separator only if a cursor is adjacent to the gap
                        if left_cursor_marker || right_cursor_marker {
                            content_spans.push(Span::styled("│", Style::default().fg(Color::Yellow)));
                        } else {
                            content_spans.push(Span::raw(" "));
                        }
                    }
                } else {
                    // trailing space for the last column
                    content_spans.push(Span::raw(" "));
                }
            }
            output.push(Spans::from(content_spans));

            // Border after row: draw horizontal; highlight when cursor is on a removed cell as well
            let mut border_spans: Vec<Span> = Vec::new();
            for col in 0..cols {
                // if the current cell is removed but has the cursor, show highlighted border
                if removed_here[col] && cursor.contains(&(row, col)) {
                    border_spans.push(Span::styled("─── ", Style::default().fg(Color::Yellow)));
                    continue;
                }

                let top_present = !removed_here[col];
                let bottom_present = if row + 1 < rows {
                    !removed.iter().any(|&(r, c)| r == row + 1 && c == col)
                } else {
                    true
                };

                if top_present && bottom_present {
                    let top_adjacent = cursor.contains(&(row, col)) && (circle_here[col] || cross_here[col]);
                    let bottom_adjacent = if row + 1 < rows {
                        let circle_below = circles.iter().any(|&(r, c)| r == row + 1 && c == col);
                        let cross_below = crosses.iter().any(|&(r, c)| r == row + 1 && c == col);
                        cursor.contains(&(row + 1, col)) && (circle_below || cross_below)
                    } else {
                        false
                    };
                    if top_adjacent || bottom_adjacent {
                        border_spans.push(Span::styled("─── ", Style::default().fg(Color::Yellow)));
                    } else {
                        border_spans.push(Span::raw("─── "));
                    }
                } else {
                    border_spans.push(Span::raw("    "));
                }
            }
            output.push(Spans::from(border_spans));
        }

        // blank separator between previews
        output.push(Spans::default());
    }

    output
}

fn increase_preview(size: &mut (usize, usize)) {
    const MAX_SIZE: usize = 12;
    if size.0 < MAX_SIZE {
        size.0 += 1;
    }
    if size.1 < MAX_SIZE {
        size.1 += 1;
    }
}

fn decrease_preview(
    size: &mut (usize, usize),
    circles: &mut Vec<(usize, usize)>,
    crosses: &mut Vec<(usize, usize)>,
    removed: &mut Vec<(usize, usize)>,
) {
    const MIN_SIZE: usize = 3;
    if size.0 > MIN_SIZE {
        size.0 -= 1;
    }
    if size.1 > MIN_SIZE {
        size.1 -= 1;
    }
    let (rows, cols) = *size;
    circles.retain(|&(r, c)| r < rows && c < cols);
    crosses.retain(|&(r, c)| r < rows && c < cols);
    removed.retain(|&(r, c)| r < rows && c < cols);
}
