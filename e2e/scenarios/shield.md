# Shield acceptance

## Scenario: two browsers enable a generated Shield and observe immunity end before a later blast

Run the production image with external mode-0444 `core/tests/fixtures/arena-v2-shield.seed`
(repeating byte 31) and `E2E_SHIELD_FIXTURE=arena-v2-shield`. The creator turns off default
Heart, turns on Shield, applies the rules, and waits for authoritative settings acceptance.
Both players ready into rules 5 / generator 2. No state or inventory is injected.

Player One bombs from (2,1), escapes through (1,1) to (1,2), and returns after fire clears
to collect the generated Shield at (3,1). Both perspectives observe its timed effect. Capture
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
