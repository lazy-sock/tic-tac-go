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

// Reverse-move used for scrambling: "pulls" objects toward the player when possible so
// that the scramble is guaranteed to be solvable by reversing these moves (i.e., by pushes).
pub fn attempt_move_reverse(circles: &mut Vec<(usize, usize)>, crosses: &mut Vec<(usize, usize)>, player_idx: usize, dr: isize, dc: isize, board: &Board) {
    let (pr, pc) = circles[player_idx];

    // position of the box we would pull from (one step behind the player in the given direction)
    let box_r_i = pr as isize - dr;
    let box_c_i = pc as isize - dc;

    // target position where the player would move to after pulling (one step forward)
    let new_r_i = pr as isize + dr;
    let new_c_i = pc as isize + dc;
    if new_r_i < 0 || new_c_i < 0 { return; }
    let new_r = new_r_i as usize;
    let new_c = new_c_i as usize;
    if new_r >= board.rows { return; }
    if new_c >= board.row_widths[new_r] { return; }

    // Helper to test occupancy
    let occupied_by_any = |r: usize, c: usize, circles: &Vec<(usize, usize)>, crosses: &Vec<(usize, usize)>| {
        circles.iter().any(|&(rr, cc)| rr == r && cc == c) || crosses.iter().any(|&(rr, cc)| rr == r && cc == c)
    };

    // If there's a box/circle one step behind the player, and the forward cell is free, pull it into the player's cell
    if box_r_i >= 0 && box_c_i >= 0 {
        let box_r = box_r_i as usize;
        let box_c = box_c_i as usize;
        if box_r < board.rows && box_c < board.row_widths[box_r] {
            // if new forward cell occupied, can't pull
            if occupied_by_any(new_r, new_c, circles, crosses) { return; }

            if let Some(idx) = circles.iter().position(|&(rr, cc)| rr == box_r && cc == box_c) {
                circles[idx] = (pr, pc);
                circles[player_idx] = (new_r, new_c);
                return;
            }
            if let Some(idx) = crosses.iter().position(|&(rr, cc)| rr == box_r && cc == box_c) {
                crosses[idx] = (pr, pc);
                circles[player_idx] = (new_r, new_c);
                return;
            }
        }
    }

    // No pull possible; if forward cell is empty, just move the player forward
    if !occupied_by_any(new_r, new_c, circles, crosses) {
        circles[player_idx] = (new_r, new_c);
    }
}
