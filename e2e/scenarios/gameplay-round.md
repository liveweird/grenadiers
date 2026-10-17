# Authoritative round transport

## Scenario: two native gameplay sockets finish one authoritative fixed-map round

Two independent browser contexts authenticate, create/join one room, ready, and receive
live public snapshots. Refreshed logical movement moves the first actor through its spawn
opening; a fresh bomb is accepted and a repeated action ID is rejected. Its explosion
removes a soft block and eliminates its owner. Both sockets observe an identical authoritative
winner snapshot and actual placement/detonation/round-end events; later result publications
advance room revision while the complete public world stays frozen.
