# ART-ATLAS artist handoff

## Scenario: baseline regions match source pixels and one edited sprite keeps its import position

The browser extracts the baseline sheet from the existing Pixi factories, packs it, imports the
runtime regions through Pixi, and compares decoded pixels. A one-pixel artist edit remains in the
same named frame and position after export.

## Scenario: source validation rejects path-like IDs and painting in authoring gutters

An unsafe stable ID is rejected before it can become a slice path. Opaque paint outside a baseline
frame, including slot gutters and reserved slots, is rejected before packing or allocation.

## Scenario: repeated preview reset and WebGL context restoration do not accumulate canvases or sprites

Three complete resets retain one preview canvas and the same bounded sprite count. A forced WebGL
context loss and restoration preserves decoded pixels from an actual imported terrain region
without duplicating scene contents.

## Scenario: independent exports match decoded runtime pixels and canonical JSON

Two independent exports of the adopted artist source decode to identical RGBA pixels. Their
canonical JSON is identical; the test deliberately makes no promise about compressed PNG bytes.

## Scenario: committed artist source validates and exports through the browser pipeline

The checked-in transparent PNG and stable-ID JSON pair passes source and runtime validation in the
actual browser pipeline, retaining 145 baseline frames and excluding four reserved entries. The 32
new walking frames and six timed-bomb poses are appended after the frozen 111-entry inventory,
without moving any existing slot. The packed runtime remains within budget at 2040 × 528.

## Scenario: all adopted directional actors retain binary alpha and bounded opaque palettes

Decode the canonical artist sheet and discover all 16 directional poses for each of four uniforms.
Require nonempty frames, binary alpha, shared 40,65 foot anchors, bounded 8–48-color palettes and
recognizable forest/plum coat families without pinning image hashes or historical pixels.

## Scenario: workbench Export button downloads the validated runtime pack

The visible Export runtime pack control performs the browser download path after validation. Four
download events expose the canonical atlas PNG and JSON plus the overview and preview PNGs.

## Scenario: workbench captures desktop and 320px reduced-motion proofs with one bounded RGBA texture

The English desktop and Polish 320-pixel reduced-motion workbenches are captured. The preview uses
one RGBA atlas inside the texture budget and a short fixed render sample remains bounded.

## Scenario: concurrent exports and reset serialize different sources without mixing files or preview ownership

Two differently edited source sheets and a reset are requested together. Each export keeps its own
PNG result, mutations complete in request order, and the final preview retains exactly one live
canvas and one owned Pixi texture set.

## Scenario: invalid upload remains visible while language and selected asset details switch to Polish

After an invalid manifest reports its concrete stable-ID error, changing the workbench language
does not replace that error with a ready message. The selected asset facts are rerendered in Polish.

## Scenario: adopted spike-pit phases are distinct binary-alpha bounded-palette tiles

Load the committed source through the browser pipeline. Check all three 48px pit frames
have opaque binary alpha, bounded palettes and distinct decoded pixel signatures.

## Scenario: adopted flame poses join across both tile axes without edge seams

Decode all six adopted flame poses. Opposing horizontal and vertical RGBA edge pixels must
match, with a nonempty opaque connector, so animated adjacent tiles have no boundary gaps.
