# Complete classic match acceptance

## Scenario: two browsers finish a drawn round, first-to-three match, and consensual rematch

Run separately against the release image with the read-only arena-v1-matches operator seed
fixture. Setup is explicit and missing acknowledgement fails. Two actual browser contexts
create/join and ready through the UI. Native keyboard movement follows the generated open
corridor; one shared bomb gives a real mutual-elimination draw. Three subsequent ordinary
round losses award the opponent a first-to-three match. Both clients agree on scores and
completed records, automatic countdowns, spawn parity, and fresh base world/upgrades.

The final hold gates rematch. One player votes and withdraws, then both consent. The same
two sockets return to occupied/unready Waiting with zero scores/count and no old world;
readying again starts actual round one and resets bomb action IDs without replacing transport.
Screenshots and Axe cover the actual desktop and 320-pixel match result. At 320 pixels, rendered
Polish text ranges for all eight HUD labels stay inside their own cells without overlapping.
A core simulation trace separately proves the fixture's walkable corridor and same-tick shared-bomb draw.
