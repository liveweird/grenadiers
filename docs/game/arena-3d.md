# Modern retro 3D arena

The production arena uses Three.js and React Three Fiber. A fixed, elevated orthographic camera
shows a miniature courtyard with shallow chamfered walls, textured brick blocks and a raised rim.
The camera has no yaw: keyboard directions still correspond to the board's rows and columns.
Rust retains movement, collisions, visibility, fuses, damage and victory; rendering never advances
gameplay time or introduces a second physics simulation.

## Characters and artwork

Players, mobs, bombs, pickups and effects use crisp billboard sprites from the canonical
`assets/art/arena-source.png` and its paired manifest. Billboards face the camera and use stable
anchors so feet remain grounded across walking frames. All four uniforms remain distinct.
Visible sprites deliberately remain readable beside terrain; this presentation rule does not
reveal actors or bombs omitted from the recipient's snapshot.

An engine-independent actor view contains public identity, heading, position and status cues.
Future model-based characters can consume that same view. Models, rigs, animation clips and
sprite-specific shadow/transparency changes would still be additional work.

## Shared material textures

The same artist sheet includes ten 48×48 surface frames:

| Surface | Stable frame IDs |
| --- | --- |
| Floor | `material.floor.top.0`, `material.floor.top.1` |
| Indestructible walls | `material.wall.top`, `material.wall.side` |
| Destructible blocks | `material.block.top`, `material.block.side` |
| Monster lairs | `material.lair.top`, `material.lair.side` |
| Arena rim | `material.rim.top`, `material.rim.side` |

Paint these named rectangles without changing frame IDs, locations, pivots or source dimensions.
World materials use a white tint, so their base appearance comes from the sheet. Lighting and
the final colour pass consistently modify the rendered result. The original sprite rectangles
remain in place. Use the [artist workflow](../../assets/art/README.md) to export guides and inspect
the full inventory; exported slices are optional review assets.

## Lighting, motion and privacy

Directional light, hemisphere fill, shadows and textured chamfers provide depth. A small final
pass adds warm colour grading and static ordered dithering. Pixel artwork retains nearest
sampling; blur, animated grain and screen shake do not obscure players or blast cells.

Positions interpolate between received snapshots without extrapolation. Reduced motion uses
static animation frames and immediate positions. Teleports, freezes, room/round changes,
admission baselines, backwards ticks and concealment/reappearance reset presentation history.
Persistent epochs preserve those resets even when React coalesces multiple updates.
Bomb pulses use received fuse facts; the renderer does not decrement a fuse or effect timer.

## Renderer and artist checks

The production build verifies its emitted module graph excludes Pixi reference drawing code.
The shared atlas retains PNG decode, binary-alpha, palette, anchor, padding and GPU texture
bounds. Renderer suspension hides the canvas immediately; final teardown releases instance
buffers, geometry, materials, textures, render targets, observers and the graphics context.

Run `npm --prefix e2e run test:three-arena` for the separate credential-free native WebGL fixture.
It covers board sizes, desktop/narrow viewports, entity removal, reset and context recovery.
The existing atlas/Pixi diagnostics remain useful for source-art integrity. Neither diagnostic
page is part of the gameplay flow. Open work and visual feedback live in [BACKLOG.md](../../BACKLOG.md).
