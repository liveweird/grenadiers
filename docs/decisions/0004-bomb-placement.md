# ADR 0004: Owned bombs and simultaneous placement

Status: accepted for the first M2 increment; explosions and damage follow separately.

## Decision

Use one authoritative bomb map in `World`. Each bomb contains its stable entity ID, cell,
owner actor ID, captured range, placement tick, and detonation deadline. Collision reads the
same record as future combat. Derive live-bomb counts from that map rather than maintaining
a second counter that could drift or release capacity twice.

`InputAction::PlaceBomb` is a one-shot logical request. Coalesce repeated requests per seat
within a tick; never retain a failed request for a later tick. `Clear` cancels an earlier
request in the same ordered input batch, while a later placement request can enable it again.
Transport action IDs and retry de-duplication remain M3 responsibilities.

Resolve movement first, then placement. Check each request independently: free capacity,
legal floor, no soft block, and no existing bomb. Discard ineligible requests before checking
same-cell conflicts. Two remaining eligible requests for one cell are both rejected; distinct
placements commit in seat order. A failed request consumes neither an ID nor capacity.

Snapshot the owner's range and the completed tick. The normal deadline is placement tick
plus 180 ticks. Checked deadline/ID exhaustion rejects placement without partially inserting
a bomb. Step-level tick overflow and excessive inputs still reject before any state mutation.

Allocate monotonically increasing entity IDs, skipping actor IDs and never recycling bomb
IDs. Seed allocation above existing bomb IDs when constructing from an existing world; an
actor with a large ID must not unnecessarily exhaust all lower unused IDs. Existing bomb
owners must match a starting actor. Preserve canonical actor/seat and bomb ordering.

Every overlapping actor receives its own collision exemption on insertion, using the M1
positive-area footprint rule. Ownership does not grant an exemption to an actor elsewhere.

## Scope and compatibility

M1's public ownerless obstacle insertion is retired before any server or browser consumer
exists. Collision fixtures use a test-only, explicitly owned insertion helper with the real
bomb type. Production bombs have no optional owner or inert placeholder mode.

This increment records fuse deadlines but does not yet execute detonations, flames, damage,
or round outcomes. It is a headless engineering prerelease, not the playable duel preview.
Tests cover placement-time snapshots, post-movement target cells, capacity, simultaneous
conflicts, invalid-request isolation, one-shot requests, identity, and shared overlap grace.
