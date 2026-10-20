# Build and test

Run everything from the repository root.

```sh
cargo build --workspace
cargo test --workspace
```

Useful variants:

```sh
cargo test --workspace -- --test-threads=1   # if a test shares a temp path
cargo test -p d2-sim                          # one crate
cargo test -p d2-sim some_test_name           # one test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p d2-client --profile dist       # release executable
```

## What the tests may use

- Synthetic values built in the test or by a test-support feature.
- Temporary directories under the system temp folder.
- The in-memory transport and a hand-built game state.

They must not:

- read a game installation or an environment variable that points at one;
- download anything;
- require a GPU, a display or an audio device;
- depend on the order of tests or on wall-clock time.

If a test would need real game files, do not add it. The public tree has no
such tests by design.

## Common failures

- `error: failed to run custom build command for alsa-sys`: install ALSA
  development headers on Linux.
- A Bevy link error on Linux: install the winit/Vulkan/udev development
  packages named by the error.
- A timeout in a long simulation test: run it with `--test-threads=1` and
  check that it has a bounded tick count.
