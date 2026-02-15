// Puzzle generation using forward-scramble (sokoban-style)
use crate::board::Board;
use crate::rules::{check_lose_flat, is_win_flat, check_cross_deadlock};
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub fn generate_puzzle(board: &Board, difficulty: Difficulty) -> (Vec<usize>, Vec<usize>, usize) {
    let mut rng = thread_rng();
    let total_cells = board.total_cells;
    let mut attempts = 0usize;
    let max_attempts = 2000usize;

    let mut circles_flat: Vec<usize> = Vec::new();
    let mut crosses_flat: Vec<usize> = Vec::new();
    let mut player_idx: usize = 0;

    loop {
        attempts += 1;

        // enumerate all possible winning triples (horizontal and vertical) on present cells
        let mut triples: Vec<Vec<(usize, usize)>> = Vec::new();
        // horizontal
        for r in 0..board.rows {
            if board.row_widths[r] < 3 { continue; }
            for c in 0..=board.row_widths[r].saturating_sub(3) {
                if board.is_cell_present(r, c) && board.is_cell_present(r, c + 1) && board.is_cell_present(r, c + 2) {
                    triples.push(vec![(r, c), (r, c + 1), (r, c + 2)]);
                }
            }
        }
        // vertical
        if board.rows >= 3 {
            for r in 0..=board.rows - 3 {
                let min_w = board.row_widths[r..r + 3].iter().cloned().min().unwrap_or(0);
                if min_w == 0 { continue; }
                for c in 0..min_w {
                    if board.is_cell_present(r, c) && board.is_cell_present(r + 1, c) && board.is_cell_present(r + 2, c) {
                        triples.push(vec![(r, c), (r + 1, c), (r + 2, c)]);
                    }
                }
            }
        }

        if triples.is_empty() {
            if attempts >= max_attempts { break; } else { continue; }
        }

        let chosen = triples.choose(&mut rng).unwrap();
        let mut circles: Vec<(usize, usize)> = chosen.clone();

        // choose player index
        player_idx = rng.gen_range(0..3);

        // place crosses randomly (avoid overlapping circles)
        let mut occupied: HashSet<usize> = HashSet::new();
        for &(r, c) in &circles { occupied.insert(board.to_flat(r, c)); }

        let (min_cross, max_cross, min_steps, max_steps) = match difficulty {
            Difficulty::Easy => (3usize, 6usize, 20usize, 60usize),
            Difficulty::Medium => (5usize, 10usize, 40usize, 200usize),
            Difficulty::Hard => (8usize, 14usize, 100usize, 400usize),
        };
        let mut cross_count = rng.gen_range(min_cross..=max_cross);
        cross_count = std::cmp::min(cross_count, total_cells.saturating_sub(3));

        // build list of available flat indices (present cells and not occupied)
        let mut available: Vec<usize> = (0..total_cells)
            .filter(|&i| board.cells[i] && !occupied.contains(&i))
            .collect();
        if available.len() < cross_count {
            if attempts >= max_attempts { break; } else { continue; }
        }
        available.shuffle(&mut rng);

        let crosses_indices: Vec<usize> = available.into_iter().take(cross_count).collect();
        let mut crosses: Vec<(usize, usize)> = crosses_indices.iter().map(|&f| board.from_flat(f)).collect();

        let crosses_flat_init: Vec<usize> = crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
        if check_lose_flat(&crosses_flat_init, board) { if attempts >= max_attempts { break; } else { continue; } }
        if check_cross_deadlock(&crosses_flat_init, board) { if attempts >= max_attempts { break; } else { continue; } }

        // scramble by making random valid moves from the winning state (reverse pushes)
        let steps_target = rng.gen_range(min_steps..=max_steps);
        let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut moves_made = 0usize;
        let mut inner_tries = 0usize;

        while moves_made < steps_target && inner_tries < steps_target * 10 {
            inner_tries += 1;
            let (dr, dc) = dirs[rng.gen_range(0..4)];

            let pre_cir: Vec<usize> = circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            let pre_cross: Vec<usize> = crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();

            crate::movement::attempt_move_reverse(&mut circles, &mut crosses, player_idx, dr, dc, board);

            let post_cir: Vec<usize> = circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            let post_cross: Vec<usize> = crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();

            if pre_cir != post_cir || pre_cross != post_cross {
                moves_made += 1;
            }
        }

        let final_circles_flat: Vec<usize> = circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
        let mut final_crosses_flat: Vec<usize> = crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
        final_crosses_flat.sort_unstable();

        // avoid trivial already-won or losing puzzles
        if is_win_flat(&final_circles_flat, board) { if attempts >= max_attempts { break; } else { continue; } }
        if check_lose_flat(&final_crosses_flat, board) { if attempts >= max_attempts { break; } else { continue; } }
        if check_cross_deadlock(&final_crosses_flat, board) { if attempts >= max_attempts { break; } else { continue; } }

        // ensure player has at least one legal move (can't be completely stuck)
        let mut has_move = false;
        let test_dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for &(dr, dc) in test_dirs.iter() {
            let mut test_circles = circles.clone();
            let mut test_crosses = crosses.clone();
            let pre_cir: Vec<usize> = test_circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            let pre_cross: Vec<usize> = test_crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            crate::movement::attempt_move_runtime(&mut test_circles, &mut test_crosses, player_idx, dr, dc, board);
            let post_cir: Vec<usize> = test_circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            let post_cross: Vec<usize> = test_crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            if pre_cir != post_cir || pre_cross != post_cross { has_move = true; break; }
        }
        if !has_move { if attempts >= max_attempts { break; } else { continue; } }

        circles_flat = final_circles_flat;
        crosses_flat = final_crosses_flat;

        break;
    }

    (circles_flat, crosses_flat, player_idx)
}
