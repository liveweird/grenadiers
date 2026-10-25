# Invisibility Cloak

Invisibility Cloak is a configurable Expanded-v5 pickup enabled in new rooms that lasts 600 physical ticks. Recollection
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
supports all 16384 subsets and remains pinned by archived gameplay v8. Archived gameplay v10 had
seventeen items through Teleporter and 131072 subsets; the live [gameplay v11](protocol-v11.md) has
22 items and supports all 4,194,304 subsets.
Rules version 5, generator version 2 and journal format 2 remain unchanged. Reload older browsers
when deploying the paired server/web upgrade.

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
`d1dc83fc` equals published v0.3.38 merge `3c210c23`; all candidate, post-merge and tag seven-job,
51-case workflows passed, and accepted image `960976c9` is promoted locally. At the 21 October
2026 18:44:23 UTC v0.3.39 native checkpoint, source `272b950` and image `9a4dc108` passed fresh
51 Compose plus 51 Kubernetes cases with zero retries or carried passes; independent raw audit is
clear and 2,320 payloads are durably sealed. At that checkpoint, v0.3.39 release CI, publication
and promotion remained pending.

At the 21 October 2026 20:56:41 UTC v0.3.40 native checkpoint, source `fa25467` preserves
those Cloak semantics while adding Stopwatch as the fifteenth source/native-verified item. All
32,768 gameplay-v9 subsets passed in debug, release and coverage, followed by fresh 52 Compose
plus 52 Kubernetes cases with zero retries or carried passes. Independent raw audit is clear and 2,434 payloads are durably sealed; hosted CI and publication remained pending at this checkpoint; published/local v0.3.39 remained
the release baseline.


Fresh reviewed Compose [English desktop](../screenshots/invisibility-cloak-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisibility-cloak-polish-320.png) captures show the
Cloak pickup before collection and readable localized HUD/settings. Root actually viewed only
these two Compose captures. Polish uses normal vertical scrolling and the unfocused-arena prompt
is expected. These stills do not prove actor privacy: the paired native assertions establish owner
visibility, exact peer-hidden actor shape, compositor removal, reconnect and current-position
reveal at the physical deadline. Kubernetes captures are retained but were not visually reviewed.

See [verification](../verification.md) for source/image identities, latency, integrity and measurement limits.
