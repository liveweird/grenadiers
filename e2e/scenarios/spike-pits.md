# Spike-pit browser scenarios

Run `playwright test --config playwright.spike-pits.config.ts` against an isolated server
using the read-only round seed `0200000000000000000000000000000000000000000000000000000000000000`
and `E2E_SPIKE_PITS_FIXTURE=arena-v3-spikes-seed2`. The test uses normal lobby controls and
keyboard input; it does not inject authoritative state.

## Scenario: spike pits warn, permit escape, kill occupants and reset across reconnect and rematch

Check default enablement and custom durations, walk a clear seeded route, trigger a warning
and escape, observe raised/retracted phases, reconnect the narrow reduced-motion peer, then
remain on a newly triggered pit and verify death. Rematch retains settings and resets pits.
