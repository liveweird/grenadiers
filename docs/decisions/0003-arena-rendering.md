# ADR 0003: PixiJS arena with a React interface

Status: implemented in M3 with exact PixiJS 8.22.0; M1 remained headless. Superseded for the
production arena by [ADR 0029](0029-three-billboard-arena.md); retained for artwork diagnostics.

## Context

Grenadiers needs a readable two-dimensional arena with tiles, animated characters, bombs,
flames, and pickups. Rust owns the deterministic simulation. React/Mantine already provides
the surrounding interface, localization, and accessible controls.

## Decision

Use **PixiJS** for the arena and keep menus, room controls, timer, score, connection state,
and settings in React/Mantine. Prefer WebGL for the initial tested browser path; WebGPU can
be evaluated separately. PixiJS 8.22.0 is pinned in the real npm lockfile and included in dependency audits.

One React-owned arena component will create and destroy the Pixi application, including its
ticker, listeners, and textures. Update retained scene objects from public server snapshots;
do not rerender React components for every animation frame. Start with simple shapes or
placeholder sprites, then add texture atlases and animation without changing simulation rules.

Render positions in cell units converted to pixels. Interpolate presentation between snapshots;
visual frame time never advances authoritative ticks, movement, fuses, damage, or victory.
Input produces logical intentions for the server. Clear held input on focus loss, and keep
meaningful status and controls available in HTML rather than only inside the canvas.

## Alternatives and consequences

- **Phaser** offers a broader framework with scenes, physics, input, and other game systems.
  PixiJS fits our narrower rendering boundary while Rust owns the rules and React owns UI.
- **Canvas 2D directly** could draw the initial grid, but would leave asset, sprite, and scene
  management to our code as animation grows.
- **DOM tiles** are easy to inspect, but we choose a dedicated rendering surface for arena
  objects and effects. Browser performance budgets still require measurement in M7.

M1 remained headless. The [browser implementation](../game/browser.md) documents retained
layers, 50 ms interpolation, focus/visibility clearing, reduced motion, device-pixel-ratio
bounds, and deterministic graphics-context teardown. Actual browser acceptance and limits
are recorded in [verification](../verification.md).

References: [PixiJS introduction](https://pixijs.com/8.x/guides/getting-started/intro),
[render loop](https://pixijs.com/8.x/guides/concepts/render-loop), and
[Phaser scenes](https://docs.phaser.io/api-documentation/class/scene).
