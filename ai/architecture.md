# Architecture map

The game is one process. `d2-client` opens a window and runs the front end;
when a game starts it builds an in-process `d2-server`, which owns the
`d2-sim` world. The two sides exchange `d2-proto` messages over
`d2-net::memory`.

```text
d2-formats ──► d2-data ──► d2-sim ──► d2-server ──► d2-client
                   │           │           │
                   └──► d2-native ──► d2-install
```

- `d2-formats` parses the file formats. It knows bytes, not game rules.
- `d2-data` reads the player's tables into typed records.
- `d2-sim` is the game. It is deterministic and has no I/O.
- `d2-proto` defines the messages. It has no behaviour.
- `d2-net` carries whole messages in order. Single player uses the in-memory
  transport.
- `d2-server` owns the authoritative game, validates intents, and runs one
  tick per frame.
- `d2-native` defines the converted asset formats and their round trips.
- `d2-install` copies the player's archives, converts them and orchestrates
  the install.
- `d2-client` is the executable: Bevy rendering, UI, input, audio, the
  bridge and the world view.

## Where to look

| Task | Start here |
|---|---|
| a game rule | `crates/d2-sim/src` |
| a protocol message | `crates/d2-proto/src` and `tables/sim/*-messages.tsv` |
| a file format | `crates/d2-formats/src` |
| a table field | `crates/d2-data/src` and `tables/data/fields.tsv` |
| a server handler | `crates/d2-server/src/adapters/handlers` |
| a screen or panel | `crates/d2-client/src/ui` |
| the world view | `crates/d2-client/src/world_view` |
| the install flow | `crates/d2-client/src/install.rs`, `crates/d2-install` |

## The tick

`d2-server` calls `d2-sim` once per tick. The client's bridge receives the
resulting messages, updates `ClientWorld`, and Bevy systems mirror that
model. Rendering never changes the model.

Do not introduce a second source of truth for game state in the client.
