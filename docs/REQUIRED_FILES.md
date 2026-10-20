# Required files

D2RUST needs the player's own, legally owned installation of the original
game. Nothing from it is committed to git and nothing is downloaded by the
game.

The first-run install asks for the folder that contains that installation,
copies the archives it needs into the D2RUST user folder, converts them to
native assets and bakes animation frames. Later runs use the converted
folder.

## What the install reads

- The data archives of the player's installation, from the folder chosen at
  the prompt or with `--game-dir`.
- Nothing outside that folder.

No list of archive names or hashes is published here. The converter opens
the archives present in the chosen folder and verifies the files it reads.
The game never compares the player's copy against a fixed build.

## What the install writes

Under the D2RUST user folder:

- `d2rust/game/`: the copied archives;
- `d2rust/native/`: the converted native assets;
- `D2RUST/cache/animation/baked/`: the baked animation frames;
- `D2RUST/saves/`, `settings.toml`, `controls.toml` and `logs/` during play.

The player's original installation is never modified.

## Saves

Characters are stored in D2RUST's own versioned format under `D2RUST/saves`.
The game can load an original game character save with `--save`; copy the
file first, because the game writes back to the file it loaded.
