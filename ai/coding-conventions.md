# Coding conventions

## General

- Match the style of the file you are in. The codebase uses `rustfmt`
  defaults, 4-space indentation and doc comments on public items.
- Keep functions small and name things after the game concept, not the
  implementation detail.
- Prefer explicit integer types in game state; avoid `usize` in persisted
  or protocol data.
- Use `thiserror` for library errors and `anyhow` only at the binary edge.
- No `unsafe`.

## Determinism

`d2-sim` must produce the same result for the same inputs on every run and
platform:

- no floating point in game state or formulas;
- no `HashMap`/`HashSet` iteration where order affects the result; use
  `BTreeMap`/`BTreeSet` or a sorted `Vec`;
- no wall clock, thread scheduling, locale or environment reads;
- draw random numbers only from the game's seeded RNG, in a fixed order.

The workspace denies `clippy::float_arithmetic` in `d2-sim`.

## Comments

- Explain why, in game terms. Do not restate the code.
- Do not add executable addresses, version-specific build numbers, private
  document paths or citations to an internal spec.
- Do not paste decompiler output or copied source.

## Tables

- Tables under `tables/` hold numeric facts and neutral identifiers.
- Do not add display names, level names, item names, monster names or
  original strings.
- If a table is parsed, update its header constant and its parser together.
- Add a test only if the table can be checked with synthetic input.

## Errors

- A malformed file should return a typed error with the path and the
  offset or field, not panic.
- A missing optional file may be `None`; a missing required file is an
  error.
- Do not log secrets or full user paths.

## Formatting

`cargo fmt --all` before finishing. Keep unrelated reformatting out of a
change.
