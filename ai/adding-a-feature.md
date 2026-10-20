# Adding a feature

A practical sequence for a change in this codebase.

## 1. Find the owning layer

Ask what the feature *is*:

- a game rule or formula → `d2-sim`;
- a new message → `d2-proto` and the message tables;
- a file or table field → `d2-formats` / `d2-data`;
- a server validation or sequence → `d2-server`;
- a screen, panel, sound or draw rule → `d2-client`;
- install or conversion behaviour → `d2-install` + `d2-client::install`.

If the feature crosses layers, add the smallest type at the lowest layer
that both sides need.

## 2. Read the neighbours

Open the module you are changing and one similar module. Match its error
type, naming and test style. Search for the table or constant you need
before adding a new one.

## 3. Write the test first

Add a focused test next to the code. Use synthetic input. Run it and see it
fail for the right reason.

## 4. Implement the smallest change

Keep the public API as small as possible. Do not add a feature flag for a
permanent feature. Do not add a plugin or hook point unless the task
explicitly asks for one.

## 5. Update the tables and docs

- If a parsed table changes, update its header, parser and test together.
- If the CLI changes, update `README.md` and `docs/LOCAL-RUN.md`.
- If the architecture changes, update `docs/ARCHITECTURE.md` and `ai/`.

## 6. Run the gates

```sh
cargo fmt --all
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Fix warnings you introduced. Do not disable a lint to make a change pass.

## 7. Describe the change

In the PR: what changed, why, how it was tested, and any limitation. Call
out a behaviour change explicitly.
