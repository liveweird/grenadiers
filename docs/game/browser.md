# Browser duel

The browser combines a React/Mantine room interface with a PixiJS 8.22 WebGL arena.
Rust owns movement, collision, bomb placement, explosions, damage, and results. Rendering
never advances the simulation or predicts a different outcome.

## Join and play

1. Open the application, choose **Create room**, and share its eight-character invitation.
2. The second player enters that code and chooses **Join room** in another browser/tab.
3. Both players choose **Ready**. Either can cancel readiness during the three-second countdown.
4. When the round starts, select the courtyard to give it keyboard focus. Move with arrows
   or W/A/S/D; press Space once per bomb. Hold a direction to keep moving.
5. A surviving player wins; simultaneous elimination or the round timeout draws. Choose
   **Leave room** to return to room admission. This preview plays one fixed-map round.

The invitation discovers a room; it is not a secret credential. Each tab requests a separate
anonymous session and keeps its credential only in memory. Closing the game socket ends
its ownership; this preview cannot reconnect to a claimed credential. A participant leaving
or losing transport during live play interrupts the other player's round.

## Input and presentation

Only a focused, live arena accepts game keys. Held directions clear when focus leaves the
arena, the window blurs, the document becomes hidden, the room changes, or the actor can no
longer play. Inputs in room forms retain their ordinary behavior. Key auto-repeat cannot
create additional bomb actions. Direction refreshes keep server-held input current; they
carry no position or clock authority.

Pixi retains terrain, block, bomb, flame, and actor layers. Public actor positions interpolate
over the 50 ms publication interval without extrapolation. Terminal states snap to the
accepted final position; reduced motion disables interpolation and pulsing effects. Player
colors, helmet shapes, and HTML seat names distinguish the two players. The canvas itself
is decorative to assistive technology; room controls, countdown/timer, and result remain HTML.

The canvas resizes to the fixed 15:13 arena aspect ratio, with device pixel ratio capped at
two. Initialization, resize observation, ticker ownership, listeners, and graphics contexts
are released on teardown, including React development StrictMode setup/cleanup. Rendering
failure produces a localized explanation and disables game input.

## Development and verification

Use the pinned Node version and run `npm --prefix web ci`, then `scripts/gates.sh web`.
The web gate includes schema drift, lint, unused code, unit coverage, and production build.
Run the deployed-stack acceptance with `scripts/gates.sh browser`; its duel scenario uses
actual native sockets, keyboard events, and compositor screenshots, plus Axe checks across
lobby, waiting, running, and result. See [verification](../verification.md) for observed results.

English and Polish are available without reconnecting the game session. **Connection
diagnostics** expands the separate diagnostic transport controls; its socket is not a seat.
No touch controller, audio, generated upgrades, scoring series, or reconnect UI is included
in this first duel preview. A narrow viewport is supported for layout and spectating;
playing currently requires a keyboard.
