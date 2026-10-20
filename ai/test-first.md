# Test-first workflow

The fastest way to change game logic without breaking it is a short
red/green loop.

## 1. Reproduce

Write one test that fails for the behaviour you are about to change. Keep
it as small as possible: one rule, one input, one expected result.

```rust
#[test]
fn a_running_character_blocks_less_often() {
    // Build the smallest world that reaches the rule.
    // Assert the old result first; it should fail after the change.
}
```

Run only that test:

```sh
cargo test -p d2-sim a_running_character_blocks_less_often
```

## 2. Make it pass

Change the rule, not the test. If the test was wrong, fix the test first and
say why in the commit.

## 3. Protect the edges

Add cases for:

- the minimum and maximum value;
- an absent unit or item;
- a non-expansion or non-hell game, if the rule differs;
- an id outside the table.

## 4. Run the crate

```sh
cargo test -p d2-sim
```

Then the workspace:

```sh
cargo test --workspace
```

## 5. Keep tests deterministic

- Do not sleep or read the clock.
- Do not rely on test order.
- Use a fixed seed and fixed tick count.
- Build input from synthetic tables or literals.

## 6. When a test is hard to write

That is a design signal. The code probably mixes I/O, rendering or global
state with the rule. Extract the rule into a pure function and test that
function.
