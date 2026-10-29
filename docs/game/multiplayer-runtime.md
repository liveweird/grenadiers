# Roster runtime

`server/src/game/rooms/roster_runtime/` provides the transport-independent runtime used by every
live two-to-four-seat configuration and board. The [v16 contract](protocol-v16.md) owns wire shape
and recipient privacy; the [multiplayer design](multiplayer-design.md) records its integration.

## Lifecycle and participation

Before the first start, an explicit human departure or expired reconnect reservation frees that
configured seat for a new admission. A vacancy cancels an initial countdown and clears readiness;
unexpired reservations remain held. Accepted settings stay unchanged. After the first start,
including rematch lobbies, departure remains permanent and seats cannot accept replacements.
The roster retains original scores and actors. Humans move between available, connected,
reserved and permanently departed states; CPU slots remain
server controlled and cannot accept human admissions. At least two eligible participants and
a connected human are required to start. A reserved human blocks readiness; a departed human
does not. CPUs cannot keep a room alive after its last retained human leaves.

After the final result, new transport losses permanently depart the affected seat. Reservations
established before that result retain their original deadlines and remain resumable; see
[reconnect behavior](reconnect-server.md).

Creator authority remains assigned during reconnect grace, but only the connected creator can
change settings. Permanent departure releases authority without transferring it. Readiness,
settings revisions, round epochs and manual-action high-water marks fence stale commands.

## Publication and admissions

Each physical update runs at 60 Hz; ordinary snapshots batch three reports at 20 Hz. Outcomes
and important lifecycle changes publish immediately. Human frames and the opaque CPU publication
are derived from the same certified physical facts before revisions or report flushing commit.
Admissions prepare a prospective lifecycle and all recipient frames, encode the private response,
and commit only after the owner delivers it. Resume retains the exact new CPU publication.
Released credentials remain retired and cannot resume or be minted for replacements. Forming
rooms accept at most 256 lifetime human admissions, bounding the retired-credential set. A
replacement cannot inherit the departed creator's settings authority.

Round construction uses rules 8 and the actual physical-round assignment. It selects generator 3
for 15×13 and generator 4 for 11×9/13×11. The configured roster remains intact, with departed
participants initially inactive. Every live room uses CPU policy 19. The exact profile table is in
[board geometry](board-sizes.md).

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
The derivation remains version 1 for private replay formats 4–7, but production recording and
export stay disabled.

## Explicit lair runtime

`RosterRuntime::new_lairs` selects a separate rules8 environment profile, including when lairs
or clouds are disabled. It publishes checked gameplay16 admissions and snapshots; the ordinary
live registry uses rules8/gameplay16. Profile-specific settings entry points reject
cross-profile updates and share the same atomic authority, revision and readiness transaction.
Disabled lairs retain their numeric settings. Creator revocation, forming-seat replacement and
the first-start lock retain the lifecycle rules above.

Rules8 controllers require `LAIRS_STANDARD` or `LAIRS_EASY`. The worker returns its finite
mob-contact lease with the bomb/escape proof and original observation. Admission keeps world,
seat, pose, effects, command epoch, age and confirmed-stop fences. Continuing movement checks
both proofs; a changed Teleporter marker or failed contact check withdraws authority. A timed
hold still ends when observations or worker results are missing. See
[CPU boundaries](multiplayer-cpu.md#lair-profile-boundaries).
