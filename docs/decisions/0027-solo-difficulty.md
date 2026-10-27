# ADR 0027: Selectable solo difficulty without delayed hazard execution

Status: accepted. Extends [ADR 0026](0026-solo-controllers.md).

## Context

The accepted CPU design varies reaction cadence while sharing physics and known-hazard
mechanics. Gameplay v12 explicitly permits only Standard; accepting Easy under that version
would create incompatible parsers without an explicit migration boundary.

## Decision

Use gameplay v13 for immutable Easy/Standard creation and matching room/controller metadata.
Keep v12 schemas and compatibility behavior archived. Clients choose only the difficulty;
the server owns the policy identity, randomness and input authority. Retain historical Standard
replay profiles and record the selected current profile in existing format-3 provenance.

Standard retains its cadence. Easy adds a provisional 12-logical-tick pause after a confirmed
Stop ending ordinary clear-board navigation. Delay fresh submission, never completed results.
New observed hazards or unavailable knowledge bypass the artificial pause. Committed routes,
Stops, origin recovery and manual action confirmation keep existing timing. Use bounded
world/epoch-scoped state; repeated observations cannot restart the deadline.

## Consequences and acceptance

Deploy server/browser together and reload older clients. Reconnect/rematch preserve the
chosen profile. This setting does not establish a measured strength ranking. Compare both
profiles on the same reference seeds/scripts, preserving deterministic accepted-input replay,
privacy, logical work limits and exact manual timing. Test bounded pause/progress, hazard
bypass, lifecycle resets, closed metadata, English/Polish selection and narrow-screen access.
