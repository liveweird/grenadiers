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
and packs the 91 active frames into one padded and extruded texture. The current pack is
2040×320 pixels: 2,611,200 decoded RGBA bytes, below the 2048px/16MiB budget. Nearest sampling,
integer pivots and single-application shadow opacity preserve the existing pixel art.

The scene uses the same animation, interpolation, status visibility and entity-removal logic
with atlas-backed drawing factories. Terrain retains its cache and deterministic variants.
Suspending or resetting a scene destroys its display nodes while retaining the atlas. Final
renderer destruction releases all region textures and their shared source. Context restoration
reuses that source and refreshes the terrain cache. Failed initialization follows the existing
localized renderer-error flow rather than substituting different artwork.

Procedural drawings remain available for template extraction and reference comparisons.
This release adopts the baseline artwork, with one explicit compositing change: status pulses
fade each completed icon once. The old procedural renderer faded every overlapping paint
stroke separately, exposing underlying colors during a pulse. A flat artist PNG retains the
finished icon rather than those hidden strokes. Pulse timing and status visibility stay shared;
this normalization affects only those decorative overlap pixels. The documented flame-frame
gap and future artist redesign remain separate visual changes.

## Verification

Run `scripts/gates.sh web`, `scripts/gates.sh e2e`, `scripts/gates.sh art-atlas` and
`scripts/gates.sh atlas-gameplay`. The last command builds the diagnostics target and runs real
Pixi/WebGL comparisons, including a flattened-opaque reference for pulsing icons and separately
reported differences from the historical per-stroke reference. Keep the required Playwright Chromium installation for browser checks;
`scripts/gates.sh all` retains its existing source/static scope.

Before release, also exercise the production image through two-player gameplay and privacy
scenarios, desktop/320px reduced-motion captures, context restoration and measured rendering
and input latency. Preserve the existing assertions, coverage floors and zero-retry policy.
