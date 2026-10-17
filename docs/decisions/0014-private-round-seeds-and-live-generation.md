# ADR 0014: Private round seeds and live generated rounds

Status: implemented; complete first-to-three match progression follows in M5.

## Decision

Production rooms use GeneratedClassicV2, pairing generator-v1 maps with classic-v2 physics.
Construct a world only at the actual countdown-to-Running boundary, using RoundAssignment's
first-round spawns/facing and the absolute lifecycle tick. Retain GenerationRecord separately
in the owner. OS entropy failure interrupts InternalError without creating substitute state.
Canceled countdowns consume no round seed; session tokens and room identities use independent
randomness. Neither the seed nor concealed/pending layers enter public DTOs or normal logs.

By default, obtain fresh 32-byte OS entropy for each actual round. Optional
GAME_ROUND_SEED_FILE is an operator reproduction setting loaded once before listening.
Read at most 66 bytes; accept exactly 64 lowercase hexadecimal bytes with at most one LF.
Missing, unreadable, malformed, or oversized files fail startup with a redacted category.
Fixed mode repeats the loaded seed for every round, independently of room/retry order. File
changes require restart. It is an explicit isolated reproduction mode, not the default game.

## Acceptance

Build one release image. First exercise default random generated-round smoke scenarios, then
recreate that same image with the public fixture mounted read-only. The fixture stays outside
web assets and the final runtime image. Compose uses a dedicated override; Kubernetes uses a
temporary Secret readable by UID 65532, preserving base probes and security restrictions.

A core trace locks generator-v1 capacity/range contents and a surviving reveal/collection route.
Separate real-browser acceptance steers only through native keyboard/public positions, proves
both HUDs and simultaneous old/new captured bomb ranges, and fails without explicit fixture
setup. Default smoke tests rely only on guaranteed openings, not a particular random map.
CI runs both modes. No browser seed/state injection or test-only executable is introduced.
