# Shield acceptance

## Scenario: two browsers enable a generated Shield and observe immunity end before a later blast

Run the production image with the pinned bytes from the archive-named mode-0444
`core/tests/fixtures/arena-v2-shield.seed` file (repeating byte 31) through generator3 and
acknowledge them with `E2E_SHIELD_FIXTURE=arena-v3-shield`. The creator turns
off all items, turns on Shield, applies the rules, and waits for authoritative settings acceptance.
Both players ready into rules8/generator3 with lairs explicitly disabled. No state or inventory is injected.

Player One bombs from (2,1), escapes through (1,1) to (1,2), and returns after fire clears
to collect the generated Shield at (3,1). Both perspectives observe its timed effect. The
collector's **Your items** region and one pickup announcement show Shield with authoritative
remaining time; after exact expiry the region returns to no active effects. Capture
the exposed pickup on desktop and the active effect in Polish at 320 pixels with reduced motion.

The collector stays on a second bomb. Both perspectives observe protection and a living
collector, with at most one prevention event per physical tick. Every retained effect observation must equal collection tick + 300 minus physical tick.
Require observations before and after that exact deadline; 299-tick or early-expiry effects
fail. Delivery may coalesce publications without changing authoritative physical time. The effect expires naturally.
A later third bomb eliminates the unprotected collector and awards Player Two the round.

Capture exposed desktop and Polish narrow layouts before collecting the timed pickup. Plant
the protected bomb immediately after collection and require its configured detonation to precede
the event-derived Shield deadline. Capture the active Polish view after both peers have proved
protection; retain the exact expiry and later lethal-blast assertions.
