# Board geometry

Live rooms use 15×13 tiles. The deterministic core also supports checked 11×9 and 13×11
arenas; room admission rejects those sizes until the full runtime and client profile supports them.
The [backlog](../../BACKLOG.md) owns activation work and status.

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

Smaller arenas require rules 6. Historical rules reject them before constructing a state.
Movement footprints, public terrain, Power blast traversal and Punch landings use the arena's
actual dimensions. Punch retains its three-cell wrapping stride: an interior axis of nine
cells visits two other coordinates before returning to its source.

`generate_variable_multiplayer_arena` uses generator 4, the raw arena seed, enabled items,
board preset, player count and round assignment. Soft blocks and concealed items use complete
reflection orbits; Oil retains the global eight-site cap. Exhausted bounded random sampling
produces an empty fallback with the same dimensions and spawn assignment.

Generators 1–3 keep their existing outputs and accepted geometry. Existing live two-player
15×13 rooms retain generator 2, and larger rooms retain generator 3. New geometry does not
change item durations, movement speeds, bomb limits or historical replay identities.

Run focused geometry checks with `cargo test -p grenadiers-core variable_board` and
`cargo test -p grenadiers-core generated_v4`. Release evidence belongs in the
[release record](../releases.md).
