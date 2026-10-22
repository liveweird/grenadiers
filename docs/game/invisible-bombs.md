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

## Verification checkpoint — visibility-startup candidate

Source `eb0b756df43fdd896cf87573a5224e4eb5f659a3`, tree
`098f2a3beb766b4fefd2316b7c7ceaf37275a826`, preserves the Invisible Bombs game contract and
changes only six E2E startup/proof files. The Chromium launcher now parses complete endpoint lines
from a bounded stream, retains redacted diagnostics for timeout and exit failures, and preserves
the original launch error when cleanup also fails. The actual cause of the earlier PR 90 startup
failure remains unknown; the parser and reporting defects are independently confirmed and tested.

Typed gate assembly `813e7c58…` retains the authenticated original-035 Rust/web computations under
input-equivalence review `1ea0a03b…`: all 65,536 masks per profile, 658 core plus 207 server tests,
516 web tests and renderer diagnostics. Fresh H repository/E2E lint, Knip, type and scenario checks,
startup and retained live-pixel units, server-mask contract 10/10, mirror unit 5/5 and current
security checks pass. A single headed diagnostic passes with zero accepted or carried cases.

Native root 40405 completed with consumed exit zero. Its source-bound matrix passes 53 Compose plus
53 Kubernetes cases, 48 reports, 114 runtime PNGs, 38 runtime records, 19 Kubernetes identities,
18 fixtures and 36 seed-mode records with zero retries and zero carry. All 80 retained latency
samples remain below 250 ms. Independent audit is clear under SHA-256
`b0c72b1ff651257930f0db8df9db3cfc1274bd6882f142336e508e699c41097e`; result review is clear under
`0cff7c9dd3bd4895cfcb426f340eca146b8c9627a55bac04c01111819b9008be`. The evidence is durably
sealed and byte-verified, with independent seal review clear under SHA-256
`87666f73ca51ac78e2d726de48f74444059e1aca636864839ab437249f353ec8`.
The two tracked screenshots are independently reviewed Compose
pre-collection presentation captures; they do not prove privacy or timing, and no Kubernetes
screenshot received visual review. PRs 89 and 90 contribute zero hosted cases. This source-bound
checkpoint makes no v0.3.41 candidate delivery, merge, publication, tag or deployment claim.
