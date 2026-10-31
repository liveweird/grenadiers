# Arena atlas rendering

Published v0.3.49 makes the canonical [artist source](../../assets/art/README.md) the
source of gameplay artwork. The server still owns movement, collision, damage, visibility and
all item rules; changing artwork does not change those rules.

## Artist changes and normal builds

Edit `assets/art/arena-source.png` within its stable frame rectangles. Keep its paired manifest,
IDs, pivots, animation metadata and blank reserved cells intact. Use the artist workbench and
`scripts/gates.sh art-atlas` to validate revisions before shipping. Optional exported slices are
review assets; gameplay uses shared texture regions.

`npm --prefix web run build` bundles the canonical PNG and JSON through Vite. The Docker web
stage copies the same inputs before that build. No Playwright installation or image-export
step is required to compile the application. Updated source pixels produce a new asset URL in
the build. The separate diagnostic pages are excluded from the production application.

## Runtime ownership

Each production renderer loads and validates the bundled source, decodes it within the PNG
budget, and packs active frames into one padded and extruded texture. The 2048px/16MiB limits,
nearest sampling, integer anchors and single-application opacity remain enforced. Canonical
metadata is independent of the Pixi drawing factories, so production Three rendering does not
load the reference engine.

The [3D arena](arena-3d.md) uses named material maps for floor, wall/block/lair tops and sides,
and trim. The same texture supplies billboard characters and effects. Suspending removes
entities and their private presentation history while retaining the owned atlas; final renderer
teardown releases geometry, materials, texture and render targets. Context restoration repaints
the current baseline. The isolated Pixi workbench retains atlas/procedural pixel parity as a
source-art integrity check rather than a comparison with the 3D perspective.

## Walking animation

Every appearance and direction has four poses. Moving actors cycle through `0 → 2 → 1 → 3`
at 110ms per pose, sharing the catalog's timing and ordering. Standing, frozen, teleported and
reduced-motion actors use pose zero; interpolated positions still come from server snapshots.
The added poses retain the existing heads, uniforms and foot baseline while articulating the
legs and arms. All four uniforms keep their established colors and headgear.

## Bomb animation

Timed bombs pulse their shell and spark from the server's remaining fuse ticks (60 ticks per
second). The pulse period is one second above two seconds remaining, half a second above one
second, and 0.2 seconds in the final second. Each pulse briefly enlarges the shell by 12.5%; its
shadow, pause icon, position and gameplay collision stay fixed. Three timed-shell drawings
show neutral, compressed and expanded phases. Short Fuse and Long Fuse use
their actual remaining time rather than a separate visual countdown.

Stopwatch-paused bombs retain a neutral shell and visible pause mark. Remote bombs and landmines
remain neutral because neither has an automatic countdown. Reduced motion and non-running
rounds also use a stable shell. Re-rendering an unchanged snapshot does not advance the fuse,
predict an explosion or remove a bomb. Neutral states use the original drawing. Bomb animation
metadata provides an authoring preview; gameplay selects poses from the authoritative fuse
phase rather than that fixed preview interval.

## Verification

Run `scripts/gates.sh web`, `scripts/gates.sh e2e`, `scripts/gates.sh art-atlas` and
`scripts/gates.sh atlas-gameplay`. The last command builds the diagnostics target and runs real
Pixi/WebGL comparisons. Its parity scenes pack the procedural baseline sheet through the same
atlas pipeline, so they verify the renderer independently of the adopted artist pixels; its
production-renderer lifecycle checks load the bundled source. Comparisons include a
flattened-opaque reference for pulsing icons and the same procedural-baseline atlas shell for
scaled-bomb smoke checks. Fixed procedural pixel fingerprints independently guard all three
pulsed bomb appearances; they do not depend on artist redraws. Differences from the historical
per-stroke reference remain explicit. Keep the required Playwright Chromium installation for browser
checks; `scripts/gates.sh all` retains its existing source/static scope.

Before release, also exercise the production image through two-player gameplay and privacy
scenarios, desktop/320px reduced-motion captures, context restoration and measured rendering
and input latency. Preserve the existing assertions, coverage floors and zero-retry policy.
