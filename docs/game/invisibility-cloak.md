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

Version 0.3.38 introduced Invisibility Cloak with generator weight 15 after Landmine. The expanded
pool contains 320 entries and preserves every earlier 8192-mask path. Its fourteen-item catalog
supports all 16384 subsets and is retained by [gameplay-v9](protocol-v9.md). Rules version 5,
generator version 2 and journal format 2 remain unchanged. Reload older browsers when deploying
the paired server/web upgrade.


Version 0.3.38 acceptance passes 51 Compose and 51 Kubernetes cases against its frozen v8
source, with zero retries or carried passes. Fresh v0.3.39/v9 acceptance remains pending. Both clients prove natural acquisition; the owner retains its actor and timed
effect while the peer receives only the hidden identity and no attributable collection cue.
Compositor pixels prove peer removal, reconnect preserves privacy, and the actor reappears at
its current settled position after exactly 600 physical ticks. Independent raw review is clear
and evidence sealed. Exact-head candidate/post-merge CI and publication remain release gates.


Fresh reviewed Compose [English desktop](../screenshots/invisibility-cloak-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisibility-cloak-polish-320.png) captures show the
Cloak pickup before collection and readable localized HUD/settings. Polish uses normal vertical
scrolling. These stills do not prove actor privacy: the paired native assertions establish owner
visibility, exact peer-hidden actor shape, compositor removal, reconnect and current-position
reveal at the physical deadline. Kubernetes captures are retained but were not visually reviewed.

See [verification](../verification.md) for source/image identities, latency, integrity and measurement limits.
