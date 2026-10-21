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

Published version 0.3.38 introduces Invisibility Cloak with generator weight 15 after Landmine.
The expanded pool contains 320 entries and preserves every earlier 8192-mask path. Its fourteen-item
catalog supports all 16384 subsets and is retained by [gameplay-v9](protocol-v9.md). Rules version 5,
generator version 2 and journal format 2 remain unchanged. Reload older browsers when deploying the
paired server/web upgrade.


At the 21 October 2026 16:47 UTC native acceptance checkpoint, corrected source `1c6724f`, tree
`403bb73f`, and image `960976c9` passed fresh 51 Compose plus
51 Kubernetes acceptance with zero retries or carried passes. The independent raw audit is clear;
the 2,298-payload final evidence seal is
`7b2f24235019d70b3207a6ed99adf6264c45347083bce397eaa873107a0ec14e`, and all 2,299 durable
files are byte-verified. Both clients prove natural acquisition; the owner retains its actor and timed
effect while the peer receives only the hidden identity and no attributable collection cue.
Compositor pixels prove peer removal, reconnect preserves privacy, and the actor reappears at
its current settled position after exactly 600 physical ticks. The earlier `4d9a751` pass and
failed PR #84 head `97a85d4` remain historical with zero carry. At that checkpoint, exact-head
candidate/post-merge CI still had to pass seven jobs and all 51 browser cases; publication and
promotion were separate gates, and published/local deployment was v0.3.37. See the
[release notes](https://github.com/liveweird/grenadiers/releases) for current release status.

The accepted raw documentation/capture commit `04c070c5` has sole parent `1c6724f`. Its tree
`d1dc83fc` equals published v0.3.38 merge `3c210c23`; all candidate and post-merge seven-job,
51-case workflows passed, and accepted image `960976c9` is promoted locally. The v0.3.39
gameplay-v9 foundation retains these Cloak semantics but has zero native acceptance or carried
passes. Its fresh matrix and release gates remain pending.


Fresh reviewed Compose [English desktop](../screenshots/invisibility-cloak-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisibility-cloak-polish-320.png) captures show the
Cloak pickup before collection and readable localized HUD/settings. Root actually viewed only
these two Compose captures. Polish uses normal vertical scrolling and the unfocused-arena prompt
is expected. These stills do not prove actor privacy: the paired native assertions establish owner
visibility, exact peer-hidden actor shape, compositor removal, reconnect and current-position
reveal at the physical deadline. Kubernetes captures are retained but were not visually reviewed.

See [verification](../verification.md) for source/image identities, latency, integrity and measurement limits.
