# Copilot instructions for tic-tac-go

Purpose
- Short guidance for Copilot/AI sessions to quickly find build/test/lint commands, architecture, and repo-specific conventions.

Build / Test / Lint (commands)
- Build: `cargo build`
- Run: `cargo run` (binary entrypoint: src/main.rs)
- Run all tests: `cargo test`
- Run a single unit test (by name): `cargo test <test_name> -- --exact`
  - Example: `cargo test my_function_test -- --exact`
- Run an integration test file (in tests/): `cargo test --test <test_file_name>`
- See test output (do not capture): `cargo test -- --nocapture`
- Format: `cargo fmt` (install with `rustup component add rustfmt` if missing)
- Check formatting: `cargo fmt -- --check`
- Lint: `cargo clippy` (install with `rustup component add clippy`); strict check: `cargo clippy -- -D warnings`

Notes: This is a small Rust binary crate; standard cargo commands apply.

High-level architecture
- Single binary crate. Entrypoint: `src/main.rs` (prints a simple message currently).
- Manifest: `Cargo.toml` at repo root (edition = 2024).
- No workspace, no dependencies (current state). Adding features usually means adding files under `src/` and/or adding integration tests under `tests/`.

Key conventions and repository-specific patterns
- Keep the main executable in `src/main.rs`. If adding a reusable library surface, add `src/lib.rs` and keep business logic there so tests can import it.
- Unit tests: place inline in modules with `#[cfg(test)]`.
- Integration tests: place in `tests/<name>.rs` and run with `cargo test --test <name>`.
- Use `cargo fmt` for formatting and `cargo clippy` for lints; prefer running `cargo clippy -- -D warnings` on CI if configured.
- When adding dependencies, update `Cargo.toml` and run `cargo build` to validate.

Docs and existing AI assistant configs
- README.md has been added and includes brief, owner-provided basic rules for tic-tac-go.
- IMPORTANT: this repository refers to "tic-tac-go" (not tic-tac-toe). Per project owner instruction, tic-tac-go is a game made by Google.
- No other assistant config files found (CLAUDE.md, .cursorrules, AGENTS.md, CONVENTIONS.md, AIDER_CONVENTIONS.md, .windsurfrules, .clinerules, etc.).

Where to look first
- `Cargo.toml` — package metadata and edition
- `src/main.rs` — program entrypoint and current behavior
- Add tests under `tests/` when adding integration tests

If editing this repository during a Copilot session
- Commit every change. This project uses git heavily; small, frequent commits are expected.
- Use feature branches for substantial work; the repository may default to a single branch but creating branches for features or fixes is permitted.
- Prefer minimal, surgical edits. After edits, run `cargo test` and `cargo fmt` locally to validate behavior and style.
- If adding functionality, prefer extracting logic into `src/lib.rs` so it can be unit-tested easily.

Contact / followups
- If additional guidance is desired (CI config, test coverage, or lint rules), add a README or CONTRIBUTING file and update this instructions file accordingly.

Git workflow (commands)
- Commit every change. This project uses git heavily; small, frequent commits are expected.
- Common commands (examples):
  - Stage all changes: `git add -A`
  - Commit staged changes: `git commit -m "<short message>"`
  - Create a feature branch: `git checkout -b feature/<name>`
  - Push a branch and set upstream: `git push -u origin feature/<name>`
  - Update main before merging: `git checkout main && git pull --rebase origin main`
- Use descriptive commit messages and open PRs for review when collaborating.
