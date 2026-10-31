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

Remote-armed and Invisible Bombs indicators are compact badges beside the player's head;
they never replace or cover the uniform. Full-frame equipment and protection cues retain
their authored placement. Invisible Bombs still suppresses the remote-armed indicator.

## Shared material textures

The same artist sheet includes ten 48×48 surface frames:

| Surface | Stable frame IDs |
| --- | --- |
| Floor | `material.floor.top.0`, `material.floor.top.1` |
| Indestructible walls | `material.wall.top`, `material.wall.side` |
| Destructible blocks | `material.block.top`, `material.block.side` |
| Monster lairs | `material.lair.top`, `material.lair.side` |
| Arena rim | `material.rim.top`, `material.rim.side` |

Lairs use a taller stone body with the dedicated `terrain.lair` barred doorway upright on
its camera-facing side. Their top and side maps remain separate editable material frames.
The lair body replaces the ordinary pillar visual at that cell; collisions remain server-owned.

Paint these named rectangles without changing frame IDs, locations, pivots or source dimensions.
World materials use a white tint, so their base appearance comes from the sheet. Lighting and
the final colour pass consistently modify the rendered result. The original sprite rectangles
remain in place. Use the [artist workflow](../../assets/art/README.md) to export guides and inspect
the full inventory; exported slices are optional review assets.

## Lighting, motion and privacy

Directional light, hemisphere fill, shadows and textured chamfers provide depth. A small final
pass adds warm colour grading and static ordered dithering. Up to four pooled, local fire lights
warm nearby terrain; public flames and burning Oil supply their positions. Gentle flicker stays
bounded when explosions overlap. Character sprites receive a subtle warm tint at their displayed
position, with public walls and blocks occluding that tint. Hidden bombs and actors supply no light.

Artist-authored flames use six poses, with brief ignition flares, small rising embers and pale
smoke. The renderer caps ignition flares at eight and particles at twelve. These are cosmetic:
the server's flame cells still own damage and duration. Sprite footprints, nearest sampling and
death-card layering preserve readable blast boundaries. The final colour pass remains restrained;
there is no full-scene blur or screen shake.

Reduced motion uses static fire artwork and illumination, with no ignition flashes, flicker,
smoke or embers. Admission into existing fire does not replay ignition. Room/round changes,
backward timelines, long observation gaps, suspension and preference changes reset transient
fire history. A fresh round-ending public blast can still show a short ignition cue.

Positions interpolate between received snapshots without extrapolation. Reduced motion uses
static animation frames and immediate positions. Teleports, freezes, room/round changes,
admission baselines, backwards ticks and concealment/reappearance reset presentation history.
Persistent epochs preserve those resets even when React coalesces multiple updates.
Bomb pulses use received fuse facts; the renderer does not decrement a fuse or effect timer.

## Death animation

Each uniform and the mob have four separate knockout poses in the artist sheet. Normal
playback lasts 600ms, at 150ms per pose; reduced motion shows only the collapsed pose for
180ms. The detached sprite and shadow then disappear. The body remains readable above blast
and burning-Oil overlays, with the hazard visible around it. These are cosmetic cues: elimination,
collision removal, scoring, bomb ownership and round transitions happen immediately on the server.

Player cues require a fresh elimination event and that recipient's currently visible dead
actor position. A cloaked opponent's hidden row cannot supply a death position, so no spatial
cue is drawn for that recipient. Events are latched before React can coalesce client callbacks.
Mob cues use the last public pose when a mob disappears from tightly continuous snapshots of
the same round and lair after fire or spike damage. A long publication gap suppresses
this inferred pose instead of showing an old location. Round-ending and match-winning blasts
can show player and mob cues together.

Death playback expires using local presentation time even if no further snapshot arrives.
Duplicate events cannot restart it. Admission/reconnect, room/round replacement, backwards
timelines, suspension and reduced-motion preference changes purge existing cues. A final kill
still animates when the authoritative phase changes directly to a round or match result.
Use `actor.<seat>.death.0` through `.3` and `mob.death.0` through `.3` for artist revisions;
keep their common `(40,65)` foot anchor and per-appearance palettes.

Small [blood pools](blood-pools.md) remain on the floor after visible player and mob bodies
disappear. They are cosmetic, bounded per tile and cleared with round/admission history.

## Renderer and artist checks

The production build verifies its emitted module graph excludes Pixi reference drawing code.
The shared atlas retains PNG decode, binary-alpha, palette, anchor, padding and GPU texture
bounds. Renderer suspension hides the canvas immediately; final teardown releases instance
buffers, geometry, materials, textures, render targets, observers and the graphics context.
After WebGL context recovery, the first new frame restores the clear colour, shadow map and
atlas upload after Three rebuilds its caches, before making the canvas visible again.

Run `npm --prefix e2e run test:three-arena` for the separate credential-free native WebGL fixture.
It covers board sizes, desktop/narrow viewports, entity removal, reset and context recovery.
The existing atlas/Pixi diagnostics remain useful for source-art integrity. Neither diagnostic
page is part of the gameplay flow. Open work and visual feedback live in [BACKLOG.md](../../BACKLOG.md).
