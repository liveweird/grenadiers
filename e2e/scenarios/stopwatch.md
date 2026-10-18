# Stopwatch

## Scenario: Stopwatch freezes voluntary actions while Timed bombs pause

Two real Chromium contexts use the repeating-byte-31 generated-v2 fixture. Both players reveal the
mirrored Stopwatch pickups with ordinary bombs, escape, and wait for fire to clear. English desktop
and Polish 320-pixel reduced-motion views capture the exposed pickups before collection.

Player One plants a real Timed bomb, collects Stopwatch, and remains mobile. Both public streams
observe the same paused bomb with a constant remaining fuse while physical ticks advance. Player
Two holds movement and attempts one ordinary placement from a free legal cell; both are blocked,
the action is not replayed after thaw, and the held direction resumes only at expiry. The original
bomb resumes and detonates at its captured fuse plus the exact paused duration.

Player Two then collects the mirrored Stopwatch. Player One's held movement and one bomb action are
blocked. Player Two places a fresh Timed bomb while holding the effect; its placement event starts
at the full configured fuse with `paused: true`. Player Two escapes normally, Player One thaws from
held input, and the fresh bomb resumes and detonates naturally. Both streams must agree on bomb IDs,
owners, pause state, placement/detonation facts, actor status, and physical deadlines. The scenario
uses no state injection, duration override, seed search, retry, or extended timeout.
