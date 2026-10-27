# Gameplay v12: solo controllers

This Standard-only wire contract is archived. The [live v13 guide](protocol-v13.md) adds
selectable difficulty; v12 clients must reload for a current server.

The paired contract is `api/gameplay-v12.schema.json`, Rust `core::protocol::v12`, and generated
`web/src/game/v12/schema.ts`. Deploy server and browser together; older clients must reload.
V12 retains the [v11 physical state, items and recipient privacy](protocol-v11.md), Expanded-v5
rules and generated-v2 arenas. Diagnostic protocol version 1 is independent.

## Creation and room metadata

`createRoom` requires `sequence` and `mode`:

```json
{"type":"createRoom","sequence":1,"mode":{"type":"solo","difficulty":"standard"}}
```

Use `{"type":"humanDuel"}` for two humans. Objects remain closed: clients cannot supply a
CPU seat, policy version, seed, input authority or arbitrary difficulty. Room mode is immutable.

Every `RoomView` includes that mode and a `controller` on each seat. A human controller is
`{"type":"human"}`; the CPU is `{"type":"cpu","difficulty":"standard"}`. Solo fixes human
seat One and occupied CPU seat Two. Only the CPU uses `connection: {"type":"serverControlled"}`;
human connection/reservation variants retain their existing meanings.

The human owns settings while connected or reserved. A closing publication may show the human
vacant/expired with no editor before the owner destroys the room. CPU occupancy does not retain
a room by itself. Personal admissions target human seats only; solo never admits seat Two.
Public-code joins to solo are rejected. CPU controllers receive no network credentials or
resume capability. See the [solo behavior](single-player.md).

## Shared validation and publication

The v12 checked wrapper validates controller metadata, normalizes CPU connection state only for
shared lifecycle validation, and retains checked v11 physical facts. The wire still publishes
`serverControlled`. Null-body admissions pass the same room/profile checks as snapshots.

Recipient projection reuses the existing privacy filters. Cloak, Invisible Bombs, private Oil,
hidden pickups, effect timers and inventory have unchanged visibility. CPU observations use
that same checked projection; they never receive the authoritative `State` or arena seed.
Archived v11 schemas reject v12 metadata and remain closed.
