# Headless match-series API

`core::room::series::MatchSeries` records trusted accepted rounds and match outcomes. It does
not own occupancy, countdown scheduling, worlds, or sockets. Production MatchLifecycle,
MatchRuntime, and MatchOwner compose it into first-to-three matches.

Call `start_round(epoch, tick)` only at an actual Running boundary. The returned existing
`game::RoundAssignment` counts actual starts, independently of epoch gaps. Commit a simulation
result once with `complete_round(epoch, result, ended_at)`. Scores cap at three; draws score
nothing. `latest_completed_round()` persists across subsequent rounds and later match forfeits.
`outcome()` is separate, final, and authoritative immediately on the third win.

`hold_deadline()` and `ready_for_next(tick)` enforce the 180-tick minimum and retained time
high-water. The future lifecycle also schedules the next countdown. All rejected operations
leave the component unchanged, including arithmetic overflow.

Pass same-tick departures together to `resolve_departures`; do not call it separately per
seat and expect simultaneous semantics. `interrupt` preserves an already final match.
After terminal hold, `set_rematch_vote(seat, ready, now)` accepts idempotent votes/withdrawal.
Both votes return `RematchStatus::Started`, clearing scores, round records, assignments, holds,
and votes while retaining epoch/time high-water. The owner validates current binding/epoch,
advances its command epoch once, and resets its world/input/seed state separately.

Run `cargo test -p grenadiers-core room::series --locked` for scoring, parity, holds,
departure precedence, rematch, and atomic boundary regressions. See [ADR 0015](../decisions/0015-match-series-and-staged-progression.md).
