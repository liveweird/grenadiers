# Typed pickup layers

Status: headless visibility/reveal/destruction behavior. Collection and upgrade application
follow separately; live rooms remain on the fixed map without pickups.

`World::with_pickups(arena, concealed, pending, exposed)` checks typed cell maps and returns
`PickupPlacementError` rather than a partial world. A concealed item must lie under a soft
block; pending/exposed items require floor with no retained block. Cells are exclusive across
all three maps. `World::new` still creates an item-free world.

`World::from_generated(generated)` builds the concealed layer and returns the private
`GenerationRecord` separately to the trusted caller. `exposed_pickups()` is the read-only
visible layer. Public domain `PickupView { cell, kind }` entries are row-major and contain no
concealed or pending information; Debug redacts those private layers. The current gameplay
wire contract does not transmit the new domain field yet. Protocol/client evolution and
activation are separate release steps.

A destroyed block moves its concealed item to Pending. The revealing explosion cannot burn
that item. It becomes Exposed only after all flames on its cell expire; overlap or renewal
extends the wait. An exposed item hit by a later blast disappears without stopping that blast.
Reveal happens before actions, so a new blast in the reveal update can destroy the item.
Pickups are nonblocking. Completed rounds freeze every layer.

Run `cargo test -p grenadiers-core pickup --locked` for layer validation, reveal/overlap/expiry,
same-update burn, public privacy, nonblocking movement, and frozen-result regressions.
[ADR 0012](../decisions/0012-pickup-phases-and-upgrades.md) fixes these phases and the upcoming
collection/upgrade boundary.
