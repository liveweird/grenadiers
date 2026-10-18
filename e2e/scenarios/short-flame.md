# Short Flame production acceptance

## Scenario: two browsers collect Short Flame and restore base bomb range after its exact deadline

Configure only Short Flame through the lobby, reveal its ordinary generated pickup at(3,1),
and collect it via movement. Both peers observe the captured range-one bomb. Its actual blast
reaches west into(2,1), stops before the known clear cell(1,1), and leaves the escaped actor alive.
Reconnect preserves the action floor and the exact collectionTick+600 timer on every snapshot.
After expiry, a newly planted ordinary range-two bomb eliminates the actor. Capture desktop
and Polish320 reduced-motion screenshots. Use the external seed; no injected game state,
relaxed budgets or retries.
