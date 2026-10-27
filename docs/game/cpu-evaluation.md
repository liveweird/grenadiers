# Deterministic CPU evaluation

The native Solo evaluation exercises the actual private CPU controller in generated ExpandedV5
rounds. It is a regression and profiling workload, not a public game mode, ranking system or claim
about human-level difficulty. Its fixed human scripts are deliberately constrained reference
inputs.

## Workloads

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

Each combination runs once for at most 1,800 running ticks, without hidden retries. A case that
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

## Interpretation and limits

Compare exact deterministic traces only under the same source, profile, seed identifiers and
logical schedule. Work is measured in planner budget units and every attempt remains at or below
the production allowance; it is not elapsed time. Timing percentiles depend on the host and should
not be mixed into trace equality or gameplay acceptance.

The matrix can expose regressions in liveness, admission, bounded work and repeatability. It cannot
establish balanced difficulty, general strategy quality or performance against people. Those
questions require broader workloads and human feedback tracked outside this guide.
