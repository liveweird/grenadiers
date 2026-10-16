# ADR 0008: Anonymous gameplay protocol and single-round preview boundary

Status: accepted; control/lifecycle, snapshot/event definitions, and checked domain adapters
and anonymous HTTP session issuance are implemented. Gameplay socket/room handlers follow.

## Decision

Preserve diagnostic `/api/v1/ws` and its protocol version. Define a separate gameplay schema
and planned `/api/v1/game/ws` endpoint. Product, diagnostic, gameplay, and rules versions are
independent. Validate the first authentication frame before any seat/room mutation; a bounded
unsupported version receives an explicit incompatibility response before credential lookup.

The session endpoint issues a 256-bit OS-random token. Keep it in tab memory and send
it only in the first socket frame, never a URL or log. Redact token Debug output and discard
payload-bearing parser errors. One live connection owns a credential. Exact Origin checks
remain required but do not establish identity; the server binds connections to seats.

Separate a public eight-character join code from a stable 32-hex room-instance identity. Room
IDs use a random 64-bit process prefix and checked monotonic 64-bit suffix, avoiding reuse and
an unbounded retired-ID set. Internal binding generations fence queued commands after either
socket replacement or same-socket leave/rejoin. Clients never choose their seat/generation.

Commands have increasing per-connection sequences; bomb action IDs increase independently per
seat/round attempt. Consume admitted IDs even when placement fails. Canceled countdowns advance
the round-attempt ID, so delayed controls cannot become valid in the next attempt. Public room
revisions order updates across response/snapshot channels; admission includes a private seat.

Use the fixed classic profile and map, two-ready countdown, one authoritative room owner, bounded
queues, 60 Hz updates, and 20 Hz full snapshots. Transport loss during Running interrupts the
preview without awarding a win; an already-resolved result remains final. Reconnect, scoring,
rematches, generated maps, and upgrades belong to their later milestones.

## Validation and implementation sequence

Use closed draft-07 schemas, checked canonical string/integer scalars, required nullable fields,
and explicit public DTO conversion. Preserve JSON integer-valued decimal/exponent forms while
keeping domain arithmetic integral. Reject oversized messages and JavaScript-unsafe ticks.
[Protocol guidance](../game/protocol.md) defines the boundary and current implementation scope.

Ship control/lifecycle definitions, snapshot/event definitions, checked projection, authenticated
transport, rooms, scheduling, browser controls, and Pixi rendering in bounded increments. Do not
advertise unimplemented endpoints in OpenAPI. Keep the domain projection from [ADR 0007](0007-public-state-events.md)
independent of Serde; [ADR 0003](0003-arena-rendering.md) keeps rendering outside authority.
