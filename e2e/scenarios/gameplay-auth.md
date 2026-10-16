# Gameplay socket authentication

## Scenario: gameplay authentication preserves credentials and connection ownership

A real browser obtains an anonymous credential in memory and sends it only in the first
WebSocket authentication frame. An unsupported protocol receives a typed incompatibility
response without consuming that credential. A supported connection then authenticates with
the same token. A duplicate connection is rejected while the original owner still receives
an authenticated response to an unseated movement command. Replaying that command sequence
is rejected. No room, seat, or browser gameplay is fabricated by this transport scenario.
