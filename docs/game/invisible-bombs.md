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

## Native acceptance checkpoint — 22 October 2026 17:00:43 UTC

Source `97a3990764592a867574b87843401d146d307f80`, tree
`8c0897440f40b8617bfdb9fd96e0be2cb3e4f9da`, and image
`sha256:11003ee18addba9badf2e2be7d8cf0c20cf743a7ccd6d65f056724083014abb2` passed 53 Compose plus 53
Kubernetes cases, 48 groups/reports and post-run integrity checks with zero retries or carried
passes. Independent audit and result review are clear. Final manifest
`ec7763fca51926fbe178e3733866f60ce664ad1ec9797c5979a57e9d02829ade` binds 2,483 payloads,
2,484 durable files and 272 durable directories under seal
`f476ec148b235a72ccb3405889b622b32f7be9cb21c0a50908ec52a90bb4cd84`. Independent seal review is
clear under hash `6aa0a2a0af2454d663852b582b606e50ce1018bbd039196cbdcb7c9dc724de90`.

Earlier attempts carry no acceptance. Capture-r1 passed its exact-source server gate, but the web
gate timed out after 300 seconds contacting the npm registry during the final audit stage after five
successful stages; it contributes no web or native acceptance. Capture-r2 and capture-r3 also
timed out during npm audit after five successful stages. Capture-r4 then passed all six web stages
under consumed root 23267 without composing any partial web result. Draft PR #87's hosted workflows exposed the obsolete
one-window timing oracle. Source `239181c` then failed its CPU-one diagnostic because PNG decoding
delayed a live assertion until after detonation. Source `97a3990` passed the corrected focused
scenario under consumed root session 2294; independent review returned
`CLEAR_DIAGNOSTIC_ONLY_NO_ACCEPTANCE`, with `nativeAccepted: false` and `acceptanceCarry: 0`; the
later 106 fresh native cases supplied the acceptance evidence. The correction captures clipped PNG buffers, proves the live
state before deferred decoding, and uses a distinct second-window stimulus while retaining the exact
expiry bounds. It changes no game behavior, timeout, retry policy, fuse, or acceptance threshold.

An independent visual review binds the fresh Compose
[English desktop](../screenshots/invisible-bombs-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisible-bombs-mobile-pl.png) pre-collection captures
under hashes `64400c142a54e92a6675dd66a60e498070b5089d90eb140a9c938bdd69a848c8` and
`475626b5267fb37fe23e4f4609dffe0df65894982407da3f42f46940b163e8fd`. The stills establish the
pre-collection presentation; paired runtime assertions establish privacy, reconnect and exact
expiry. Hosted delivery had not started at this native checkpoint.
