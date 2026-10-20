# Clean-room rules for agents

D2RUST is a from-scratch reimplementation. The repository is public and
must remain free of material from the original game.

## Never add

- game data, archives, executables, saves, extracted sprites, sounds,
  fonts, text tables or baked animation packs;
- original level, item, monster, NPC, quest or skill display names in
  code, comments, tests or tables;
- decompiler output, assembly, copied source or a line-by-line translation;
- executable addresses, build-specific version numbers, private spec
  citations or paths into a private repository;
- screenshots or logs that contain game content or personal paths.

## Always

- describe behaviour in game terms and neutral identifiers;
- keep display strings out of the repository; read them from the player's
  files at run time;
- use synthetic values in tests;
- leave the install step as the only code that reads the player's files,
  and keep it local;
- report a suspected leak instead of working around it.

## When a task seems to need game data

Do not check in a fixture. Either build a synthetic equivalent, or make the
code path testable without the data. If neither is possible, explain the
limitation in the change description rather than adding the file.

## Names

Use neutral labels such as `quest-0`, `waypoint-1`, `npc-146`, `monster-3`,
`item-12`. Do not use a name from the game as an identifier, a test name or
a comment example.
