use crate::board::Board;

/// Attempt to move the player at `player_idx` by (dr, dc) in the runtime (forward) direction.
/// If the destination contains a movable object (circle or cross), attempt to push it one cell.
pub fn attempt_move_runtime(circles: &mut Vec<(usize, usize)>, crosses: &mut Vec<(usize, usize)>, player_idx: usize, dr: isize, dc: isize, board: &Board) {
    let (player_r, player_c) = circles[player_idx];
    let dest_r_i = player_r as isize + dr;
    let dest_c_i = player_c as isize + dc;

    // destination must be within board bounds
    if dest_r_i < 0 || dest_c_i < 0 { return; }
    let dest_r = dest_r_i as usize;
    let dest_c = dest_c_i as usize;
    if dest_r >= board.rows || dest_c >= board.row_widths[dest_r] { return; }

    // helper queries
    let find_circle_index = |r: usize, c: usize| circles.iter().position(|&(rr, cc)| rr == r && cc == c);
    let find_cross_index = |r: usize, c: usize| crosses.iter().position(|&(rr, cc)| rr == r && cc == c);
    let occupied_any = |r: usize, c: usize| find_circle_index(r, c).is_some() || find_cross_index(r, c).is_some();

    // If destination occupied by another circle, try to push that circle one step further
    if let Some(other_circle_idx) = find_circle_index(dest_r, dest_c) {
        let push_r_i = dest_r_i + dr;
        let push_c_i = dest_c_i + dc;
        if push_r_i < 0 || push_c_i < 0 { return; }
        let push_r = push_r_i as usize;
        let push_c = push_c_i as usize;
        if push_r >= board.rows || push_c >= board.row_widths[push_r] { return; }
        if occupied_any(push_r, push_c) { return; }

        // perform push
        circles[other_circle_idx] = (push_r, push_c);
        circles[player_idx] = (dest_r, dest_c);
        return;
    }

    // If destination occupied by a cross, try to push the cross one step further
    if let Some(cross_idx) = find_cross_index(dest_r, dest_c) {
        let push_r_i = dest_r_i + dr;
        let push_c_i = dest_c_i + dc;
        if push_r_i < 0 || push_c_i < 0 { return; }
        let push_r = push_r_i as usize;
        let push_c = push_c_i as usize;
        if push_r >= board.rows || push_c >= board.row_widths[push_r] { return; }
        if occupied_any(push_r, push_c) { return; }

        // perform push
        crosses[cross_idx] = (push_r, push_c);
        circles[player_idx] = (dest_r, dest_c);
        return;
    }

    // empty destination: move player
    circles[player_idx] = (dest_r, dest_c);
}

/// Reverse-move used for scrambling: attempt to "pull" an object from behind the player into
/// the player's current cell and move the player forward. This is the inverse of a forward push.
pub fn attempt_move_reverse(circles: &mut Vec<(usize, usize)>, crosses: &mut Vec<(usize, usize)>, player_idx: usize, dr: isize, dc: isize, board: &Board) {
    let (player_r, player_c) = circles[player_idx];

    // source cell (one step behind the player in the given direction)
    let source_r_i = player_r as isize - dr;
    let source_c_i = player_c as isize - dc;

    // forward cell where the player would step into after pulling
    let forward_r_i = player_r as isize + dr;
    let forward_c_i = player_c as isize + dc;

    // forward must be valid
    if forward_r_i < 0 || forward_c_i < 0 { return; }
    let forward_r = forward_r_i as usize;
    let forward_c = forward_c_i as usize;
    if forward_r >= board.rows || forward_c >= board.row_widths[forward_r] { return; }

    // helper queries
    let find_circle_index = |r: usize, c: usize| circles.iter().position(|&(rr, cc)| rr == r && cc == c);
    let find_cross_index = |r: usize, c: usize| crosses.iter().position(|&(rr, cc)| rr == r && cc == c);
    let occupied_any = |r: usize, c: usize| find_circle_index(r, c).is_some() || find_cross_index(r, c).is_some();

    // If there's an object one step behind the player and the forward cell is free, pull it into player's cell
    if source_r_i >= 0 && source_c_i >= 0 {
        let source_r = source_r_i as usize;
        let source_c = source_c_i as usize;
        if source_r < board.rows && source_c < board.row_widths[source_r] {
            // forward cell must be free to pull
            if occupied_any(forward_r, forward_c) { return; }

            if let Some(circle_idx) = find_circle_index(source_r, source_c) {
                circles[circle_idx] = (player_r, player_c);
                circles[player_idx] = (forward_r, forward_c);
                return;
            }
            if let Some(cross_idx) = find_cross_index(source_r, source_c) {
                crosses[cross_idx] = (player_r, player_c);
                circles[player_idx] = (forward_r, forward_c);
                return;
            }
        }
    }

    // Otherwise, if forward is empty, just move the player forward
    if !occupied_any(forward_r, forward_c) {
        circles[player_idx] = (forward_r, forward_c);
    }
}
