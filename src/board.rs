// Board utilities for tic-tac-go
use rand::{Rng, thread_rng};

pub struct Board {
    pub rows: usize,
    pub cols: usize,
    pub row_widths: Vec<usize>,
    pub row_offsets: Vec<usize>,
    pub total_cells: usize,
    pub default_grid_w: u16,
    pub default_grid_h: u16,
}

impl Board {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let rows: usize = rng.gen_range(3..=8);
        let min_cols = (20 + rows - 1) / rows;
        let max_cols = min_cols + 8;
        let cols: usize = rng.gen_range(min_cols..=max_cols);

        let mut row_widths = vec![cols; rows];

        let mut row_offsets = vec![0usize; rows];
        for i in 1..rows {
            row_offsets[i] = row_offsets[i - 1] + row_widths[i - 1];
        }
        let total_cells = row_offsets[rows - 1] + row_widths[rows - 1];

        let default_grid_w: u16 = (4 * cols + 1) as u16;
        let default_grid_h: u16 = (2 * rows + 1) as u16;

        Board { rows, cols, row_widths, row_offsets, total_cells, default_grid_w, default_grid_h }
    }

    pub fn to_flat(&self, r: usize, c: usize) -> usize {
        self.row_offsets[r] + c
    }

    pub fn from_flat(&self, mut idx: usize) -> (usize, usize) {
        let mut r = 0usize;
        while r < self.rows {
            let start = self.row_offsets[r];
            let w = self.row_widths[r];
            if idx < start + w {
                return (r, idx - start);
            }
            r += 1;
        }
        panic!("invalid flat index {}", idx);
    }
}
