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

Format8 binds rules9 and CPU21–25, with archived mob navigation1 and the same
generator/layout/lifecycle/randomness identities.
It reconstructs one-mob respawn timers, solid movement, Bracelet contacts and atomic Teleporter
exchanges from accepted inputs. Format7 retains rules8 and its original overlap/periodic lairs.
Every CPU entry must match its header policy; mixed or unknown policy versions are rejected.

Format9 adds an explicit immutable mob-navigation identity: navigation2, rules9 and CPU23/24/25.
Its envelope reuses the checked format8 accepted-input body, header/provenance checks, limits,
seed derivation and normalized executor. The standalone format8 reader always reconstructs
navigation1; a format9 envelope cannot be interpreted as an archived recording. Unknown outer
versions, navigation identities or CPU combinations are rejected before execution. Runtime
inspection includes the navigation policy even before a round starts. Lobby changes, later rounds
and rematches preserve it. Recording/export remains test-only.

Format10 is the private server-side accepted-operation profile for rules10, CPU26, navigation2
and immutable spike-pit placement1. Its checked header binds the complete spike room
configuration, lifecycle3, board-dependent generator3/4, simulation and environment randomness1,
lair layout1, spike-pit layout1 and the private simulation-root derivation. Atomic lobby records
retain base settings, clouds, respawn-lair tuning and spike-pit tuning together, including numeric
durations while pits are disabled. Round markers still bind the private generation seed,
assignment and generation record. The shared executor reconstructs admissions, reconnects,
accepted human and CPU inputs, fences, settled departures, publications, interruptions and later
rounds without rerunning CPU policy.

Format10 does not widen format8 or format9: their readers remain closed to rules9 and their
documented CPU identities, while the format10 reader requires CPU26 and rejects mixed provenance.
The recorder, reader and runtime inspection remain under `cfg(test)` and add no production
recording or export path. Truncated, incomplete, malformed, misordered, marker-inconsistent and
cross-profile journals are rejected before being treated as exact replays.

Format11 retains rules10/CPU26 and the same checked accepted-operation executor, but explicitly
binds navigation3. Its reader rejects navigation2, previous outer versions and mixed CPU provenance;
format10 remains navigation2. The new identity changes only deterministic mob decisions, not public
observations, ordinary input submission, lifecycle or recording/export availability.

Format12 retains rules10/navigation3 and binds CPU27, including its persistent-burning-Oil
plant support. Format11 remains CPU26. The closed reader rejects older outer versions and
mixed CPU provenance while reusing the same accepted-operation executor; it does not rerun
planners or enable production recording/export.

Format13 explicitly selects gameplay19 publication with rules10/navigation3. Its finite pairing
is CPU27 for Easy/Standard (or no CPU) and CPU28 for Hard; every CPU provenance entry must match
the room-wide tier and policy. Formats11/12 remain gameplay18 and reject Hard even when header,
body configuration and CPU provenance agree. All versions reuse accepted inputs rather than
rerunning planners. Production recording/export stays disabled.

Format14 keeps gameplay19/rules10/navigation3 and advances only Hard to CPU29. Its reader
requires Hard29 or Easy/Standard27 with exact header/body/provenance agreement; format13 remains
Hard28 and rejects future policy29 even when a forged journal is internally consistent.

Format15 retains gameplay19/rules10/navigation3 and binds CPU31 on Easy/Standard (or no CPU),
CPU32 on Hard. Header, body and per-seat provenance must agree. Format14 rejects those newer
identities even when every forged metadata field agrees. This changes no physics or replay
execution semantics and does not enable production recording/export.

Format16 retains gameplay19/rules10/navigation3 and binds CPU33 on Easy/Standard (or no CPU),
CPU34 on Hard. Header, body and per-seat provenance must agree. Format15 retains CPU31/32 and
rejects future metadata; production recording/export remains disabled.

Format17 retains gameplay19/rules10/navigation3 and binds CPU33 on Easy/Standard (or no CPU),
CPU35 on Hard. Header, body and per-seat provenance must agree. Format16 rejects future Hard35
metadata; production recording/export remains disabled.

Format18 retains gameplay19/rules10/navigation3 and binds CPU36 on Easy/Standard (or no CPU),
CPU37 on Hard. Header, body and per-seat provenance must agree. Format17 retains CPU33/35 and
rejects future CPU36/37 metadata, including internally consistent forged identities.
Production recording/export remains disabled.

Format19 retains gameplay19/rules10/navigation3 and binds CPU38 on Easy/Standard (or no CPU),
CPU39 on Hard. Format18 retains CPU36/37 and rejects future CPU38/39 metadata even when its
header, body and provenance agree. This private format19 does not change live WebSocket19.
Production recording/export remains disabled.

Formats20 through 27 retain gameplay19/rules10/navigation3 while advancing their closed CPU
pairs through CPU40/41, 42/43, 44/45, 46/47, 48/49, 50/51, 52/53 and 54/55 respectively.
Format28 binds CPU56 on Easy/Standard and CPU57 on Hard. Archived format29 retains CPU56/58;
archived format30 binds gameplay19, CPU56/59 and ManualV1 Punch activation. Current private
format31 binds gameplay20, CPU60/61 and explicit Punch activation version2. Every reader requires
exact agreement among its header, body and per-seat provenance, and an older reader rejects later policy identities even when all
forged metadata agrees. Accepted inputs are replayed directly; planners are not rerun. These
private formats do not select the live socket or enable production recording/export.

Format31 records accepted movement and Bomb actions, not an invented Equipment action for contact
Punch. Exact inspection compares the complete private State, including each actor's contact latch
and cooldown history, so reconstruction must reproduce the same contact, landing and latch expiry.

Formats6 through 31 normalize their closed operations into shared executors rather than
duplicating lifecycle mechanics. Their headers and accepted source/controller identities remain
distinct; each reader accepts only the finite identity combinations defined for its profile.

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
