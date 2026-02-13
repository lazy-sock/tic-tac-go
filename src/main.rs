use std::error::Error;
use std::io::{self, Stdout};
use std::time::Duration;
use std::collections::{HashSet, VecDeque};

use rand::{thread_rng, Rng};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Paragraph, Borders};
use ratatui::Terminal;
use ratatui::style::{Color, Style, Modifier};

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
    // Grid size
    let n: usize = 7;
    let default_grid_w: u16 = (4 * n + 1) as u16; // 4*n + 1 characters wide
    let default_grid_h: u16 = (2 * n + 1) as u16; // 2*n + 1 lines tall

    // Helper: determine reachability of a win state via BFS considering push rules
    fn reachable_win(initial_positions: &[(usize, usize)], player_idx: usize, n: usize) -> bool {
        use std::collections::VecDeque;

        // small helper to check win (three contiguous in same row or same column)
        fn is_win(pos: &[(usize, usize)]) -> bool {
            if pos.len() < 3 { return false; }
            // row
            if pos.iter().all(|&(r, _)| r == pos[0].0) {
                let mut cols: Vec<usize> = pos.iter().map(|&(_, c)| c).collect();
                cols.sort_unstable();
                if cols[1] == cols[0] + 1 && cols[2] == cols[1] + 1 { return true; }
            }
            // column
            if pos.iter().all(|&(_, c)| c == pos[0].1) {
                let mut rows: Vec<usize> = pos.iter().map(|&(r, _)| r).collect();
                rows.sort_unstable();
                if rows[1] == rows[0] + 1 && rows[2] == rows[1] + 1 { return true; }
            }
            false
        }

        let mut q: VecDeque<((usize, usize), [(usize, usize); 2])> = VecDeque::new();
        let mut visited: HashSet<u32> = HashSet::new();

        let p0 = initial_positions[player_idx];
        let mut others = [initial_positions[(player_idx + 1) % 3], initial_positions[(player_idx + 2) % 3]];
        // canonicalize others order
        let mut o0 = others[0].0 * n + others[0].1;
        let mut o1 = others[1].0 * n + others[1].1;
        if o0 > o1 { others.swap(0, 1); o0 = others[0].0 * n + others[0].1; o1 = others[1].0 * n + others[1].1; }

        // encode state into u32: p(8 bits) | o0(8 bits) | o1(8 bits)
        let encode = |p: (usize, usize), o: [(usize, usize); 2]| -> u32 {
            let p_i = (p.0 * n + p.1) as u32;
            let o0_i = (o[0].0 * n + o[0].1) as u32;
            let o1_i = (o[1].0 * n + o[1].1) as u32;
            (p_i << 16) | (o0_i << 8) | o1_i
        };

        let key = encode(p0, others);
        visited.insert(key);
        q.push_back((p0, others));

        while let Some((p, o)) = q.pop_front() {
            let positions_vec = vec![p, o[0], o[1]];
            if is_win(&positions_vec) { return true; }

            for (dr, dc) in [(-1,0),(1,0),(0,-1),(0,1)] {
                let new_r_i = p.0 as isize + dr;
                let new_c_i = p.1 as isize + dc;
                if new_r_i < 0 || new_c_i < 0 || new_r_i >= n as isize || new_c_i >= n as isize { continue; }
                let new_r = new_r_i as usize;
                let new_c = new_c_i as usize;

                // check if target is one of the others
                if (o[0].0 == new_r && o[0].1 == new_c) || (o[1].0 == new_r && o[1].1 == new_c) {
                    let other_idx = if o[0].0 == new_r && o[0].1 == new_c { 0 } else { 1 };
                    let push_r_i = new_r_i + dr;
                    let push_c_i = new_c_i + dc;
                    if push_r_i < 0 || push_c_i < 0 || push_r_i >= n as isize || push_c_i >= n as isize { continue; }
                    let push_r = push_r_i as usize;
                    let push_c = push_c_i as usize;
                    // cannot push into the other circle
                    if (o[0].0 == push_r && o[0].1 == push_c) || (o[1].0 == push_r && o[1].1 == push_c) { continue; }
                    // create new state with pushed circle
                    let mut new_others = o;
                    new_others[other_idx] = (push_r, push_c);
                    // canonicalize
                    let mut new_o0 = new_others[0].0 * n + new_others[0].1;
                    let mut new_o1 = new_others[1].0 * n + new_others[1].1;
                    if new_o0 > new_o1 { new_others.swap(0,1); new_o0 = new_others[0].0 * n + new_others[0].1; new_o1 = new_others[1].0 * n + new_others[1].1; }
                    let new_key = encode((new_r, new_c), new_others);
                    if !visited.contains(&new_key) {
                        visited.insert(new_key);
                        q.push_back(((new_r, new_c), new_others));
                    }
                } else {
                    // empty target; move player
                    let new_others = o;
                    let mut new_o0 = new_others[0].0 * n + new_others[0].1;
                    let mut new_o1 = new_others[1].0 * n + new_others[1].1;
                    if new_o0 > new_o1 { let mut no = new_others; no.swap(0,1); let new_key = encode((new_r, new_c), no); if !visited.contains(&new_key) { visited.insert(new_key); q.push_back(((new_r, new_c), no)); } } else { let new_key = encode((new_r, new_c), new_others); if !visited.contains(&new_key) { visited.insert(new_key); q.push_back(((new_r, new_c), new_others)); } }
                }
            }
        }
        false
    }

    // Generate three distinct positions but ensure they are winnable
    let mut rng = thread_rng();
    let mut attempts = 0;
    let max_attempts = 2000;
    let mut positions: Vec<(usize, usize)>;
    let mut player_idx: usize;
    loop {
        attempts += 1;
        let mut occupied = HashSet::new();
        positions = Vec::new();
        while positions.len() < 3 {
            let r = rng.gen_range(0..n);
            let c = rng.gen_range(0..n);
            if occupied.insert((r, c)) {
                positions.push((r, c));
            }
        }
        player_idx = rng.gen_range(0..positions.len());
        if reachable_win(&positions, player_idx, n) {
            break;
        }
        if attempts >= max_attempts {
            // deterministic fallback: place on center row contiguous
            positions.clear();
            let center_row = n / 2;
            positions.push((center_row, 2));
            positions.push((center_row, 3));
            positions.push((center_row, 4));
            player_idx = 1; // middle as player
            break;
        }
    }
    fn check_win(positions: &[(usize, usize)]) -> bool {
        if positions.len() < 3 {
            return false;
        }
        // row win
        if positions.iter().all(|&(r, _)| r == positions[0].0) {
            let mut cols: Vec<usize> = positions.iter().map(|&(_, c)| c).collect();
            cols.sort_unstable();
            if cols[1] == cols[0] + 1 && cols[2] == cols[1] + 1 {
                return true;
            }
        }
        // column win
        if positions.iter().all(|&(_, c)| c == positions[0].1) {
            let mut rows: Vec<usize> = positions.iter().map(|&(r, _)| r).collect();
            rows.sort_unstable();
            if rows[1] == rows[0] + 1 && rows[2] == rows[1] + 1 {
                return true;
            }
        }
        false
    }

    // Helper to attempt movement: dr/dc are -1/0/1
    // This handles pushing a single adjacent circle if the cell beyond it is free.
    fn attempt_move(positions: &mut Vec<(usize, usize)>, player_idx: usize, dr: isize, dc: isize, n: usize) {
        let (r, c) = positions[player_idx];
        let new_r_i = r as isize + dr;
        let new_c_i = c as isize + dc;
        // check bounds for the player's move
        if new_r_i < 0 || new_c_i < 0 || new_r_i >= n as isize || new_c_i >= n as isize {
            return;
        }
        let new_r = new_r_i as usize;
        let new_c = new_c_i as usize;

        // If target cell is empty, simply move the player
        if !positions.iter().any(|&(rr, cc)| rr == new_r && cc == new_c) {
            positions[player_idx] = (new_r, new_c);
            return;
        }

        // Otherwise, there's a circle to push. Find its index.
        if let Some(other_idx) = positions.iter().position(|&(rr, cc)| rr == new_r && cc == new_c) {
            // compute push target for the other circle
            let push_r_i = new_r_i + dr;
            let push_c_i = new_c_i + dc;
            // cannot push out of bounds
            if push_r_i < 0 || push_c_i < 0 || push_r_i >= n as isize || push_c_i >= n as isize {
                return;
            }
            let push_r = push_r_i as usize;
            let push_c = push_c_i as usize;
            // if the push target is occupied, refuse the move (can't push two circles)
            if positions.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) {
                return;
            }
            // perform the push: move the other circle, then move the player
            positions[other_idx] = (push_r, push_c);
            positions[player_idx] = (new_r, new_c);
        }
    }

    // initial win check
    let mut won = check_win(&positions);

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // ensure grid fits terminal
            let grid_w = if default_grid_w + 2 > size.width { size.width.saturating_sub(2) } else { default_grid_w };
            let grid_h = if default_grid_h + 2 > size.height { size.height.saturating_sub(2) } else { default_grid_h };

            let x = (size.width.saturating_sub(grid_w)) / 2;
            let y = (size.height.saturating_sub(grid_h)) / 2;
            let area = Rect::new(x, y, grid_w, grid_h);

            let mut lines: Vec<Spans> = Vec::new();

            // Top border
            let mut top = String::new();
            top.push('┌');
            for col in 0..n {
                top.push_str("───");
                if col != n - 1 {
                    top.push('┬');
                } else {
                    top.push('┐');
                }
            }
            lines.push(Spans::from(Span::raw(top)));

            for row in 0..n {
                // Content line with optional circles
                let mut span_line: Vec<Span> = Vec::new();
                span_line.push(Span::raw("│"));
                for col in 0..n {
                    if let Some(idx) = positions.iter().position(|&(rr, cc)| rr == row && cc == col) {
                        let is_player = idx == player_idx;
                        let symbol = if is_player { "●" } else { "○" };
                        let style = if is_player { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::LightBlue) };
                        span_line.push(Span::raw(" "));
                        span_line.push(Span::styled(symbol.to_string(), style));
                        span_line.push(Span::raw(" │"));
                    } else {
                        span_line.push(Span::raw("   │"));
                    }
                }
                lines.push(Spans::from(span_line));

                // Middle border or bottom
                if row != n - 1 {
                    let mut mid = String::new();
                    mid.push('├');
                    for col in 0..n {
                        mid.push_str("───");
                        if col != n - 1 {
                            mid.push('┼');
                        } else {
                            mid.push('┤');
                        }
                    }
                    lines.push(Spans::from(Span::raw(mid)));
                } else {
                    let mut bot = String::new();
                    bot.push('└');
                    for col in 0..n {
                        bot.push_str("───");
                        if col != n - 1 {
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

            // If won, render an overlay message centered on screen
            if won {
                let overlay_w = std::cmp::min(30, size.width.saturating_sub(4));
                let overlay_h = 5u16;
                let ox = (size.width.saturating_sub(overlay_w)) / 2;
                let oy = (size.height.saturating_sub(overlay_h)) / 2;
                let o_area = Rect::new(ox, oy, overlay_w, overlay_h);

                let mut msg_lines: Vec<Spans> = Vec::new();
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(" YOU WON! ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD))));
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled("press q to quit", Style::default().fg(Color::White))));

                let overlay = Paragraph::new(msg_lines).block(Block::default().borders(Borders::ALL).title("Victory"));
                f.render_widget(overlay, o_area);
            }
        })?;

        // Input handling: arrows and WASD. movement blocked by walls and other circles
        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => match c.to_ascii_lowercase() {
                        'q' => break,
                        'w' => if !won { attempt_move(&mut positions, player_idx, -1, 0, n) },
                        'a' => if !won { attempt_move(&mut positions, player_idx, 0, -1, n) },
                        's' => if !won { attempt_move(&mut positions, player_idx, 1, 0, n) },
                        'd' => if !won { attempt_move(&mut positions, player_idx, 0, 1, n) },
                        _ => {}
                    },
                    KeyCode::Up => if !won { attempt_move(&mut positions, player_idx, -1, 0, n) },
                    KeyCode::Left => if !won { attempt_move(&mut positions, player_idx, 0, -1, n) },
                    KeyCode::Down => if !won { attempt_move(&mut positions, player_idx, 1, 0, n) },
                    KeyCode::Right => if !won { attempt_move(&mut positions, player_idx, 0, 1, n) },
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
            // re-evaluate win state after handling input
            won = check_win(&positions);
        }
    }
    Ok(())
}
