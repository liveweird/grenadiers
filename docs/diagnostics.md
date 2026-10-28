# Realtime diagnostics

The diagnostic WebSocket at `/api/v1/ws` uses the current contract in
`api/realtime.schema.json`. Its v2 welcome is exactly `protocolVersion: 2` and
`maxPlayers: 4`. The player count describes match capacity; it is not the transport
connection limit, which remains independently enforced by server admission controls.

The previous closed v1 contract is retained at `api/realtime-v1.schema.json` with its
original `protocolVersion: 1` and `maxPlayers: 2` literals. It is an archive for compatibility
tests and generated historical types, not an additional production validator or negotiated
runtime protocol.

After editing either schema, run:

```sh
npm --prefix web run gen:realtime
npm --prefix web run gen:validators
npm --prefix web run check:realtime
npm --prefix web run check:validators
```
