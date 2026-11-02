# Gameplay socket authentication

## Scenario: gameplay authentication preserves credentials and connection ownership

A real browser obtains an anonymous credential in memory and sends it only in the first
WebSocket authentication frame. Retired protocol 19 receives a typed v20 incompatibility
response even when its presented token is invalid, proving version rejection precedes token
claim. A v20 connection then authenticates with the issued token. A duplicate v20 connection is
rejected while the original owner still receives
an authenticated response to an unseated movement command. Replaying that command sequence
is rejected. No room, seat, or browser gameplay is fabricated by this transport scenario.
