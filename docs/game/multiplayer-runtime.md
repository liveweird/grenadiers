# Roster runtime

`server/src/game/rooms/roster_runtime/` provides the transport-independent runtime used by live
three- and four-seat configurations and by two-seat rooms on smaller boards. The live registry
retains the established two-seat path only for 15×13. The [v14 contract](protocol-v14.md) owns wire shape
and recipient privacy; the [multiplayer design](multiplayer-design.md) records its integration.

## Lifecycle and participation

The roster lifecycle retains every original seat, score and actor after departure. Humans move
between never admitted, connected, reserved and permanently departed states; CPU slots remain
server controlled and cannot accept human admissions. At least two eligible participants and
a connected human are required to start. A reserved human blocks readiness; a departed human
does not. CPUs cannot keep a room alive after its last retained human leaves.

Creator authority remains assigned during reconnect grace, but only the connected creator can
change settings. Permanent departure releases authority without transferring it. Readiness,
settings revisions, round epochs and manual-action high-water marks fence stale commands.

## Publication and admissions

Each physical update runs at 60 Hz; ordinary snapshots batch three reports at 20 Hz. Outcomes
and important lifecycle changes publish immediately. Human frames and the opaque CPU publication
are derived from the same certified physical facts before revisions or report flushing commit.
Admissions prepare a prospective lifecycle and all recipient frames, encode the private response,
and commit only after the owner delivers it. Resume retains the exact new CPU publication.

Round construction uses rules 6 and the actual physical-round assignment. It selects generator 3
for 15×13 and generator 4 for 11×9/13×11. The configured roster remains intact, with departed
participants initially inactive. Live two-player 15×13 routing retains rules 5, generator 2 and
CPU policy 16. Every smaller-board room and every three/four-seat room uses the roster runtime and
CPU policy 17. The exact profile table is in [board geometry](board-sizes.md).

## Boundaries and randomness

During a running boundary, physics and scoring settle before departures and exact grace expiry.
Elimination appends events to the current report and can complete the current tick without
advancing twice. Completed physics, owned bombs and final match outcomes remain frozen. During
idle phases, departures precede countdown advancement. Shutdown cleanup cancels rematch consensus
so pending departures cannot start another match.

Simulation randomness uses an independent OS-generated private room root. Derivation version 1
uses ChaCha20 with the root key, the nonzero physical epoch as its 64-bit stream, and the first
32 bytes as the round seed. Derivation does not advance shared state or depend on request order;
arena fixtures and CPU streams cannot alter it. Roots and derived seeds redact debug output.
The derivation remains version 1 for private replay formats 4 and 5, but production recording and
export stay disabled.
