# Local run

Everything here runs on your own machine. The project has no CI dependency
and no service to contact.

## Requirements

- Rust 1.99 or newer.
- Windows 10/11 or a Linux desktop with a Vulkan-capable GPU.
- Your own copy of the original game.

## Build

```sh
cargo build --workspace
cargo build -p d2-client --profile dist
```

## First run

```sh
cargo run -p d2-client --release -- play
```

With no converted data the game asks for the folder that contains your
original game. You can pass it instead:

```sh
cargo run -p d2-client --release -- setup --game-dir "/path/to/your/game"
```

The install step copies the archives it needs into the D2RUST user folder,
converts them, and bakes animations. It does not modify the source folder.

## Later runs

```sh
cargo run -p d2-client --release -- play
```

The converted folder is reused. Useful options:

```text
--game-dir DIR       the source folder for a fresh install
--native DIR         an already converted asset folder
--source native|mpq  converted assets (default) or archives (debug builds)
--res WxH            the frame
--save FILE.d2s      load a character
--new CLASS NAME     start a new character in memory
--difficulty D       normal, nightmare or hell
--anim-dir DIR       a different animation bake folder
--no-dev             turn the developer panel off
```

Saves, settings and logs are written under the D2RUST user folder:
on Windows `%APPDATA%\d2rust`, on Linux `$XDG_DATA_HOME/d2rust` (usually
`~/.local/share/d2rust`). Character saves are under the `D2RUST` documents
folder used by the client.

## Tests

```sh
cargo test --workspace
```

The unit tests use synthetic data. They do not read a game installation.

## Troubleshooting

- `no game folder was chosen`: pass `--game-dir` or set `D2_GAME_DIR`.
- `no native folder`: run `setup` again, or pass `--native DIR`.
- A Vulkan/adapter error: update the GPU driver; a software Vulkan
  implementation also works but is slow.
- A crash writes `crash.log` under the logs folder named in the message.
