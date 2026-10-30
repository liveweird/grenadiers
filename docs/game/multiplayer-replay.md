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

## Environment recordings

Format6 binds rules7, CPU18 and environment randomness version1 in addition to lifecycle3,
simulation randomness1 and derivation1. It chooses generator3 for 15×13 or generator4 for
smaller boards. The header records the initial environment configuration; accepted lobby
settings operations record each subsequent base-settings and environment change together.
Cloud-disabled recordings still use this explicit new profile.

The closed format6 operations normalize into the shared deterministic executor. Older
format4/5 headers and operation parsers retain their original rules and policy identities.
Replay submits accepted CPU inputs without rerunning planning. Inspection includes the
environment configuration, actor timers and complete simulation RNG progress. These recorders
remain test-only; this adds no production recording or export path.

Format7 separately binds rules8, CPU20 (or archived CPU19) and lair layout1, alongside the same lifecycle,
generator selection and randomness versions. Its header retains complete lair tuning, including
numeric choices while disabled. Atomic settings records carry base rules, clouds and tuning
together. Mob actions are not journal entries: deterministic simulation regenerates births,
pursuit, contacts and deaths from accepted player inputs. Exact state inspection includes mob
positions, retreat anchors, lair deadlines, living counts and simulation randomness.

Formats6/7 normalize their closed operations into the shared executor rather than duplicating
lifecycle mechanics. Their headers and accepted source/controller identities remain distinct;
older formats cannot claim the new rules or CPU policy.

## Exactness and resource limits

Runtime inspection compares lifecycle, complete simulation state including RNG progress, pending
inputs/reports, and publication/settings revisions. Tests exercise two to four participants,
Easy/Standard CPU actions, real Teleporter collection, cancellation, departures, reserved-seat
expiry, rematch consensus, publication and interruption ordering.

Readers reject cross-profile headers: format 4 cannot claim a small board or generator 4, and
format 5 cannot claim 15×13 or generator 3. Formats4/5 keep their 64-byte record ceiling and
4 MiB resident limit; format6 permits 72-byte records within 5 MiB. Every profile retains the
60,000-record cap, and memory accounting includes header-owned allocations. Truncated or
incomplete journals are rejected. A failed round
construction records the attempted boundary as incomplete rather than fabricating a seed or round.
The lifecycle profile is pinned to the core implementation at compile time and checked at replay.

Run `cargo test -p grenadiers-server replay_v4 --no-fail-fast` for the focused verification suite.
The [runtime guide](multiplayer-runtime.md) owns physical settlement; the
[CPU guide](multiplayer-cpu.md) owns observation and policy boundaries.
