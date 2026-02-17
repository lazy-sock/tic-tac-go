// Constructive puzzle generation using reverse-play BFS (sokoban-style).
//
// Algorithm (based on proven Sokoban level generators like miki151/sokoban):
// 1. Start from a solved state (circles in a winning triple)
// 2. Place crosses strategically, avoiding deadlocks
// 3. Explore reverse moves via BFS to build a tree of reachable states
// 4. Pick the state with maximum distance from the solution as the puzzle
// 5. Run multiple iterations with randomized configurations, keep the best
//
// This guarantees every generated puzzle is solvable by construction since
// the initial state was reached by reversing valid moves from a solution.

use crate::board::Board;
use crate::rules::{check_cross_deadlock, check_lose_flat, is_win_flat};
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use std::collections::{HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct ReverseState {
    circles: Vec<usize>,
    crosses: Vec<usize>,
}

impl ReverseState {
    fn new(circles: &[(usize, usize)], crosses: &[(usize, usize)], board: &Board) -> Self {
        let circles: Vec<usize> = circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
        let mut crosses: Vec<usize> = crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
        crosses.sort_unstable();
        ReverseState { circles, crosses }
    }
}

/// Enumerate all winning triples (3 consecutive present cells, horizontal and vertical).
fn enumerate_triples(board: &Board) -> Vec<Vec<(usize, usize)>> {
    let mut triples: Vec<Vec<(usize, usize)>> = Vec::new();
    for r in 0..board.rows {
        if board.row_widths[r] < 3 {
            continue;
        }
        for c in 0..=board.row_widths[r].saturating_sub(3) {
            if board.is_cell_present(r, c)
                && board.is_cell_present(r, c + 1)
                && board.is_cell_present(r, c + 2)
            {
                triples.push(vec![(r, c), (r, c + 1), (r, c + 2)]);
            }
        }
    }
    if board.rows >= 3 {
        for r in 0..=board.rows - 3 {
            let min_w = board.row_widths[r..r + 3]
                .iter()
                .cloned()
                .min()
                .unwrap_or(0);
            if min_w == 0 {
                continue;
            }
            for c in 0..min_w {
                if board.is_cell_present(r, c)
                    && board.is_cell_present(r + 1, c)
                    && board.is_cell_present(r + 2, c)
                {
                    triples.push(vec![(r, c), (r + 1, c), (r + 2, c)]);
                }
            }
        }
    }
    triples
}

/// Place crosses on the board, avoiding deadlocks and the lose condition.
/// Uses a heuristic: prefer cells at moderate distance from circles to create
/// interesting obstacles without trivial deadlocks.
fn place_crosses(
    board: &Board,
    circles: &[(usize, usize)],
    count: usize,
    rng: &mut impl Rng,
) -> Option<Vec<(usize, usize)>> {
    let occupied: HashSet<usize> = circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
    let mut available: Vec<usize> = (0..board.total_cells)
        .filter(|&i| board.cells[i] && !occupied.contains(&i))
        .collect();
    if available.len() < count {
        return None;
    }

    // Shuffle available cells, then sort by a heuristic score:
    // prefer moderate Manhattan distance from circle centroid (not too close, not too far)
    available.shuffle(rng);
    let centroid_r: f64 =
        circles.iter().map(|&(r, _)| r as f64).sum::<f64>() / circles.len() as f64;
    let centroid_c: f64 =
        circles.iter().map(|&(_, c)| c as f64).sum::<f64>() / circles.len() as f64;

    // Sort by distance from centroid, breaking ties randomly (already shuffled)
    available.sort_by_key(|&f| {
        let (r, c) = board.from_flat(f);
        let dist = ((r as f64 - centroid_r).abs() + (c as f64 - centroid_c).abs()) as usize;
        // Prefer distance 2..6 from centroid (sweet spot for interesting obstacles)
        if dist < 2 {
            10 + dist
        } else if dist <= 6 {
            dist
        } else {
            5 + dist
        }
    });

    let mut crosses: Vec<(usize, usize)> = Vec::new();
    for &f in &available {
        if crosses.len() >= count {
            break;
        }
        let pos = board.from_flat(f);
        crosses.push(pos);
        let cross_flat: Vec<usize> = crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
        if check_lose_flat(&cross_flat, board) || check_cross_deadlock(&cross_flat, board) {
            crosses.pop();
        }
    }
    if crosses.len() >= count {
        Some(crosses)
    } else {
        None
    }
}

/// Perform BFS over reverse moves from a solved state.
/// Returns the farthest reachable state (circles, crosses in rc form) and its depth,
/// along with the total number of unique states explored.
///
/// The key insight: every state found this way is guaranteed solvable because
/// we reached it by undoing valid forward moves from a known solution.
fn reverse_bfs(
    board: &Board,
    init_circles: &[(usize, usize)],
    init_crosses: &[(usize, usize)],
    player_idx: usize,
    max_nodes: usize,
    rng: &mut impl Rng,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>, usize) {
    let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    let init_state = ReverseState::new(init_circles, init_crosses, board);
    let mut visited: HashSet<ReverseState> = HashSet::new();
    visited.insert(init_state);

    // BFS queue: (circles_rc, crosses_rc, depth)
    let mut queue: VecDeque<(Vec<(usize, usize)>, Vec<(usize, usize)>, usize)> = VecDeque::new();
    queue.push_back((init_circles.to_vec(), init_crosses.to_vec(), 0));

    let mut best_circles = init_circles.to_vec();
    let mut best_crosses = init_crosses.to_vec();
    let mut best_depth = 0usize;
    // Collect multiple candidates at high depth for random selection
    let mut best_candidates: Vec<(Vec<(usize, usize)>, Vec<(usize, usize)>, usize)> = Vec::new();

    let mut nodes = 0usize;

    while let Some((circles, crosses, depth)) = queue.pop_front() {
        nodes += 1;
        if nodes > max_nodes {
            break;
        }

        // Track the best (deepest) state found
        if depth > best_depth {
            best_depth = depth;
            best_candidates.clear();
        }
        if depth == best_depth {
            best_candidates.push((circles.clone(), crosses.clone(), depth));
            // Cap stored candidates to avoid memory bloat
            if best_candidates.len() > 50 {
                let idx = rng.gen_range(0..best_candidates.len() - 1);
                best_candidates.swap_remove(idx);
            }
        }

        // Try all 4 reverse-move directions
        for &(dr, dc) in &dirs {
            let mut new_circles = circles.clone();
            let mut new_crosses = crosses.clone();

            crate::movement::attempt_move_reverse(
                &mut new_circles,
                &mut new_crosses,
                player_idx,
                dr,
                dc,
                board,
            );

            // Check if the state actually changed
            let cir_flat_before: Vec<usize> =
                circles.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            let cir_flat_after: Vec<usize> = new_circles
                .iter()
                .map(|&(r, c)| board.to_flat(r, c))
                .collect();
            let crs_flat_before: Vec<usize> =
                crosses.iter().map(|&(r, c)| board.to_flat(r, c)).collect();
            let crs_flat_after: Vec<usize> = new_crosses
                .iter()
                .map(|&(r, c)| board.to_flat(r, c))
                .collect();

            if cir_flat_before == cir_flat_after && crs_flat_before == crs_flat_after {
                continue;
            }

            // Reject states that cause losing or deadlock conditions
            if check_lose_flat(&crs_flat_after, board) {
                continue;
            }
            if check_cross_deadlock(&crs_flat_after, board) {
                continue;
            }

            // Reject if circles are already in a winning position (trivial)
            if is_win_flat(&cir_flat_after, board) {
                continue;
            }

            let new_state = ReverseState::new(&new_circles, &new_crosses, board);
            if visited.insert(new_state) {
                queue.push_back((new_circles, new_crosses, depth + 1));
            }
        }
    }

    // Pick a random candidate from the best depth tier for variety
    if !best_candidates.is_empty() {
        let chosen = &best_candidates[rng.gen_range(0..best_candidates.len())];
        best_circles = chosen.0.clone();
        best_crosses = chosen.1.clone();
        best_depth = chosen.2;
    }

    (best_circles, best_crosses, best_depth)
}

/// Check that the player has at least one safe legal move from this position.
fn has_safe_move(
    board: &Board,
    circles: &[(usize, usize)],
    crosses: &[(usize, usize)],
    player_idx: usize,
) -> bool {
    let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for &(dr, dc) in &dirs {
        let mut test_circles = circles.to_vec();
        let mut test_crosses = crosses.to_vec();
        let pre_cir: Vec<usize> = test_circles
            .iter()
            .map(|&(r, c)| board.to_flat(r, c))
            .collect();
        let pre_crs: Vec<usize> = test_crosses
            .iter()
            .map(|&(r, c)| board.to_flat(r, c))
            .collect();

        crate::movement::attempt_move_runtime(
            &mut test_circles,
            &mut test_crosses,
            player_idx,
            dr,
            dc,
            board,
        );

        let post_cir: Vec<usize> = test_circles
            .iter()
            .map(|&(r, c)| board.to_flat(r, c))
            .collect();
        let post_crs: Vec<usize> = test_crosses
            .iter()
            .map(|&(r, c)| board.to_flat(r, c))
            .collect();

        if post_cir == pre_cir && post_crs == pre_crs {
            continue;
        }
        if check_lose_flat(&post_crs, board) {
            continue;
        }
        if check_cross_deadlock(&post_crs, board) {
            continue;
        }
        return true;
    }
    false
}

pub fn generate_puzzle_constructive(
    board: &Board,
    difficulty: Difficulty,
) -> (Vec<usize>, Vec<usize>, usize) {
    let mut rng = thread_rng();

    let triples = enumerate_triples(board);
    if triples.is_empty() {
        return (Vec::new(), Vec::new(), 0);
    }

    // Difficulty parameters:
    //   cross_range: how many crosses to place
    //   min_depth: minimum solution depth (reverse BFS depth) to accept
    //   max_depth: don't accept puzzles deeper than this (keeps difficulty bounded)
    //   node_budget: BFS exploration budget per iteration
    //   iterations: how many random configurations to try
    let (cross_range, min_depth, max_depth, node_budget, iterations) = match difficulty {
        Difficulty::Easy => ((3usize, 5usize), 3usize, 10usize, 5_000usize, 30usize),
        Difficulty::Medium => ((4usize, 8usize), 6usize, 25usize, 20_000usize, 25usize),
        Difficulty::Hard => ((5usize, 10usize), 10usize, 80usize, 50_000usize, 20usize),
    };

    let mut best_result: Option<(Vec<usize>, Vec<usize>, usize, usize)> = None; // (circles, crosses, player_idx, depth)

    for _ in 0..iterations {
        // Pick a random winning triple
        let triple = triples.choose(&mut rng).unwrap();
        let circles: Vec<(usize, usize)> = triple.clone();

        // Pick a random player index (which circle is the player)
        let player_idx = rng.gen_range(0..3);

        // Pick a random cross count within the range
        let cross_count = rng
            .gen_range(cross_range.0..=cross_range.1)
            .min(board.total_cells.saturating_sub(3));

        // Place crosses
        let crosses = match place_crosses(board, &circles, cross_count, &mut rng) {
            Some(c) => c,
            None => continue,
        };

        // The circles are currently in the winning position.
        // Run reverse BFS to find the farthest reachable state.
        let (result_circles, result_crosses, depth) =
            reverse_bfs(board, &circles, &crosses, player_idx, node_budget, &mut rng);

        // Filter by difficulty depth range
        if depth < min_depth {
            continue;
        }
        let effective_depth = depth.min(max_depth);

        // Skip if already won or lost or deadlocked
        let result_cir_flat: Vec<usize> = result_circles
            .iter()
            .map(|&(r, c)| board.to_flat(r, c))
            .collect();
        let result_crs_flat: Vec<usize> = result_crosses
            .iter()
            .map(|&(r, c)| board.to_flat(r, c))
            .collect();
        if is_win_flat(&result_cir_flat, board) {
            continue;
        }
        if check_lose_flat(&result_crs_flat, board) {
            continue;
        }
        if check_cross_deadlock(&result_crs_flat, board) {
            continue;
        }

        // Ensure player has at least one safe move
        if !has_safe_move(board, &result_circles, &result_crosses, player_idx) {
            continue;
        }

        // Keep the best puzzle found so far (highest depth within range)
        let dominated = match &best_result {
            Some((_, _, _, best_d)) => effective_depth > *best_d,
            None => true,
        };
        if dominated {
            let mut crs_sorted = result_crs_flat;
            crs_sorted.sort_unstable();
            best_result = Some((result_cir_flat, crs_sorted, player_idx, effective_depth));
        }
    }

    match best_result {
        Some((circles, crosses, player_idx, _)) => (circles, crosses, player_idx),
        None => (Vec::new(), Vec::new(), 0),
    }
}
