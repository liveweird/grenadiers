# ADR 0012: Pickup visibility, collection, and upgrades

Status: visibility/reveal/destruction layers implemented; collection is the next slice.

## Visibility and damage phases

Store typed pickups in mutually exclusive concealed, pending, and exposed cell maps.
Concealed contents require a soft block; pending/exposed contents require floor without a
block. Checked construction rejects duplicate cells and invalid layer placement atomically.
Keep generation metadata separate from the active world. Public domain views include only
row-major exposed `{cell, kind}` entries; Debug hides concealed/pending locations and kinds.

After exclusive flame expiry, expose pending contents only on flame-free cells. Apply the
immutable explosion plan without changing its ray/chain rules: burn already exposed items in
new blast cells, and move contents of destroyed blocks to pending. The revealing blast never
burns pending contents; renewed/overlapping flames postpone reveal. An item exposed before
a new blast in the same update burns normally. Pickups never block movement or blast rays.
Completed results freeze these transitions with the rest of the world.

## Collection decision

The following collection contract is fixed before its dependent implementation. Enable it
through distinct `classic_v2` rules, preserving the fixed preview's `classic_v1` behavior.
After all fire damage and before final result determination, eligible living actors collect
items only when their final centers lie in the item's cell. Choose the smaller squared
integer distance to the cell center; use explicit round-seat priority for exact ties.
Consume an item even at its upgrade cap and permit collection beneath a bomb.

Capacity and range cap at eight; speed uses the five existing levels. Placement precedes
collection, so old and same-update bombs retain their captured range. Capacity helps a later
placement; speed changes the following movement update without resetting fractional residuals.
Emit only a collection event before RoundEnded; visible snapshots represent reveal/burn changes.
This preserves the bounded event-window argument instead of emitting per-cell blast events.

Use actual nonzero round number for alternating spawn assignment and collection tie priority.
Canceled countdown attempt IDs are not round numbers. Existing constructors retain first-round
assignment; an explicit constructor supports the future match owner without moving arbitrary
fixture ActorInit positions implicitly. Protocol compatibility and item rendering precede
live generated-room activation. See [pickup API](../game/pickups.md).
