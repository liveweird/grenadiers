# Reverse Controls

## Rules

Reverse Controls is a configurable timed curse introduced in v0.3.31. Collection activates or
refreshes it for exactly 600 physical simulation ticks. The simulation keeps the raw held keys
and their recency unchanged, then moves in the opposite direction while the effect is active.

Collection happens after movement, so movement on the collection tick is ordinary and inversion
starts on the next tick. Expiry happens before movement, so the collection tick plus 600 uses the
raw direction normally. If expiry and recollection share a tick, that tick is ordinary and the
next is reversed. A refresh while already active preserves current turn assistance.

## State and reconnect

Starting or ending inversion clears a pending assisted turn while preserving position, heading,
held-key state, recency and fractional movement budget. A stationary actor does not rotate;
heading follows actual motion. Death and round reset clear the effect. Reconnect clears held input
under the existing input policy and restores the authoritative remaining timer.

## Availability and verification

New rooms enable the option; Classic and explicit empty expanded settings remain supported. Fresh same-image
native acceptance passes all 33 Compose and 33 Kubernetes browser checks.
Final exact-head CI and publication are recorded separately in [verification](../verification.md).

Screenshots: [desktop pickup](../screenshots/reverse-controls-desktop.png) and
[Polish 320px after expiry](../screenshots/reverse-controls-polish-320.png).
