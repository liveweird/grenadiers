# Gameplay protocol

Status: control/lifecycle, snapshot/event contracts, and checked domain conversion. `api/gameplay.schema.json`, independent Rust
`core::protocol` types, and generated `web/src/game/schema.ts` define messages; gameplay
endpoints, room handlers, and browser controls remain planned.
Diagnostic `/api/v1/ws` and `api/realtime.schema.json` remain unchanged.

## Authentication and compatibility

The planned gameplay socket is `/api/v1/game/ws`; session issuance will arrive with its HTTP
handler and OpenAPI operation. The first application frame is:

```json
{"type":"authenticate","protocolVersion":1,"token":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}
```

The token is a secret 64-character lowercase hexadecimal credential, held only in tab memory.
It never belongs in a URL or log. A syntactically valid unsupported version is parsed so the
server can return `incompatibleProtocol` before credential binding. Supported gameplay version
is 1, independently of the product release and diagnostic protocol. Parser failures expose
only a generic error; token Debug formatting is redacted.

## Commands and responses

All listed fields are required; objects reject extra fields. Counter values are 1–2,147,483,647.

| Client type | Additional fields |
| --- | --- |
| `authenticate` | `protocolVersion` (0–65,535), `token` |
| `createRoom` | `sequence` |
| `joinRoom` | `sequence`, `roomCode` |
| `ready` | `sequence`, `roomId`, `roundId`, `ready` |
| `movement` | `sequence`, `roomId`, `roundId`, `direction` (cardinal direction or explicit null) |
| `placeBomb` | `sequence`, `roomId`, `roundId`, `actionId` |
| `leaveRoom` | `sequence`, `roomId`, `roundId` |

| Server type | Additional fields |
| --- | --- |
| `authenticated` | `protocolVersion: 1` |
| `incompatibleProtocol` | `supportedVersion: 1` |
| `roomState` | `room`, personalized `youSeat` |
| `leftRoom` | `roomId`, acknowledged `sequence` |
| `error` | typed `code`, `sequence` (counter or explicit null) |
| `snapshot` | public `room`, `state`, and `events` |

HTTP session responses are separate from socket messages.

## Public snapshots and events

A snapshot is identical for both seats; only admission `roomState` includes `youSeat`.
`state` contains `runningUpdates` and `remainingTicks` (each 0–10,800), the arena, exactly two
actors, up to 16 bombs, and up to 195 flames. The arena has width 15, height 13, exactly 195
row-major floor/wall terrain entries, and up to 195 unique soft-block cells.

Actors expose ID, seat, fixed-point position, heading, speed level (0–4), alive status,
capacity/range (1–8), and live-bomb count (0–8). Bombs include ID, cell, owner, captured range,
placement tick, and detonation tick. Flames include cell and exclusive expiry tick. Room metadata
carries tick, rules/map versions, and outcome; these are not duplicated in game state.
Private held input, movement fractions, collision grace, allocator state, and hidden contents
are excluded. The checked domain adapter enforces canonical ordering and cross-field invariants before
runtime publication.

Each snapshot carries 0–64 events with absolute `tick`, per-tick `index` (0–63), and tagged
`kind`: `bombPlaced`/`bombDetonated` carry a full bomb record; `actorEliminated` carries ID/seat;
`roundEnded` carries the result. Identity is `(roomId, roundId, tick, index)`. Placement and
explosion within one update remain separate events. Events are best-effort presentation cues;
full snapshots remain authoritative. The planned 20 Hz publisher accumulates at most three
completed 60 Hz updates per normal snapshot.

## Checked domain conversion

Use `core::protocol::project_state(&public_state)` to obtain a `ProjectedRound` containing
checked absolute tick, optional wire outcome, and game state. It validates the fixed rules/map
dimensions, clock sum, seat/ID order, positions, upgrades, unique occupied cells, bomb owners,
actual live-bomb counts, and outcome/liveness consistency. Conversion errors contain fixed
categories rather than payloads. No direct serialization of internal `World` or `State` exists.

`project_events(&reports)` accepts at most three consecutive completed updates and 64 events.
Each event must belong to its report tick; indices restart at zero per tick and preserve actual
phase order. Placement and detonation within the same tick remain observable. Result states
freeze world contents while the absolute clock advances, so old bomb/flame deadlines remain
valid in frozen projections. Conversion neither clamps values nor advances the simulation.

The future room publisher must combine these checked values with matching room tick/phase,
apply the encoder's frame limit, and bound delivery. The adapter alone does not create rooms
or validate transport ownership.

## Identity, ordering, and lifecycle

Room IDs are 32 lowercase hex characters and identify a room instance. Join codes are public,
eight canonical characters from `A-HJ-NP-Z2-9`. Neither proves seat ownership. The authenticated
server binding supplies the seat; no client command includes a seat, position, damage, or result.

Sequence numbers increase across create/join/leave and gameplay commands on one connection.
A structurally valid command consumes its sequence before semantic handling. Bomb action IDs
increase independently per seat/round and remain consumed after an unsuccessful placement.
Invalid room/round bindings cannot mutate another seat's high-water marks.

`RoomView` carries identity/code, round attempt, revision, absolute tick, rules version 1,
`fixed_v1` map identity, two canonical seat statuses, and a tagged phase. Phases are Waiting,
Countdown with deadline, Running, Result with fixed outcome/time, or Interrupted with reason/time.
An interrupted preview has no fabricated winner. Ticks are nonnegative JavaScript-safe integers.

The target runtime cancels countdown on unready/departure, clears readiness, and advances the
round-attempt ID. Leaving acknowledges removal while retaining an authenticated unseated
socket. Both same-socket rejoining and replacement sockets invalidate old queued bindings.
Public revisions order state across outbound channels; admission precedes a recipient's first
snapshot. See [ADR 0008](../decisions/0008-gameplay-protocol.md).

## Contract checks

`npm --prefix web run gen:gameplay` regenerates gameplay types; `check:gameplay` detects drift
without writing. The shared web gate runs this alongside existing diagnostic checks and AJV
validation tests. Rust conformance tests compare strict parsing and serialization with the same
schema, including missing/null fields, scalar boundaries, integer-valued number forms, version
rejection responses, credential redaction, all event variants, nested bounds, and real
frame-size enforcement. A maximal structural snapshot encodes to 33,989 UTF-8 bytes against
the inclusive 65,536-byte server limit; an actual oversized snapshot is rejected. TypeScript types do not enforce string patterns
or numeric/array bounds at runtime; AJV and Rust validation do.
