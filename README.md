# tic-tac-go

IMPORTANT: this project implements "tic-tac-go" (not tic-tac-toe).

Short description
- Minimal Rust skeleton for implementing the tic-tac-go game.

Basic rules (provided by project owner)
- Aim: form exactly three circles in a straight line (three in a row) to score/win.
- Crosses act as movable obstacles.
- You lose if three crosses become aligned in a straight line.
- Your character is a circle that can push other crosses or circles.
- The playing field may have a random shape; implementations should account for varied board geometry.

Running locally
- Build: `cargo build`
- Run: `cargo run`

Notes
- These rules are the concise, owner-provided basics to include in this repository's README; implementational details and edge cases should be defined in code or additional docs.

## Copilot Git Identity

- Keep your personal git identity in the global config so manual commits are authored to your account:
  - git config --global user.name "Your Name"
  - git config --global user.email "you@example.com"

- Repository-local user.name and user.email have been removed so commits default to your global identity.

- When Copilot (the automated agent) needs to make commits, it must use a per-commit Copilot identity so commits by Copilot are clearly attributed to Copilot and do not override your personal identity. Use one of these safe methods (preferred):
  - Per-command config:
    - git -c user.name="Copilot CLI" -c user.email="copilot@local" commit -m "..."
  - Environment variables:
    - GIT_AUTHOR_NAME="Copilot CLI" GIT_AUTHOR_EMAIL="copilot@local" GIT_COMMITTER_NAME="Copilot CLI" GIT_COMMITTER_EMAIL="copilot@local" git commit -m "..."

- Recommended helper script: scripts/copilot-commit.sh — makes a single commit with the Copilot identity (do not make this script change your global config). Use it like:
  - ./scripts/copilot-commit.sh -m "message"

- Human developers must continue to commit normally (git commit ...) so commits remain authored to your global identity.

## Generator algorithm and difficulty

This project uses a sokoban-style reverse-scramble generator to produce puzzles that are guaranteed solvable: start from a simple winning (solved) state and apply legal reverse moves ("pulls") so every generated puzzle can be solved by reversing those moves. The approach below summarizes research (notably Parberry 2003 and related sokoban literature) and explains practical techniques for measuring and tuning difficulty beyond simply counting crosses.

Key takeaways
- Reverse-scramble (backwards generation) is reliable for solvability: it produces puzzles whose forward solution exists by construction.
- Pushes (number of box pushes) correlate much more strongly with human-perceived difficulty than raw move-counts; push-optimal solutions are particularly important.
- Deadlocks (static patterns like 2×2 blocks or corner pushes, and dynamic/corral deadlocks) are critical — small deadlock motifs can make a puzzle unsolvable or dramatically harder.
- Structural complexity (box dependencies, corridor layouts, constrained storage) matters: puzzles that require coordinated multi-box sequences are harder than those with many free spaces.

Why cross-count alone is insufficient
- The number of obstacle pieces (crosses) does not capture spatial arrangement: a few crosses placed in tight corridors can be far harder than many crosses in open space.
- Difficulty arises from interactions: how boxes block each other, access to goals, forced sequences of pushes, and the branching factor during search.
- Instead of cross-count, use structural and solver-aware metrics (pushes, branching factor, dependency graph features) to rank difficulty.

Recommended generator design (practical roadmap)
1. Core generator (reverse-scramble)
   - Start from a solved/winning layout (three-circle line placements and empty board otherwise).
   - Apply a sequence of legal reverse pushes (pulls) to boxes and circles. Parameterize by a target number of pushes (not mere moves) and by a scramble depth distribution.
   - To increase diversity, bias selection of reverse moves toward those that move boxes into walls, narrow corridors, or areas that create dependencies (probabilistic weighting rather than uniform random).

2. Difficulty metrics (score, not just count)
   - Minimal pushes (use a push-aware solver to compute or estimate the minimum number of pushes to win).
   - Branching-factor profile: how many distinct pushes are available on average during solution search (higher branching often means higher complexity).
   - Box dependency graph size / conflicts: how many boxes' paths interfere or require ordering.
   - Trap motifs count: number of narrow corridors, one-way stashes, or near-wall lockers introduced.
   - Combine these into a weighted score (e.g., score = α*min_pushes + β*branching + γ*dependencies + δ*trap_score).

3. Deadlock detection & safety
   - Implement fast conservative deadlock checks (already added: 2×2 and corner checks). Extend with corral detection and bipartite matching for unreachable box-goal pairings if goals are introduced.
   - Reject or repair scrambles that create unavoidable deadlocks.

4. Trap motifs and curated difficulty features
   - Maintain a small library of "trap motifs" (tunnel+dead-end, tight corner stash, short U-shaped corridor) that are known to increase difficulty when boxes are placed inside.
   - During scrambling, occasionally (with probability tuned by difficulty) attempt to apply a motif: bias reverse moves to place boxes into those motifs, then validate solvability.

5. Validation and tuning loop
   - After scrambling, run a final push-aware solver pass to validate solvability and compute minimal pushes/score.
   - If the puzzle score is outside the requested difficulty band, continue scrambling (more reverse-pushes with appropriate bias) or retry with different seeds; include a timeout/fallback to a deterministic layout.

6. Performance considerations
   - Use compact state encodings for the solver (bitsets/u64 masks, or packed integers) to avoid heavy allocations and speed up searches.
   - Precompute neighbor lists and valid board adjacencies to speed move checks on irregular boards.
   - Run heavy validation (solver) asynchronously or in a background thread to avoid blocking the UI; keep a fast conservative filter to reject obvious bad states early.

Implementation notes for this repository
- The current code already adopted reverse-scramble (reverse/pull moves) and added conservative deadlock checks; this is the correct baseline.
- Next steps to improve difficulty control (recommended order):
  1. Add a Difficulty enum and generator signature like `generate_puzzle(board: &Board, difficulty: Difficulty)` so caller can request Easy/Medium/Hard.
  2. Replace cross-count heuristics with a scoring function driven by push-based solver metrics (minimal pushes, branching) and motif counts.
  3. Implement a small motif library and bias reverse-move selection to place boxes in motif locations for higher difficulty.
  4. Optimize solver and run final validation asynchronously; keep the conservative checks synchronous to quickly reject bad scrambles.

Practical trade-offs
- Full push-optimal solving for every candidate is expensive; use a staged pipeline: cheap conservative filters → motif/structural scoring → occasional full solver validation for final selection.
- If generation time is critical for UX, generate puzzles in a background thread and show a quick deterministic fallback while the solver validates.

References and further reading
- Parberry, I. (2003). Puzzle Generators and Complexity. GAMEON-NA METH 2003. (https://ianparberry.com/pubs/GAMEON-NA_METH_03.pdf)
- Classic sokoban literature on deadlocks, push-aware heuristics, and level generation (see Sokoban research surveys and solver implementations).

---

(Generator design details and planned improvements are recorded here to guide future changes to the `src/generator.rs` implementation.)
