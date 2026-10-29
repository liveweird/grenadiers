# Heart acceptance

## Scenario: two browsers reveal a generated Heart and observe one charge protect its collector

Run the production image with the pinned bytes from the archive-named
`core/tests/fixtures/arena-v2-heart.seed` file through generator3 and acknowledge them with
`E2E_HEART_FIXTURE=arena-v3-heart`. Use **All off**, explicitly enable
Heart, apply the rules, and wait for authoritative settings acceptance. No test-only world or
inventory endpoint is used.

Two real browser contexts start a rules7/generator3 room. Player one destroys the
nearby soft block, escapes the ordinary blast, waits for its fire to expire and collects the
exposed Heart. Both browsers observe the equipped charge. The desktop exposed-pickup capture,
the existing Polish 320-pixel rival capture, and `heart-collector-equipped-polish-320.png` cover
the art, labels and responsive layout. The collector's English **Your items** region and one
pickup announcement show Heart. Its Polish **Twoje przedmioty** region then visibly shows
**Wyposażenie: Serce** at 320 pixels, while the Polish rival's own region remains empty and does
not expose that equipment.

The player deliberately stays on a second bomb. Both perspectives receive one Heart protection
event, a living actor, an empty equipment slot and positive physical-tick damage grace. After
consumption the collector's localized narrow bar shows an empty gear slot and positive protection
time; after grace expires it shows `0:00`. A third bomb eliminates the unprotected player. There is exactly one protection
event and the normal winner result. Preserve public summaries and failure artifacts, with
personal admission traces disabled and retries zero.
