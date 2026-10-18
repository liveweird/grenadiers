# Heart acceptance

## Scenario: two browsers reveal a generated Heart and observe one charge protect its collector

Run the production image with the external `core/tests/fixtures/arena-v2-heart.seed` fixture
and `E2E_HEART_FIXTURE=arena-v2-heart`. Keep the default Heart-enabled settings; do not select
the Classic preset. No test-only world or inventory endpoint is used.

Two real browser contexts start an Expanded v5/generator v2 room. Player one destroys the
nearby soft block, escapes the ordinary blast, waits for its fire to expire and collects the
exposed Heart. Both browsers observe the equipped charge. Desktop exposed-pickup and Polish
320-pixel equipped-state screenshots cover the new art, labels and responsive layout.

The player deliberately stays on a second bomb. Both perspectives receive one Heart protection
event, a living actor, an empty equipment slot and positive physical-tick damage grace. After
grace expires, a third bomb eliminates the unprotected player. There is exactly one protection
event and the normal winner result. Preserve public summaries and failure artifacts, with
personal admission traces disabled and retries zero.
