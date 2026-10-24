# Renderer lifecycle diagnostics

## Scenario: real Pixi renderer lifecycle matrix cell remains observable during full-page capture

The test runs once per lifecycle matrix cell: six projects pair a primary and a peer state, each
empty (`E`), active (`A`) or suspended (`S`). Two isolated Chromium contexts, a 1440-pixel primary
and a 320-pixel reduced-motion peer, load the static renderer diagnostic build with all
cross-origin requests blocked. An active or suspended page is driven through initialize, warmup and
finish; an empty page is never initialized and must stay empty with no canvas and zero updates.
Fixed renderer state must match the cell: no further updates while animation frames keep arriving,
viewport and document sizes as expected, and no unexpected requests. Axe and horizontal-fit checks pass on both pages. A
full-page primary screenshot must then leave update counts, mode, suspend count, and warm history
unchanged. Lifecycle evidence and stall diagnostics are attached, and cleanup failures never mask
the primary failure.
