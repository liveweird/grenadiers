# Live difficulty comparison — 2026-11-05

The [backlog](../../BACKLOG.md#single-player) owns follow-up work. The reusable procedure is in
[CPU evaluation](../game/cpu-evaluation.md#live-roster-difficulty-comparison).

## Workload and source

Source `b0ccf05e85930a5fbf0bf33fdeb82b9672da2af7` runs one fixed-script human and three CPUs
on the same 15×13 arena/environment/settings per matched case: all items, ×5 capacity weighting,
storm, lairs and spikes. Easy62, Standard62 and Hard67 use generator7, pit-layout2,
gameplay21/ContactV2 and private Replay36.

Four arena seeds, two CPU seed sets, two human scripts and three levels produce 48 cases.
Every case terminates naturally, remains in physical round one and reconstructs the complete
owner inspection through both the shadow recorder and accepted-input replay. Real planning,
accepted movement and living cell transitions are nonzero; no planner failure or outstanding
work is hidden. All charged plans remain within 65,536 units.

The first gameplay run passed, but sandboxed BSD resource capture failed on `kern.clockrate`.
The unchanged command then completed with the required host access. All 48 deterministic case
results match between those runs; timing samples are separate. The completed runner retains
clean source/binary identity in `target/cpu-live-round-measurements/results.jsonl`.

## Observations and limits

Each row contains 16 runs. All committed CPU RNG draw counts are zero; both CPU seed sets
produce identical trajectories. There are therefore **eight distinct arena/script trajectories
per level**, not 16 independent behavioral samples.

| Level | CPU bombs | Bombs / 1,000 living CPU updates | Cell transitions / 1,000 living CPU updates | Logical planning work | Idle/scripted human wins |
| --- | ---: | ---: | ---: | ---: | ---: |
| Easy62 | 148 | 2.638 | 19.144 | 79,208,594 | 0 |
| Standard62 | 148 | 2.638 | 19.144 | 79,208,594 | 0 |
| Hard67 | 74 | 1.515 | 21.664 | 173,349,970 | 6 |

Easy and Standard match exactly after excluding the difficulty label. Hard uses fewer bombs,
more planning work and slightly more cell transitions per living update. These reference scripts
and bot-versus-bot outcomes do not establish general strength. Only eight pickup events occur
across the matrix, so pickup coverage is limited. Hard67's separate useful-combination witness
remains valid.

The completed isolated run samples 16,788 plans: p50 129,291 ns, p95 1,201,584 ns,
p99 1,632,875 ns and maximum 1,820,208 ns. These host-specific timings exclude shadow/replay
work and are not hardware requirements. `stationaryUpdates` counts unchanged **cells**, including
subtile movement. Intent counters include retained post-elimination summaries and must not be
interpreted as active-intent duration.

## Confirmed Easy integration issue

Source inspection finds three independent blockers: the Easy whitelist stops before62;
ordinary-navigation arming requires an uncertified movement shape that rules10 admission rejects;
and any spike-pit support withholds the old pause. Active certified programs also clear its arm.
Adding the missing profile alone would not restore current Easy pacing.

A new identity must preserve archived62 and arm only after actual certified navigation
completion plus stopped confirmation, with fresh bounded stationary-safety checks and immediate
cancellation for new danger. No Hard allocation change is justified by this aggregate alone:
its half-budget path requires eligible visible bombs. Arena53/fixed-idle is a reproducible future
Hard diagnostic, requiring its first divergent planning basis and event timeline.

The [v1.3.27 release record](../releases.md#v1327--2026-11-05) owns final verification totals
and exact source/image identities.
