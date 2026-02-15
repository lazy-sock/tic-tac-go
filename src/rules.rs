// Game rules and search helpers
use crate::board::Board;
use std::collections::{HashMap, HashSet};

pub fn is_win_flat(positions: &[usize], board: &Board) -> bool {
    if positions.len() < 3 {
        return false;
    }
    let mut by_row: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut by_col: HashMap<usize, Vec<usize>> = HashMap::new();
    for &p in positions {
        let (r, c) = board.from_flat(p);
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
}

pub fn check_lose_flat(crosses: &[usize], board: &Board) -> bool {
    if crosses.len() < 3 {
        return false;
    }
    let mut by_row: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut by_col: HashMap<usize, Vec<usize>> = HashMap::new();
    for &p in crosses {
        let (r, c) = board.from_flat(p);
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
}

/// Cheap conservative deadlock checks for crosses:
/// - detect any 2x2 filled block of crosses (unsolvable in general)
/// - detect crosses in convex board corners (no adjacent cells in two orthogonal dirs)
/// - detect crosses blocked on two orthogonal sides by other crosses/walls (conservative)
pub fn check_cross_deadlock(crosses: &[usize], board: &Board) -> bool {
    if crosses.is_empty() {
        return false;
    }
    let mut set: HashSet<(usize, usize)> = HashSet::new();
    for &p in crosses {
        let (r, c) = board.from_flat(p);
        set.insert((r, c));
    }

    // 2x2 block check
    for &(r, c) in set.iter() {
        if c + 1 < board.row_widths[r]
            && r + 1 < board.rows
            && c < board.row_widths[r + 1]
            && c + 1 < board.row_widths[r + 1]
            && set.contains(&(r, c + 1))
            && set.contains(&(r + 1, c))
            && set.contains(&(r + 1, c + 1))
        {
            return true;
        }
    }

    // corner / blocked-by-crosses checks (conservative)
    for &(r, c) in set.iter() {
        let up_missing = r == 0 || c >= board.row_widths[r - 1];
        let down_missing = r + 1 >= board.rows || c >= board.row_widths[r + 1];
        let left_missing = c == 0;
        let right_missing = c + 1 >= board.row_widths[r];

        if (up_missing && left_missing)
            || (up_missing && right_missing)
            || (down_missing && left_missing)
            || (down_missing && right_missing)
        {
            return true;
        }

        // consider neighboring crosses as blockers (only crosses count as blockers here)
        let up_blocked = up_missing || set.contains(&(r.saturating_sub(1), c));
        let down_blocked = down_missing || set.contains(&(r + 1, c));
        let left_blocked = left_missing || set.contains(&(r, c.saturating_sub(1)));
        let right_blocked = right_missing || set.contains(&(r, c + 1));

        if (up_blocked && left_blocked)
            || (up_blocked && right_blocked)
            || (down_blocked && left_blocked)
            || (down_blocked && right_blocked)
        {
            return true;
        }
    }

    false
}
