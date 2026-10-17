# Network room admission

## Scenario: rooms bind two independent players and preserve authentication after leaving

Three independent browser contexts obtain separate in-memory credentials and authenticate
native game sockets. One creates a room; a second joins by its public invitation code. Each
receives its own seat and identical public room state, while a third receives roomFull.
Leaving is acknowledged only after removal; the remaining player observes the vacancy and
the third context can take it. The departing socket creates a different room without another
authentication handshake. No readiness or physics is simulated by this admission scenario.
