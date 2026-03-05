use std::{error::Error, fs, io::Stdout, path::PathBuf, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Clear, Wrap},
};

use crate::{board::Board, database::upload};

struct PuzzleItem {
    path: PathBuf,
    file_name: String,
    rows: usize,
    cols: usize,
    created_at: Option<u64>,
}

fn parse_number(s: &str, key: &str) -> Option<u64> {
    if let Some(pos) = s.find(key) {
        let mut i = pos + key.len();
        let bytes = s.as_bytes();
        while i < bytes.len()
            && (bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t')
        {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && (bytes[i] >= b'0' && bytes[i] <= b'9') {
            i += 1;
        }
        if i > start {
            if let Ok(n) = s[start..i].parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}

fn parse_pairs(s: &str, key: &str) -> Vec<(usize, usize)> {
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    if let Some(pos) = s.find(key) {
        // find the first '[' after the key
        if let Some(rel) = s[pos..].find('[') {
            let start = pos + rel;
            let bytes = s.as_bytes();
            let mut depth: i32 = 0;
            let mut end = start;
            for i in start..bytes.len() {
                match bytes[i] as char {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end > start {
                let sub = &s[start..=end];
                // collect numbers
                let mut nums: Vec<usize> = Vec::new();
                let mut cur = String::new();
                for ch in sub.chars() {
                    if ch.is_ascii_digit() {
                        cur.push(ch);
                    } else {
                        if !cur.is_empty() {
                            if let Ok(n) = cur.parse::<usize>() {
                                nums.push(n);
                            }
                            cur.clear();
                        }
                    }
                }
                // group into pairs
                let mut it = nums.chunks(2);
                for chunk in it {
                    if chunk.len() == 2 {
                        pairs.push((chunk[0], chunk[1]));
                    }
                }
            }
        }
    }
    pairs
}

fn parse_pair_single(s: &str, key: &str) -> Option<(usize, usize)> {
    if let Some(pos) = s.find(key) {
        if let Some(rel) = s[pos..].find('[') {
            let start = pos + rel;
            let bytes = s.as_bytes();
            let mut nums: Vec<usize> = Vec::new();
            let mut cur = String::new();
            for i in start..bytes.len() {
                let ch = bytes[i] as char;
                if ch.is_ascii_digit() {
                    cur.push(ch);
                } else {
                    if !cur.is_empty() {
                        if let Ok(n) = cur.parse::<usize>() {
                            nums.push(n);
                        }
                        cur.clear();
                    }
                    if ch == ']' {
                        break;
                    }
                }
            }
            if nums.len() >= 2 {
                return Some((nums[0], nums[1]));
            }
        }
    }
    None
}

fn board_from_dims(
    rows: usize,
    cols: usize,
    removed: &[(usize, usize)],
) -> Result<Board, Box<dyn std::error::Error>> {
    if rows == 0 || cols == 0 {
        return Err("Invalid rows or cols".into());
    }
    let row_widths = vec![cols; rows];
    let mut row_offsets = vec![0usize; rows];
    for i in 1..rows {
        row_offsets[i] = row_offsets[i - 1] + row_widths[i - 1];
    }
    let total_cells = row_offsets[rows - 1] + row_widths[rows - 1];
    let default_grid_w: u16 = (4 * cols + 1) as u16;
    let default_grid_h: u16 = (2 * rows + 1) as u16;
    let mut cells = vec![true; total_cells];
    for &(r, c) in removed.iter() {
        if r < rows && c < cols {
            let idx = row_offsets[r] + c;
            if idx < total_cells {
                cells[idx] = false;
            }
        }
    }
    Ok(Board {
        rows,
        cols,
        row_widths,
        row_offsets,
        total_cells,
        cells,
        default_grid_w,
        default_grid_h,
    })
}

fn load_puzzle_board(
    path: &PathBuf,
) -> Result<
    (
        Board,
        Vec<(usize, usize)>,
        Vec<(usize, usize)>,
        Vec<(usize, usize)>,
        Option<(usize, usize)>,
        Option<u64>,
    ),
    Box<dyn std::error::Error>,
> {
    let contents = fs::read_to_string(path)?;
    let rows = parse_number(&contents, "\"rows\":").ok_or("missing rows")? as usize;
    let cols = parse_number(&contents, "\"cols\":").ok_or("missing cols")? as usize;
    let created_at = parse_number(&contents, "\"created_at\":");
    let circles = parse_pairs(&contents, "\"circles\":");
    let crosses = parse_pairs(&contents, "\"crosses\":");
    let removed = parse_pairs(&contents, "\"removed\":");
    let player = parse_pair_single(&contents, "\"player\":");
    let board = board_from_dims(rows, cols, &removed)?;
    Ok((board, circles, crosses, removed, player, created_at))
}

fn read_puzzles() -> Vec<PuzzleItem> {
    let mut puzzles = Vec::new();
    if let Ok(entries) = fs::read_dir("puzzles") {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("json") {
                let file_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let mut rows = 0usize;
                let mut cols = 0usize;
                let mut created_at = None;
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Some(r) = parse_number(&contents, "\"rows\":") {
                        rows = r as usize;
                    }
                    if let Some(c) = parse_number(&contents, "\"cols\":") {
                        cols = c as usize;
                    }
                    if let Some(ts) = parse_number(&contents, "\"created_at\":") {
                        created_at = Some(ts);
                    }
                }
                puzzles.push(PuzzleItem {
                    path,
                    file_name,
                    rows,
                    cols,
                    created_at,
                });
            }
        }
    }
    puzzles.sort_by_key(|p| p.file_name.clone());
    puzzles
}

pub fn show_browser(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let mut puzzles = read_puzzles();
    let mut selected: usize = 0;
    let mut status_msg: Option<String> = None;
    let mut error_popup: Option<String> = None;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Cap the browser overlay to a reasonable maximum roughly equal to
            // two-thirds of a 1920x1080 screen (approx 1280×720 pixels).
            // Using approximate character cell size (8×16 px) this maps to ~160 cols × 45 rows.
            const SCREEN_W_PX: u16 = 1920;
            const SCREEN_H_PX: u16 = 1080;
            const MAX_W_PX: u16 = SCREEN_W_PX * 2 / 3;
            const MAX_H_PX: u16 = SCREEN_H_PX * 2 / 3;
            const PX_PER_COL: u16 = 8;
            const PX_PER_ROW: u16 = 16;
            let max_cols = (MAX_W_PX / PX_PER_COL).max(20u16); // ensure sane minimum
            let max_rows = (MAX_H_PX / PX_PER_ROW).max(10u16);

            let overlay_w = std::cmp::min(size.width.saturating_sub(4), max_cols);
            let overlay_h = std::cmp::min(size.height.saturating_sub(4), max_rows);

            let ox = (size.width.saturating_sub(overlay_w)) / 2;
            let oy = (size.height.saturating_sub(overlay_h)) / 2;
            let area = Rect::new(ox, oy, overlay_w, overlay_h);

            let block = Block::default()
                .title("Puzzle Browser")
                .borders(Borders::ALL);
            f.render_widget(block, area);

            // Build list lines
            let mut lines: Vec<Spans> = Vec::new();
            lines.push(Spans::from(Span::styled(
                " Available puzzles (Enter=select, d=delete, q=quit, u=upload) ",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Spans::from(Span::raw("")));
            if let Some(ref msg) = status_msg {
                lines.push(Spans::from(Span::styled(
                    msg.as_str(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Spans::from(Span::raw("")));
            }
            if puzzles.is_empty() {
                lines.push(Spans::from(Span::raw(
                    "No puzzles found. Create some via Create mode.",
                )));
            } else {
                for (i, p) in puzzles.iter().enumerate() {
                    let label = match p.created_at {
                        Some(ts) => format!("{}  —  {}x{}  —  {}", p.file_name, p.rows, p.cols, ts),
                        None => format!("{}  —  {}x{}", p.file_name, p.rows, p.cols),
                    };
                    if i == selected {
                        lines.push(Spans::from(Span::styled(
                            label,
                            Style::default().bg(Color::Yellow).fg(Color::Black),
                        )));
                    } else {
                        lines.push(Spans::from(Span::raw(label)));
                    }
                }
            }

            let inner = Rect::new(
                area.x.saturating_add(1),
                area.y.saturating_add(1),
                area.width.saturating_sub(2),
                area.height.saturating_sub(2),
            );
            let para = Paragraph::new(lines).alignment(Alignment::Left);
            f.render_widget(para, inner);

            // show error popup if set
            if let Some(ref err) = error_popup {
                let max_w = size.width.saturating_sub(10);
                let ew = std::cmp::min(max_w, 80u16);
                let mut err_lines: Vec<Spans> = Vec::new();
                err_lines.push(Spans::from(Span::styled(
                    " Error ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                err_lines.push(Spans::from(Span::raw("")));
                for line in err.lines() {
                    err_lines.push(Spans::from(Span::styled(line, Style::default().fg(Color::Red))));
                }
                err_lines.push(Spans::from(Span::raw("")));
                err_lines.push(Spans::from(Span::raw("Press any key to close")));
                let eh = std::cmp::min((err_lines.len() as u16) + 4, size.height.saturating_sub(4));
                let ex = (size.width.saturating_sub(ew)) / 2;
                let ey = (size.height.saturating_sub(eh)) / 2;
                let earea = Rect::new(ex, ey, ew, eh);
                let err_para = Paragraph::new(err_lines)
                    .alignment(Alignment::Left)
                    .block(Block::default().borders(Borders::ALL).title("Error"))
                    .wrap(Wrap { trim: true });
                f.render_widget(Clear, earea);
                f.render_widget(err_para, earea);
            }
        })?;

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                if error_popup.is_some() {
                    // close error popup on any key press
                    error_popup = None;
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('d') => {
                            if !puzzles.is_empty() {
                                if let Some(p) = puzzles.get(selected) {
                                    let file_name = p.file_name.clone();
                                    let path = p.path.clone();
                                    match fs::remove_file(&path) {
                                        Ok(()) => {
                                            status_msg = Some(format!("Deleted {}", file_name));
                                            puzzles = read_puzzles();
                                            if puzzles.is_empty() {
                                                selected = 0;
                                            } else if selected >= puzzles.len() {
                                                selected = puzzles.len() - 1;
                                            }
                                        }
                                        Err(e) => {
                                            error_popup = Some(format!("Failed to delete {}: {}", file_name, e));
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !puzzles.is_empty() && selected + 1 < puzzles.len() {
                                selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            if !puzzles.is_empty() {
                                if let Some(p) = puzzles.get(selected) {
                                    match load_puzzle_board(&p.path) {
                                        Ok((
                                            board,
                                            circles,
                                            crosses,
                                            _removed,
                                            player,
                                            _created_at,
                                        )) => {
                                            // determine player index (if player marked, find its index among circles)
                                            let player_idx = if let Some(player_pos) = player {
                                                circles
                                                    .iter()
                                                    .position(|&p| p == player_pos)
                                                    .unwrap_or(0usize)
                                            } else {
                                                if !circles.is_empty() { 0usize } else { 0usize }
                                            };
                                            if let Err(e) = crate::game::run_puzzle(
                                                terminal, board, circles, crosses, player_idx,
                                            ) {
                                                eprintln!("Failed to run puzzle: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to load puzzle: {}", e);
                                        }
                                    }
                                }
                            }
                            return Ok(());
                        }
                        KeyCode::Char('u') => {
                            if puzzles.is_empty() {
                                status_msg = Some("No puzzle to upload".to_string());
                            } else if let Some(p) = puzzles.get(selected) {
                                match fs::read_to_string(&p.path) {
                                    Ok(json) => {
                                        match upload(&p.file_name, &json) {
                                            Ok(id) => {
                                                if id.is_empty() {
                                                    status_msg = Some(format!("Uploaded {}", p.file_name));
                                                } else {
                                                    status_msg = Some(format!("Uploaded {} (id {})", p.file_name, id));
                                                }
                                            }
                                            Err(e) => {
                                                error_popup = Some(format!("Upload failed: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error_popup = Some(format!("Failed to read {}: {}", p.file_name, e));
                                    }
                                }
                                puzzles = read_puzzles();
                                if puzzles.is_empty() {
                                    selected = 0;
                                } else if selected >= puzzles.len() {
                                    selected = puzzles.len() - 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
