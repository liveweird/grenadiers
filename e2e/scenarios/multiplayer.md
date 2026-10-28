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

These cases require a gameplay v14 endpoint. They must not run against a v13 deployment.
