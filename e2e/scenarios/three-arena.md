# Native Three arena acceptance

## Scenario: production GPU finish pass preserves sRGB midtones and separates primary colors

An isolated manual Fiber root renders red, green, blue, and mid-gray unlit swatches through the
production finish pass. Final framebuffer samples must keep the mid-gray channels in a broad sRGB
range instead of collapsing toward linear-byte values, while each primary remains dominant. The
native GPU calibration strip is attached as a screenshot in both browser projects.

## Scenario: native camera frames all board sizes and the complete public entity showcase

The credential-free diagnostic page creates the production Three.js/R3F arena through its public
renderer factory and feeds only public snapshot messages. Desktop Chromium and a 320-pixel,
reduced-motion Chromium context render the 11x9, 13x11, and 15x13 boards. Screenshot attachments
cover empty baselines and a static showcase with all four actor appearances, every public pickup,
timed/remote/contact bombs, equipment and status cues, flame, unlit and burning oil, storm cloud,
lair, and mobs. Each showcase must differ from its board baseline without page, shader, WebGL, or
CSP errors.

## Scenario: projected actor and bomb pixels revert exactly when public rows are removed

Known actor and bomb cells are projected with the production orthographic camera helpers. Adding
their public rows changes native canvas pixels; removing them must reproduce the whole empty canvas
byte-for-byte, including shadows.

## Scenario: native death artwork advances every appearance and expires without lingering pixels

The normal-motion desktop project injects one public presentation death cue at a time for actor
appearances one through four and a mob. Native WebGL screenshots at all four frames and after the
600-millisecond lifetime must show changing artwork for every appearance.
The terminal screenshot is captured without another snapshot and must equal an independently
created pool-only reference byte-for-byte, proving that neither the corpse nor its shadow lingers
while the cosmetic pool persists. An isolated fixture clock holds each age during capture so
software-renderer screenshot latency cannot skip frames.

## Scenario: reduced motion holds every death on frame three then expires from the same birth time

The narrow reduced-motion project injects the same five public death appearances. Two native WebGL
screenshots before expiry must be byte-identical and distinct from the empty canvas, proving the
static frame-three treatment. After 180 milliseconds from the cue's unchanged birth time, a
screenshot captured without another snapshot must equal a pool-only reference with no corpse or
shadow. The isolated fixture clock changes presentation time only; native animation frames still
drive the production renderer.

## Scenario: native death body remains readable over ordinary and burning-oil flames

The desktop project freezes actor-one death frame zero in a floor cell and captures empty,
death-only, flame-only, and combined native WebGL pixels for both an ordinary blast flame and a
burning oil barrel. At least 75 percent of the death-only changed-pixel mask must remain identical
in the combined image, while both controls independently differ from empty and the combined image
still differs from death-only. This proves the body stays readable above either visible flame layer
instead of accepting a merely nonempty combined canvas.

## Scenario: narrow native remote badge preserves actor body pixels and adds a visible cue

The narrow reduced-motion project captures empty, actor-only, and actor-with-remote-cue native
WebGL canvases from public snapshots. At least 85 percent of the actor-only changed-pixel mask must
remain identical when the cue is present, while the combined image must add visible pixels outside
that mask. This rejects a full-scale centered bomb shell that replaces the uniform while proving
the compact remote badge is actually drawn.

## Scenario: native isolated lair entrance is visible for visual review

The desktop project renders a single public lair replacing a hard-wall cell at the center of an
otherwise empty arena and attaches both the empty baseline and lair canvas. The lair image must
differ from empty, providing a focused visual-review artifact for the taller body and upright front
entrance; production geometry unit tests retain the precise orientation assertions.

## Scenario: native blood pools remain beneath living actors and flames

The desktop project creates recipient-safe player and mob pools in separate cells, then places a
living actor over one and an ordinary flame over the other through a later public snapshot. Native
empty, foreground-only, pools-only, and combined canvases must retain at least 80 percent of the
foreground mask while adding visible pool pixels. This supplies a visual ground-decal artifact and
proves actors and hazards remain legible above the cosmetic stains.

## Scenario: native spike-pit phases remain readable on every board and in reduced motion

Desktop and narrow reduced-motion projects render authoritative idle, warning, and raised pit
rows at the center of every supported board. Each phase must change native ground pixels from the
empty baseline, and adjacent phases must remain visually distinct. Reduced motion receives the
same static authoritative frames without a locally advanced fuse or damage state.

## Scenario: duplicate death cues do not darken blood pools and lifecycle resets leave no ghosts

The desktop project creates a pool from a fresh public cue, replays the identical envelope, and
requires byte-identical native pixels so duplicates cannot darken the stain. Presentation reset,
suspension followed by retained-envelope replay, room replacement, and a fresh admission snapshot
must each reproduce the empty baseline with no old or reconstructed pools.

## Scenario: teleport, suspend, room replacement, context restoration, and destroy leave no ghosts

A teleport changes the rendered position, suspension hides the canvas immediately, and a
replacement room reproduces the empty baseline without presentation history. When
`WEBGL_lose_context` is available, a real loss hides the canvas and restoration redraws the
nonempty static showcase. Destroy removes the canvas and cancels its animation frame.
