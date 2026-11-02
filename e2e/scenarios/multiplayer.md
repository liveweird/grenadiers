# Multiplayer room configurations

## Scenario: three humans join an explicit three-seat match

Three independent browser contexts create and join an all-human three-seat room through the
localized custom-match form. The creator observes all three canonical player labels.

## Scenario: four humans share the four-seat lobby at 320 pixels

Four independent contexts join an all-human room. The fourth renders the complete four-seat
lobby at 320 CSS pixels without horizontal overflow.

## Scenario: human CPU human assigns both human seats without exposing a CPU join

Two human contexts create and join a three-controller Human/CPU/Human room. The server-owned CPU
is visible with Standard difficulty and never consumes a human admission.

## Scenario: one human and three CPUs creates a ready four-controller lobby

The creator submits Human/CPU/CPU/CPU with explicit Standard difficulty. All four controller rows
are visible, no invitation is offered without a human vacancy, and the sole human can ready
without any spectator or replacement flow.

## Scenario: three CPUs escape their first bombs in a default four-player match

One human creates a default Human/CPU/CPU/CPU match through the UI. Real planner workers
place each CPU's first bomb. Public placement/detonation events identify ownership, and every
CPU must survive its first detonation and the following flame interval. The test uses ordinary
room controls with no injected game state, fixed private AI seed or direct CPU input.

## Scenario: three CPUs keep moving with all optional features disabled

The creator chooses All off and applies the accepted settings revision before starting a
Human/CPU/CPU/CPU room. All three real CPU workers must make physical progress while lairs,
clouds and items remain disabled; the published lair and mob arrays stay empty.

These cases require the active gameplay v20 endpoint. They must not run against an archived deployment.
