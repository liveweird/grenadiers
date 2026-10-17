# Browser match client

Status: active in the application alongside the v3 server. No separate route or selectable
protocol mode is exposed.

The v3 client validates every message against its schema and authenticates with version 3.
Room updates require the same room, a strictly newer revision, and a nondecreasing command
epoch. A newer-epoch snapshot is sufficient even if a watch channel coalesces the preceding
room view. Epoch changes reset bomb action IDs and arena/control identity while retaining
transport command sequence. Old revisions/epochs and unrelated rooms cannot replace state.

Ready is limited to Waiting and initial countdowns. Rematch consent requires MatchResult,
an elapsed authoritative hold (`room.tick >= rematchAt`), and both seats occupied; a player
can withdraw consent. Consensus is represented only by the server's fresh unready Waiting
view. Admission, correlated leave, bounded counters, authentication failure, and disposal
retain the existing client lifecycle discipline.

The English/Polish panel shows actual round number, first-to-three scores, authoritative
countdown/hold timers, latest completed round, and independent match outcome. A forfeit
never derives a fabricated round result from frozen physics. Authoritative vacancy takes
precedence over stale live/dead actors in a terminal snapshot. The Pixi renderer is shared
through its narrow presentation boundary and resets on room/epoch changes.

Unit/conformance/build checks exercise transport and presentation. Actual browser screenshots,
accessibility checks, and full first-to-three/rematch acceptance complement those checks;
[verification](../verification.md) records observed evidence.
