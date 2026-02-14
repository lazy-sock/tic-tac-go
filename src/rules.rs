// Game rules and search helpers
use std::collections::{HashMap, HashSet, VecDeque};
use crate::board::Board;

pub fn is_win_flat(positions: &[usize], board: &Board) -> bool {
    if positions.len() < 3 { return false; }
    let mut by_row: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut by_col: HashMap<usize, Vec<usize>> = HashMap::new();
    for &p in positions {
        let (r, c) = board.from_flat(p);
        by_row.entry(r).or_default().push(c);
        by_col.entry(c).or_default().push(r);
    }
    for (_r, mut cols_vec) in by_row.into_iter() {
        if cols_vec.len() < 3 { continue; }
        cols_vec.sort_unstable();
        for i in 0..cols_vec.len().saturating_sub(2) {
            if cols_vec[i + 1] == cols_vec[i] + 1 && cols_vec[i + 2] == cols_vec[i + 1] + 1 {
                return true;
            }
        }
    }
    for (_c, mut rows_vec) in by_col.into_iter() {
        if rows_vec.len() < 3 { continue; }
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
    if crosses.len() < 3 { return false; }
    let mut by_row: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut by_col: HashMap<usize, Vec<usize>> = HashMap::new();
    for &p in crosses {
        let (r, c) = board.from_flat(p);
        by_row.entry(r).or_default().push(c);
        by_col.entry(c).or_default().push(r);
    }
    for (_r, mut cols_vec) in by_row.into_iter() {
        if cols_vec.len() < 3 { continue; }
        cols_vec.sort_unstable();
        for i in 0..cols_vec.len().saturating_sub(2) {
            if cols_vec[i + 1] == cols_vec[i] + 1 && cols_vec[i + 2] == cols_vec[i + 1] + 1 {
                return true;
            }
        }
    }
    for (_c, mut rows_vec) in by_col.into_iter() {
        if rows_vec.len() < 3 { continue; }
        rows_vec.sort_unstable();
        for i in 0..rows_vec.len().saturating_sub(2) {
            if rows_vec[i + 1] == rows_vec[i] + 1 && rows_vec[i + 2] == rows_vec[i + 1] + 1 {
                return true;
            }
        }
    }
    false
}

pub fn reachable_win(circles_flat: &[usize], player_idx: usize, crosses_flat: &[usize], board: &Board) -> bool {
    let mut q: VecDeque<(usize, [usize; 2], Vec<usize>)> = VecDeque::new();
    let mut visited: HashSet<Vec<u16>> = HashSet::new();
    let p0 = circles_flat[player_idx];
    let mut others = [circles_flat[(player_idx + 1) % 3], circles_flat[(player_idx + 2) % 3]];
    if others[0] > others[1] { others.swap(0,1); }
    let mut crosses = crosses_flat.to_vec();
    crosses.sort_unstable();

    let encode = |p: usize, o: &[usize; 2], x: &Vec<usize>| -> Vec<u16> {
        let mut key = Vec::with_capacity(3 + x.len());
        key.push(p as u16);
        key.push(o[0] as u16);
        key.push(o[1] as u16);
        for &xx in x { key.push(xx as u16); }
        key
    };

    visited.insert(encode(p0, &others, &crosses));
    q.push_back((p0, others, crosses.clone()));

    let mut nodes = 0usize;
    let max_nodes = 200_000usize;

    while let Some((p, o, x)) = q.pop_front() {
        nodes += 1;
        if nodes > max_nodes { return false; }
        let posv = vec![p, o[0], o[1]];
        if is_win_flat(&posv, board) { return true; }

        for (dr, dc) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)].iter().cloned() {
            let (pr, pc) = board.from_flat(p);
            let new_r_i = pr as isize + dr;
            let new_c_i = pc as isize + dc;
            if new_r_i < 0 || new_c_i < 0 { continue; }
            let new_r = new_r_i as usize;
            let new_c = new_c_i as usize;
            if new_r >= board.rows { continue; }
            if new_c >= board.row_widths[new_r] { continue; }
            let p1 = board.to_flat(new_r, new_c);

            let mut occupied_by_circle: Option<usize> = None;
            if o[0] == p1 { occupied_by_circle = Some(0); } else if o[1] == p1 { occupied_by_circle = Some(1); }

            if let Some(other_idx) = occupied_by_circle {
                let push_r_i = new_r_i + dr;
                let push_c_i = new_c_i + dc;
                if push_r_i < 0 || push_c_i < 0 { continue; }
                let push_r = push_r_i as usize;
                let push_c = push_c_i as usize;
                if push_r >= board.rows { continue; }
                if push_c >= board.row_widths[push_r] { continue; }
                let p2 = board.to_flat(push_r, push_c);
                if o[0] == p2 || o[1] == p2 { continue; }
                if x.iter().any(|&xx| xx == p2) { continue; }
                let mut new_o = o;
                new_o[other_idx] = p2;
                if new_o[0] > new_o[1] { new_o.swap(0,1); }
                let k = encode(p1, &new_o, &x);
                if visited.contains(&k) { continue; }
                if check_lose_flat(&x, board) { continue; }
                visited.insert(k);
                q.push_back((p1, new_o, x.clone()));
            } else if let Some(cross_idx) = x.iter().position(|&xx| xx == p1) {
                let push_r_i = new_r_i + dr;
                let push_c_i = new_c_i + dc;
                if push_r_i < 0 || push_c_i < 0 { continue; }
                let push_r = push_r_i as usize;
                let push_c = push_c_i as usize;
                if push_r >= board.rows { continue; }
                if push_c >= board.row_widths[push_r] { continue; }
                let p2 = board.to_flat(push_r, push_c);
                if o[0] == p2 || o[1] == p2 || p == p2 { continue; }
                if x.iter().any(|&xx| xx == p2) { continue; }
                let mut new_x = x.clone();
                new_x[cross_idx] = p2;
                new_x.sort_unstable();
                if check_lose_flat(&new_x, board) { continue; }
                let k = encode(p1, &o, &new_x);
                if visited.contains(&k) { continue; }
                visited.insert(k);
                q.push_back((p1, o, new_x));
            } else {
                let k = encode(p1, &o, &x);
                if visited.contains(&k) { continue; }
                if check_lose_flat(&x, board) { continue; }
                visited.insert(k);
                q.push_back((p1, o, x.clone()));
            }
        }
    }
    false
}
