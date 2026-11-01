# Interrupted CPU escape investigation — 2026-11-01

Four controlled physical-owner episodes on source release
[v1.3.4](https://github.com/liveweird/grenadiers/releases/tag/v1.3.4), commit
`25a68b7e6491857887ba28617987d958fcb6a6eb`, isolate an interrupted-escape weakness.
Work status belongs in [BACKLOG.md](../../BACKLOG.md).

## Construction and results

The fixture uses visible terrain from a captured match, legal centered starts at (1,9)/(13,9),
all item types enabled, actual Short Fuse pickup collection where selected, and controlled
empty hidden loot. It uses Hard35, gameplay19/rules10/navigation3 and the normal queue, owner,
input admission and physical updates. Runtime/State clocks and assignment are validated before
installation. Jobs are pumped before each update; recorded admission rejections are zero.
Cloud and pits are disabled in all four cases. This is a controlled scene, not restoration of
private state or a replay of the captured match.

| Case | Fuse | Lairs | Result | Maximum planning work |
| --- | --- | --- | --- | ---: |
| One CPU | 90 ticks | Disabled | Original escape retained; survives beyond flames | 4,143 |
| Two CPUs | 90 ticks | Disabled | Both routes invalidated; both die | 65,398 |
| Two CPUs | 180 ticks | Disabled | Both die; additional idle/work-exhaustion behavior | 65,536 |
| Two CPUs | 90 ticks | Enabled | Both routes invalidated; both die | 7,581 |

The normal-fuse case is **not** a successful control, and its additional behavior is not
generalized to the enabled-lair dispatch. No mobs are born during the fourth case's escape.

In the enabled-lair case, both bombs are born at tick 191 and detonate at 281. First publication 193
invalidates both original routes while retaining valid own-origin exit grace. Decisions at
196,202,…,256 admit eleven South-3 pulses. Origin grace is correctly consumed after the footprint
leaves the planted cell, by 231; ordinary recovery then continues. From decision 262 the CPU stops.
Both die at 281, at (1536,11411)/(13824,11411), before reaching the turn band.

## Causal boundary

`ForecastEvolution::bombs_match` requires the exact expected bomb set. Each original plant
certificate expects its own new bomb, so the other CPU's simultaneous bomb invalidates it,
even at the opposite corner. This is intentional withdrawal of stale authority.

Conditional recovery permits three moving updates followed by stopping, publication and fresh
planning. The controlled case advances one such pulse every six updates: 33 moving updates
before refusal, while the escape needs approximately 57. Existing conditional routes also have
three stopped updates per leg and at most eight legs; merely assembling current pulses into a
route cannot meet this deadline. The surviving single-CPU case establishes that the original
uninterrupted route has enough time. Worker congestion is unnecessary for this reproduction.

This does not justify ignoring a newly observed bomb, treating owned bombs as stationary,
raising work limits, extending current conditional pulses or guaranteeing safety against arbitrary
future rival actions.

## Repair design boundary

A separately reviewed continuation contract would need a complete conditional escape from fresh
public facts, named stationary assumptions, full forecast/contact checks, correct consumption of
origin grace, and a certified stopped suffix at every checkpoint. It could carry motion across
successful checkpoints while authorizing at most three conditional moving updates before fresh
validation. Missing, stale or changed observations must withdraw continuation at an explicit update.
Existing pulse contracts and archived policies must remain unchanged.

Activation needs new opt-in policy/replay identities, deterministic queued-owner evidence that
both Short Fuse CPUs plant and survive, and negative coverage for stale observations, changed
bombs, displacement, exhausted work and delayed workers. A stopped suffix is bounded protection
under its checked assumptions; stopping after an unexpected action cannot promise survival
against every later change.

## Reproduce the diagnostic

Extract a clean source archive of v1.3.4 into a temporary directory and apply the
[historical diagnostic patch](fixtures/short-fuse-v134.patch) there. From that directory run:

```sh
CARGO_BUILD_JOBS=1 cargo test -p grenadiers-server --locked short_fuse_simultaneous_owner_diagnostic --lib -- --nocapture
CARGO_BUILD_JOBS=1 cargo test -p grenadiers-server --locked short_fuse_lairs_enabled_owner_diagnostic --lib -- --nocapture
```

The first command runs the three lairs-disabled episodes; the second runs the enabled-lair
case. The patch adds a test fixture and diagnostic tracing, without changing planning, physics
or admission. A passing diagnostic harness means the episode executed; it does not mean its
CPUs survived. Convert outcome assertions into actual regression checks when implementing the
repair. Complete original logs/results remain locally under `target/quality-review/short-fuse-v134-*`.
