# Authoritative round transport

## Scenario: two native gameplay sockets finish one authoritative generated classic round

Two independent browser contexts authenticate, create/join one room, ready, and receive
live public snapshots with generated-v1/classic-v2 metadata, mirrored blocks, safe spawn
openings, and no initially exposed pickups. Refreshed logical movement moves the first actor through its spawn
opening; a fresh bomb is accepted and a repeated action ID is rejected. Its explosion
eliminates its owner. Block destruction and upgrade effects are verified by the separate
reproducible pickup scenario. Both sockets observe an identical authoritative
winner snapshot and actual placement/detonation/round-end events; later result publications
advance room revision while the complete public world stays frozen.
