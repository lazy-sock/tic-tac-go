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
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn puzzle_to_json(
    rows: usize,
    cols: usize,
    circles: &[(usize, usize)],
    crosses: &[(usize, usize)],
    removed: &[(usize, usize)],
    player: Option<(usize, usize)>,
    created_at: u64,
) -> String {
    // Build a serde_json object to avoid manual string concatenation bugs.
    let circles_json: Vec<serde_json::Value> = circles
        .iter()
        .map(|&(r, c)| serde_json::json!([r, c]))
        .collect();
    let crosses_json: Vec<serde_json::Value> = crosses
        .iter()
        .map(|&(r, c)| serde_json::json!([r, c]))
        .collect();
    let removed_json: Vec<serde_json::Value> = removed
        .iter()
        .map(|&(r, c)| serde_json::json!([r, c]))
        .collect();
    let player_json = if let Some((r, c)) = player {
        serde_json::json!([r, c])
    } else {
        serde_json::Value::Null
    };

    let obj = serde_json::json!({
        "rows": rows,
        "cols": cols,
        "created_at": created_at,
        "circles": circles_json,
        "crosses": crosses_json,
        "removed": removed_json,
        "player": player_json
    });

    serde_json::to_string(&obj).unwrap_or_default()
}

fn save_puzzle_to_file(json: &str, created_at: u64) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = PathBuf::from("puzzles");
    create_dir_all(&dir)?;
    let filename = format!("puzzle-{}.json", created_at);
    let path = dir.join(filename);
    let mut file = File::create(&path)?;
    file.write_all(json.as_bytes())?;
    Ok(path)
}

