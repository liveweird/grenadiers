# Multiplayer CPU foundation

The shared CPU implementation uses multiplayer policy v17 on rules 6 for live three- and
four-seat rooms. Live two-player rooms retain policy v16 and rules 5. The
[CPU contract guide](cpu-contracts.md) describes planner boundaries; the
[roster runtime](multiplayer-runtime.md) supplies the certified publication seam.

## Observations and compatibility

Rules 1–5 require two canonical actors and retain the 16-bomb observation/forecast cap.
Rules 6 accepts two to four canonical actors and 32 bombs. Forecast evolution, history and
hypothetical placement use the selected rules' limit rather than raising a global legacy cap.

`protocol::v14::observe_cpu_recipient` derives observations from the same checked recipient
projection used by humans. Cloaked actors, unavailable bomb knowledge, timed effects and
individual Teleporter markers remain private. Planning never receives the complete simulation
or a private arena/simulation random stream.

## Policies and controller ownership

`CpuProfile::MULTIPLAYER_STANDARD` and `MULTIPLAYER_EASY` select Utility v17. Existing
`CURRENT`, `CURRENT_EASY` and historical replay profile checks remain v16. Multiplayer uses
the established utility and bounded forecasting machinery with explicit rules-6 branches.

Each controller owns its participant/world identity, memory, RNG, pending work, route and
cadence. Cross-seat results cannot commit plans or random draws. Cancellation of one controller
does not cancel another. The shared planner retains two workers, a 32-job queue and the
existing per-job work allowance.

Punch and Bracelet proofs inspect all relevant visible rivals and third-party blockers.
Unknown hidden rivals withdraw proofs that cannot be certified. Bracelet also rejects
third-party Bracelet threats that could counter-shove the CPU off its certified approach.
These additional multiplayer guards do not change rules-5 admission semantics.

## Focused checks

Run `cargo test -p grenadiers-core cpu:: --lib` and
`cargo test -p grenadiers-server game::rooms::cpu:: --lib` for planner/controller regressions.
The v14 adapter tests compare CPU facts with human recipient projections. Additional cases
cover forecast capacity, rival interference, both difficulties, independent controllers and
player four. Live multi-CPU scheduling is owned by the room owner/socket integration rather than
this projection API.
