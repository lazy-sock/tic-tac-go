use std::{error::Error, io::Stdout, time::Duration, fs, path::PathBuf};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Rect, Alignment},
    widgets::{Block, Borders, Paragraph},
    text::{Span, Spans},
    style::{Style, Color, Modifier},
};

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
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t') {
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

fn read_puzzles() -> Vec<PuzzleItem> {
    let mut puzzles = Vec::new();
    if let Ok(entries) = fs::read_dir("puzzles") {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() { continue; }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("json") {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let mut rows = 0usize;
                let mut cols = 0usize;
                let mut created_at = None;
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Some(r) = parse_number(&contents, "\"rows\":") { rows = r as usize; }
                    if let Some(c) = parse_number(&contents, "\"cols\":") { cols = c as usize; }
                    if let Some(ts) = parse_number(&contents, "\"created_at\":") { created_at = Some(ts); }
                }
                puzzles.push(PuzzleItem { path, file_name, rows, cols, created_at });
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

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("browser").borders(Borders::ALL);
            let area = Rect::new(0, 0, size.width, size.height);
            f.render_widget(block, area);

            // Build list lines
            let mut lines: Vec<Spans> = Vec::new();
            lines.push(Spans::from(Span::styled(
                " Puzzles (Enter=select, q=quit) ",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Spans::from(Span::raw("")));
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
        })?;

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
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
                                eprintln!("Selected puzzle: {}", p.path.display());
                            }
                        }
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}
