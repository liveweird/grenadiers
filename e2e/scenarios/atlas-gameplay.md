# Atlas gameplay regression

## Scenario: atlas gameplay matches procedural pixels and retains renderer resources through lifecycle changes

The isolated actual-WebGL fixture packs the procedural baseline sheet through the production atlas
pipeline and compares those scene pixels with the procedural reference at device pixel ratios one
and two, independently of the adopted artist source. It covers fractional interpolation, the actor,
bomb, Flame and Oil animation phases, reduced motion, a result with a hidden actor, changed
terrain, reset, and context restoration. The same run initializes the production renderer with
the bundled artist source, retains one canvas and atlas across suspend/resume and reduced-motion
changes, restores its context, measures cold startup and a bounded retained-frame sample, and
releases each comparison and production texture owner on final destruction.
The retained production texture also includes the two appended, art-only actor appearances while
the live gameplay protocol continues to render its existing two seats.
