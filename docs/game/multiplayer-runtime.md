# Roster runtime

`server/src/game/rooms/roster_runtime/` provides a transport-independent runtime for checked
2–4-seat configurations. It is dormant: live sockets still use the existing two-player runtime.
The [v14 contract](protocol-v14.md) owns wire shape and recipient privacy; the
[multiplayer design](multiplayer-design.md) owns the intended live integration.

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

Round construction uses rules 6, generation 3 and the actual physical-round assignment. The
configured roster remains intact, with departed participants initially inactive. Only 15×13 is
accepted; smaller boards fail before entropy or construction. Legacy two-player routing,
rules 5, generation 2 and CPU policy 16 are unaffected.

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
