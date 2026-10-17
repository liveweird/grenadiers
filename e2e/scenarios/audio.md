# Optional procedural sound

## Scenario: native sound gesture creates Web Audio cues and mute prevents later playback

A pinned native Chromium session exposes the read-only CDP WebAudio domain before loading the
game. No context exists before the localized sound-button gesture. Enabling creates one running
real-time context, and a native Space press in a live two-player round produces a validated
public bomb-placement event and a real oscillator node. Muting suspends the context; the later
detonation creates no node. Resuming consumes the current baseline without replay, natural tab
hiding suspends again, and returning shows the paused state until another explicit resume gesture;
that resume runs the same context without replay. Bounded owned-browser teardown then closes every context and its process.
Desktop English and 320-pixel Polish views retain screenshots and pass Axe. Tracing stays disabled.
CDP graph/state evidence proves browser audio processing, not hardware audibility or sound quality.
The target closes before CDP can reliably expose final context destruction; disposal and
`AudioContext.close()` are covered by unit tests rather than claimed as observed browser evidence.
