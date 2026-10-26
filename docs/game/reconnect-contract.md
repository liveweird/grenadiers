# Archived v4 reconnect contract

Historical v4 reference. Gameplay v11 now serves the endpoint; see the current
[server reconnect guide](reconnect-server.md), [browser reconnect guide](reconnect-browser.md),
and [v11 contract](protocol-v11.md). The v4 schema, generated types and browser conformance remain
archived; the active Rust/browser implementation uses v11.

Gameplay v4 is retained in `api/gameplay-v4.schema.json` and generated
`web/src/game/v4/schema.ts`; there is no current Rust v4 module. At activation the existing
gameplay endpoint and its single owner/runtime used v4, without an alternate endpoint or runtime
selector. Diagnostic v1, classic-v2 physics, and generator-v1 were unchanged. V2 and v3 were
already historical contracts.

A new distinct, redacted ResumeToken is a 64-character lowercase-hex seat capability. Ordinary
session authentication precedes `resumeRoom {sequence, roomId, resumeToken}`. Only personal
`admitted` responses carry it, alongside the acknowledging sequence, room, seat, nullable
lastBombActionId, and nullable full state. Admission has no events field; clients use an empty
event baseline. Shared RoomState contains only room; Snapshot retains room/state/events.

SeatStatus adds a closed connection variant: vacant, connected, disconnected with expiresAt,
or expired. Result carries holdUntil, the earliest eligible restart tick; it may remain in the
past while waiting for return. Command epoch remains independent of actual round count and
latest completed epoch. `seatReplaced {roomId}` stops an old binding's automatic retry;
`resumeRejected` gives uniform capability rejection without exposing matching-seat details.

All object properties are required, unknown fields are rejected, and nullable fields require
explicit null. Authentication/incompatibility inputs retain their bounded numeric negotiation
range while successful authentication is fixed to 4. V4 errors are namespaced separately.
Physical state/event definitions exactly match v3 and use local schema references.

The client frame cap remains 4,096 UTF-8 bytes and the server cap 65,536. Maximal structural
fixtures encode a complete Snapshot to 44,874 bytes and Admitted to 31,444 bytes, including
widest scalar metadata, full arrays, capability, seat and action high-water. These are structural
bounds, not reachable gameplay claims. Oversized encoding is rejected.

Run `npm --prefix web run check:gameplay` for retained-schema generation drift and
`npm --prefix web test -- src/game/v4/conformance.test.ts` for the archived v4 browser contract.
Use `npm --prefix web run gen:gameplay` only when intentionally regenerating retained declarations.
The archived conformance suite covers authentication, reconnect capability handling,
missing/null/extra fields and physical parity. Current Rust publication and reconnect behavior
are covered by v11 and the linked reconnect guides; there is no `protocol::v4` Rust test filter.
