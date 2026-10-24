# Browser match client

Status: historical v3 browser behavior from the classic-match release. The active application
and existing gameplay endpoint now use the [reconnect browser](reconnect-browser.md); no
separate route or selectable protocol mode is exposed.

The archived v3 client validated every message against its schema and authenticated with version 3.
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

Historical unit and conformance checks retain this transport and presentation behavior. Current
browser screenshots, accessibility checks, and full first-to-three/rematch acceptance complement them;
[verification](../verification.md) records observed evidence.
