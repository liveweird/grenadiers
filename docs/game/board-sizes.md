# Board geometry

Live custom rooms support exactly 11×9, 13×11 and 15×13 tiles. The creation choice is immutable
through rounds, reconnect and rematch; 15×13 is the default. **Create room** and **Play solo against CPU** always
create 15×13 rooms regardless of the custom form's selected **Board size** / **Rozmiar planszy**.

| Configuration | Runtime profile | Rules | Generator | CPU policy |
| --- | --- | --- | --- | --- |
| Two to four seats, 15×13 | Roster owner | 9 | 3 | 23 |
| Two to four seats, 11×9 or 13×11 | Roster owner | 9 | 4 | 23 |

## Checked presets

`room::BoardSize` is the dimension identity. `Arena::new_for_board` requires exactly
`width × height` terrain entries. Both coordinate axes must be in bounds before converting a
cell to a row-major index; an invalid column must never alias a valid cell in the next row.
`Arena::new` retains its original 195-entry, 15×13 constructor.

Borders remain indestructible walls. Spawn corners and their two inward neighboring cells
must be floor without soft blocks. Two players use opposite corners; three-player rounds
rotate the omitted corner, and four-player assignments rotate around all corners. Floor
connectivity validation ignores soft blocks, while spawn escape reservations are checked separately.

## Simulation and generation

Live arenas require rules9. Retained rules6/CPU17, rules7/CPU18 and rules8/CPU20 support the same presets
without being selected for current rooms.
Movement footprints, public terrain, Power blast traversal and Punch landings use the arena's
actual dimensions. Punch retains its three-cell wrapping stride: an interior axis of nine
cells visits two other coordinates before returning to its source.

`generate_variable_multiplayer_arena` uses generator 4, the raw arena seed, enabled items,
board preset, player count and round assignment. Soft blocks and concealed items use complete
reflection orbits; Oil retains the global eight-site cap. Exhausted bounded random sampling
produces an empty fallback with the same dimensions and spawn assignment.

Generators 1–3 keep their existing outputs and accepted geometry. Live rooms use generator 3 on
15×13 and generator 4 on 11×9/13×11. New geometry does not change item durations, movement speeds,
the 32-bomb multiplayer limit, range/item bounds or
historical replay identities.

The [desktop selector](../screenshots/board-sizes-desktop.png) and
[Polish mobile arena](../screenshots/board-sizes-mobile-pl.png) illustrate the creation controls
and an 11×9 running match.
