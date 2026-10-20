# Crate boundaries

The dependency direction is fixed:

```text
d2-formats
d2-data
d2-native      (depends on formats and data)
d2-sim         (depends on data and formats)
d2-proto
d2-net         (depends on proto)
d2-server      (depends on sim, data, formats, native, proto, net)
d2-install     (depends on data, formats, native)
d2-client      (depends on everything above; the only Bevy crate)
```

Rules:

- A crate may depend only on crates below it in that list.
- Only `d2-client` depends on Bevy.
- `d2-sim` performs no I/O: no file reads, no network, no wall clock, no
  environment variables.
- `d2-proto` contains data and encoding/decoding, not game rules.
- `d2-formats` does not know about game concepts; `d2-data` maps bytes to
  typed records.
- `d2-server` validates and sequences; it does not duplicate game formulas.
- `d2-install` may read and write files; it must not depend on the client.
- New crates need a clear reason. Prefer adding to an existing crate when
  the responsibility already belongs there.

If a change seems to need an upward dependency, move the shared type down
instead.
