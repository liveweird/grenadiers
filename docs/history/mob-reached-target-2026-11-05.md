# Reached-target mob navigation — 2026-11-05

## Cause and boundary

A broader seeded round sweep found a long Standard CPU pause. Its goals alternated between
blocks `(1,6)` and `(2,7)`, both approached through `(1,7)`. The physical safety layer repeatedly
certified Stop while two mobs remained against the northern wall. A retained full planner trace
disproved discarded-memory and false-progress explanations: stalled-target changes and deferrals
were committed, and the final result retained a certified Stop with contact authority.

The upstream navigation defect was a zero-distance BFS comparison. At its target cell,
`current.checked_sub(1)` is `None`; an unreachable neighboring cell also has distance `None`.
The old selector therefore chose that blocked neighbor. Once off center, the mob retained its
heading and collision clipped it against the wall. A focused old-policy fixture reproduces the
captured `(1536,7488)` pose from center `(1536,7680)`. The final scene alone does not identify
which protected actor triggered the historical departure; the zero-distance fixture and full
old/new episode establish the reachable defect and the fix's effect.

Navigation4 stops ordinary pursuit at zero distance. Existing bomb/spike escape branches run
first, stationary contact remains lethal after protection/grace, and a moved/dead target can be
reselected. Navigation1–3 retain their recorded behavior. Replay40 binds navigation4 with the
same Easy68/Standard62/Hard70 profiles; Replay39 remains navigation3. No CPU scoring, work
allowance, physics speed, generator, public protocol or item setting changes. A proposed broader
Standard routing policy was deferred after this smaller upstream fix cleared the reproduced case.

## Physical paired regression

Both runs use arena3, simulation root113, CPU seeds71/72/73 and a fixed-idle human. They share the
same queued-owner evaluator and each independently verifies its retained shadow and accepted-input
replay. Neither arm retries, searches seeds, injects state or copies the other arm's current state.

| Seat Four, Standard62 | Navigation3 / Replay39 | Navigation4 / Replay40 |
| --- | ---: | ---: |
| Longest stay in one cell, running updates | 5,276 | 183 |
| Natural round duration | 10,800 (timeout) | 2,851 (winner Four) |
| Alive at result | Yes | Yes |

The regression requires a natural result, reduced stationary duration, survival and departure from
the old exact position. A smaller span obtained by eliminating the actor does not pass. This is
one deterministic scripted case, not a general difficulty ranking or survival guarantee.

## Broader comparison

Clean source `b16f9beaf14b4a037b8ed624f87fb648d04d1dae` passes both release runners: 48 reference
cases in 12.66s and 192 seed-sweep cases in 48.74s. All 240 terminate naturally with exact
owner/shadow/input replay and complete bounded work/queue accounting. All 48 reference reports
match navigation3; 188 of 192 seed-sweep reports match. The four changed reports are Standard,
arena3, fixed-idle, two simulation roots and two CPU seed sets. CPU RNG draws are zero, so paired
CPU-seed streams repeat rather than provide independent trials. The largest Standard cell span
in the broader sweep falls from 5,276 to 232 updates; Easy228 and Hard240 are unchanged.

These are comparisons of complete deterministic measurement reports, not a comparison of every
unrecorded mob trajectory. Timing depends on the host and changed rounds; it does not establish a
performance speedup. Source/binary identities, raw reports and the full retained diagnostic remain
under ignored `target/quality-review/mob4-*` and `standard-stall-proof-*`. Full release acceptance
belongs in the [v1.3.32 release record](../releases.md#v1332--2026-11-05).
