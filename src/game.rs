use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};

use crate::board::Board;
use crate::generator;
use crate::movement;
use crate::rules::{check_lose_flat, is_win_flat};

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    difficulty: generator::Difficulty,
) -> Result<(), Box<dyn Error>> {
    // Create board and helpers
    let board = Board::random();
    let rows = board.rows;
    let cols = board.cols;
    let row_widths = &board.row_widths;
    let to_flat = |r: usize, c: usize| board.to_flat(r, c);
    let from_flat = |idx: usize| board.from_flat(idx);
    let default_grid_w = board.default_grid_w;
    let default_grid_h = board.default_grid_h;

    // Generate puzzle
    let (mut circles_flat, mut crosses_flat, mut player_idx) =
        generator::generate_puzzle(&board, difficulty);

    // fallback deterministic layout if generation failed
    if circles_flat.is_empty() {
        let center_row = rows / 2;
        let c2 = std::cmp::min(2, row_widths[center_row].saturating_sub(1));
        let c3 = std::cmp::min(3, row_widths[center_row].saturating_sub(1));
        let c4 = std::cmp::min(4, row_widths[center_row].saturating_sub(1));
        circles_flat = vec![
            to_flat(center_row, c2),
            to_flat(center_row, c3),
            to_flat(center_row, c4),
        ];
        player_idx = 1;
        crosses_flat = Vec::new();
        'outer: for r in 0..rows {
            for c in 0..row_widths[r] {
                let f = to_flat(r, c);
                if circles_flat.contains(&f) {
                    continue;
                }
                crosses_flat.push(f);
                if crosses_flat.len() >= 5 {
                    break 'outer;
                }
            }
        }
    }

    // convert flat positions to (r,c)
    let mut circles: Vec<(usize, usize)> = circles_flat.iter().map(|&f| from_flat(f)).collect();
    let mut crosses: Vec<(usize, usize)> = crosses_flat.iter().map(|&f| from_flat(f)).collect();

    // initial win/lose checks
    let mut circles_flat_now: Vec<usize> = circles.iter().map(|&(r, c)| to_flat(r, c)).collect();
    let mut crosses_flat_now: Vec<usize> = crosses.iter().map(|&(r, c)| to_flat(r, c)).collect();
    let mut won = is_win_flat(&circles_flat_now, &board);
    let mut lost = check_lose_flat(&crosses_flat_now, &board);

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // ensure grid fits terminal
            let grid_w = if default_grid_w + 2 > size.width {
                size.width.saturating_sub(2)
            } else {
                default_grid_w
            };
            let grid_h = if default_grid_h + 2 > size.height {
                size.height.saturating_sub(2)
            } else {
                default_grid_h
            };

            let x = (size.width.saturating_sub(grid_w)) / 2;
            let y = (size.height.saturating_sub(grid_h)) / 2;
            let area = Rect::new(x, y, grid_w, grid_h);

            let mut lines: Vec<Spans> = Vec::new();

            // Top border (based on max cols)
            let mut top = String::new();
            top.push('┌');
            for col in 0..cols {
                top.push_str("───");
                if col != cols - 1 {
                    top.push('┬');
                } else {
                    top.push('┐');
                }
            }
            lines.push(Spans::from(Span::raw(top)));

            for row in 0..rows {
                // Content line with optional circles or crosses
                let mut span_line: Vec<Span> = Vec::new();
                span_line.push(Span::raw("│"));
                for col in 0..cols {
                    if col < row_widths[row] {
                        if let Some(idx) =
                            circles.iter().position(|&(rr, cc)| rr == row && cc == col)
                        {
                            let is_player = idx == player_idx;
                            let symbol = "o";
                            let style = if is_player {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::LightBlue)
                            };
                            span_line.push(Span::raw(" "));
                            span_line.push(Span::styled(symbol.to_string(), style));
                            span_line.push(Span::raw(" │"));
                            continue;
                        }
                        if let Some(_) = crosses.iter().position(|&(rr, cc)| rr == row && cc == col)
                        {
                            let style = Style::default().fg(Color::Red);
                            span_line.push(Span::raw(" "));
                            span_line.push(Span::styled("x".to_string(), style));
                            span_line.push(Span::raw(" │"));
                            continue;
                        }
                        span_line.push(Span::raw("   │"));
                    } else {
                        // absent cell at edge: render empty space
                        span_line.push(Span::raw("   │"));
                    }
                }
                lines.push(Spans::from(span_line));

                // Middle border or bottom
                if row != rows - 1 {
                    let mut mid = String::new();
                    mid.push('├');
                    for col in 0..cols {
                        mid.push_str("───");
                        if col != cols - 1 {
                            mid.push('┼');
                        } else {
                            mid.push('┤');
                        }
                    }
                    lines.push(Spans::from(Span::raw(mid)));
                } else {
                    let mut bot = String::new();
                    bot.push('└');
                    for col in 0..cols {
                        bot.push_str("───");
                        if col != cols - 1 {
                            bot.push('┴');
                        } else {
                            bot.push('┘');
                        }
                    }
                    lines.push(Spans::from(Span::raw(bot)));
                }
            }

            let paragraph = Paragraph::new(lines).block(Block::default());
            f.render_widget(paragraph, area);

            // Render difficulty centered under the board
            let diff_label = match difficulty {
                generator::Difficulty::Easy => "Easy",
                generator::Difficulty::Medium => "Medium",
                generator::Difficulty::Hard => "Hard",
            };
            let diff_text = format!("Difficulty: {}", diff_label);
            let diff_lines = vec![Spans::from(Span::styled(
                diff_text,
                Style::default().fg(Color::White),
            ))];
            let diff_y = y.saturating_add(grid_h);
            if diff_y < size.height {
                let diff_area = Rect::new(x, diff_y, grid_w, 1);
                let diff_para = Paragraph::new(diff_lines).alignment(Alignment::Center);
                f.render_widget(diff_para, diff_area);
            }

            // If won, render an overlay message centered on screen
            if won {
                let overlay_w = std::cmp::min(36, size.width.saturating_sub(4));
                let overlay_h = 5u16;
                let ox = (size.width.saturating_sub(overlay_w)) / 2;
                let oy = (size.height.saturating_sub(overlay_h)) / 2;
                let o_area = Rect::new(ox, oy, overlay_w, overlay_h);

                let mut msg_lines: Vec<Spans> = Vec::new();
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(
                    " YOU WON! ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(
                    "press q to quit",
                    Style::default().fg(Color::White).bg(Color::Black),
                )));

                let overlay = Paragraph::new(msg_lines)
                    .style(Style::default().bg(Color::Black))
                    .block(Block::default().borders(Borders::ALL).title("Victory").style(Style::default().bg(Color::Black)));
                f.render_widget(Clear, o_area);
                f.render_widget(Block::default().style(Style::default().bg(Color::Black)), o_area);
                f.render_widget(overlay, o_area);
            }

            // If lost, render an overlay message centered on screen
            if lost {
                let overlay_w = std::cmp::min(36, size.width.saturating_sub(4));
                let overlay_h = 5u16;
                let ox = (size.width.saturating_sub(overlay_w)) / 2;
                let oy = (size.height.saturating_sub(overlay_h)) / 2;
                let o_area = Rect::new(ox, oy, overlay_w, overlay_h);

                let mut msg_lines: Vec<Spans> = Vec::new();
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(
                    " YOU LOST! three crosses aligned ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )));
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(
                    "press q to quit",
                    Style::default().fg(Color::White).bg(Color::Black),
                )));

                let overlay = Paragraph::new(msg_lines)
                    .style(Style::default().bg(Color::Black))
                    .block(Block::default().borders(Borders::ALL).title("Defeat").style(Style::default().bg(Color::Black)));
                f.render_widget(Clear, o_area);
                f.render_widget(Block::default().style(Style::default().bg(Color::Black)), o_area);
                f.render_widget(overlay, o_area);
            }
        })?;

        // Input handling: arrows and WASD. movement blocked by walls and other objects
        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => match c.to_ascii_lowercase() {
                        'q' => break,
                        'w' => {
                            if !won && !lost {
                                movement::attempt_move_runtime(
                                    &mut circles,
                                    &mut crosses,
                                    player_idx,
                                    -1,
                                    0,
                                    &board,
                                )
                            }
                        }
                        'a' => {
                            if !won && !lost {
                                movement::attempt_move_runtime(
                                    &mut circles,
                                    &mut crosses,
                                    player_idx,
                                    0,
                                    -1,
                                    &board,
                                )
                            }
                        }
                        's' => {
                            if !won && !lost {
                                movement::attempt_move_runtime(
                                    &mut circles,
                                    &mut crosses,
                                    player_idx,
                                    1,
                                    0,
                                    &board,
                                )
                            }
                        }
                        'd' => {
                            if !won && !lost {
                                movement::attempt_move_runtime(
                                    &mut circles,
                                    &mut crosses,
                                    player_idx,
                                    0,
                                    1,
                                    &board,
                                )
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Up => {
                        if !won && !lost {
                            movement::attempt_move_runtime(
                                &mut circles,
                                &mut crosses,
                                player_idx,
                                -1,
                                0,
                                &board,
                            )
                        }
                    }
                    KeyCode::Left => {
                        if !won && !lost {
                            movement::attempt_move_runtime(
                                &mut circles,
                                &mut crosses,
                                player_idx,
                                0,
                                -1,
                                &board,
                            )
                        }
                    }
                    KeyCode::Down => {
                        if !won && !lost {
                            movement::attempt_move_runtime(
                                &mut circles,
                                &mut crosses,
                                player_idx,
                                1,
                                0,
                                &board,
                            )
                        }
                    }
                    KeyCode::Right => {
                        if !won && !lost {
                            movement::attempt_move_runtime(
                                &mut circles,
                                &mut crosses,
                                player_idx,
                                0,
                                1,
                                &board,
                            )
                        }
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
            // re-evaluate win/lose state after handling input
            circles_flat_now = circles.iter().map(|&(r, c)| to_flat(r, c)).collect();
            crosses_flat_now = crosses.iter().map(|&(r, c)| to_flat(r, c)).collect();
            won = is_win_flat(&circles_flat_now, &board);
            lost = check_lose_flat(&crosses_flat_now, &board);
        }
    }

    Ok(())
}
