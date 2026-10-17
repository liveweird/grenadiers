# ADR 0015: Match facts and staged first-to-three progression

Status: headless MatchSeries and MatchLifecycle implemented; contract/runtime/browser integration follows.

## Decision

Separate pure first-to-three series facts from occupancy, physics, transport, and rendering.
Keep two bounded scores, one active round, one latest immutable completed-round record, and
one independent final match outcome. Draws award no point; the third win finalizes immediately.
A completed first/second round result may coexist with a later forfeit or interruption.
Only a finalized match suppresses subsequent termination. Store no unbounded round history.

Use actual nonzero round number for the existing game RoundAssignment. Room command epochs
are separate monotonic fences: gaps and canceled countdowns consume no actual round number.
Preserve the last started epoch across rematch. Checked time/epoch/number failures are atomic.
Readiness queries respect the retained absolute-time high-water.

Hold results for 180 authoritative ticks. The pure MatchLifecycle owns the following automatic
180-tick countdown. Rematch votes are idempotent and withdrawable, rejected before the terminal
hold expires. Both eligible votes reset series facts; the owner advances the room epoch once
and returns occupied but unready seats to Waiting. Socket sequences and bindings remain intact.

Resolve departures as one authoritative-tick batch after accepted simulation/scoring. Both
explicit departures abandon; any transport loss interrupts without a fabricated winner;
one explicit departure forfeits to the participating opponent. Before the first actual start,
no match result is invented. A third-win match result already committed in that update wins.

## Integration sequence

Keep live gameplay v2 unchanged while adding the pure MatchLifecycle and dormant namespaced
v3 contracts. V3 adds truthful scores, actual round count, completed-round history, countdown
kind, match results, and rematch votes. It does not add physics rules or gameplay events.
Then build real server adapters and browser match controls, sharing authentication and arena
rendering. Switch production server/browser together only after bounded ownership, projection,
shutdown, and real-browser acceptance pass. No placeholder match facts or second public
transport endpoint is needed. Reserve v0.3.0 for completed M4+M5 acceptance.
