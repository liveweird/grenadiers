# CPU intent diagnostics

Use the exact accepted application image in isolated stacks. With `CPU_DEBUG_ENABLED=true`, run
`E2E_CPU_DEBUG_ENABLED=true npm --prefix e2e run test:cpu-debug`. With the flag unset/false,
run `E2E_CPU_DEBUG_ENABLED=false npm --prefix e2e run test:cpu-debug-disabled`.

## Scenario: normal gameplay makes no CPU diagnostic requests without URL opt-in

A real four-seat match progresses without rendering the panel or issuing diagnostic requests.

## Scenario: opted-in CPU intents transition from lobby to play and render in English and Polish without blocking controls

A human creates a Standard match with three CPUs through the native UI at `/?cpuDebug=1`.
Read-only lobby responses contain unavailable rows, then running play produces strictly validated
intent rows bound to the current room/round/seat. The localized expandable panel shows goals and
switch reasons, normal keyboard movement remains functional, and EN/PL screenshots plus accessibility
checks verify the rendered view. No state or CPU-input injection is used; no network traces or
credential headers are persisted.

## Scenario: operator-disabled CPU diagnostics return no intent and stay disabled across game start

The URL opt-in cannot enable server diagnostics. The panel reports the operator-disabled state
in both languages; the route returns404 without exposing intent or requiring a usable capability.
Server/unit and client projection tests separately cover hidden-target/bomb-blind redaction,
retired credentials, reconnect generations and bounded polling.
