# ADR 0019: Reserved-seat reconnect and private replay evidence

Status: accepted; staged in v0.3.1–v0.3.8 and activated with gameplay v4 in v0.3.9.

## Decision

Keep ordinary session credentials single-connection. A separate stable random seat capability
supports resume after fresh authentication and may replace a half-open binding. Capabilities
stay in tab memory and personal admission frames, never shared notices, URLs, logs or journals.
Exact connection/generation fences reject old actions and delayed guard drops. Replacement
sends a private notification and closes with application code 4001.

Reserve a disconnected participant for 600 authoritative ticks through running and held rounds.
Running physics/scoring precedes the complete loss batch and expiry; idle losses precede any
automatic start. A third win is final before forfeiture, while first/second wins retain separate
forfeit facts. Staggered dual expiry waits for an eligible return or both deadlines. An eligible
disconnected opponent can win after explicit abandonment. Prematch loss releases immediately.

Hold completed rounds while disconnected; canceled inter-countdowns advance only command epoch.
Return starts a fresh countdown on the next idle tick. Preexisting terminal reservations keep
their original deadline; new post-final losses create no new grace. Rematch requires both
connected. The coordinated v4 server/browser switch replaces the legacy v3 interruption path; v3 DTOs
remain historical compatibility references.

Capture accepted semantic operations privately, including immediate input fences, actual round
seeds and boundary lifecycle events. Replay uses the real runtime. The 60,000-record journal
stays below 4 MiB per room; truncation and irreproducible construction failures are explicit
and cannot certify a complete replay. Compare full semantic private checkpoints, excluding
transport-only publication revision. Profile 2 explicitly records report-draining operations; profile 1 retains its original flush semantics.

## Consequences

V4 closed DTOs describe reserved connections, held results, full personal admission and action
high-water truthfully. Structural contracts, projection, helpers and browser are reviewed in
bounded increments; one coordinated activation migrates the existing owner/runtime/socket.
No second physics implementation, production protocol flag, alternate endpoint or public replay
service is introduced. Restart discards process-local matches, capabilities and journals;
PostgreSQL remains a future option for durable identity and statistics.
