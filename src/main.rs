use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use rand::{Rng, thread_rng};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, Paragraph};

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
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    // Automatically choose a random rectangular board size (no CLI options). Ensure at least 20 slots.
    let mut rng = thread_rng();
    // choose rows between 3 and 8
    let rows: usize = rng.gen_range(3..=8);
    // choose columns so rows*cols >= 20, allow some variation
    let min_cols = (20 + rows - 1) / rows;
    let max_cols = min_cols + 8;
    let cols: usize = rng.gen_range(min_cols..=max_cols);

    // Generate slight edge variation: per-row widths (remove cells from right edge only, no holes)
    let mut rng = thread_rng();
    let mut row_widths = vec![cols; rows];
    let total_cells_initial = rows * cols;
    let max_removable = if total_cells_initial > 20 {
        total_cells_initial - 20
    } else {
        0
    };
    let mut removable = if max_removable > 0 {
        rng.gen_range(0..=max_removable)
    } else {
        0
    };
    // remove from last rows right side
    let mut idx = rows;
    while removable > 0 && idx > 0 {
        idx -= 1;
        let can_remove = row_widths[idx].saturating_sub(1);
        let r = std::cmp::min(can_remove, removable);
        row_widths[idx] = row_widths[idx].saturating_sub(r);
        removable -= r;
    }

    // row offsets for flat indexing
    let mut row_offsets = vec![0usize; rows];
    for i in 1..rows {
        row_offsets[i] = row_offsets[i - 1] + row_widths[i - 1];
    }
    let total_cells = row_offsets[rows - 1] + row_widths[rows - 1];

    let to_flat = |r: usize, c: usize| -> usize { row_offsets[r] + c };
    let from_flat = |mut idx: usize| -> (usize, usize) {
        let mut r = 0usize;
        while r < rows {
            let start = row_offsets[r];
            let w = row_widths[r];
            if idx < start + w {
                return (r, idx - start);
            }
            r += 1;
        }
        panic!("invalid flat index {}", idx);
    };

    let default_grid_w: u16 = (4 * cols + 1) as u16; // use cols for layout width
    let default_grid_h: u16 = (2 * rows + 1) as u16;

    // Check win for circles (three contiguous in row or column)
    let is_win_flat = |positions: &[usize]| -> bool {
        if positions.len() < 3 {
            return false;
        }
        use std::collections::HashMap;
        let mut by_row: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut by_col: HashMap<usize, Vec<usize>> = HashMap::new();
        for &p in positions {
            let (r, c) = from_flat(p);
            by_row.entry(r).or_default().push(c);
            by_col.entry(c).or_default().push(r);
        }
        for (_r, mut cols_vec) in by_row.into_iter() {
            if cols_vec.len() < 3 {
                continue;
            }
            cols_vec.sort_unstable();
            for i in 0..cols_vec.len().saturating_sub(2) {
                if cols_vec[i + 1] == cols_vec[i] + 1 && cols_vec[i + 2] == cols_vec[i + 1] + 1 {
                    return true;
                }
            }
        }
        for (_c, mut rows_vec) in by_col.into_iter() {
            if rows_vec.len() < 3 {
                continue;
            }
            rows_vec.sort_unstable();
            for i in 0..rows_vec.len().saturating_sub(2) {
                if rows_vec[i + 1] == rows_vec[i] + 1 && rows_vec[i + 2] == rows_vec[i + 1] + 1 {
                    return true;
                }
            }
        }
        false
    };

    // Check lose: any three crosses contiguous in row or column
    let check_lose_flat = |crosses: &[usize]| -> bool {
        if crosses.len() < 3 {
            return false;
        }
        use std::collections::HashMap;
        let mut by_row: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut by_col: HashMap<usize, Vec<usize>> = HashMap::new();
        for &p in crosses {
            let (r, c) = from_flat(p);
            by_row.entry(r).or_default().push(c);
            by_col.entry(c).or_default().push(r);
        }
        for (_r, mut cols_vec) in by_row.into_iter() {
            if cols_vec.len() < 3 {
                continue;
            }
            cols_vec.sort_unstable();
            for i in 0..cols_vec.len().saturating_sub(2) {
                if cols_vec[i + 1] == cols_vec[i] + 1 && cols_vec[i + 2] == cols_vec[i + 1] + 1 {
                    return true;
                }
            }
        }
        for (_c, mut rows_vec) in by_col.into_iter() {
            if rows_vec.len() < 3 {
                continue;
            }
            rows_vec.sort_unstable();
            for i in 0..rows_vec.len().saturating_sub(2) {
                if rows_vec[i + 1] == rows_vec[i] + 1 && rows_vec[i + 2] == rows_vec[i + 1] + 1 {
                    return true;
                }
            }
        }
        false
    };

    // BFS reachability: can circles reach a win without ever creating a losing cross-line?
    let reachable_win =
        |circles_flat: &[usize], player_idx: usize, crosses_flat: &[usize]| -> bool {
            use std::collections::VecDeque;
            // state: (player_pos, other_circles[2], crosses_vec)
            let mut q: VecDeque<(usize, [usize; 2], Vec<usize>)> = VecDeque::new();
            let mut visited: HashSet<Vec<u16>> = HashSet::new();
            let p0 = circles_flat[player_idx];
            let mut others = [
                circles_flat[(player_idx + 1) % 3],
                circles_flat[(player_idx + 2) % 3],
            ];
            if others[0] > others[1] {
                others.swap(0, 1);
            }
            let mut crosses = crosses_flat.to_vec();
            crosses.sort_unstable();

            let encode = |p: usize, o: &[usize; 2], x: &Vec<usize>| -> Vec<u16> {
                let mut key = Vec::with_capacity(3 + x.len());
                key.push(p as u16);
                key.push(o[0] as u16);
                key.push(o[1] as u16);
                for &xx in x {
                    key.push(xx as u16);
                }
                key
            };

            let key0 = encode(p0, &others, &crosses);
            visited.insert(key0);
            q.push_back((p0, others, crosses.clone()));

            let mut nodes = 0usize;
            let max_nodes = 200_000usize;

            while let Some((p, o, x)) = q.pop_front() {
                nodes += 1;
                if nodes > max_nodes {
                    return false;
                }
                let posv = vec![p, o[0], o[1]];
                if is_win_flat(&posv) {
                    return true;
                }

                // generate moves
                for (dr, dc) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let (pr, pc) = from_flat(p);
                    let new_r_i = pr as isize + dr;
                    let new_c_i = pc as isize + dc;
                    if new_r_i < 0 || new_c_i < 0 {
                        continue;
                    }
                    let new_r = new_r_i as usize;
                    let new_c = new_c_i as usize;
                    if new_r >= rows {
                        continue;
                    }
                    if new_c >= row_widths[new_r] {
                        continue;
                    }
                    let p1 = to_flat(new_r, new_c);

                    // check if occupied by other circle
                    let mut occupied_by_circle: Option<usize> = None;
                    if o[0] == p1 {
                        occupied_by_circle = Some(0);
                    } else if o[1] == p1 {
                        occupied_by_circle = Some(1);
                    }

                    if let Some(other_idx) = occupied_by_circle {
                        // try push circle
                        let push_r_i = new_r_i + dr;
                        let push_c_i = new_c_i + dc;
                        if push_r_i < 0 || push_c_i < 0 {
                            continue;
                        }
                        let push_r = push_r_i as usize;
                        let push_c = push_c_i as usize;
                        if push_r >= rows {
                            continue;
                        }
                        if push_c >= row_widths[push_r] {
                            continue;
                        }
                        let p2 = to_flat(push_r, push_c);
                        // cannot push into another circle
                        if o[0] == p2 || o[1] == p2 {
                            continue;
                        }
                        // cannot push into a cross
                        if x.iter().any(|&xx| xx == p2) {
                            continue;
                        }
                        let mut new_o = o;
                        new_o[other_idx] = p2;
                        if new_o[0] > new_o[1] {
                            new_o.swap(0, 1);
                        }
                        let k = encode(p1, &new_o, &x);
                        if visited.contains(&k) {
                            continue;
                        }
                        // crosses unchanged; check losing crosses (unchanged)
                        if check_lose_flat(&x) {
                            continue;
                        }
                        visited.insert(k);
                        q.push_back((p1, new_o, x.clone()));
                    } else if let Some(cross_idx) = x.iter().position(|&xx| xx == p1) {
                        // try push cross
                        let push_r_i = new_r_i + dr;
                        let push_c_i = new_c_i + dc;
                        if push_r_i < 0 || push_c_i < 0 {
                            continue;
                        }
                        let push_r = push_r_i as usize;
                        let push_c = push_c_i as usize;
                        if push_r >= rows {
                            continue;
                        }
                        if push_c >= row_widths[push_r] {
                            continue;
                        }
                        let p2 = to_flat(push_r, push_c);
                        // cannot push into circle
                        if o[0] == p2 || o[1] == p2 || p == p2 {
                            continue;
                        }
                        // cannot push into another cross
                        if x.iter().any(|&xx| xx == p2) {
                            continue;
                        }
                        let mut new_x = x.clone();
                        new_x[cross_idx] = p2;
                        new_x.sort_unstable();
                        // if new crosses cause losing, skip
                        if check_lose_flat(&new_x) {
                            continue;
                        }
                        let k = encode(p1, &o, &new_x);
                        if visited.contains(&k) {
                            continue;
                        }
                        visited.insert(k);
                        q.push_back((p1, o, new_x));
                    } else {
                        // empty target
                        let k = encode(p1, &o, &x);
                        if visited.contains(&k) {
                            continue;
                        }
                        // crosses unchanged; ensure not losing
                        if check_lose_flat(&x) {
                            continue;
                        }
                        visited.insert(k);
                        q.push_back((p1, o, x.clone()));
                    }
                }
            }
            false
        };

    // Generate circles and crosses such that puzzle is solvable
    let mut attempts = 0usize;
    let max_attempts = 2000usize;

    let mut circles_flat: Vec<usize> = Vec::new();
    let mut crosses_flat: Vec<usize> = Vec::new();
    let mut player_idx: usize = 0;

    loop {
        attempts += 1;
        circles_flat.clear();
        crosses_flat.clear();
        let mut occupied = HashSet::new();
        // select 3 unique positions for circles
        while circles_flat.len() < 3 {
            let f = rng.gen_range(0..total_cells);
            if occupied.insert(f) {
                circles_flat.push(f);
            }
        }
        // select crosses
        let mut cross_count = rng.gen_range(5..=10);
        cross_count = std::cmp::min(cross_count, total_cells.saturating_sub(3));
        while crosses_flat.len() < cross_count {
            let f = rng.gen_range(0..total_cells);
            if occupied.insert(f) {
                crosses_flat.push(f);
            }
        }
        // ensure crosses aren't immediately losing
        crosses_flat.sort_unstable();
        if check_lose_flat(&crosses_flat) {
            if attempts >= max_attempts {
                break;
            } else {
                continue;
            }
        }
        player_idx = rng.gen_range(0..3);
        if reachable_win(&circles_flat, player_idx, &crosses_flat) {
            break;
        }
        if attempts >= max_attempts {
            break;
        }
    }

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
        // scatter a few crosses that don't immediately lose
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

    // Helper to attempt movement during runtime (allows pushing a single object: circle or cross)
    let attempt_move_runtime = |circles: &mut Vec<(usize, usize)>,
                                crosses: &mut Vec<(usize, usize)>,
                                player_idx: usize,
                                dr: isize,
                                dc: isize| {
        let (r, c) = circles[player_idx];
        let new_r_i = r as isize + dr;
        let new_c_i = c as isize + dc;
        if new_r_i < 0 || new_c_i < 0 {
            return;
        }
        let new_r = new_r_i as usize;
        let new_c = new_c_i as usize;
        if new_r >= rows {
            return;
        }
        if new_c >= row_widths[new_r] {
            return;
        }
        // occupied by circle?
        if let Some(idx) = circles
            .iter()
            .position(|&(rr, cc)| rr == new_r && cc == new_c)
        {
            // try push circle
            let push_r_i = new_r_i + dr;
            let push_c_i = new_c_i + dc;
            if push_r_i < 0 || push_c_i < 0 {
                return;
            }
            let push_r = push_r_i as usize;
            let push_c = push_c_i as usize;
            if push_r >= rows {
                return;
            }
            if push_c >= row_widths[push_r] {
                return;
            }
            if circles.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) {
                return;
            }
            if crosses.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) {
                return;
            }
            circles[idx] = (push_r, push_c);
            circles[player_idx] = (new_r, new_c);
            return;
        }
        // occupied by cross?
        if let Some(idx) = crosses
            .iter()
            .position(|&(rr, cc)| rr == new_r && cc == new_c)
        {
            let push_r_i = new_r_i + dr;
            let push_c_i = new_c_i + dc;
            if push_r_i < 0 || push_c_i < 0 {
                return;
            }
            let push_r = push_r_i as usize;
            let push_c = push_c_i as usize;
            if push_r >= rows {
                return;
            }
            if push_c >= row_widths[push_r] {
                return;
            }
            if circles.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) {
                return;
            }
            if crosses.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) {
                return;
            }
            crosses[idx] = (push_r, push_c);
            circles[player_idx] = (new_r, new_c);
            return;
        }
        // empty
        circles[player_idx] = (new_r, new_c);
    };

    // initial win/lose checks
    let mut circles_flat_now: Vec<usize> = circles.iter().map(|&(r, c)| to_flat(r, c)).collect();
    let mut crosses_flat_now: Vec<usize> = crosses.iter().map(|&(r, c)| to_flat(r, c)).collect();
    let mut won = is_win_flat(&circles_flat_now);
    let mut lost = check_lose_flat(&crosses_flat_now);

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
                    " 🎉 YOU WON! 🎉 ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                msg_lines.push(Spans::from(Span::raw("")));
                msg_lines.push(Spans::from(Span::styled(
                    "press q to quit",
                    Style::default().fg(Color::White),
                )));

                let overlay = Paragraph::new(msg_lines)
                    .block(Block::default().borders(Borders::ALL).title("Victory"));
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
                    Style::default().fg(Color::White),
                )));

                let overlay = Paragraph::new(msg_lines)
                    .block(Block::default().borders(Borders::ALL).title("Defeat"));
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
                                attempt_move_runtime(&mut circles, &mut crosses, player_idx, -1, 0)
                            }
                        }
                        'a' => {
                            if !won && !lost {
                                attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, -1)
                            }
                        }
                        's' => {
                            if !won && !lost {
                                attempt_move_runtime(&mut circles, &mut crosses, player_idx, 1, 0)
                            }
                        }
                        'd' => {
                            if !won && !lost {
                                attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, 1)
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Up => {
                        if !won && !lost {
                            attempt_move_runtime(&mut circles, &mut crosses, player_idx, -1, 0)
                        }
                    }
                    KeyCode::Left => {
                        if !won && !lost {
                            attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, -1)
                        }
                    }
                    KeyCode::Down => {
                        if !won && !lost {
                            attempt_move_runtime(&mut circles, &mut crosses, player_idx, 1, 0)
                        }
                    }
                    KeyCode::Right => {
                        if !won && !lost {
                            attempt_move_runtime(&mut circles, &mut crosses, player_idx, 0, 1)
                        }
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
            // re-evaluate win/lose state after handling input
            circles_flat_now = circles.iter().map(|&(r, c)| to_flat(r, c)).collect();
            crosses_flat_now = crosses.iter().map(|&(r, c)| to_flat(r, c)).collect();
            won = is_win_flat(&circles_flat_now);
            lost = check_lose_flat(&crosses_flat_now);
        }
    }
    Ok(())
}
