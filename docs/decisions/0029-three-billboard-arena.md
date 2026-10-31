# ADR 0029: Three.js arena with billboard characters

Status: accepted. Supersedes [ADR 0003](0003-arena-rendering.md) for the production arena;
Pixi remains an isolated artwork and reference-rendering tool.

## Context

Grenadiers needs a modern retro miniature world with visible depth, detailed pixel characters,
and artist-editable surface textures. React owns the surrounding interface and Rust owns the
deterministic game. Changing presentation must preserve that authority and recipient privacy.

## Decision

Use Three.js and React Three Fiber behind the existing `ArenaRenderer` interface. A manual
Fiber root registers only required classes. Production imports exclude Pixi and its procedural
drawing factories; the emitted module graph and unchanged bundle budgets enforce this boundary.

Use an elevated orthographic camera with a north-aligned normal view. Bounded, optional
round-boundary camera motion returns to that view; reduced motion keeps it stationary. One world unit represents one tile;
simulation X/Y maps to world X/Z. Build shallow, chamfered terrain and a raised tray. Floors,
tops, sides and trim sample named material frames in the canonical artist sheet. Keep characters,
mobs and effects as anchored billboard sprites initially. An engine-independent actor view
separates snapshot interpretation from character artwork, allowing a later model-based visual.

Apply warm lighting and a restrained static colour-grade/dither pass. Preserve readable hazards,
nearest texture sampling, reduced motion, and existing keyboard directions. Render only public
presentation snapshots. Wall depth never grants access to concealed entities or world state.

## Consequences and acceptance

Model assets, rigs and animation remain additional work if full 3D characters are adopted.
Billboard-specific UVs, transparency and shadows will also need adapting. Arena geometry,
snapshot interpretation, networking and simulation can remain shared.

Preserve interpolation without extrapolation, persistent reset/visibility epochs, authoritative
bomb animation, synchronous suspension, safe context restoration and deterministic GPU ownership.
Require native WebGL colour, visibility, lifecycle, board-size and viewport acceptance, plus
unchanged coverage, asset-validation and compressed-bundle limits. Retain Pixi pixel parity as
an artwork integrity check, separate from evaluating the new perspective.

See the [3D arena guide](../game/arena-3d.md) and
[R3F custom roots](https://r3f.docs.pmnd.rs/api/canvas#custom-canvas).
