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

    // Helpers for flat indices
    let to_flat = |r: usize, c: usize| -> usize { r * n + c };
    let from_flat = |idx: usize| -> (usize, usize) { (idx / n, idx % n) };

    // Check win for circles (three contiguous in row or column)
    fn is_win_flat(positions: &[usize], n: usize) -> bool {
        if positions.len() < 3 { return false; }
        // same row?
        let r0 = positions[0] / n;
        if positions.iter().all(|&p| p / n == r0) {
            let mut cols: Vec<usize> = positions.iter().map(|&p| p % n).collect();
            cols.sort_unstable();
            if cols[1] == cols[0] + 1 && cols[2] == cols[1] + 1 { return true; }
        }
        // same column?
        let c0 = positions[0] % n;
        if positions.iter().all(|&p| p % n == c0) {
            let mut rows: Vec<usize> = positions.iter().map(|&p| p / n).collect();
            rows.sort_unstable();
            if rows[1] == rows[0] + 1 && rows[2] == rows[1] + 1 { return true; }
        }
        false
    }

    // Check lose condition: any three crosses contiguous in a row or column
    fn check_lose_flat(crosses: &[usize], n: usize) -> bool {
        if crosses.len() < 3 { return false; }
        // rows
        for r in 0..n {
            let mut cols: Vec<usize> = crosses.iter().filter(|&&p| p / n == r).map(|&p| p % n).collect();
            if cols.len() < 3 { continue; }
            cols.sort_unstable();
            for i in 0..cols.len()-2 {
                if cols[i+1] == cols[i] + 1 && cols[i+2] == cols[i+1] + 1 { return true; }
            }
        }
        // columns
        for c in 0..n {
            let mut rows: Vec<usize> = crosses.iter().filter(|&&p| p % n == c).map(|&p| p / n).collect();
            if rows.len() < 3 { continue; }
            rows.sort_unstable();
            for i in 0..rows.len()-2 {
                if rows[i+1] == rows[i] + 1 && rows[i+2] == rows[i+1] + 1 { return true; }
            }
        }
        false
    }

    // BFS reachability: can circles reach a win without ever creating a losing cross-line?
    let reachable_win = |circles_flat: &[usize], player_idx: usize, crosses_flat: &[usize], n: usize| -> bool {
        // state: (player_pos, other_circles[2], crosses_vec)
        let mut q: VecDeque<(usize, [usize;2], Vec<usize>)> = VecDeque::new();
        let mut visited: HashSet<Vec<u8>> = HashSet::new();
        let p0 = circles_flat[player_idx];
        let mut others = [circles_flat[(player_idx + 1) % 3], circles_flat[(player_idx + 2) % 3]];
        if others[0] > others[1] { others.swap(0,1); }
        let mut crosses = crosses_flat.to_vec();
        crosses.sort_unstable();

        let encode = |p: usize, o: &[usize;2], x: &Vec<usize>| -> Vec<u8> {
            let mut key = Vec::with_capacity(3 + x.len());
            key.push(p as u8);
            key.push(o[0] as u8);
            key.push(o[1] as u8);
            for &xx in x { key.push(xx as u8); }
            key
        };

        let key0 = encode(p0, &others, &crosses);
        visited.insert(key0);
        q.push_back((p0, others, crosses.clone()));

        let mut nodes = 0usize;
        let max_nodes = 200_000usize;

        while let Some((p, o, x)) = q.pop_front() {
            nodes += 1;
            if nodes > max_nodes { return false; }
            let posv = vec![p, o[0], o[1]];
            if is_win_flat(&posv, n) { return true; }

            // generate moves
            for (dr, dc) in [(-1,0),(1,0),(0,-1),(0,1)] {
                let pr = p / n; let pc = p % n;
                let new_r_i = pr as isize + dr; let new_c_i = pc as isize + dc;
                if new_r_i < 0 || new_c_i < 0 || new_r_i >= n as isize || new_c_i >= n as isize { continue; }
                let new_r = new_r_i as usize; let new_c = new_c_i as usize;
                let p1 = to_flat(new_r, new_c);

                // check if occupied by other circle
                let mut occupied_by_circle: Option<usize> = None;
                if o[0] == p1 { occupied_by_circle = Some(0); }
                else if o[1] == p1 { occupied_by_circle = Some(1); }

                if let Some(other_idx) = occupied_by_circle {
                    // try push circle
                    let push_r_i = new_r_i + dr; let push_c_i = new_c_i + dc;
                    if push_r_i < 0 || push_c_i < 0 || push_r_i >= n as isize || push_c_i >= n as isize { continue; }
                    let push_r = push_r_i as usize; let push_c = push_c_i as usize; let p2 = to_flat(push_r, push_c);
                    // cannot push into another circle
                    if o[0] == p2 || o[1] == p2 { continue; }
                    // cannot push into a cross
                    if x.iter().any(|&xx| xx == p2) { continue; }
                    let mut new_o = o;
                    new_o[other_idx] = p2;
                    if new_o[0] > new_o[1] { new_o.swap(0,1); }
                    let k = encode(p1, &new_o, &x);
                    if visited.contains(&k) { continue; }
                    // crosses unchanged; check losing crosses (unchanged)
                    if check_lose_flat(&x, n) { continue; }
                    visited.insert(k);
                    q.push_back((p1, new_o, x.clone()));
                } else if let Some(cross_idx) = x.iter().position(|&xx| xx == p1) {
                    // try push cross
                    let push_r_i = new_r_i + dr; let push_c_i = new_c_i + dc;
                    if push_r_i < 0 || push_c_i < 0 || push_r_i >= n as isize || push_c_i >= n as isize { continue; }
                    let push_r = push_r_i as usize; let push_c = push_c_i as usize; let p2 = to_flat(push_r, push_c);
                    // cannot push into circle
                    if o[0] == p2 || o[1] == p2 || p == p2 { continue; }
                    // cannot push into another cross
                    if x.iter().any(|&xx| xx == p2) { continue; }
                    let mut new_x = x.clone();
                    new_x[cross_idx] = p2;
                    new_x.sort_unstable();
                    // if new crosses cause losing, skip
                    if check_lose_flat(&new_x, n) { continue; }
                    let k = encode(p1, &o, &new_x);
                    if visited.contains(&k) { continue; }
                    visited.insert(k);
                    q.push_back((p1, o, new_x));
                } else {
                    // empty target
                    let k = encode(p1, &o, &x);
                    if visited.contains(&k) { continue; }
                    // crosses unchanged; ensure not losing
                    if check_lose_flat(&x, n) { continue; }
                    visited.insert(k);
                    q.push_back((p1, o, x.clone()));
                }
            }
        }
        false
    };

    // Generate circles and crosses such that puzzle is solvable
    let mut rng = thread_rng();
    let mut attempts = 0usize;
    let max_attempts = 2000usize;

    let mut circles_flat: Vec<usize> = Vec::new();
    let mut crosses_flat: Vec<usize> = Vec::new();
    let mut player_idx: usize = 0;

    loop {
        attempts += 1;
        circles_flat.clear(); crosses_flat.clear();
        let mut occupied = HashSet::new();
        // select 3 unique positions for circles
        while circles_flat.len() < 3 {
            let r = rng.gen_range(0..n);
            let c = rng.gen_range(0..n);
            let f = to_flat(r,c);
            if occupied.insert(f) { circles_flat.push(f); }
        }
        // select crosses
        let cross_count = rng.gen_range(5..=10);
        while crosses_flat.len() < cross_count {
            let r = rng.gen_range(0..n);
            let c = rng.gen_range(0..n);
            let f = to_flat(r,c);
            if occupied.insert(f) { crosses_flat.push(f); }
        }
        // ensure crosses aren't immediately losing
        crosses_flat.sort_unstable();
        if check_lose_flat(&crosses_flat, n) { if attempts >= max_attempts { break } else { continue } }
        player_idx = rng.gen_range(0..3);
        if reachable_win(&circles_flat, player_idx, &crosses_flat, n) { break }
        if attempts >= max_attempts { break }
    }

    // fallback deterministic layout if generation failed
    if circles_flat.is_empty() {
        let center_row = n / 2;
        circles_flat = vec![to_flat(center_row,2), to_flat(center_row,3), to_flat(center_row,4)];
        player_idx = 1;
        crosses_flat = Vec::new();
        // scatter a few crosses that don't immediately lose
        for r in 0..n {
            for c in 0..n {
                let f = to_flat(r,c);
                if circles_flat.contains(&f) { continue; }
                if crosses_flat.len() >= 5 { break; }
                crosses_flat.push(f);
            }
            if crosses_flat.len() >= 5 { break; }
        }
    }

    // convert flat positions to (r,c)
    let mut circles: Vec<(usize, usize)> = circles_flat.iter().map(|&f| from_flat(f)).collect();
    let mut crosses: Vec<(usize, usize)> = crosses_flat.iter().map(|&f| from_flat(f)).collect();

    // Helper to attempt movement during runtime (allows pushing a single object: circle or cross)
    fn attempt_move_runtime(circles: &mut Vec<(usize, usize)>, crosses: &mut Vec<(usize, usize)>, player_idx: usize, dr: isize, dc: isize, n: usize) {
        let (r, c) = circles[player_idx];
        let new_r_i = r as isize + dr; let new_c_i = c as isize + dc;
        if new_r_i < 0 || new_c_i < 0 || new_r_i >= n as isize || new_c_i >= n as isize { return; }
        let new_r = new_r_i as usize; let new_c = new_c_i as usize;
        // occupied by circle?
        if let Some(idx) = circles.iter().position(|&(rr, cc)| rr == new_r && cc == new_c) {
            // try push circle
            let push_r_i = new_r_i + dr; let push_c_i = new_c_i + dc;
            if push_r_i < 0 || push_c_i < 0 || push_r_i >= n as isize || push_c_i >= n as isize { return; }
            let push_r = push_r_i as usize; let push_c = push_c_i as usize;
            if circles.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
            if crosses.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
            circles[idx] = (push_r, push_c);
            circles[player_idx] = (new_r, new_c);
            return;
        }
        // occupied by cross?
        if let Some(idx) = crosses.iter().position(|&(rr, cc)| rr == new_r && cc == new_c) {
            let push_r_i = new_r_i + dr; let push_c_i = new_c_i + dc;
            if push_r_i < 0 || push_c_i < 0 || push_r_i >= n as isize || push_c_i >= n as isize { return; }
            let push_r = push_r_i as usize; let push_c = push_c_i as usize;
            if circles.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
            if crosses.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
            crosses[idx] = (push_r, push_c);
            circles[player_idx] = (new_r, new_c);
            return;
        }
        // empty
        circles[player_idx] = (new_r, new_c);
    }

    // initial win/lose checks
    let mut circles_flat_now: Vec<usize> = circles.iter().map(|&(r,c)| to_flat(r,c)).collect();
    let mut crosses_flat_now: Vec<usize> = crosses.iter().map(|&(r,c)| to_flat(r,c)).collect();
    let mut won = is_win_flat(&circles_flat_now, n);
    let mut lost = check_lose_flat(&crosses_flat_now, n);

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
                // Content line with optional circles or crosses
                let mut span_line: Vec<Span> = Vec::new();
                span_line.push(Span::raw("│"));
                for col in 0..n {
                    if let Some(idx) = circles.iter().position(|&(rr, cc)| rr == row && cc == col) {
                        let is_player = idx == player_idx;
                        let symbol = "o";
                        let style = if is_player { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::LightBlue) };
                        span_line.push(Span::raw(" "));
                        span_line.push(Span::styled(symbol.to_string(), style));
                        span_line.push(Span::raw(" │"));
                    } else if let Some(_) = crosses.iter().position(|&(rr, cc)| rr == row && cc == col) {
                        let style = Style::default().fg(Color::Red);
                        span_line.push(Span::raw(" "));
                        span_line.push(Span::styled("x".to_string(), style));
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
                let overlay_w = std::cmp::min(36, size.width.saturating_sub(4));
                let overlay_h = 5u16;
                let ox = (size.width.saturating_sub(overlay_w)) / 2;
                let oy = (size.height.saturating_sub(overlay_h)) / 2;
                let o_area = Rect::new(ox, oy, overlay_w, overlay_h);

                let mut msg_lines: Vec<Spans> = Vec::new();
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(" 🎉 YOU WON! 🎉 ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD))));
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled("press q to quit", Style::default().fg(Color::White))));

                let overlay = Paragraph::new(msg_lines).block(Block::default().borders(Borders::ALL).title("Victory"));
                f.render_widget(overlay, o_area);
            }

            // If lost, render an overlay message centered on screen
            if lost {
                let overlay_w = std::cmp::min(36, size.width.saturating_sub(4));
                let overlay_h = 5u16;
                let ox = (size.width.saturating_sub(overlay_w)) / 2;
                let oy = (size.height.saturating_sub(overlay_h)) / 2 + 6;
                let o_area = Rect::new(ox, oy, overlay_w, overlay_h);

                let mut msg_lines: Vec<Spans> = Vec::new();
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(" YOU LOST! three crosses aligned ", Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD))));
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled("press q to quit", Style::default().fg(Color::White))));

                let overlay = Paragraph::new(msg_lines).block(Block::default().borders(Borders::ALL).title("Defeat"));
                f.render_widget(overlay, o_area);
            }
        })?;

        // Input handling: arrows and WASD. movement blocked by walls and other objects
        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => match c.to_ascii_lowercase() {
                        'q' => break,
                        'w' => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, -1, 0, n) },
                        'a' => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, -1, n) },
                        's' => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, 1, 0, n) },
                        'd' => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, 1, n) },
                        _ => {}
                    },
                    KeyCode::Up => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, -1, 0, n) },
                    KeyCode::Left => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, -1, n) },
                    KeyCode::Down => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, 1, 0, n) },
                    KeyCode::Right => if !won && !lost { attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, 1, n) },
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
            // re-evaluate win/lose state after handling input
            circles_flat_now = circles.iter().map(|&(r,c)| to_flat(r,c)).collect();
            crosses_flat_now = crosses.iter().map(|&(r,c)| to_flat(r,c)).collect();
            won = is_win_flat(&circles_flat_now, n);
            lost = check_lose_flat(&crosses_flat_now, n);
        }
    }
    Ok(())
}
