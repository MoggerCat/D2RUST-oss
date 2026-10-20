# Playtest

This page describes a local playtest of a build you made yourself. There is
no launcher, download service or release pipeline in this project.

## 1. Build

```sh
cargo build -p d2-client --profile dist
```

Run `target/dist/D2RUST-Game` (Linux) or `target/dist/D2RUST-Game.exe`
(Windows). The release executable opens no console window; fatal errors are
shown in a message box and written to the log.

## 2. Install

On the first start, point the game at the folder that holds your own copy of
the original game. The install copies, converts and bakes locally; see
`README.md` and `docs/LOCAL-RUN.md`.

## 3. Play

Double-click the executable, or run:

```sh
D2RUST-Game play
```

The main menu opens. **Single Player** leads to character select and
character create. From a command line you can skip the menu:

| Command | What |
|---|---|
| `D2RUST-Game play --new sorceress MyName` | a new character in memory |
| `D2RUST-Game play --save "path/to/MyName.d2s"` | load a save |
| `--difficulty normal\|nightmare\|hell` | difficulty |
| `--res 640x480\|800x600\|1068x600` | the game frame |
| `--seed N` | a fixed game seed, to repeat a run |

Saves, settings and logs stay in the D2RUST user folder. The game never
writes into the folder you selected during install.

## 4. Keys

Defaults: left click walk/attack/interact, right click the right skill,
`Ctrl` run, `R` walk/run toggle, `Shift` stand still, `Alt` show items,
`A`/`C` character, `I`/`B` inventory, `T` skill tree, `S` skill choice,
`Q` quests, `M` message log, `Tab` automap, `F1`–`F8` skill hotkeys,
`1`–`4` belt, `Esc` game menu. Rebind in Configure Controls.

## 5. Developer panel

The in-game developer panel is a debugging aid for D2RUST itself, not a
feature of the original game. It is on by default in development builds.
Turn it off with `--no-dev`. Press `` ` `` to open or close it; the console
accepts commands such as `help`, `heal`, `level`, `gold`, `item`, `spawn`
and `save`/`load`. Do not use a run with developer commands as a reference
for the original game's behaviour.

## 6. Reporting a problem

Collect:

1. `game.log` and, after a crash, `crash.log` from the logs folder.
2. A screenshot of the window or message box.
3. The save file, if the problem is reproducible from it.
4. The exact command, what you did and what you expected.

Do not attach game data archives or extracted assets.
