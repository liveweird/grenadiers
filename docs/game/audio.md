# Optional procedural sound

The active browser synthesizes short retro cues for local walking, bomb placement, detonation,
pickup collection, actor elimination, qualified public mob elimination presentations, and accepted
public special-weapon activity. Special cues
come only from weapon pickup events or bombs whose accepted event identifies a non-standard blast
or trigger. The protocol has no weapon-use event, so the browser does not guess one from input or
inventory changes. Sound is off by default. It has no music, countdown warning, spatial
positioning, inferred terrain destruction, victory cue, or attribution. All essential game state and
warnings remain visual.

**Enable sound** is a native button outside the arena. Its explicit gesture creates or resumes
one browser `AudioContext`; restoring `grenadiers.audio.v1` only shows **Sound paused** and never
constructs a context automatically. Muting, hiding the document, reconnecting, leaving the room,
or unmounting stops active voices. Hidden return requires **Resume sound**. The preference stores
exact bounded v2 enabled and 0–100 volume data. Existing exact
`{ "version": 1, "enabled": boolean }` data remains readable at the quiet 60% default. The
localized native volume slider persists changes without creating an `AudioContext`.

Audio follows validated live-contract state after the client accepts it. Event identity is room ID
plus event tick and index. The cursor advances while muted, seeds silently on room, round,
admission, enable, and visibility boundaries, rejects old revisions, and never queues history for
replay. Mob elimination reuses the presentation tracker's guarded continuous-snapshot result; the
audio cursor does not independently infer death from disappearance. Each snapshot coalesces every
cue kind once in critical-first priority. Walking compares
only the accepted visible local player's positions, requires plausible small movement accumulated
over a six-tick cadence, and seeds silently on admission and teleport epochs. Gaps over six ticks
and large displacements reset the travel accumulator instead of producing a catch-up step.
It never observes an opponent's movement, including a cloaked one. Short contact displacements can contribute to this cosmetic local movement cue, because the
public position has no forced-movement marker. There is no dedicated or attributed Bracelet shove cue.

One volume-scaled low-gain master feeds at most four scheduled oscillator/gain voices. Live volume
changes use a short gain ramp to avoid clicks. Cue kinds
use distinct pitch, waveform, duration, and gain shapes with small deterministic pitch variation.
Same-time repeats are coalesced and rate-limited; a footstep never starts alongside a critical
voice and is displaced when a critical cue arrives. Every oscillator has a fixed stop time and
disconnects when ended or silenced. There are no audio assets, downloads, timers, random buffers,
server changes, or game-command inference.

Run `scripts/gates.sh web` and `scripts/gates.sh e2e`. With this worktree's built frontend served
by the native server, run:

```sh
E2E_BASE_URL=http://127.0.0.1:8098 npm --prefix e2e run test:audio
```

The natural-visibility helper launches headed Chromium. On Linux, provide a display with:

```sh
xvfb-run -a env E2E_BASE_URL=http://127.0.0.1:8098 npm --prefix e2e run test:audio
```

The dedicated pinned-Chromium scenario observes real Web Audio context and oscillator lifecycle
through read-only CDP events. This verifies browser audio processing, not hardware audibility,
volume suitability, or listening quality. The owned-browser helper bounds teardown and closes all
contexts, the process, and its temporary profile. Because closing that browser also closes the CDP
target, final context destruction is not claimed as observable protocol evidence; unit tests cover
explicit disposal and `AudioContext.close()`.

Reviewed [desktop and 320-pixel Polish screenshots](../screenshots/audio.md) show the actual interface.
