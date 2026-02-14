// Puzzle generation
use crate::board::Board;
use crate::rules::{check_lose_flat, reachable_win};
use rand::{Rng, thread_rng};
use std::collections::HashSet;

pub fn generate_puzzle(board: &Board) -> (Vec<usize>, Vec<usize>, usize) {
    let mut attempts = 0usize;
    let max_attempts = 2000usize;
    let mut circles_flat: Vec<usize> = Vec::new();
    let mut crosses_flat: Vec<usize> = Vec::new();
    let mut player_idx: usize = 0;
    let total_cells = board.total_cells;
    let mut rng = thread_rng();

    loop {
        attempts += 1;
        circles_flat.clear();
        crosses_flat.clear();
        let mut occupied = HashSet::new();
        while circles_flat.len() < 3 {
            let f = rng.gen_range(0..total_cells);
            if occupied.insert(f) {
                circles_flat.push(f);
            }
        }
        let mut cross_count = rng.gen_range(5..=10);
        cross_count = std::cmp::min(cross_count, total_cells.saturating_sub(3));
        while crosses_flat.len() < cross_count {
            let f = rng.gen_range(0..total_cells);
            if occupied.insert(f) {
                crosses_flat.push(f);
            }
        }
        crosses_flat.sort_unstable();
        if check_lose_flat(&crosses_flat, board) {
            if attempts >= max_attempts { break; } else { continue; }
        }
        player_idx = rng.gen_range(0..3);
        if reachable_win(&circles_flat, player_idx, &crosses_flat, board) {
            break;
        }
        if attempts >= max_attempts { break; }
    }

    (circles_flat, crosses_flat, player_idx)
}
