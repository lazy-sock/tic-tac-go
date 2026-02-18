use crate::board::Board;

// Helper query functions
fn find_circle_index(circles: &[(usize, usize)], r: usize, c: usize) -> Option<usize> {
    circles.iter().position(|&(rr, cc)| rr == r && cc == c)
}

fn find_cross_index(crosses: &[(usize, usize)], r: usize, c: usize) -> Option<usize> {
    crosses.iter().position(|&(rr, cc)| rr == r && cc == c)
}

fn occupied_any(
    circles: &[(usize, usize)],
    crosses: &[(usize, usize)],
    r: usize,
    c: usize,
) -> bool {
    find_circle_index(circles, r, c).is_some() || find_cross_index(crosses, r, c).is_some()
}

/// Attempt to move the player at `player_idx` by (dr, dc) in the runtime (forward) direction.
/// If the destination contains a movable object (circle or cross), attempt to push it one cell.
pub fn attempt_move_runtime(
    circles: &mut [(usize, usize)],
    crosses: &mut [(usize, usize)],
    player_idx: usize,
    direction_row: isize,
    direction_column: isize,
    board: &Board,
) {
    let (player_row, player_column) = circles[player_idx];
    let destination_row_i = player_row as isize + direction_row;
    let destination_column_i = player_column as isize + direction_column;

    // destination must be within board bounds and present
    if destination_row_i < 0 || destination_column_i < 0 {
        return;
    }
    let destination_row = destination_row_i as usize;
    let destination_column = destination_column_i as usize;
    if destination_row >= board.rows || destination_column >= board.row_widths[destination_row] || !board.is_cell_present(destination_row, destination_column) {
        return;
    }

    // If destination occupied by another circle, try to push that circle one step further
    if let Some(other_circle_idx) = find_circle_index(circles, destination_row, destination_column)
    {
        let push_row_i = destination_row_i + direction_row;
        let push_column_i = destination_column_i + direction_column;
        if push_row_i < 0 || push_column_i < 0 {
            return;
        }
        let push_row = push_row_i as usize;
        let push_column = push_column_i as usize;
        if push_row >= board.rows || push_column >= board.row_widths[push_row] || !board.is_cell_present(push_row, push_column) {
            return;
        }
        if occupied_any(circles, crosses, push_row, push_column) {
            return;
        }

        // perform push
        circles[other_circle_idx] = (push_row, push_column);
        circles[player_idx] = (destination_row, destination_column);
        return;
    }

    // If destination occupied by a cross, try to push the cross one step further
    if let Some(cross_idx) = find_cross_index(crosses, destination_row, destination_column) {
        let push_row_i = destination_row_i + direction_row;
        let push_column_i = destination_column_i + direction_column;
        if push_row_i < 0 || push_column_i < 0 {
            return;
        }
        let push_row = push_row_i as usize;
        let push_column = push_column_i as usize;
        if push_row >= board.rows || push_column >= board.row_widths[push_row] || !board.is_cell_present(push_row, push_column) {
            return;
        }
        if occupied_any(circles, crosses, push_row, push_column) {
            return;
        }

        // perform push
        crosses[cross_idx] = (push_row, push_column);
        circles[player_idx] = (destination_row, destination_column);
        return;
    }

    // empty destination: move player
    circles[player_idx] = (destination_row, destination_column);
}

/// Reverse-move used for scrambling: attempt to "pull" an object from behind the player into
/// the player's current cell and move the player forward. This is the inverse of a forward push.
pub fn attempt_move_reverse(
    circles: &mut [(usize, usize)],
    crosses: &mut [(usize, usize)],
    player_idx: usize,
    dr: isize,
    dc: isize,
    board: &Board,
) {
    let (player_row, player_column) = circles[player_idx];

    // source cell (one step behind the player in the given direction)
    let source_row_i = player_row as isize - dr;
    let source_column_i = player_column as isize - dc;

    // forward cell where the player would step into after pulling
    let forward_row_i = player_row as isize + dr;
    let forward_column_i = player_column as isize + dc;

    // forward must be valid and present
    if forward_row_i < 0 || forward_column_i < 0 {
        return;
    }
    let forward_row = forward_row_i as usize;
    let forward_column = forward_column_i as usize;
    if forward_row >= board.rows || forward_column >= board.row_widths[forward_row] || !board.is_cell_present(forward_row, forward_column) {
        return;
    }

    // If there's an object one step behind the player and the forward cell is free, pull it into player's cell
    if source_row_i >= 0 && source_column_i >= 0 {
        let source_row = source_row_i as usize;
        let source_column = source_column_i as usize;
        if source_row < board.rows && source_column < board.row_widths[source_row] && board.is_cell_present(source_row, source_column) {
            // forward cell must be free to pull
            if occupied_any(circles, crosses, forward_row, forward_column) {
                return;
            }

            if let Some(circle_idx) = find_circle_index(circles, source_row, source_column) {
                circles[circle_idx] = (player_row, player_column);
                circles[player_idx] = (forward_row, forward_column);
                return;
            }
            if let Some(cross_idx) = find_cross_index(crosses, source_row, source_column) {
                crosses[cross_idx] = (player_row, player_column);
                circles[player_idx] = (forward_row, forward_column);
                return;
            }
        }
    }

    // Otherwise, if forward is empty, just move the player forward
    if !occupied_any(circles, crosses, forward_row, forward_column) {
        circles[player_idx] = (forward_row, forward_column);
    }
}
