# Clean room

D2RUST is our own Rust code. It runs on the player's own copy of the
original game and contains none of the publisher's code, assets or text.
This is a process note, not legal advice.

## Code

- All code is written by us, in our own words and structure.
- Decompiler output, assembly and near-verbatim C are never pasted or
  line-translated into this repository, in code or in comments.
- No third-party source is adapted. Dependencies come from crates.io; see
  `THIRD_PARTY_NOTICES.md`.
- Comments explain what D2RUST does and why, in game terms. They do not
  cite executable addresses, version-specific builds, private specifications
  or an internal reverse-engineering record.

## Data

- `tables/` holds small numeric tables the game logic uses: ids, counts,
  frame and layout values, function indices, flags and state numbers. They
  are measured facts, not the publisher's files.
- Display names are being replaced with neutral labels (`quest-0`,
  `waypoint-1`, `npc-146`, ...). Some development tables still carry an
  original label; those are known cleanup items, not publisher data.
- Some tables still carry executable addresses because the current
  dispatch uses them as opaque keys. Removing them needs the code and the
  table key to move to a neutral id together; that work is not complete.
  They are identifiers only, never executable content.
- Everything else (data archives, executables, saves, extracted assets,
  full text or binary tables) comes from the player's own installation at
  run time and is never committed or shipped.
- The install step reads the player's files, converts them locally and
  writes the result under the player's D2RUST folder. Nothing is uploaded.

## Release

A release contains our source code, our own build scripts and documentation,
and nothing from the game. No baked animation pack is published; baking
happens on the player's machine during install.
