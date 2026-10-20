# Slow foot

Slow foot is an opt-in Expanded v5 curse. Each pickup adds one independent 600-physical-tick
stack, up to six. A stack halves movement speed, so one through six active stacks use
`1/2`, `1/4`, `1/8`, `1/16`, `1/32`, or `1/64` of the actor's current speed upgrade.

## Timing and arithmetic

Collection occurs after movement and starts affecting the following physical tick. Each stack
keeps its own deadline; expiry happens before movement at the exact collection tick plus 600.
Death and round reset clear every stack. Reconnect and replay retain the original deadlines.

All five base rates are divisible by 64. Movement therefore uses the exact integer rate
`base_rate / 2^stack_count`, then the existing 1/60-second residual accumulator. No minimum
movement is forced: six stacks at base speed legitimately produce some zero-budget ticks while
damage and stationary footprint checks still run. Changes in stack count and speed upgrades keep
the fractional residual, and blocked whole-unit movement is discarded as usual.

## Collection and compatibility

An actor with six active stacks is ineligible for another Slow-foot pickup. Eligibility is checked
before distance and tie arbitration, allowing an eligible rival to collect it. If both actors are
capped, the pickup remains and no collection event is emitted. Expiry frees capacity before pickup
collection on that tick; the replacement stack affects movement on the next tick.

Slow foot uses catalog bit 15 and a final generator weight of 15 after Diarrhea. The expanded
catalog supports all 512 subsets while preserving every prior 256-mask generator path. Gameplay
wire version 8, rules version 5, generator version 2, and journal format 2 remain unchanged.

## Acceptance and presentation

Fresh same-image acceptance passes 46 Compose and 46 Kubernetes cases. The ordinary-key
Slow Foot scenario proves one stack's half-speed movement, matching peer positions, exact
600-tick expiry, restored base speed and stopped movement after key release. Domain and server
checks cover independently expiring stacks, the six-stack cap, death/reset and replay.

The [English desktop capture](../screenshots/slow-foot-desktop.png) and
[Polish 320px reduced-motion capture](../screenshots/slow-foot-polish-320.png) show the exposed
pickup and ninth lobby switch before collection. No active Slow Foot timer is visible in these
stills. Runtime behavior is established by assertions; see [verification](../verification.md)
for the preserved failed route, unexecuted preparation and final accepted-source evidence.
