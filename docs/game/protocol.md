# Gameplay protocol

Status: control/lifecycle contract foundation. `api/gameplay.schema.json`, independent Rust
`core::protocol` types, and generated `web/src/game/schema.ts` define messages; gameplay
endpoints, room handlers, and browser controls are not implemented by this increment.
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

The future snapshot/event contract extends this foundation separately; no untyped placeholder
payload is exposed. HTTP session responses are separate from socket messages.

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
rejection responses, and credential redaction. TypeScript types do not enforce string patterns
or numeric/array bounds at runtime; AJV and Rust validation do.
