# Atlas gameplay regression

## Scenario: atlas gameplay matches procedural pixels and retains renderer resources through lifecycle changes

The isolated actual-WebGL fixture packs the procedural baseline sheet through the production atlas
pipeline and compares those scene pixels with the procedural reference at device pixel ratios one
and two, independently of the adopted artist source. It covers fractional interpolation, the actor,
bomb, Flame and Oil animation phases, reduced motion, a result with a hidden actor, changed
terrain, reset, and context restoration. Timed bomb shells and sparks use authoritative remaining
ticks at all three urgency bands; paused, remote and contact shells retain their neutral pose.
All four actor appearances exercise the 0,2,1,3 walk cycle in every direction at 110ms steps;
paused, reduced-motion and non-running scenes retain pose zero. Semantic evidence requires exactly
one selected actor or shell frame, including when timed-shell frame zero is hidden. The normalized
comparison uses all three procedural-baseline atlas shell poses before applying their scale,
without relaxing the zero-difference assertion at either device pixel ratio. Separate fixed
pixel fingerprints cover pulsed Standard, Piercing and Power fixtures and the independent raw
procedural renderer. They detect a shared atlas frame/pivot/transform error; the artist's PNG
does not participate in these fixtures.
The same run initializes the production renderer with
the bundled artist source, retains one canvas and atlas across suspend/resume and reduced-motion
changes, restores its context, measures cold startup and a bounded retained-frame sample, and
releases each comparison and production texture owner on final destruction.
The retained production texture includes all four player appearances used by live matches.
