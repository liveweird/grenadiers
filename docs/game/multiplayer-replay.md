# Multiplayer replay verification

`server/src/game/rooms/replay_v4/` provides test-only recorders and deterministic replayers for
roster matches. They are compiled under `cfg(test)`; production recording and export remain
disabled, so no durable migration is required. Historical formats 1–3 retain their original
implementation and accepted profiles.

## Header and operation boundaries

Format 4 strictly binds a 15×13 checked room configuration, lifecycle profile 3, rules 6,
generator 3, simulation randomness 1, derivation 1 and an independently generated private
simulation root. Format 5 retains the same journal operations and runner but strictly binds an
11×9 or 13×11 configuration and generator 4. Both formats bind lifecycle profile 3, simulation
randomness 1, derivation 1 and CPU policy 17. Every configured CPU has exactly one canonical
provenance entry with matching difficulty; humans cannot acquire CPU entries. Private roots and
seeds redact debug output.

The recorder stores accepted human and CPU actions, committed admissions, input fences, settle
boundaries, publication flushing and interruption. Replay submits the recorded CPU actions through
the runtime's controller fence; it does not execute CPU policies or workers again.

Every actual physical round requires exactly one marker containing its epoch, assignment and
generation record. The replayer queues the recorded arena seeds and derives simulation randomness
from the private root and recorded epoch. Missing, extra or mismatched markers are rejected.
Removing an old simulation during rematch cleanup never creates a round marker.

## Exactness and resource limits

Runtime inspection compares lifecycle, complete simulation state including RNG progress, pending
inputs/reports, and publication/settings revisions. Tests exercise two to four participants,
Easy/Standard CPU actions, real Teleporter collection, cancellation, departures, reserved-seat
expiry, rematch consensus, publication and interruption ordering.

Readers reject cross-profile headers: format 4 cannot claim a small board or generator 4, and
format 5 cannot claim 15×13 or generator 3. Records remain at most 64 bytes with a 60,000-record cap. Memory accounting includes header-owned
allocations and stays below 4 MiB. Truncated or incomplete journals are rejected. A failed round
construction records the attempted boundary as incomplete rather than fabricating a seed or round.
The lifecycle profile is pinned to the core implementation at compile time and checked at replay.

Run `cargo test -p grenadiers-server replay_v4 --no-fail-fast` for the focused verification suite.
The [runtime guide](multiplayer-runtime.md) owns physical settlement; the
[CPU guide](multiplayer-cpu.md) owns observation and policy boundaries.
