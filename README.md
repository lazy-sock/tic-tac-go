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
