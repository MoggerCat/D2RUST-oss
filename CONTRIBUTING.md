# Contributing to D2RUST

Thanks for taking a look. D2RUST is a from-scratch, single-player
reimplementation. Contributions are welcome as patches or pull requests,
provided they respect the clean-room rules below.

## Before you start

- Read `docs/ARCHITECTURE.md` and `docs/CLEAN_ROOM.md`.
- Build the workspace with `cargo build --workspace`.
- Run the unit tests with `cargo test --workspace`.
- Do not add game files, extracted assets, saves, baked animation packs or
  any other material from the original game. The repository must stay
  useful without a game installation.
- Do not add multiplayer, launcher, updater, mod-loading or plugin code.
  These are intentionally out of scope.

## Clean-room rules

- Write code in your own words and structure.
- Do not paste decompiler output, assembly, copied source or near-verbatim
  translations into the repository, including comments.
- Describe behaviour in game terms. Do not add executable addresses,
  version-specific build numbers, internal spec citations or references to
  private notes.
- Numeric facts may be used, but do not add tables of original display
  names. Names shown to the player must come from the player's own files at
  run time.
- If you are unsure whether a change is clean, open an issue first.

## Code style

- Run `cargo fmt --all` and keep `cargo clippy --workspace --all-targets`
  free of new warnings.
- Keep game rules in `d2-sim`; keep I/O, rendering and UI in `d2-client`.
  `d2-server` owns the authoritative state and validates intents.
- No `unsafe` (the workspace forbids it).
- Game logic must be deterministic: no wall-clock reads, no unordered
  iteration over hash maps, no floating point in simulation state.
- Prefer small, focused modules and tests near the code they cover.

## Tests

- Add a unit test for new game rules and edge cases.
- Tests must not require the original game files. Use synthetic values and
  the readers' test-support features.
- `cargo test --workspace` must pass on a clean checkout.

## Pull requests

- Keep changes focused; unrelated cleanups belong in a separate PR.
- Explain what behaviour changed and why.
- Include the commands you ran and their results.
- By contributing you agree that your contribution is licensed under the
  MIT License.
