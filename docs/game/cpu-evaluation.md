# Deterministic CPU evaluation

These deterministic workloads measure controller behavior and planning cost. They are regression
and profiling tools, not rankings or evidence of human-level strength. Fixed human scripts are
constrained reference inputs.

## Live roster difficulty comparison

```sh
scripts/measure-game.sh --cpu-live-rounds
```

This separate release-profile matrix uses the actual four-seat roster owner with one human and
three CPUs, all items enabled, live ×5 extra-bomb weighting, storm, lairs and spikes. It binds
generator7, pit-layout2, gameplay21/ContactV2, Easy68, Standard62 and Hard70 through private
Replay39. The 15×13 arena, environment and initial physical state match across difficulties.
Arena, simulation-root and CPU seeds are explicit independent inputs.

The 48 cases combine four arena seeds, two CPU seed sets, `fixed-idle` and `bomb-retreat` human
scripts, and three difficulty levels. Commands use normal movement refreshes and accepted
manual IDs. Each episode stops at its first natural round result or 10,800 running updates.
Natural timeout is a draw; reaching a measurement bound without a result is censored.

A test-only bounded trace captures committed operations from the real owner. A retained shadow
uses the existing recorder; its complete inspection must match the owner, and accepted-input
Replay39 must reconstruct that same inspection. Tracing stays disabled in production. Overflow,
unsupported post-setup admissions, incomplete queue accounting and replay divergence fail the
measurement. Short tests require actual queued work, accepted CPU inputs and physical progress.

Reports include real bomb/pickup/elimination events, living-cell movement and stationary spans,
alive updates, unattributed block destruction, intent, admission denials, complete queue/work
counts and RNG draws. Easy cadence counters record arms, completed pauses, deferred planning,
bypasses and guard work; Standard/Hard counters stay zero. The full matrix requires actual Easy
deferral and completed pauses, alongside physical progress. Dead time does not inflate stationary metrics. Planner timing excludes
shadow/replay work; wrapper wall time includes the whole workload. Run timing comparisons on a
quiet host. Output under `target/cpu-live-round-measurements/` retains source, toolchain, binary
identity and terminal/censored results. Interpret pressure and movement alongside survival;
these scripts do not establish a monotonic difficulty or win-rate ordering.

`stationaryUpdates` and its longest span mean no **cell transition**, including movement within
one cell. They do not measure complete physical stillness. Intent counts sample the last-known
controller summary and may persist after elimination; they are not active-goal durations.
The [dated comparison](../history/cpu-live-evaluation-2026-11-05.md) records the observed seed
coverage and current Easy/Standard equivalence.

## Broader seed soak

```sh
scripts/measure-game.sh --cpu-live-soak
```

This opt-in 192-case sweep complements the fixed comparison matrix. It uses eight different
arena seeds, two independent simulation-root seeds (including a different item-randomness
stream), two CPU seed sets, both reference scripts and all three difficulties. Each episode
runs through its first natural result or the same 10,800-update bound. The real queued owner,
complete shadow inspection, accepted-input replay and work/queue invariants remain mandatory.
There are no retries or seed searches. The regular comparison matrix and its output stay unchanged.

Reports under `target/cpu-live-soak-measurements/` retain source and binary identity, every seed,
living movement, stationary spans, events and bounded planning work. Investigate long stationary
spans in context: safe waiting, movement within a cell and frozen actors are not themselves
proof of a stall. This sweep tests deterministic simulation episodes, not wall-clock network
load, continuous multi-round operation or human difficulty.

## Archived Solo workloads

The archived native Solo evaluation retains CPU15 and generated ExpandedV5 rounds. Its
`--cpu`, `--cpu-rounds`, `--cpu-easy` and `--cpu-easy-rounds` meanings and output paths remain
unchanged; they do not measure the live roster profiles above.

Routine tests use small fixed cases to check deterministic traces, bounded termination and the
controller's queue/admission invariants. Arena-generation seeds and private CPU seeds are supplied
separately: changing one does not silently change the other. Generated smoke coverage uses the
same all-items settings and owner/Driver path as a normal Solo round; it does not replace the
targeted planting, escape, goal, privacy and replay regressions.

