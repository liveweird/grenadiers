## Scenario: Bracelet shoves a stationary rival one tile without an action or private cue

- Create a two-player room with only Bracelet enabled and the repeating-byte-31 production seed.
- Use ordinary Bomb and movement input to reveal Player One's Bracelet and clear the top and right
  perimeter routes while both players remain alive.
- Stage Player One at `(10,1)` and stationary Player Two at `(11,1)` with no live bomb or flame.
- Move Player One east and verify both recipient streams place Player Two exactly one tile east at
  `(12,1)` without a Player Two command, Equipment action, public event, or private history field.
- Capture the stable English desktop and Polish 320-pixel reduced-motion views and verify the
  destination tile is visibly occupied without horizontal overflow.

Before the shove, the unequipped rival walks into the stationary holder and is blocked at
the cell or footprint boundary. Holding movement cannot overlap them or share a tile, and it does not cause
a reverse shove. The rival then returns to its staging cell for the existing Bracelet proof.
