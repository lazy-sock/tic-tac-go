use crate::{board::Board, rules::is_win_flat};

enum Step {
    Left,
    Right,
    Top,
    Bottom,
}

pub fn solve_puzzle(puzzle: Board) {
    let mut steps: Vec<Step> = Vec::new();
    while !is_win_flat(&[42], &puzzle) {
        // &[42] is a placeholder
        steps.push(Step::Left);
    }
}
