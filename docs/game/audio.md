# Optional procedural sound

The active v4 browser can synthesize four short cues from accepted public events: bomb placement,
detonation, pickup collection, and actor elimination. Sound is off by default. It has no music,
countdown warning, spatial positioning, inferred destruction, victory cue, or attribution. All
essential game state and warnings remain visual.

**Enable sound** is a native button outside the arena. Its explicit gesture creates or resumes
one browser `AudioContext`; restoring `grenadiers.audio.v1` only shows **Sound paused** and never
constructs a context automatically. Muting, hiding the document, reconnecting, leaving the room,
or unmounting stops active voices. Hidden return requires **Resume sound**. The preference stores
only exact `{ "version": 1, "enabled": boolean }` data, with a 64-character input bound.

Audio follows validated v4 state after the client accepts it. Event identity is room ID plus event
tick and index. The cursor advances while muted, seeds silently on room, round, admission, enable,
and visibility boundaries, rejects old revisions, and never queues history for replay. Each
snapshot coalesces every cue kind once in elimination, detonation, pickup, placement priority.

One low-gain master feeds at most four scheduled oscillator/gain voices. Every oscillator has a
fixed stop time and disconnects when ended or silenced. There are no assets, downloads, timers,
random buffers, server changes, or game-command inference.

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
