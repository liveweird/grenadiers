# Reverse Controls production acceptance

## Scenario: two browsers hold raw east through Reverse Controls and exact expiry

Enable only Reverse Controls, reveal its deterministic pickup at cell (3,1), and capture the
desktop view before collection. Hold the physical east input continuously: the actor first moves
east, collects the pickup, reverses west to the left wall, then resumes east movement at the exact
600-tick expiry. Both peers observe the same countdown while every outgoing movement command remains
raw east. Release the key, verify both peers see an unchanged position across at least three later
authoritative snapshots spanning six physical ticks, and capture the Polish 320px reduced-motion view.
