# Architecture

## Overview

D2RUST runs an authoritative game server in the same process as the client.
The client sends intents to that server; the server updates the simulation
and returns messages; the client renders a mirror of the server's world.
There is no network transport in this tree.

```text
   player's own game files
            │  read at install time, converted locally
            ▼
   d2-formats ──► d2-data
        │             │
        │             ▼
        │        d2-install ──► d2-native
        │             │
        ▼             ▼
   d2-client ◄──► d2-proto ◄──► d2-server
        │                          │
        └──────────────► d2-sim ◄──┘
                       (deterministic)
```

## Crates

| Crate | Responsibility |
|---|---|
| `d2-formats` | parsers for MPQ, DC6, DCC, DT1, DS1, COF, palettes and string tables |
| `d2-data` | typed readers for the game's text and binary tables |
| `d2-sim` | deterministic game rules; no I/O, rendering or Bevy |
| `d2-proto` | the client/server message types and their wire encoding |
| `d2-net` | the `Transport` interface and the in-memory transport |
| `d2-server` | the authoritative server: message queues, dispatch, host loop |
| `d2-native` | native asset formats and their round-trip readers/writers |
| `d2-install` | copy, conversion and install orchestration |
| `d2-client` | Bevy client: rendering, UI, input, audio, bridge, executable |

Dependency direction is one-way: `d2-formats` and `d2-data` at the bottom,
then `d2-sim`, then `d2-server`, then `d2-client`. `d2-install` depends on
the format/data/native layers and is used by the client. A crate must not
depend on a crate above it in that list.

## The game loop

- `d2-sim` runs a fixed 60 Hz logic tick. Unit update order, RNG draw order
  and integer arithmetic are fixed, so the same inputs always produce the
  same game.
- `d2-server` owns the `d2-sim` game, validates every client intent, and
  runs one tick per frame.
- The client sends C→S messages and receives S→C messages over
  `d2-net::memory`.
- The client's bridge decodes messages into a plain Rust world model.
  Bevy entities only mirror that model for drawing.
- The client does not predict game outcomes. It interpolates animation
  between ticks with its own presentation rules, and the server decides
  every result.

## Install and data

The game ships no data. The first-run install step copies the archives the
player selects into a local folder, converts them to native assets with
`d2-install`/`d2-native`, and bakes animation frames from the converted
assets. Later runs read only the converted folder.

`d2-data` reads the player's own tables at run time. The tables committed
to this repository contain numeric layout facts and neutral identifiers;
display strings come from the player's files.

## Persistence

Characters are stored in D2RUST's own versioned save format under the
player's D2RUST folder. Settings and key bindings are written there too.
The player's game installation is never modified.

## Boundaries worth keeping

- Game rules belong in `d2-sim`.
- Message formats belong in `d2-proto`.
- File formats belong in `d2-formats`/`d2-native`.
- The client may read the model; it may not decide outcomes.
- No crate outside `d2-client` may depend on Bevy.
- No `unsafe` code.
