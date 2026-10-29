# Gameplay protocol

Status: gameplay contracts, checked domain conversion, HTTP session issuance, and authenticated
gameplay WebSockets with authoritative configurable two-to-four-seat matches are implemented.
`api/gameplay-v15.schema.json`, Rust `core::protocol::v15`, and generated
`web/src/game/v15/schema.ts` define active socket messages. V2 through v14 remain historical
or archived compatibility contracts.
V14 retains the v11 physical/privacy model while adding a checked two-to-four-controller room
configuration. See the [v14 contract](protocol-v14.md) and [v13 compatibility](protocol-v13.md).
See [ADR 0021](../decisions/0021-gameplay-retention.md) for the distinction between live support,
archived wire contracts and private replay retention.
The browser validates these messages and drives the playable generated-map match. Diagnostic
`/api/v1/ws` separately uses diagnostic protocol v2 in `api/realtime.schema.json`.

## Protocol version guide

| Version | Guide | Network support |
| --- | --- | --- |
| v2 | [ADR 0013](../decisions/0013-gameplay-v2-and-room-profiles.md) (no standalone guide) | Archived contract |
| v3 | [Historical guide](../history/protocols/protocol-v3.md) | Archived contract |
| v4 | [Historical guide](../history/protocols/protocol-v4.md) | Archived contract |
| v5 | [Historical guide](../history/protocols/protocol-v5.md) | Archived contract |
| v6 | [Historical guide](../history/protocols/protocol-v6.md) | Archived contract |
| v7 | [Historical guide](../history/protocols/protocol-v7.md) | Archived contract |
| v8 | [Historical guide](../history/protocols/protocol-v8.md) | Archived contract |
| v9 | [Historical guide](../history/protocols/protocol-v9.md) | Archived contract |
| v10 | [Historical guide](../history/protocols/protocol-v10.md) | Archived contract |
| v11 | [Physical compatibility guide](protocol-v11.md) | Archived contract |
| v12 | [Standard-only solo guide](protocol-v12.md) | Archived contract |
| v13 | [Compatibility guide](protocol-v13.md) | Archived contract |
| v14 | [Archived multiplayer contract](protocol-v14.md) | Archived/rejected before credential claim |
| v15 | [Environment contract](protocol-v15.md) | Live multiplayer contract |

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
{"type":"authenticate","protocolVersion":14,"token":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}
```

The token is a secret 64-character lowercase hexadecimal credential, held only in tab memory.
It never belongs in a URL or log. A syntactically valid unsupported version is parsed so the
server can return `incompatibleProtocol` before credential binding. Supported gameplay version
is 14, independently of the product release and diagnostic protocol. Parser failures expose
only a generic error; token Debug formatting is redacted.

A successful claim atomically binds one connection. Duplicate claims receive `connectionInUse`
and cannot displace the owner. Unknown/expired/revoked credentials receive `authenticationFailed`.
Closing or canceling the owner revokes its credential; issue a new token for a new connection.
Claimed credentials do not expire under the unclaimed-token timer. Gameplay sockets have their
own 128-connection budget, including sockets awaiting authentication; diagnostic connections use
an independent 16-connection budget.

The resume token is a separate 32-byte seat capability encoded as 64 lowercase hexadecimal
characters. It stays only in the owner and admitted tab memory, never in URLs, shared messages,
storage, logs, or replay. Each retry obtains and authenticates a fresh ordinary session before
sending `resumeRoom`; the stable seat capability is not an authentication-session token.

Authentication has an absolute five-second deadline; Ping/Pong cannot extend it. Upgrades use
a global 32-token burst with eight-per-second refill. Each socket admits a burst of 30 inbound
messages with 60-per-second refill, including control frames. Client frames/messages are at
most 4 KiB; encoded v14 server messages are at most 96 KiB with finite write buffering. Sends have
a five-second deadline. Server Ping runs every ten seconds, no inbound activity for 30 seconds
closes the socket, and unseated authentication expires after an absolute five minutes.

Malformed/binary input, failed authentication, repeated authentication, or rate exhaustion
closes the offending socket with fixed reasons. Create/join bind a configured human seat in a
two-to-four-seat room; CPU seats remain server-owned and at least one human is required.
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
| `createRoom` | `sequence`, complete `configuration` (two to four controllers, creator human, CPU difficulty or null, board) |
| `joinRoom` | `sequence`, `roomCode` |
| `resumeRoom` | `sequence`, `roomId`, private `resumeToken` |
| `ready` | `sequence`, `roomId`, `roundId`, `settingsRevision`, `ready` |
| `updateSettings` | `sequence`, `roomId`, `roundId`, `settingsRevision`, complete `settings` |
| `movement` | `sequence`, `roomId`, `roundId`, `direction` (cardinal direction or explicit null) |
| `manualAction` | `sequence`, `roomId`, `roundId`, `actionId`, `action` (`bomb` or active Punch `equipment`) |
| `leaveRoom` | `sequence`, `roomId`, `roundId` |
| `rematch` | `sequence`, `roomId`, `roundId`, `ready` |

| Server type | Additional fields |
| --- | --- |
| `authenticated` | `protocolVersion: 14` |
| `incompatibleProtocol` | `supportedVersion` (0–65,535; this server emits 14) |
| `admitted` | acknowledged `sequence`, shared `room`, `youSeat`, private `resumeToken`, nullable `lastActionId`, nullable `state` |
| `roomState` | shared `room` only |
| `seatReplaced` | `roomId`; old binding closes with code 4001 |
| `leftRoom` | `roomId`, acknowledged `sequence` |
| `error` | typed `code`, `sequence` (counter or explicit null) |
| `snapshot` | public `room`, `state`, and `events` |

HTTP session responses are separate from socket messages.

## Public snapshots and events

Room metadata is shared; physical snapshots are prepared separately for each authenticated seat.
The default all-items enabled set still uses recipient-specific projections. Personal
`admitted` carries seat identity, capability, action floor and an optional recipient baseline,
without historical events. `state` contains `physicalTick`, `runningUpdates` and `remainingTicks`
(each running counter 0–10,800), the arena, exactly the configured two to four actors, up to 32
bombs, up to 195 finite flames, and up to 195 exposed pickups. Finite flames and
burning Oil together occupy at most 195 distinct cells. The arena has width 15, height 13, exactly 195
row-major floor/wall terrain entries, and up to 195 unique soft-block cells.

Visible actors expose ID, seat, position, heading, speed (0–4), alive status, capacity/range
(1–8), authoritative owned-bomb count, inventory, effects, remaining damage grace and the required
`voluntaryActionsFrozen` boolean, derived from the living opponent's active Stopwatch, plus nullable
`lastTeleportTick` shared by both visible actors after an authoritative swap. Hidden
actors contain only ID, seat and alive status; owners always receive their own visible actor.
Bombs carry ID, cell, owner, range, placement tick, blast mode and a typed trigger. Timed
triggers report remaining ticks at `physicalTick` and pause state; no untimed deadline is invented.
Timed bombs may pause only with enabled Stopwatch and validated private captured-fuse history.
Enabled Piercing and Power bombs retain their special blasts. Enabled Remote and Contact bombs use
untimed Standard triggers with no invented fuse.
Flames include cell and exclusive expiry tick.
The required sorted `oilBarrels` array contains at most eight materialized arena objects in `unlit`
or `burning` state. Burning Oil is persistent round fire and remains separate from finite flames.
Concealed/pending locations and typed Oil grace never enter recipient frames.
Room metadata carries the global tick, rules/map versions, scores, actual round number,
latest completed round, and independent match outcome; these are not duplicated in game state.
The required `pickups` array contains closed `{cell, kind}` entries for ordinary upgrades or an
enabled optional item. Empty means no exposed items. Concealed and pending items,
round assignments, generation records, and seeds never enter these messages.
Private held input, movement fractions, bomb-exit grace, allocator state, and hidden contents
are excluded. The checked domain adapter enforces canonical ordering and cross-field invariants before
runtime publication. Pickups must be unique, row-major, on floor without blocks or flames;
bomb and actor overlap is permitted.

Each snapshot carries 0–192 events with absolute `tick`, per-tick `index` (0–191), and tagged
`kind`: `bombPlaced`/`bombDetonated` carry a full bomb record; `actorEliminated` carries ID/seat;
`pickupCollected` carries collector ID/seat, cell, and kind; `roundEnded` carries the result.
`damagePrevented` reports a consumed Heart with source `heart`. Identity is
`(roomId, roundId, tick, index)`. Placement and
explosion within one update remain separate events. Events are best-effort presentation cues;
full snapshots remain authoritative. The 20 Hz publisher accumulates at most three
completed 60 Hz updates per normal snapshot.

## Checked domain conversion

Use `core::protocol::v14::capture_expanded` to observe the complete State and reports, then
`bind_expanded` to certify the room profile, configuration, settings, lifecycle epoch, physical tick and
trusted round generation record.
Opaque captures cannot be serialized directly. Raw server DTO fields alone do not establish
canonical validity. Validate every actor, bomb, ownership count, effect and event before filtering;
a malformed hidden record fails the whole operation. Binding rejects unsupported optional state
even when a caller provides matching enabled/available arrays. Archived Classic binding remains strict.

`prepare_snapshots` builds every recipient encoding before any can be published. Only
then may the runtime advance revision, record one report flush and clear reports once. Report
windows contain at most three consecutive updates and 192 unfiltered events. Capture audiences
at emission, filter before assigning contiguous recipient-local indexes per tick, and never
truncate. Historical event records need not exist in the final world. Admission prepares a
recipient-specific event-free body before ownership/replacement commit.

Running physical time equals room time. Result and MatchResult bodies remain frozen at their
own completed-round epoch/result/end tick while room time advances; unfinished terminal bodies
bind to the independent match outcome. A later forfeit cannot replace the earlier completed
physics tick. Inter-round countdowns publish room views without an old world. V14 encoders retain
the 96 KiB ceiling. Conversion neither clamps invalid timestamps nor advances simulation.

## Identity, ordering, and lifecycle

Room IDs are 32 lowercase hex characters and identify a room instance. Join codes are public,
eight canonical characters from `A-HJ-NP-Z2-9`. Neither proves seat ownership. The authenticated
server binding supplies the seat; no client command includes a seat, position, damage, or result.

Sequence numbers increase across create/join/resume/leave and gameplay commands on one connection.
A structurally valid command consumes its sequence before semantic handling. Manual action IDs
increase independently per seat/round and remain consumed after an unsuccessful placement.
Invalid room/round bindings cannot mutate another seat's high-water marks. Resume reports the
accepted floor; the next admitted manual action ID must be greater. A floor of 2,147,483,647 exhausts bomb actions
for that epoch. Unsupported equipment commands consume only the connection sequence,
not the manual high-water, mailbox budget or journal; a later bomb may reuse that action ID.

`RoomView` carries identity/code, independent command epoch `roundId`, revision, global tick,
rules/map versions, actual `roundNumber`, canonical per-seat scores (0..9, bounded by the selected
target), nullable `lastRound`, the configured two to four canonical seat statuses (connection,
occupancy/readiness/rematch consent), the immutable room configuration, and a tagged phase. A
connection is vacant, connected, disconnected with exclusive `expiresAt`, or expired. Actual
round count increases only on starts, remains on the preceding round during inter-round
countdown, and resets on rematch. Epochs also advance across cancellation and rematch, so
spawn/tie assignment uses actual round count rather than command epoch.

Shared room views and snapshot metadata also carry the complete checked `settings`,
`settingsRevision`, nullable original `settingsEditor`, and `availableItems`. Settings also
carry `enabledItems`; v10 remains pinned to its 21-item Bracelet catalog while active v15
has the unchanged 22-item catalog beginning with Kick, Punch, Bracelet and Wall Pass, with Oil Barrel immediately
before Teleporter. New live rooms enable the full available catalog and every supported subset is allowed.
Only actual changes increment
the settings revision and clear readiness; Ready commands acknowledge that exact revision.
See [v6 settings and authority](../history/protocols/protocol-v6.md) for bounds, no-op
acknowledgements and edit locks.

Phases are Waiting; initial/inter-round Countdown with deadline; Running; nonfinal Result
with `holdUntil`; MatchResult with independent outcome, end time, and `rematchAt`;
or prematch Interrupted with reason/end time. Holds last 180 ticks, followed by a 180-tick
inter-round countdown. Draws award no score. Last-round records persist across later rounds,
forfeits, and interruptions; rematch clears them. See the
[v4 field reference](../history/protocols/protocol-v4.md).

Eligible human players may set or withdraw rematch votes after the final hold. Consensus advances
one epoch and returns occupied, unready Waiting without changing transport sequence or room
revision ownership. Countdown cancellation advances epoch without consuming an actual round
or seed. A newer-epoch snapshot can be accepted directly when an intermediate room view is
coalesced. Old room/epoch/revision data cannot overwrite newer state.

Leaving acknowledges removal while retaining an authenticated unseated socket. A rejected leave
retains the seat capability; acknowledged leave clears it. If transport is lost while leave is
pending, the browser honors the explicit local departure instead of retrying. Running
boundaries commit physics/scoring before batched departures; idle phases apply departure
before automatic transitions. Departure and grace expiry settle elimination, forfeit, or
abandonment against the complete eligible roster; CPUs cannot keep a room alive after its last
retained human leaves. Eligible transport loss reserves the seat for 600 global ticks. Reaching the
configured win target and already final outcomes take precedence. Ownership is bound to connection generation and private
capability, never to invitation code. See [bounded owner](match-owner.md).

## Profile and version evolution

An immutable server room profile owns state construction and its rules/map labels. Publication
rejects state whose checked rules version disagrees with that profile. Live two-seat 15×13 rooms
use Expanded-v5 rules, generator v2, and CPU policy 16. Three/four-seat 15×13 rooms use rules 6,
generator v3, and CPU policy 17. Every 11×9/13×11 room uses rules 6, generator v4, and CPU policy
17; see [board geometry](board-sizes.md). Snapshots initially have an
empty pickup array because contents are concealed; only exposed items become public.
Fixed-classic-v1 remains an explicit unit fixture, not a production selector.

Gameplay v14 is incompatible with earlier versions. Reload clients when upgrading the single server. Both
old and future versions are rejected before credential lookup, so rejection cannot claim or
revoke a token. `supportedVersion` accepts the bounded input range rather than only v14,
allowing a new client to understand an older server's incompatibility response.
See [ADR 0013](../decisions/0013-gameplay-v2-and-room-profiles.md).

## Contract checks

`npm --prefix web run gen:gameplay` regenerates gameplay types; `check:gameplay` detects drift
without writing. The shared web gate runs this alongside existing diagnostic checks and AJV
validation tests. Rust conformance tests compare strict parsing and serialization with the same
schema, including missing/null fields, scalar boundaries, integer-valued number forms, version
rejection responses, credential redaction, all event variants, nested bounds, and real
frame-size enforcement. Conservative v14 schema-recursive proofs keep Snapshot and Admitted at
or below the inclusive 98,304-byte ceiling. Separate
schema-valid representatives and oversize rejection tests remain. These are structural bounds,
not reachable gameplay claims. TypeScript types do not enforce string patterns
or numeric/array bounds at runtime; AJV and Rust validation do.

## Graceful shutdown

A sticky stop signal bypasses command queues. Owners interrupt unfinished matches with
ServerStopped, atomically settle pending departures, and retain one final checked publication
with the closing reason. Existing final matches remain final. Members receive their authenticated
seat perspective of that publication
before close; unseated and authenticating sockets close directly. Delivery is best effort
within a five-second bound. The process explicitly drains upgraded game/diagnostic sessions
and room cleanup before runtime exit; HTTP graceful shutdown alone does not await upgrades.

## Archived wire compatibility

Core bomb views/events distinguish Timed, Remote and Contact triggers. This internal Rust
representation does not change gameplay-v6 or its archived wire DTOs: wire bombs retain
mandatory `detonatesAt`. The v6 checked projection accepts Timed records and rejects either
untimed variant, including event-only records. One unsupported record fails the complete
snapshot/report window; it is never omitted or assigned a substitute deadline. Publication
failure does not advance revision or discard reports. Current Expanded rules also admit Remote
and Contact bombs when their corresponding items are enabled; archived v6 continues to reject
untimed live and event-only records.

Room configuration and per-seat controller/connection metadata follow the [v14 contract](protocol-v14.md).
CPU seats are server-controlled and have no network capability.
