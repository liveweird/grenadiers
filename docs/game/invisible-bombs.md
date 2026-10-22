# Invisible Bombs

Invisible Bombs is an optional ten-second curse in Expanded-v5 rooms. While it is active, the
affected player receives no live bomb records and no placement or detonation events. This applies
to their own and their opponent's Timed, Remote and Contact bombs, including paused Timed bombs and
special blast types. Bomb collision, capacity, contact, chains and detonation remain authoritative;
flames, terrain, pickups, results and each actor's live-bomb count stay public.

The effect lasts exactly 600 physical ticks. Collection applies immediately through tick `T+599`,
expiry happens before actions at `T+600`, and recollection refreshes the deadline. Death and round
reset clear it. Resume preserves the original physical deadline and sends an event-free current
view. A surviving bomb appears at its current cell and remaining fuse after expiry or death; no
historical placement or sound is replayed.

Bomb event audiences are captured when the event occurs. Each event begins with the Cloak owner's
audience, then removes every seat blinded at that tick. An event may therefore be visible to both,
one seat, or neither. Earlier visible history remains visible after collection, and hidden history
does not become visible after expiry. Complete bomb and event facts are validated before either
recipient view is filtered and both recipient frames are prepared before publication advances.

Version 0.3.41 adds Invisible Bombs as gameplay v9's sixteenth item and generator-v2's final weight
15. The all-items mask is `1048560`; all 65,536 subsets are accepted. The new interval is `[335,
350)` and the sole-item pool is 115. Every earlier catalog mask and generator path is unchanged.
Archived gameplay v8 remains pinned to its fourteen-item Cloak-era catalog.

## Verification checkpoint - fresh native execution complete

Source 035c6a1263674601874e82c31b5c731c794cb38a, tree
d447fa8099ded815a37060d79117021ea89a61b7, preserves the Invisible Bombs runtime contract and
changes the browser proof schedule and diagnostics. Both page captures now run concurrently while
Player One completes the remaining normal retreat. Capture geometry is prepared before collection
and revalidated without scrolling; sanitized receipts retain selected bomb events, exact physical
deadlines, capture boundaries, comparison hashes and every returned comparison PNG on failure.

Fresh full gates pass for this source: debug, release and coverage each cover all 65,536 masks;
debug and release pass 658 core plus 207 server tests and two doctests; coverage passes the same
core/server counts; Rust line coverage is 92.38127315594477%. All 516 web tests pass with
95.43/91.25/95.9/97.82 statement/branch/function/line coverage. Renderer diagnostics, 54 static
scenario contracts, Cargo deny and both configured high-threshold npm audits pass. The web audit
still records two moderate Vitest/mocker 5.0.1 advisories; no high or critical advisory is accepted.

The earlier PR 88 delivery is disqualified with zero hosted carry. Run 39820850960 was cancelled
with seven non-server jobs cancelled without steps; only the aggregate server job's upstream guard
failed and its later steps were skipped. Push run 39820856196 passed all nine jobs. Pull-request run
39820865780 failed a real browser assertion after the original five-second bomb naturally expired
across sequential captures. The correction's single CPU-1 diagnostic passed in 45.8 seconds:
placement 1150, natural detonation 1450, captures 1231..1330, and reconnect 1220 + 506 preserving
the original curse deadline 1726. That run is diagnostic only and contributes zero accepted cases.

Preparation and prebrowser review are clear for the new source, image and archive. Native root
30427 completed with consumed exit zero: 53 Compose plus 53 Kubernetes cases, all 48 groups and
reports, 114 runtime PNGs, 38 runtime records, 19 Kubernetes identities, 18 fixtures and 36
seed-mode records pass with zero retries or carried results. All 80 latency observations remain
below 250 ms. Root reviewed two fresh Compose pre-collection captures for presentation; those
stills do not prove privacy or timing, which the native assertions and sanitized JSON cover.
Independent native audit and result review are clear. The final manifest seals 2,486 payloads;
2,487 files and the separate source archive are byte-verified and read-only, and independent seal
review is clear. No v0.3.41 candidate, merge, publication, tag or deployment is claimed.
