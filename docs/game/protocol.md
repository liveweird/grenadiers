# Gameplay protocol

Status: gameplay contracts, checked domain conversion, HTTP session issuance, and authenticated
gameplay WebSockets with authoritative configurable two-seat matches are implemented.
`api/gameplay-v6.schema.json`, Rust `core::protocol::v6`, and generated
`web/src/game/v6/schema.ts` define active socket messages. V2 through v5 remain historical. V6 preserves reconnect semantics and adds configurable rules; see [the migration note](protocol-v6.md).
The browser validates these messages and drives the playable generated-map match. Diagnostic `/api/v1/ws` and
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
{"type":"authenticate","protocolVersion":6,"token":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}
```

The token is a secret 64-character lowercase hexadecimal credential, held only in tab memory.
It never belongs in a URL or log. A syntactically valid unsupported version is parsed so the
server can return `incompatibleProtocol` before credential binding. Supported gameplay version
is 6, independently of the product release and diagnostic protocol. Parser failures expose
only a generic error; token Debug formatting is redacted.

A successful claim atomically binds one connection. Duplicate claims receive `connectionInUse`
and cannot displace the owner. Unknown/expired/revoked credentials receive `authenticationFailed`.
Closing or canceling the owner revokes its credential; issue a new token for a new connection.
Claimed credentials do not expire under the unclaimed-token timer. Gameplay and diagnostic
sockets share one 128-connection semaphore, including sockets awaiting authentication.

The resume token is a separate 32-byte seat capability encoded as 64 lowercase hexadecimal
characters. It stays only in the owner and admitted tab memory, never in URLs, shared messages,
storage, logs, or replay. Each retry obtains and authenticates a fresh ordinary session before
sending `resumeRoom`; the stable seat capability is not an authentication-session token.

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
or attempts return `staleRoom`. Ready applies in Waiting/initial Countdown; movement/bombs require
Running. Other phases return `invalidPhase`. Fresh sequences are consumed before semantic rejection; replay returns
`staleSequence`. Invalid, unavailable, retired, or closing resume attempts uniformly return
`resumeRejected`. See [room admission](rooms.md#network-admission) for lifetime and queue bounds.

## Commands and responses

All listed fields are required; objects reject extra fields. Counter values are 1–2,147,483,647.

| Client type | Additional fields |
| --- | --- |
| `authenticate` | `protocolVersion` (0–65,535), `token` |
| `createRoom` | `sequence` |
| `joinRoom` | `sequence`, `roomCode` |
| `resumeRoom` | `sequence`, `roomId`, private `resumeToken` |
| `ready` | `sequence`, `roomId`, `roundId`, `settingsRevision`, `ready` |
| `updateSettings` | `sequence`, `roomId`, `roundId`, `settingsRevision`, complete `settings` |
| `movement` | `sequence`, `roomId`, `roundId`, `direction` (cardinal direction or explicit null) |
| `placeBomb` | `sequence`, `roomId`, `roundId`, `actionId` |
| `leaveRoom` | `sequence`, `roomId`, `roundId` |
| `rematch` | `sequence`, `roomId`, `roundId`, `ready` |

| Server type | Additional fields |
| --- | --- |
| `authenticated` | `protocolVersion: 6` |
| `incompatibleProtocol` | `supportedVersion` (0–65,535; this server emits 6) |
| `admitted` | acknowledged `sequence`, shared `room`, `youSeat`, private `resumeToken`, nullable `lastBombActionId`, nullable `state` |
| `roomState` | shared `room` only |
| `seatReplaced` | `roomId`; old binding closes with code 4001 |
| `leftRoom` | `roomId`, acknowledged `sequence` |
| `error` | typed `code`, `sequence` (counter or explicit null) |
| `snapshot` | public `room`, `state`, and `events` |

HTTP session responses are separate from socket messages.

## Public snapshots and events

A snapshot and shared room state are identical for both seats. Only personal `admitted` carries
seat identity, capability, action floor, and an optional full state baseline; it carries no events.
`state` contains `runningUpdates` and `remainingTicks` (each 0–10,800), the arena, exactly two
actors, up to 16 bombs, up to 195 flames, and up to 195 exposed pickups. The arena has width 15, height 13, exactly 195
row-major floor/wall terrain entries, and up to 195 unique soft-block cells.

Actors expose ID, seat, fixed-point position, heading, speed level (0–4), alive status,
capacity/range (1–8), and live-bomb count (0–8). Bombs include ID, cell, owner, captured range,
placement tick, and detonation tick. Flames include cell and exclusive expiry tick. Room metadata carries the global tick, rules/map versions, scores, actual round number,
latest completed round, and independent match outcome; these are not duplicated in game state.
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
phase order. Placement and detonation within the same tick remain observable. Result and MatchResult hold physics completely frozen while the room clock advances.
The final partial report window is published once; later hold snapshots contain no repeated events.
Conversion neither clamps values nor advances the simulation.

The runtime checks Running physics tick against the room tick. A completed frozen world
must match its own latest completed-round epoch/result/end tick. An unfinished terminal
world keeps no round result and freezes at the independent match outcome tick. A completed
hold followed by forfeit still binds physics to the earlier completed-round tick. Inter-round
countdowns publish room views without an old world. The encoder enforces the frame limit;
these adapters do not create rooms or validate transport ownership.

## Identity, ordering, and lifecycle

Room IDs are 32 lowercase hex characters and identify a room instance. Join codes are public,
eight canonical characters from `A-HJ-NP-Z2-9`. Neither proves seat ownership. The authenticated
server binding supplies the seat; no client command includes a seat, position, damage, or result.

Sequence numbers increase across create/join/resume/leave and gameplay commands on one connection.
A structurally valid command consumes its sequence before semantic handling. Bomb action IDs
increase independently per seat/round and remain consumed after an unsuccessful placement.
Invalid room/round bindings cannot mutate another seat's high-water marks. Resume reports the
accepted floor; the next bomb ID must be greater. A floor of 2,147,483,647 exhausts bomb actions
for that epoch.

`RoomView` carries identity/code, independent command epoch `roundId`, revision, global tick,
rules/map versions, actual `roundNumber`, scores (0..9 per seat, bounded by the selected target), nullable `lastRound`, two
canonical seat statuses (connection, occupancy/readiness/rematch consent), and a tagged phase. A
connection is vacant, connected, disconnected with exclusive `expiresAt`, or expired. Actual
round count increases only on starts, remains on the preceding round during inter-round
countdown, and resets on rematch. Epochs also advance across cancellation and rematch, so
spawn/tie assignment uses actual round count rather than command epoch.

Shared room views and snapshot metadata also carry the complete checked `settings`,
`settingsRevision` and nullable original `settingsEditor` seat. Only actual changes increment
the settings revision and clear readiness; Ready commands acknowledge that exact revision.
See [v6 settings and authority](protocol-v6.md) for bounds, no-op acknowledgements and edit locks.

Phases are Waiting; initial/inter-round Countdown with deadline; Running; nonfinal Result
with `holdUntil`; MatchResult with independent outcome, end time, and `rematchAt`;
or prematch Interrupted with reason/end time. Holds last 180 ticks, followed by a 180-tick
inter-round countdown. Draws award no score. Last-round records persist across later rounds,
forfeits, and interruptions; rematch clears them. See [v4 field reference](protocol-v4.md).

Two occupied players may set/withdraw rematch votes after the final hold. Consensus advances
one epoch and returns occupied, unready Waiting without changing transport sequence or room
revision ownership. Countdown cancellation advances epoch without consuming an actual round
or seed. A newer-epoch snapshot can be accepted directly when an intermediate room view is
coalesced. Old room/epoch/revision data cannot overwrite newer state.

Leaving acknowledges removal while retaining an authenticated unseated socket. A rejected leave
retains the seat capability; acknowledged leave clears it. If transport is lost while leave is
pending, the browser honors the explicit local departure instead of retrying. Running
boundaries commit physics/scoring before batched departures; idle phases apply departure
before automatic transitions. Explicit leave forfeits and two explicit leaves abandon. Eligible
transport loss reserves the seat for 600 global ticks; one expiry forfeits to an eligible opponent,
both expiry abandons, and a disconnected opponent delays the decision until settled. Reaching the
configured win target and already final outcomes take precedence. Ownership is bound to connection generation and private
capability, never to invitation code. See [bounded owner](match-owner.md).

## Profile and version evolution

An immutable server room profile owns state construction and its rules/map labels. Publication
rejects state whose checked rules version disagrees with that profile. Production now uses configurable-v4 rules and generated-v1 arenas. Snapshots initially have an
empty pickup array because contents are concealed; only exposed items become public.
Fixed-classic-v1 remains an explicit unit fixture, not a production selector.

Gameplay v6 is incompatible with earlier versions. Reload clients when upgrading the single server. Both
old and future versions are rejected before credential lookup, so rejection cannot claim or
revoke a token. `supportedVersion` accepts the bounded input range rather than only v6,
allowing a new client to understand an older server's incompatibility response.
See [ADR 0013](../decisions/0013-gameplay-v2-and-room-profiles.md).

## Contract checks

`npm --prefix web run gen:gameplay` regenerates gameplay types; `check:gameplay` detects drift
without writing. The shared web gate runs this alongside existing diagnostic checks and AJV
validation tests. Rust conformance tests compare strict parsing and serialization with the same
schema, including missing/null fields, scalar boundaries, integer-valued number forms, version
rejection responses, credential redaction, all event variants, nested bounds, and real
frame-size enforcement. Maximal structural v6 Snapshot and Admitted fixtures fit the inclusive
65,536-byte server limit; an actual oversized message is rejected. These fixtures prove structural
bounds rather than reachable gameplay size. TypeScript types do not enforce string patterns
or numeric/array bounds at runtime; AJV and Rust validation do.

## Graceful shutdown

A sticky stop signal bypasses command queues. Owners interrupt unfinished matches with
ServerStopped, atomically settle pending departures, and retain one final checked publication
with the closing reason. Existing final matches remain final. Members receive that publication
before close; unseated and authenticating sockets close directly. Delivery is best effort
within a five-second bound. The process explicitly drains upgraded game/diagnostic sessions
and room cleanup before runtime exit; HTTP graceful shutdown alone does not await upgrades.
