# Gameplay protocol

Status: gameplay contracts, checked domain conversion, HTTP session issuance, and authenticated
gameplay WebSockets with authoritative two-seat rounds are implemented. `api/gameplay.schema.json`, independent Rust
`core::protocol` types, and generated `web/src/game/schema.ts` define socket messages.
The browser validates these messages and drives the playable fixed-map duel. Diagnostic `/api/v1/ws` and
`api/realtime.schema.json` remain unchanged.

## Authentication and compatibility

`POST /api/v1/game/session` issues an anonymous credential. Send exactly one configured
`Origin` header and no query parameters; no request body is needed. The 201 response is a
closed JSON object `{"token":"<64 lowercase hex characters>"}` with `Cache-Control: no-store`.
No cookie or Location is set. An unclaimed credential expires after 300 seconds and grants no
room or seat. Restart discards all credentials.

The registry is bounded to 128 issued or claimed credentials, with a global burst of 32 and refill of eight
per second. Disallowed/missing/duplicate or null Origin returns 403; a nonempty query returns 400;
rate exhaustion returns 429 with `Retry-After: 1`; capacity/entropy failure returns a generic
503 problem. Every handler response disables caching. HTTP tracing records method/path,
excluding query strings. The operation and failures are defined in `api/openapi.json`.

Connect to `/api/v1/game/ws` with one configured non-null Origin and no query parameters.
Its first application frame must be:

```json
{"type":"authenticate","protocolVersion":2,"token":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}
```

The token is a secret 64-character lowercase hexadecimal credential, held only in tab memory.
It never belongs in a URL or log. A syntactically valid unsupported version is parsed so the
server can return `incompatibleProtocol` before credential binding. Supported gameplay version
is 2, independently of the product release and diagnostic protocol. Parser failures expose
only a generic error; token Debug formatting is redacted.

A successful claim atomically binds one connection. Duplicate claims receive `connectionInUse`
and cannot displace the owner. Unknown/expired/revoked credentials receive `authenticationFailed`.
Closing or canceling the owner revokes its credential; issue a new token for a new connection.
Claimed credentials do not expire under the unclaimed-token timer. Gameplay and diagnostic
sockets share one 128-connection semaphore, including sockets awaiting authentication.

Authentication has an absolute five-second deadline; Ping/Pong cannot extend it. Upgrades use
a global 32-token burst with eight-per-second refill. Each socket admits a burst of 30 inbound
messages with 60-per-second refill, including control frames. Client frames/messages are at
most 4 KiB; encoded server messages are at most 64 KiB with finite write buffering. Sends have
a five-second deadline. Server Ping runs every ten seconds, no inbound activity for 30 seconds
closes the socket, and unseated authentication expires after an absolute five minutes.

