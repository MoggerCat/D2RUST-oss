# Review checklist

Use this before declaring a change finished.

## Correctness

- [ ] The change does what the task asked, and nothing unrelated.
- [ ] The owning layer is right (see `crate-boundaries.md`).
- [ ] Errors are typed and include the failing file/field.
- [ ] No panics on malformed input, missing optional data or empty tables.
- [ ] No behaviour change is hidden inside a refactor.

## Determinism

- [ ] `d2-sim` has no floating point, wall clock, unordered iteration or
  unseeded randomness.
- [ ] RNG draw order is unchanged unless the task changes it.

## Clean room

- [ ] No game assets, data, saves, names or extracted files.
- [ ] No executable addresses, build-specific versions or private spec
  citations.
- [ ] New tables contain numeric facts and neutral labels only.
- [ ] Comments explain behaviour, not a disassembly.

## Tests

- [ ] A new rule has a test that fails without the change.
- [ ] Tests use synthetic data and need no game installation.
- [ ] `cargo test --workspace` passes.

## Build

- [ ] `cargo build --workspace` passes.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` has no new
  warnings.

## Documentation

- [ ] `README.md` or `docs/` updated if the CLI, install or architecture
  changed.
- [ ] `ai/` updated if the change alters a rule described there.
- [ ] The PR description names the commands run and their results.
