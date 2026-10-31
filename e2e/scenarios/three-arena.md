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

## Scenario: teleport, suspend, room replacement, context restoration, and destroy leave no ghosts

A teleport changes the rendered position, suspension hides the canvas immediately, and a
replacement room reproduces the empty baseline without presentation history. When
`WEBGL_lose_context` is available, a real loss hides the canvas and restoration redraws the
nonempty static showcase. Destroy removes the canvas and cancels its animation frame.
