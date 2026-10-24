# Atlas gameplay regression

## Scenario: atlas gameplay matches procedural pixels and retains renderer resources through lifecycle changes

The isolated actual-WebGL fixture compares production atlas scene pixels with the procedural
reference at device pixel ratios one and two. It covers fractional interpolation, the actor,
bomb, Flame and Oil animation phases, reduced motion, a result with a hidden actor, changed
terrain, reset, and context restoration. The same run initializes
the production renderer, retains one canvas and atlas across suspend/resume and reduced-motion
changes, restores its context, measures cold startup and a bounded retained-frame sample, and
releases each comparison and production texture owner on final destruction.