pub fn show_create_placeholder(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let mut preview = (5usize, 5usize);
    let mut cursor = vec![(0usize, 0usize)];
    let mut circles: Vec<(usize, usize)> = Vec::new();
    let mut crosses: Vec<(usize, usize)> = Vec::new();
    let mut removed: Vec<(usize, usize)> = Vec::new();
    // track a single player O (optional)
    let mut player: Option<(usize, usize)> = None;
    let mut error_msg: Option<String> = None;
    let mut success_msg: Option<String> = None;

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
                player,
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
                " Backspace on empty cell to delete. ",
            )));
            lines.push(Spans::from(Span::raw(" Space on empty cell to add. ")));
            lines.push(Spans::from(Span::raw(" Press R to restore all cells. ")));
            lines.push(Spans::from(Span::raw(" Press Enter to save puzzle. ")));
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

            // show error popup if set
            if let Some(err) = &error_msg {
                let ew = std::cmp::min(50, size.width.saturating_sub(10));
                let eh = 5u16;
                let ex = (size.width.saturating_sub(ew)) / 2;
                let ey = (size.height.saturating_sub(eh)) / 2;
                let earea = Rect::new(ex, ey, ew, eh);
                let mut err_lines: Vec<Spans> = Vec::new();
                err_lines.push(Spans::from(Span::styled(
                    " Error ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                err_lines.push(Spans::from(Span::raw("")));
                err_lines.push(Spans::from(Span::raw(err.as_str())));
                err_lines.push(Spans::from(Span::raw("")));
                err_lines.push(Spans::from(Span::raw("Press any key to continue")));
                let err_para = Paragraph::new(err_lines)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("Error"));
                f.render_widget(Clear, earea);
                f.render_widget(err_para, earea);
            }

            // show success popup if set
            if let Some(msg) = &success_msg {
                let ew = std::cmp::min(50, size.width.saturating_sub(10));
                let eh = 5u16;
                let ex = (size.width.saturating_sub(ew)) / 2;
                let ey = (size.height.saturating_sub(eh)) / 2;
                let earea = Rect::new(ex, ey, ew, eh);
                let mut ok_lines: Vec<Spans> = Vec::new();
                ok_lines.push(Spans::from(Span::styled(
                    " Saved ",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )));
                ok_lines.push(Spans::from(Span::raw("")));
                ok_lines.push(Spans::from(Span::raw(msg.as_str())));
                ok_lines.push(Spans::from(Span::raw("")));
                ok_lines.push(Spans::from(Span::raw("Press any key to return to home screen")));
                let ok_para = Paragraph::new(ok_lines)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("Saved"));
                f.render_widget(Clear, earea);
                f.render_widget(ok_para, earea);
            }
        })?;

        if event::poll(Duration::from_millis(150))?
            && let Event::Key(key) = event::read()?
        {
            if error_msg.is_some() {
                // clear error popup on any key press
                error_msg = None;
            } else if success_msg.is_some() {
                // after successful save, any key returns to home screen
                return Ok(());
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('o')
                    | KeyCode::Char('O')
                    | KeyCode::Char('x')
                    | KeyCode::Char('X')
                    | KeyCode::Backspace => {
                        edit_cell(key.code, &cursor, &mut circles, &mut crosses, &mut removed, &mut player)
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        // Increase matrix size (append to bottom/right)
                        increase_preview(&mut preview);
                    }
                    KeyCode::Char('-') => {
                        // Decrease matrix size and drop any marks that fall outside
                        decrease_preview(&mut preview, &mut circles, &mut crosses, &mut removed, &mut player);
                        // Ensure cursor remains within bounds
                        if let Some(pos) = cursor.get_mut(0) {
                            pos.0 = std::cmp::min(pos.0, preview.0.saturating_sub(1));
                            pos.1 = std::cmp::min(pos.1, preview.1.saturating_sub(1));
                        }
                    }
                    KeyCode::Char(' ') => {
                        // Restore the single removed cell under the cursor (if any)
                        if let Some(&pos) = cursor.get(0) {
                            if let Some(idx) = removed.iter().position(|&p| p == pos) {
                                removed.remove(idx);
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // Validate circle count before saving
                        if circles.len() != 3 {
                            error_msg = Some(format!("Puzzle must contain exactly 3 circles; found {}.", circles.len()));
                        } else {
                            // Serialize and save puzzle as JSON
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let json =
                                puzzle_to_json(preview.0, preview.1, &circles, &crosses, &removed, player, now);
                            match save_puzzle_to_file(&json, now) {
                                Ok(path) => {
                                    success_msg = Some(format!("Saved puzzle to {}", path.display()));
                                }
                                Err(e) => {
                                    error_msg = Some(format!("Failed to save puzzle: {}", e));
                                }
                            }
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
}

fn edit_cell(
    key: KeyCode,
    cursor: &[(usize, usize)],
    circles: &mut Vec<(usize, usize)>,
    crosses: &mut Vec<(usize, usize)>,
    removed: &mut Vec<(usize, usize)>,
    player: &mut Option<(usize, usize)>,
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
            // remove cross if present
            if let Some(idx) = crosses.iter().position(|&p| p == pos) {
                crosses.remove(idx);
            }
            // if there's already a circle at this position, mark it as the player
            if circles.contains(&pos) {
                *player = Some(pos);
                return;
            }
            // add circle if missing, but enforce a maximum of 3
            if !circles.contains(&pos) {
                if circles.len() >= 3 {
                    // limit reached; do not add another circle
                    return;
                }
                circles.push(pos);
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if removed.contains(&pos) {
                return;
            }
            if let Some(idx) = circles.iter().position(|&p| p == pos) {
                circles.remove(idx);
                // if the removed circle was the player, clear player
                if player.as_ref().map(|p| *p == pos).unwrap_or(false) {
                    *player = None;
                }
            }
            if !crosses.contains(&pos) {
                crosses.push(pos);
            }
        }
        KeyCode::Backspace => {
            if let Some(idx) = circles.iter().position(|&p| p == pos) {
                circles.remove(idx);
                if player.as_ref().map(|p| *p == pos).unwrap_or(false) {
                    *player = None;
                }
            } else if let Some(idx) = crosses.iter().position(|&p| p == pos) {
                crosses.remove(idx);
            } else if !removed.contains(&pos) {
                // delete the empty cell
                removed.push(pos);
                // if player was on this cell, clear it
                if player.as_ref().map(|p| *p == pos).unwrap_or(false) {
                    *player = None;
                }
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
    player: Option<(usize, usize)>,
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
                    let is_player = player.map(|p| p == (row, col)).unwrap_or(false);
                    if is_player {
                        content_spans.push(Span::styled(
                            "o".to_string(),
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        content_spans.push(Span::styled(
                            "o".to_string(),
                            Style::default().fg(Color::LightBlue),
                        ));
                    }
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
                    let left_cursor_marker = cursor.contains(&(row, col))
                        && (circle_here[col] || cross_here[col] || removed_here[col]);
                    let right_cursor_marker = cursor.contains(&(row, col + 1))
                        && (circle_here[col + 1] || cross_here[col + 1] || removed_here[col + 1]);

                    if left_present && right_present {
                        if left_cursor_marker || right_cursor_marker {
                            content_spans
                                .push(Span::styled("│", Style::default().fg(Color::Yellow)));
                        } else {
                            content_spans.push(Span::raw("│"));
                        }
                    } else {
                        // draw separator only if a cursor is adjacent to the gap
                        if left_cursor_marker || right_cursor_marker {
                            content_spans
                                .push(Span::styled("│", Style::default().fg(Color::Yellow)));
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
                let top_removed = removed_here[col];
                let top_filled = circle_here[col] || cross_here[col];
                let top_cursor = cursor.contains(&(row, col));

                let bottom_removed = if row + 1 < rows {
                    removed.iter().any(|&(r, c)| r == row + 1 && c == col)
                } else {
                    false
                };
                let bottom_filled = if row + 1 < rows {
                    circles.iter().any(|&(r, c)| r == row + 1 && c == col)
                        || crosses.iter().any(|&(r, c)| r == row + 1 && c == col)
                } else {
                    false
                };
                let bottom_cursor = if row + 1 < rows {
                    cursor.contains(&(row + 1, col))
                } else {
                    false
                };

                // If either adjacent removed cell has the cursor, highlight the border
                if (top_removed && top_cursor) || (bottom_removed && bottom_cursor) {
                    border_spans.push(Span::styled("─── ", Style::default().fg(Color::Yellow)));
                    continue;
                }

                let top_present = !top_removed;
                let bottom_present = if row + 1 < rows {
                    !bottom_removed
                } else {
                    true
                };

                if top_present && bottom_present {
                    // highlight if cursor is adjacent to a filled cell
                    if (top_cursor && top_filled) || (bottom_cursor && bottom_filled) {
                        border_spans.push(Span::styled("─── ", Style::default().fg(Color::Yellow)));
                    } else {
                        border_spans.push(Span::raw("─── "));
                    }
                } else {
                    // gap: highlight if cursor is adjacent at all
                    if top_cursor || bottom_cursor {
                        border_spans.push(Span::styled("─── ", Style::default().fg(Color::Yellow)));
                    } else {
                        border_spans.push(Span::raw("    "));
                    }
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
    player: &mut Option<(usize, usize)>,
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
    if let Some((r, c)) = *player {
        if r >= rows || c >= cols {
            *player = None;
        }
    }
}