The larger ignored matrix combines four arena seeds, two independent CPU seeds and two named
human scripts:

- `idle` submits no human input.
- `bomb-retreat` repeats a 360-tick corner loop: East at tick 0, Stop at 20, Bomb at
  21, West at 22, Stop at 42, South at 43, Stop at 63, North at 270 and Stop at 290.
  Held movement refreshes every ten ticks within each leg, matching normal input expiry.
  Its opening plants from `(2,1)`, retreats through `(1,1)` past the full blast footprint
  into `(1,2)`, and waits
  past the first ordinary fuse and flame window before returning. This is a fixed safe-opening
  workload, not a guarantee against later CPU interference.

The opening matrix runs each combination once for at most 1,800 running ticks, without hidden retries. A case that
reaches its bound is reported as `censored`, never converted into a draw or win. A physical round
result terminates the case normally.

The deterministic result records seed identifiers and schedule names, terminal or censored state,
running ticks, CPU survival, accepted movement and manual inputs, first observed CPU Bomb placement
and pickup ticks, visited cells, stationary ticks, planning attempts/results/cancellations, total and maximum bounded
planning work, and committed RNG draws. Optional first-event values remain null when their event
was not observed. The evaluator does not infer causes such as self-elimination.

## Running the checks

Run routine coverage in the normal test profile:

```sh
cargo test -p grenadiers-server cpu --lib
```

Run the full matrix separately through the release measurement runner. It supplies exact source
and toolchain identity, uses the release profile and writes the captured report under
`target/cpu-measurements/`:

```sh
scripts/measure-game.sh --cpu
```

The report includes the actual profile/debug-assertion state, room profile, matrix definition,
deterministic case results and planner timing percentiles. Planner timings measure actual planning
work separately from deterministic decisions; they are descriptive observations with no hardware
threshold. The evaluator does not claim the production-disabled replay recording policy because
this test harness records evaluation data.

## Full rounds and replay

Use the same declared seed/script combinations through the first natural round result or the
existing 10,800-update round limit:

```sh
scripts/measure-game.sh --cpu-rounds
```

This separate workload writes to `target/cpu-round-measurements/`; `--cpu` retains its opening
matrix and output path. A natural timeout is a real Draw result. A measurement cap without a
result stays censored. Every generated episode must replay its accepted inputs to the exact
runtime inspection through the existing replay engine, without rerunning the CPU policy.
Truncated journals and replay mismatches fail the run; recording capacity is unchanged.
Replay verification is reported separately and is outside planner timing samples.

## Interpretation and limits

Compare exact deterministic traces only under the same source, profile, seed identifiers and
logical schedule. Work is measured in planner budget units and every attempt remains at or below
the production allowance; it is not elapsed time. Timing percentiles depend on the host and should
not be mixed into trace equality or gameplay acceptance.

The matrix can expose regressions in liveness, admission, bounded work and repeatability. It cannot
establish balanced difficulty, general strategy quality or performance against people. Those
questions require broader workloads and human feedback tracked outside this guide.

## Comparing difficulty settings

Use `scripts/measure-game.sh --cpu-easy` and `--cpu-easy-rounds` for separately labeled Easy
matrices under `target/cpu-easy-measurements/` and `target/cpu-easy-round-measurements/`.
Standard retains `--cpu` and `--cpu-rounds`. Both use the same seeds, scripts, caps and replay
checks. Compare cadence, pickup/progress, stationary time, planning cost and outcomes; slower
decisions need not produce a monotonic survival or win-rate ordering.

## Current arena strategy regression

`cargo test -p grenadiers-server queued_cpu23 --lib` exercises the real queued four-seat
controller with Golden Flame, a storm and current mob navigation. It records distinct physical
bomb identities, crate count, collection and movement after collection, alive-player updates,
settled cell transitions and bounded work. Worker proposals alone do not establish actual bomb
use, and sub-tile motion does not establish navigation progress.

The fixed generated scene supplements synthetic cases for unreachable pickups, changed goal
values, attained goals, optional planting budgets and safe alternate first movements. Historical
CPU21/22 fixtures retain explicit archived profiles. These tests establish concrete progress and
safety regressions; they do not rank strength against human players.
