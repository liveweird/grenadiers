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

## Sealed native acceptance checkpoint — 2026-10-22 00:55:23 UTC

Corrected source `cb10ec7a143ed62128de3b96765b715cab346acf`, tree
`9d38e0cbe6abf6e7601a44dea1162acdde386e28`, and image
`sha256:d08b93af6eec7791e63ac423f12b33a43ee8c21e931423750f9a1a37ab63a3bf`
passed complete 65,536-mask debug, release and coverage runs and fresh 53 Compose plus 53
Kubernetes cases. All 48 groups passed with zero retries and zero carried passes.

The initial corrected-source native attempts remain zero-carry history. One exposed a stale
Diarrhea catalog oracle; another exposed the Polish arena locator. A focused run then showed the
first Invisible Bombs window had expired before the late placement. The final two-window scenario
derives a fresh 600-tick deadline from each collection, swaps viewer roles, filters privacy at the
exact event-tick boundary, and polls for the sighted pre-deadline detonation. These were test-oracle
and route corrections; game behavior, fuse 300, timeout, retry policy and timing bounds did not
change. The initial Kubernetes premeasurement capture 54425 rejected a filename assumption;
observer 14269 was deliberately stopped with consumed exit 143 before any latency test. Fresh
observer 43361 and the complete matrix passed without a test retry.

Root reviewed the fresh Compose [English desktop](../screenshots/invisible-bombs-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisible-bombs-mobile-pl.png) pre-collection captures.
They show the exposed pickup and readable localized presentation; privacy, reconnect and exact
expiry are proved by paired runtime assertions. Kubernetes stills were retained without visual
review. Independent audit decoded all 48 raw reports, checked 114 PNG hashes and structures,
rehashed 285 provenance records (282 unique), and checked 80 raw latency samples. It returned
CLEAR. The result review is also CLEAR. The final manifest
seals 2,492 payloads under SHA256
`819545a47add9a28bb4394fe8b4baa5b66f503090b1a575c4dbf39d2beb5ce15`; 2,493 durable files,
274 directories and the separate source archive are byte-verified and read-only. Independent seal
review is CLEAR. Hosted candidate, merge, tag and promotion checks have not run. See the
[release notes](https://github.com/liveweird/grenadiers/releases) for current release status.
