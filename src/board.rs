// Board utilities for tic-tac-go
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;

pub struct Board {
    pub rows: usize,
    pub cols: usize,
    pub row_widths: Vec<usize>,
    pub row_offsets: Vec<usize>,
    pub total_cells: usize,
    // per-cell existence mask: true == cell exists / is playable
    pub cells: Vec<bool>,
    pub default_grid_w: u16,
    pub default_grid_h: u16,
}

impl Board {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let rows: usize = rng.gen_range(3..=8);
        let min_cols = 20_usize.div_ceil(rows);
        let max_cols = min_cols + 8;
        let cols: usize = rng.gen_range(min_cols..=max_cols);

        let row_widths = vec![cols; rows];

        let mut row_offsets = vec![0usize; rows];
        for i in 1..rows {
            row_offsets[i] = row_offsets[i - 1] + row_widths[i - 1];
        }
        let total_cells = row_offsets[rows - 1] + row_widths[rows - 1];

        let default_grid_w: u16 = (4 * cols + 1) as u16;
        let default_grid_h: u16 = (2 * rows + 1) as u16;

        // start with all cells present
        let mut cells = vec![true; total_cells];

        // Decide roughly how many holes to carve out (as fraction of total cells)
        let hole_frac: f64 = rng.gen_range(0.06..0.16); // 6%..16% holes
        let mut target_holes = ((total_cells as f64) * hole_frac).round() as usize;
        if target_holes == 0 && total_cells > 8 {
            target_holes = 1;
        }
        target_holes = std::cmp::min(target_holes, total_cells.saturating_sub(6));

        if target_holes > 0 {
            // seeded random-walk blobs
            let seeds = rng.gen_range(1..=3);

            let to_rc = |mut idx: usize| -> (usize, usize) {
                let mut r = 0usize;
                while r < rows {
                    let start = row_offsets[r];
                    let w = row_widths[r];
                    if idx < start + w {
                        return (r, idx - start);
                    }
                    r += 1;
                }
                panic!("invalid flat index in generator {}", idx);
            };
            let rc_to_idx = |r: usize, c: usize| row_offsets[r] + c;

            // pick some seed positions
            let mut seeds_pos: Vec<usize> = Vec::new();
            for _ in 0..seeds {
                let mut f = rng.gen_range(0..total_cells);
                // nudge away from extreme edges so blobs look nicer
                let (sr, sc) = to_rc(f);
                if sc == 0 && row_widths[sr] >= 2 {
                    f = rc_to_idx(sr, 1);
                }
                if sc + 1 >= row_widths[sr] && row_widths[sr] >= 2 {
                    f = rc_to_idx(sr, row_widths[sr].saturating_sub(2));
                }
                seeds_pos.push(f);
            }

            let mut removed = 0usize;
            let mut attempts = 0usize;
            while removed < target_holes && attempts < target_holes * 20 {
                attempts += 1;
                let seed_idx = rng.gen_range(0..seeds_pos.len());
                let mut cur = seeds_pos[seed_idx];
                let steps = rng.gen_range(1..=4);
                for _ in 0..steps {
                    let (r, c) = to_rc(cur);
                    if cells[cur] {
                        cells[cur] = false;
                        removed += 1;
                        if removed >= target_holes {
                            break;
                        }
                    }

                    // gather neighbors (4-way), respecting row widths
                    let mut neighbors: Vec<usize> = Vec::new();
                    if c > 0 {
                        neighbors.push(rc_to_idx(r, c - 1));
                    }
                    if c + 1 < row_widths[r] {
                        neighbors.push(rc_to_idx(r, c + 1));
                    }
                    if r > 0 && c < row_widths[r - 1] {
                        neighbors.push(rc_to_idx(r - 1, c));
                    }
                    if r + 1 < rows && c < row_widths[r + 1] {
                        neighbors.push(rc_to_idx(r + 1, c));
                    }
                    if neighbors.is_empty() {
                        break;
                    }
                    neighbors.shuffle(&mut rng);
                    cur = *neighbors.first().unwrap();
                }

                // jitter the seed occasionally
                if rng.gen_bool(0.18) {
                    seeds_pos[seed_idx] = rng.gen_range(0..total_cells);
                }
            }

            // Ensure the remaining cells are globally connected; if not, carve simple corridors
            // Build components of present cells
            let mut seen = vec![false; total_cells];
            let mut components: Vec<Vec<usize>> = Vec::new();
            for i in 0..total_cells {
                if !cells[i] || seen[i] {
                    continue;
                }
                // BFS
                let mut q = vec![i];
                seen[i] = true;
                let mut qi = 0usize;
                while qi < q.len() {
                    let cur = q[qi];
                    qi += 1;
                    let (r, c) = to_rc(cur);
                    if c > 0 {
                        let n = rc_to_idx(r, c - 1);
                        if cells[n] && !seen[n] { seen[n] = true; q.push(n); }
                    }
                    if c + 1 < row_widths[r] {
                        let n = rc_to_idx(r, c + 1);
                        if cells[n] && !seen[n] { seen[n] = true; q.push(n); }
                    }
                    if r > 0 && c < row_widths[r - 1] {
                        let n = rc_to_idx(r - 1, c);
                        if cells[n] && !seen[n] { seen[n] = true; q.push(n); }
                    }
                    if r + 1 < rows && c < row_widths[r + 1] {
                        let n = rc_to_idx(r + 1, c);
                        if cells[n] && !seen[n] { seen[n] = true; q.push(n); }
                    }
                }
                components.push(q);
            }

            if components.len() > 1 {
                // connect smaller components to the largest with simple manhattan corridors
                let mut largest_idx = 0usize;
                for (i, comp) in components.iter().enumerate() {
                    if comp.len() > components[largest_idx].len() { largest_idx = i; }
                }
                for (i, comp) in components.iter().enumerate() {
                    if i == largest_idx { continue; }
                    let src = comp[0];
                    // find nearest cell in largest
                    let mut best = None;
                    let mut best_dist = usize::MAX;
                    for &t in &components[largest_idx] {
                        let (sr, sc) = to_rc(src);
                        let (tr, tc) = to_rc(t);
                        let dist = sr.abs_diff(tr) + sc.abs_diff(tc);
                        if dist < best_dist { best_dist = dist; best = Some(t); }
                    }
                    if let Some(dest) = best {
                        let (mut r, mut c) = to_rc(src);
                        let (tr, tc) = to_rc(dest);
                        while r != tr {
                            if r < tr { r += 1; } else { r -= 1; }
                            if c >= row_widths[r] { c = row_widths[r].saturating_sub(1); }
                            let idx = rc_to_idx(r, c);
                            if !cells[idx] { cells[idx] = true; }
                        }
                        while c != tc {
                            if c < tc { c += 1; } else { c -= 1; }
                            if c >= row_widths[r] { break; }
                            let idx = rc_to_idx(r, c);
                            if !cells[idx] { cells[idx] = true; }
                        }
                    }
                }
            }
        }

        Board {
            rows,
            cols,
            row_widths,
            row_offsets,
            total_cells,
            cells,
            default_grid_w,
            default_grid_h,
        }
    }

    pub fn to_flat(&self, r: usize, c: usize) -> usize {
        self.row_offsets[r] + c
    }

    pub fn from_flat(&self, idx: usize) -> (usize, usize) {
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

    pub fn is_cell_present(&self, r: usize, c: usize) -> bool {
        let idx = self.to_flat(r, c);
        self.cells[idx]
    }
}
