# Browser matches

The browser combines a React/Mantine room interface with a PixiJS 8.22 WebGL arena.
Rust owns movement, collision, bomb placement, explosions, damage, and results. Rendering
never advances the simulation or predicts a different outcome.

## Join and play

1. Open the application, choose **Create room**, and share its eight-character invitation.
2. The second player enters that code and chooses **Join room** in another browser/tab.
3. The creator can [configure gameplay](settings.md) while unready. Both choose **Ready**;
   either can cancel readiness during the three-second countdown.
4. When the round starts, select the courtyard to give it keyboard focus. Move with arrows
   or W/A/S/D; press Space once per bomb. Hold a direction to keep moving.
5. A surviving player wins the round; simultaneous elimination or the round timeout draws.
   Reach the selected win target (three by default) to take the match. Nonfinal rounds hold their result for three seconds,
   then start a three-second automatic countdown with a fresh world and base upgrades.
6. After the final three-second hold, **Play again** records rematch consent. Either player
   can withdraw it. Both votes return the same connected players to an unready lobby with
   zero scores; both choose **Ready** to start a new match. **Leave room** returns to admission.

The invitation discovers a room; it is not a secret credential. Each tab requests a separate
anonymous session. Personal `admitted` state gives that tab a distinct resume capability kept only
in memory. An unexpected socket loss reserves the seat for 600 authoritative ticks and triggers at
most three attempts, each with a fresh HTTP session and socket authentication. The latest world
stays visible but controls remain disabled until a full personal baseline returns. Explicit live
departure forfeits the match; two same-boundary departures abandon it. Completed-round records
remain visible independently of a later forfeit/interruption; an already-final match stays final.

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
two. The fixed logical surface uses crisp pixelated CSS scaling. Initialization, ticker ownership, listeners, and graphics contexts
are released on teardown, including React development StrictMode setup/cleanup. Rendering
failure produces a localized explanation and disables game input.

The [pixel-art courtyard](pixel-art.md) uses weathered terrain, original directional player
poses, grenades, connected flame cells and distinct pickup symbols. Reduced motion fixes
decorative animation frames while preserving authoritative position and hazard information.

## Development and verification

Use the pinned Node version and run `npm --prefix web ci`, then `scripts/gates.sh web`.
The web gate includes schema drift, lint, unused code, unit coverage, and production build.
Run the deployed-stack acceptance with `scripts/gates.sh browser`; its duel scenario uses
actual native sockets, keyboard events, and compositor screenshots, plus Axe checks across
lobby, waiting, running, and result. See [verification](../verification.md) for observed results.

Accepted Classic-v7 HUD examples: [English desktop](../screenshots/gameplay-v7-desktop.png)
and [Polish 320px](../screenshots/gameplay-v7-polish-320.png). Narrow screens use two stat
columns; desktop retains four. All eight labels and values wrap within their own cells.

All active-v8 Playwright configurations disable raw framework traces because personal admission
frames contain capabilities. Failure screenshots, public reports, and redacted summaries remain.
Run reconnect acceptance separately with
`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:reconnect`.

English and Polish are available without reconnecting the game session. **Connection
diagnostics** expands the separate diagnostic transport controls; its socket is not a seat.
The panel shows recovery, disconnected/expired seats, and the grace countdown. `seatReplaced`,
close code 4001, `resumeRejected`, and exhausted retries stop recovery and return to admission.
No touch controller is included. A narrow viewport is supported for layout and spectating;
playing currently requires a keyboard. The five keyboard actions can be changed and restored
through the localized [controls dialog](controls.md); the validated preference stays in this browser.

Optional [procedural sound](audio.md) starts muted. **Enable sound** below the arena creates
audio only from that gesture; placement, detonation, pickup and elimination cues follow accepted
public events. Essential information stays visual. Muting, hiding, disconnecting or leaving stops
voices. Returning from a hidden tab requires **Resume sound**; old events are discarded.

## Exposed items and player upgrades

Gameplay-v8 snapshots supply visible pickup cells and kinds under Expanded-v5 rules. Pixi draws distinct bomb,
cross/range, speed and Heart marks above blocks and below bombs/flames/actors; it never guesses
concealed contents. Retained drawings are reused while unchanged and destroyed when removed,
replaced, or reset. The UI performs no collection simulation.

Both player rows show available bombs (capacity minus live bombs), capacity, range, and speed
levels 1–5 from authoritative actors. Running/nonfinal-result rows show alive or eliminated; pre-start rows show readiness. Final
match rows show rematch consent, and authoritative vacant seats remain open despite frozen actors. Labels are localized in English and Polish. The existing single-column
layout bounds the upgrade rows at narrow widths. Expanded-v5 rooms also show current equipment
and physical-tick damage grace. A bounded corner aura marks Heart protection; reduced motion
keeps it static. Reconnecting renders current protection without replaying its consumption.

## Match facts and timers

The header shows round wins and actual round number. The clock uses authoritative room
deadlines during countdown/result/rematch holds and remaining Running time during play.
Result physics freezes while the room clock advances. Inter-round countdowns clear the old
world from shared presentation, though the server retains a completed body for a canceled
countdown. A lone expiry forfeits to the eligible opponent; both expired seats abandon, while an
offline opponent can delay settlement until its own deadline. A target-reaching win already committed at a
boundary takes precedence over expiry. Rematch clears scores and completed-round history. A newer-epoch snapshot can
reset presentation directly even if an intermediate room view was coalesced.

The separate complete-match fixture uses native keyboard controls to prove a real draw,
three wins, fresh round/spawn assignments, consent withdrawal, and rematch over the same
two sockets. See [deployment](../deployment.md) for explicit read-only fixture setup.

Historical [desktop and 320-pixel Polish captures](../screenshots/reconnect.md) record the v4
interface. The active shared settings panel retains the revision fencing introduced in v6.

## Arena renderer ownership

One Pixi application is initialized lazily for the mounted arena. Room transitions suspend its
ticker, hide the canvas and clear the scene; the next authoritative baseline is painted before
the same canvas is revealed. Room changes keep one renderer rather than destroying and
recreating its WebGL context. Final component unmount releases its resources synchronously.

Initialization uses the latest snapshot. A result arriving after final unmount is destroyed;
a result arriving while waiting stays suspended. Failed initialization disables controls and
can retry on a later room activation, without retrying on every snapshot. Reduced-motion
preferences and focus/input clearing retain their existing behavior.
