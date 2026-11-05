# Certified Easy cadence — 2026-11-05

## Problem and boundary

The previous live comparison found Easy62 and Standard62 identical. The archived cadence gate
omitted current profiles, required a movement shape rejected by current rules, rejected any
spike metadata and was cleared by active certified execution. Easy68 retains Standard62 planner
output and adds a separate post-completion controller state machine. Archived Easy62 remains
unchanged; private Replay37 closes Easy68/Standard62/Hard67 while Replay36 keeps its original
constructor and exact-replay coverage.

Only an actually admitted ordinary movement pulse may arm a dormant candidate. Completion and
physical Stop precede the fixed confirmation-plus-12-tick deadline. Fresh publication checks
withdraw the optional pause for danger or changed authority; active escapes and manual/Teleporter
execution remain immediate. The stationary guard supports distant pits and active storm clouds,
charges variable scans and retains the existing work limits.

## Physical evidence

The generated default-feature paired queued-owner test records real Easy deferrals and resumed
settled-cell movement; Standard records no cadence activity. A separate 720-update, 60 Hz
native two-worker episode observes physical movement, an owned bomb, confirmed Stop at update
198, resumed settled-cell movement at 223, and the first safety bypass at 672. The focused native
run records 16 arms, 14 completions, 56 deferrals and one bypass; all 18 dequeued jobs are delivered
with no failures, cancellations or drops and maximum planner work 57,986. The episode was extended
from 480 to 720 updates because the shorter duration did not reach a bypass; its seed/content and
assertions were not changed. Full-profile repetitions belong to the release runner below.

## Matched live comparison

Clean source `dd0130f22229395599d47bd45c738927aeaaca20` passes the actual release runner,
`scripts/measure-game.sh --cpu-live-rounds`, in 10.93 seconds. All 48 episodes terminate naturally
and match both their shadow and exact accepted-input replay. Standard/Hard's 32 complete
measurements equal v1.3.27 byte-for-byte after omitting the newly added zero cadence field.

| Metric | Easy68 | Standard62 | Hard67 |
| --- | ---: | ---: | ---: |
| CPU bombs placed | 136 | 148 | 74 |
| Living CPU updates | 57,272 | 56,100 | 48,836 |
| Cell transitions | 1,012 | 1,074 | 1,058 |
| Total planner work | 72,986,160 | 79,208,594 | 173,349,970 |
| Completed pauses | 340 | 0 | 0 |
| Deferred planning publications | 1,380 | 0 | 0 |

Easy records completed pauses in 14 of 16 cases, 2,600 armed candidates, 2,254 bypasses and
108,160 separate guard-work units. All planner attempts stay at or below 65,536 work units.
Both CPU seed sets still make zero RNG draws, leaving eight distinct arena/script trajectories
per level, each repeated twice. This establishes exercised safe pacing, not a strength ranking.
Last-known intent counts may persist after death; unchanged-cell spans include subtile movement.
Human playtesting remains necessary.

The ignored raw artifact is `target/cpu-live-round-measurements/results.jsonl`; the retained
comparison is `target/quality-review/easy68-comparison-summary.json`. Full local, Linux and
same-image runtime evidence is recorded in the [release record](../releases.md#v1328--2026-11-05).
