# Viewport-contained game shell

## Scenario: the game shell contains play and settings at every supported viewport

Real Chromium visits the lobby at 1440×900, 1024×768, 390×844, and 320×568. The document and
body have no horizontal or vertical overflow, the courtyard remains inside the viewport, and the
quick duel, solo, join, and settings controls remain reachable. Opening the settings pane exposes
the custom board controls without page scrolling; closing it hides those controls from interaction
and restores focus to the opener. A screenshot records each closed layout.

## Scenario: a running four-player HUD remains contained at every supported viewport

Four real browser contexts create, join, and start one authoritative match at the supported viewport
sizes. Every player keeps the courtyard, eight-field own-status bar, keyboard button, and sound
control inside the initial viewport without document scrolling. Screenshots record the live four-seat
score strip and each player's private HUD.
The narrow client then duplicates rendered status entries and displays a pickup notice to stress the
bounded horizontal inventory region: every timer remains present and reachable while the arena and
local controls stay fixed in view.
