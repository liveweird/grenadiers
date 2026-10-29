# Authoritative round transport

## Scenario: two native gameplay sockets finish one authoritative cloud-disabled generated round

Two independent browser contexts authenticate on gameplay v15, create/join one room, disable optional items
and the storm cloud atomically
through revision-fenced settings after confirming a fresh room enables every available item,
ready, and receive live public snapshots with
generator3/rules7 metadata, a null cloud, mirrored blocks, safe spawn
openings, and no initially exposed pickups. Refreshed logical movement moves the first actor through its spawn
opening. Equipment with Punch disabled is rejected with `invalidPhase` without consuming its manual action ID;
a fresh bomb reuses that ID and is accepted and a repeated action ID is rejected. Its explosion
eliminates its owner. Block destruction and upgrade effects are verified by the separate
reproducible pickup scenario. Both sockets observe an identical authoritative
winner snapshot and actual placement/detonation/round-end events; later result publications
advance room revision while the complete public world stays frozen.
