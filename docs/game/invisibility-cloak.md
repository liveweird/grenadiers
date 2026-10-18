# Invisibility Cloak

Invisibility Cloak is an opt-in Expanded-v5 pickup that lasts 600 physical ticks. Recollection
refreshes that deadline. It coexists with equipment, weapons and other effects, changes no movement,
damage or bomb rule, and clears on death and round reset.

The owner receives their complete actor. An opponent receives exactly the actor's ID, seat and alive
status while the effect is active; position, heading, inventory, effects, grace and derived stats are
absent. Bombs, flames, blocks, pickups and results remain public, so visible world changes can still
reveal clues. Expiry reveals the actor at their current position without interpolating from a stale
location. Disconnect and resume preserve the physical deadline and do not replay private history.

Pickup collection, bomb placement and detonation, and damage-prevention events are private to an
affected cloaked seat when each event occurs. Elimination and round-result events remain public.
Audience is captured at emission and is not recomputed from the final world in a publication window.
Both complete recipient encodings are validated before either is published; legacy protocols reject
the private semantics.

Version 0.3.38 introduced Invisibility Cloak with generator weight 15 after Landmine. Its expanded
pool contained 320 entries and preserved every earlier 8192-mask path. That fourteen-item catalog
supports all 16384 subsets and remains pinned by archived gameplay v8. Active
[gameplay v9](protocol-v9.md) now has sixteen items through Invisible Bombs and supports all 65536 subsets.
Rules version 5, generator version 2 and journal format 2 remain unchanged. Reload older browsers
when deploying the paired server/web upgrade.
