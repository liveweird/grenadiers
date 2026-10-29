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

Each arena renderer loads and validates the bundled source, decodes it within the PNG budget,
and packs the 145 active frames into one padded and extruded texture. The current pack is
2040×528 pixels: 4,308,480 decoded RGBA bytes, below the 2048px/16MiB budget. Nearest sampling,
integer pivots and single-application shadow opacity preserve the existing pixel art.

The scene uses the same animation, interpolation, status visibility and entity-removal logic
with atlas-backed drawing factories. Terrain retains its cache and deterministic variants.
Suspending or resetting a scene destroys its display nodes while retaining the atlas. Final
renderer destruction releases all region textures and their shared source. Context restoration
reuses that source and refreshes the terrain cache. Failed initialization follows the existing
localized renderer-error flow rather than substituting different artwork.

Procedural drawings remain available for template extraction and reference comparisons.
v0.3.53 replaced the adopted source with a redrawn, AI-generated sheet; the current source is
a Napoleonic redraw (shako and bicorne actors, repainted terrain, bombs, flames, pickups and
cues; see the [provenance](../../assets/art/README.md#provenance)). The original 95 frame contracts and pixels are preserved. Sixteen appended player-three/four
poses and additional animation frames extend the authoring sheet to 1120×1680 with matching pivots/foot anchors; the texture
budget and gameplay collision geometry are unchanged. Live rooms support two to four human/CPU seats.
v0.3.49 adopted the baseline artwork, with one explicit compositing change: status pulses
fade each completed icon once. The old procedural renderer faded every overlapping paint
stroke separately, exposing underlying colors during a pulse. A flat artist PNG retains the
finished icon rather than those hidden strokes. Pulse timing and status visibility stay shared;
this normalization affects only those decorative overlap pixels.

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
