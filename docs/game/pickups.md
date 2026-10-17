# Typed pickup layers

Status: headless visibility, collection, upgrades, and round assignment implemented.
Live rooms remain on the fixed map without pickups until wire/client activation.

`World::with_pickups(arena, concealed, pending, exposed)` checks typed cell maps and returns
`PickupPlacementError` rather than a partial world. A concealed item must lie under a soft
block; pending/exposed items require floor with no retained block. Cells are exclusive across
all three maps. `World::new` still creates an item-free world.

`World::from_generated(generated)` builds the concealed layer and returns the private
`GenerationRecord` separately to the trusted caller. `exposed_pickups()` is the read-only
visible layer. Public domain `PickupView { cell, kind }` entries are row-major and contain no
concealed or pending information; Debug redacts those private layers. Gameplay v2 transmits only exposed items and collection events. Browser item rendering and
generated-room activation remain separate release steps.

A destroyed block moves its concealed item to Pending. The revealing explosion cannot burn
that item. It becomes Exposed only after all flames on its cell expire; overlap or renewal
extends the wait. An exposed item hit by a later blast disappears without stopping that blast.
Reveal happens before actions, so a new blast in the reveal update can destroy the item.
Pickups are nonblocking. Completed rounds freeze every layer.

With `RulesConfig::classic_v2()`, collection runs after every fire-damage phase and before
round-result determination. A living actor's final center must occupy the exposed item's cell.
The nearer center wins a contested item; exact ties use `RoundAssignment::priority_seat()`.
Collection under a bomb is allowed. An item disappears even if its upgrade is already capped.

Capacity and range increase to at most eight; speed advances through five levels. A bomb
captures range when placed, including placement earlier in the collection update. Increased
capacity helps later placements, and increased speed affects the next movement update without
resetting fractional residuals. `PickupCollected` precedes any `RoundEnded` domain event.
Classic-v1 deliberately keeps exposed items inert.

`RoundAssignment::new(NonZeroU32)` uses the actual match round: odd rounds assign seat one to
the row-major first spawn and give it tie priority; even rounds reverse both. Constructors
without an assignment use round one. Explicit assignments never relocate supplied fixture
actors; the match owner selects their spawn positions. Canceled countdowns do not advance rounds.

Run `cargo test -p grenadiers-core --locked` for layer validation, reveal/overlap/expiry,
same-update burn, public privacy, collection order/ties, upgrade caps, preserved old-bomb range,
movement residuals, assignment parity, nonblocking movement, and frozen-result regressions.
[ADR 0012](../decisions/0012-pickup-phases-and-upgrades.md) documents these phases and the collection/upgrade boundary.
