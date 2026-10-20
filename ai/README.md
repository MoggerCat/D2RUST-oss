# Guides for AI coding agents

These guides are for coding agents working on D2RUST. They describe the
code, the build and the rules of the project. They intentionally contain no
private infrastructure, credentials, internal process or company details.

Read in this order:

1. [`architecture.md`](architecture.md) — the data flow and the crates.
2. [`crate-boundaries.md`](crate-boundaries.md) — what belongs where.
3. [`build-and-test.md`](build-and-test.md) — the commands that must pass.
4. [`coding-conventions.md`](coding-conventions.md) — style and determinism.
5. [`clean-room.md`](clean-room.md) — what must never enter the tree.
6. [`adding-a-feature.md`](adding-a-feature.md) — a change from idea to PR.
7. [`test-first.md`](test-first.md) — a practical red/green loop.
8. [`review-checklist.md`](review-checklist.md) — what to check before finishing.

Rules that apply to every task:

- Work from the repository, not from memory. Read the code you are changing.
- Keep the tree buildable at every step.
- Never add game data, assets or names of original game content.
- Run `cargo build --workspace` and `cargo test --workspace` before you finish.
- If a test needs a game installation, it does not belong in this tree.
