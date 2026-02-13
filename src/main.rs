use std::error::Error;
use std::io::{self, Stdout};
use std::time::Duration;
use std::collections::HashSet;

use rand::{thread_rng, Rng};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Paragraph};
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

    // Generate three distinct circle positions (row, col)
    let mut rng = thread_rng();
    let mut occupied = HashSet::new();
    let mut positions: Vec<(usize, usize)> = Vec::new();
    while positions.len() < 3 {
        let r = rng.gen_range(0..n);
        let c = rng.gen_range(0..n);
        if occupied.insert((r, c)) {
            positions.push((r, c));
        }
    }
    // Choose one circle to be the player
    let player_idx = rng.gen_range(0..positions.len());

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
        })?;

        // Exit on 'q' or Esc
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }
    Ok(())
}
