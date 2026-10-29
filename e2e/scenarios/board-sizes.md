# Variable board sizes

## Scenario: all human controller counts run on every board preset

Independent browser contexts create, join, and start every combination of two through four human
controllers on 11×9, 13×11, and 15×13. Each running world has the selected dimensions, exact
terrain capacity, complete roster, rules7, the matching generator3/generator4 profile, one enabled
cloud, and authoritative movement from the highest occupied seat.

## Scenario: CPU controller counts and difficulties make authoritative progress on every board preset

For every board, controller count, and Easy or Standard difficulty, one human starts a match with
the remaining seats server-controlled. Real snapshots must show each configured CPU moving or
owning a live bomb, and at least one CPU-owned bomb in the room. Configured controller metadata
alone is not accepted as evidence of progress. Every combination also verifies the rules7 cloud
environment independently of controller count and difficulty.

## Scenario: quick duel and solo retain the default board

Changing the custom-match selector does not affect either quick creation path. A quick human duel
and a solo room both report the default 15×13 board from the server.

## Scenario: a small four-human room reconnects and rematches without changing geometry

Four humans play an 11×9 match with the fourth browser at 320 pixels and reduced motion. The
rendered canvas has square per-tile geometry and no horizontal overflow. Seat four moves and
reconnects, three stationary opponents eliminate themselves, all four vote for a rematch, and the
new world retains 11×9 while resetting every score.
