# Live round camera presentation

## Scenario: real snapshot-free countdowns produce camera intros on the first and subsequent rounds

Two real human clients in isolated browser contexts configure Classic rules and play through two round starts. The primary
browser freezes only its local presentation clock; the native server continues to own countdowns,
fuses and results. It never fabricates a game snapshot or lifecycle cue. Static central arena pixels
must change between the opening pose and the neutral pose in each round, covering initial renderer
creation and the retained renderer's snapshot-free countdown suspension. The first player plants a
real bomb to end round one. Both clients leave and both owned contexts close. Public screenshots
exclude transport credentials; browser errors and CSP violations fail the test.