Malformed/binary input, failed authentication, repeated authentication, or rate exhaustion
closes the offending socket with fixed reasons. Create/join now bind one of two room seats;
already-bound admission returns `alreadyInRoom`, full rooms return `roomFull`, and unknown
codes return `roomNotFound`. Unbound room commands return `notInRoom`; mismatched ownership
or attempts return `staleRoom`. Ready applies in Waiting/Countdown; movement/bombs require
Running. Other phases return `invalidPhase`. Fresh sequences are consumed before semantic rejection; replay returns
`staleSequence`. See [room admission](rooms.md#network-admission) for lifetime and queue bounds.

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
| `authenticated` | `protocolVersion: 2` |
| `incompatibleProtocol` | `supportedVersion` (0–65,535; this server emits 2) |
| `roomState` | `room`, personalized `youSeat` |
| `leftRoom` | `roomId`, acknowledged `sequence` |
| `error` | typed `code`, `sequence` (counter or explicit null) |
| `snapshot` | public `room`, `state`, and `events` |

HTTP session responses are separate from socket messages.

## Public snapshots and events

A snapshot is identical for both seats; only admission `roomState` includes `youSeat`.
`state` contains `runningUpdates` and `remainingTicks` (each 0–10,800), the arena, exactly two
actors, up to 16 bombs, up to 195 flames, and up to 195 exposed pickups. The arena has width 15, height 13, exactly 195
row-major floor/wall terrain entries, and up to 195 unique soft-block cells.

Actors expose ID, seat, fixed-point position, heading, speed level (0–4), alive status,
capacity/range (1–8), and live-bomb count (0–8). Bombs include ID, cell, owner, captured range,
placement tick, and detonation tick. Flames include cell and exclusive expiry tick. Room metadata
carries tick, rules/map versions, and outcome; these are not duplicated in game state.
The required `pickups` array contains closed `{cell, kind}` entries with kind `bombCapacity`,
`blastRange`, or `movementSpeed`. Empty means no exposed items. Concealed and pending items,
round assignments, generation records, and seeds never enter these messages.
Private held input, movement fractions, collision grace, allocator state, and hidden contents
are excluded. The checked domain adapter enforces canonical ordering and cross-field invariants before
runtime publication. Pickups must be unique, row-major, on floor without blocks or flames;
bomb and actor overlap is permitted.

Each snapshot carries 0–64 events with absolute `tick`, per-tick `index` (0–63), and tagged
`kind`: `bombPlaced`/`bombDetonated` carry a full bomb record; `actorEliminated` carries ID/seat;
`pickupCollected` carries collector ID/seat, cell, and kind; `roundEnded` carries the result. Identity is `(roomId, roundId, tick, index)`. Placement and
explosion within one update remain separate events. Events are best-effort presentation cues;
full snapshots remain authoritative. The 20 Hz publisher accumulates at most three
completed 60 Hz updates per normal snapshot.

## Checked domain conversion

Use `core::protocol::project_state(&public_state)` to obtain a `ProjectedRound` containing
checked absolute tick, supported rules version, optional wire outcome, and game state. It validates the fixed arena
dimensions, clock sum, seat/ID order, positions, upgrades, unique occupied cells, bomb owners,
actual live-bomb counts, and outcome/liveness consistency. Conversion errors contain fixed
categories rather than payloads. No direct serialization of internal `World` or `State` exists.

`project_events(&reports)` accepts at most three consecutive completed updates and 64 events.
Each event must belong to its report tick; indices restart at zero per tick and preserve actual
phase order. Placement and detonation within the same tick remain observable. Result states
freeze world contents while the absolute clock advances, so old bomb/flame deadlines remain
valid in frozen projections. Conversion neither clamps values nor advances the simulation.

The room publisher combines these checked values with matching room tick/phase, applies
the encoder's frame limit, and bounds delivery. The adapter alone does not create rooms
or validate transport ownership.

## Identity, ordering, and lifecycle

Room IDs are 32 lowercase hex characters and identify a room instance. Join codes are public,
eight canonical characters from `A-HJ-NP-Z2-9`. Neither proves seat ownership. The authenticated
server binding supplies the seat; no client command includes a seat, position, damage, or result.

Sequence numbers increase across create/join/leave and gameplay commands on one connection.
A structurally valid command consumes its sequence before semantic handling. Bomb action IDs
increase independently per seat/round and remain consumed after an unsuccessful placement.
Invalid room/round bindings cannot mutate another seat's high-water marks.

`RoomView` carries identity/code, round attempt, revision, absolute tick, rules version (1 or 2),
map identity (`fixed_v1` or `generated_v1`), two canonical seat statuses, and a tagged phase. Phases are Waiting,
Countdown with deadline, Running, Result with fixed outcome/time, or Interrupted with reason/time.
An interrupted preview has no fabricated winner. Ticks are nonnegative JavaScript-safe integers.

The runtime cancels countdown on unready/departure, clears readiness, and advances the
round-attempt ID. Leaving acknowledges removal while retaining an authenticated unseated
socket. Both same-socket rejoining and replacement sockets invalidate old queued bindings.
Public revisions order state across outbound channels; admission precedes a recipient's first
snapshot. See [ADR 0008](../decisions/0008-gameplay-protocol.md).

## Profile and version evolution

An immutable server room profile owns state construction and its rules/map labels. Publication
rejects state whose checked rules version disagrees with that profile. The current profile
still uses classic-v1 rules, the fixed map, and an empty pickup array; schema support for
classic-v2/generated-v1 does not activate generated rounds.

Gameplay v2 is incompatible with v1. Reload clients when upgrading the single server. Both
old and future versions are rejected before credential lookup, so rejection cannot claim or
revoke a token. `supportedVersion` accepts the bounded input range rather than only v2,
allowing a new client to understand an older server's incompatibility response.
See [ADR 0013](../decisions/0013-gameplay-v2-and-room-profiles.md).

## Contract checks

`npm --prefix web run gen:gameplay` regenerates gameplay types; `check:gameplay` detects drift
without writing. The shared web gate runs this alongside existing diagnostic checks and AJV
validation tests. Rust conformance tests compare strict parsing and serialization with the same
schema, including missing/null fields, scalar boundaries, integer-valued number forms, version
rejection responses, credential redaction, all event variants, nested bounds, and real
frame-size enforcement. A maximal structural v2 snapshot encodes to 44,466 UTF-8 bytes against
the inclusive 65,536-byte server limit; an actual oversized snapshot is rejected. TypeScript types do not enforce string patterns
or numeric/array bounds at runtime; AJV and Rust validation do.
