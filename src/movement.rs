use crate::board::Board;

pub fn attempt_move_runtime(circles: &mut Vec<(usize, usize)>, crosses: &mut Vec<(usize, usize)>, player_idx: usize, dr: isize, dc: isize, board: &Board) {
    let (r, c) = circles[player_idx];
    let new_r_i = r as isize + dr;
    let new_c_i = c as isize + dc;
    if new_r_i < 0 || new_c_i < 0 {
        return;
    }
    let new_r = new_r_i as usize;
    let new_c = new_c_i as usize;
    if new_r >= board.rows { return; }
    if new_c >= board.row_widths[new_r] { return; }
    // occupied by circle?
    if let Some(idx) = circles.iter().position(|&(rr, cc)| rr == new_r && cc == new_c) {
        let push_r_i = new_r_i + dr;
        let push_c_i = new_c_i + dc;
        if push_r_i < 0 || push_c_i < 0 { return; }
        let push_r = push_r_i as usize;
        let push_c = push_c_i as usize;
        if push_r >= board.rows { return; }
        if push_c >= board.row_widths[push_r] { return; }
        if circles.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
        if crosses.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
        circles[idx] = (push_r, push_c);
        circles[player_idx] = (new_r, new_c);
        return;
    }
    // occupied by cross?
    if let Some(idx) = crosses.iter().position(|&(rr, cc)| rr == new_r && cc == new_c) {
        let push_r_i = new_r_i + dr;
        let push_c_i = new_c_i + dc;
        if push_r_i < 0 || push_c_i < 0 { return; }
        let push_r = push_r_i as usize;
        let push_c = push_c_i as usize;
        if push_r >= board.rows { return; }
        if push_c >= board.row_widths[push_r] { return; }
        if circles.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
        if crosses.iter().any(|&(rr, cc)| rr == push_r && cc == push_c) { return; }
        crosses[idx] = (push_r, push_c);
        circles[player_idx] = (new_r, new_c);
        return;
    }
    // empty
    circles[player_idx] = (new_r, new_c);
}
