# Browser matches

The browser combines a React/Mantine room interface with a Three.js/React Three Fiber WebGL arena.
Rust owns movement, collision, bomb placement, explosions, damage, and results. Rendering
never advances the simulation or predicts a different outcome.

## Join and play

1. Open the application, choose **Create room**, select the custom **Board size** / **Rozmiar
   planszy** if desired, and share its eight-character invitation. The default is 15×13; Quick
   duel and Solo always use 15×13.
2. Each other configured human enters that code and chooses **Join room** in another browser/tab.
3. The creator can [configure gameplay](settings.md) while unready. All connected humans choose
   **Ready**; any of them can cancel readiness during the three-second countdown.
4. When the round starts, select the courtyard to give it keyboard focus. Move with arrows
   or W/A/S/D; press Space once per bomb. Hold a direction to keep moving.
5. A surviving player wins the round; simultaneous elimination or the round timeout draws.
   Reach the selected win target (three by default) to take the match. Nonfinal rounds hold their result for three seconds,
   then start a three-second automatic countdown with a fresh world and base upgrades.
6. After the final three-second hold, **Play again** records rematch consent. Either player
   can withdraw it. Consensus among the retained humans returns the same connected players to an
   unready lobby with zero scores; they choose **Ready** to start a new match. **Leave room**
   returns to admission.

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

The renderer retains instanced terrain and shared-atlas billboard objects. Public actor positions
interpolate over the observed publication interval, clamped to 16.7–100 ms, without extrapolation. Terminal states snap to the
accepted final position; reduced motion disables interpolation and pulsing effects. Player
colors, headgear, and HTML seat names distinguish the configured players. The canvas itself
is decorative to assistive technology; room controls, countdown/timer, and result remain HTML.

The canvas resizes to the authoritative arena aspect ratio (11:9, 13:11 or 15:13), with device
pixel ratio capped at two. The board-relative logical surface uses crisp pixelated CSS scaling. Initialization, frame-loop ownership, listeners, and graphics contexts
are released on teardown, including React development StrictMode setup/cleanup. Rendering
failure produces a localized explanation and disables game input.

The [modern retro courtyard](arena-3d.md) uses weathered terrain, original directional player
poses, grenades, connected flame cells and distinct pickup symbols. Reduced motion fixes
decorative animation frames while preserving authoritative position and hazard information.

Beside the arena, the localized **Your items** / **Twoje przedmioty** region is derived only
from the viewer's visible actor. It shows permanent upgrade totals, equipment and weapon slots,
active effect timers, independent Slow-foot stack timers, frozen-action status and remaining
damage grace. Opponent-private cached state never enters this region. Timers use authoritative
snapshot ticks and are not live announcements; a separate polite notice announces each fresh
own pickup once and clears across death, round changes, departure and fresh admission.

## Development and verification

Use the pinned Node version and run `npm --prefix web ci`, then `scripts/gates.sh web`.
The web gate includes schema drift, lint, unused code, unit coverage, and production build.
Run the deployed-stack acceptance with `scripts/gates.sh browser`; its duel scenario uses
actual native sockets, keyboard events, and compositor screenshots, plus Axe checks across
lobby, waiting, running, and result. See [verification](../verification.md) for observed results.

Accepted Classic-v7 HUD examples: [English desktop](../screenshots/gameplay-v7-desktop.png)
and [Polish 320px](../screenshots/gameplay-v7-polish-320.png). Narrow screens use two stat
columns; desktop retains four. All eight labels and values wrap within their own cells.
Accepted Heart examples: [exposed pickup on desktop](../screenshots/heart-desktop.png) and
[equipped Heart in Polish at 320px](../screenshots/heart-polish-320.png).
Accepted Shield examples: [exposed pickup on desktop](../screenshots/shield-desktop.png) and
[active immunity in Polish at 320px](../screenshots/shield-polish-320.png).
Accepted Golden Flame examples: [exposed pickup on desktop](../screenshots/golden-flame-desktop.png)
and [active effect in Polish at 320px](../screenshots/golden-flame-polish-320.png).
Accepted Short Fuse examples: [exposed pickup on desktop](../screenshots/short-fuse-desktop.png)
and [active effect in Polish at 320px](../screenshots/short-fuse-polish-320.png).
Accepted Short Flame examples: [exposed pickup on desktop](../screenshots/short-flame-desktop.png)
and [active curse in Polish at 320px](../screenshots/short-flame-polish-320.png).

All Playwright configurations disable raw framework traces because personal admission
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

Live gameplay snapshots supply visible pickup cells and kinds under Expanded-v5 rules. The renderer draws
distinct bomb, cross/range, speed, Heart and Shield marks above blocks and below
bombs/flames/actors; it never guesses concealed contents. Retained drawings are reused while unchanged and destroyed when removed,
replaced, or reset. The UI performs no collection simulation.

All player rows show available bombs (capacity minus live bombs), capacity, range, and speed
levels 1–5 from authoritative actors. Running/nonfinal-result rows show alive or eliminated; pre-start rows show readiness. Final
match rows show rematch consent, and authoritative vacant seats remain open despite frozen actors. Labels are localized in English and Polish. Player cards stack at narrow widths, with two columns inside each
stat grid. Expanded-v5 rooms also show current equipment
and physical-tick damage grace. A bounded corner aura marks Heart grace; a separate shield ring marks timed immunity; reduced motion
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

One manual Fiber root is initialized lazily for the mounted arena. Room transitions stop its
frame loop, hide the canvas and clear the scene; the next authoritative baseline is painted
before the same canvas is revealed. Room changes retain the renderer and graphics context.
Final unmount hides/removes the canvas synchronously and releases the owned Fiber/GPU resources.

Initialization uses the latest snapshot. A result arriving after final unmount is destroyed;
a result arriving while waiting stays suspended. Failed initialization disables controls and
can retry on a later room activation, without retrying on every snapshot. Reduced-motion
preferences and focus/input clearing retain their existing behavior.

See the [active own-player bar at 320px in Polish](../screenshots/item-bar-polish-320.png).
The collector is equipped, while inventory and timers still come from authoritative snapshots.
Capture provenance is preserved in
[game acceptance history](../history/game-acceptance.md#own-player-bar-v0360).
