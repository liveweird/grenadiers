# Configurable keyboard controls

Defaults are the arrow keys or W A S D for movement, and Space for bombs. Punch, Kick and Bracelet
activate through movement contact; there is no separate Equipment command. E is unassigned by
default, but remains available for a custom movement or Bomb binding.

Open **Keyboard controls** outside the arena to change North, East, South, West or Bomb.
Choosing **Change** replaces that action’s aliases with one physical key. **Restore defaults**
restores the original aliases. Supported keys are ArrowUp/Right/Down/Left, physical KeyA–Z,
Digit0–9, Space and Enter. Modifier chords, browser-reserved keys and auto-repeat do not change a
binding. Tab keeps normal focus navigation; Escape cancels capture, then dismisses the dialog.
A duplicate key produces an inline error and leaves the mapping unchanged.

## Input lifecycle

Opening settings disables arena input and immediately sends a movement stop if needed. The
match clock continues. Changing a mapping clears movement through the old callback before
installing the new mapping. A key held across that change stays blocked until native key-up;
input is neither replayed nor buffered. Bomb placement remains one action per fresh physical
press, with the existing action-ID/deduplication rules. Repeated presses also trigger an armed
remote bomb through the same Bomb action.

Focus, blur, visibility changes, room/round identity, periodic movement refresh and unmount
cleanup apply equally to custom mappings. Focus the courtyard before moving. Unrelated keys and
editable descendants keep their native behavior.

## Stored preferences and migration

The non-secret preference is stored under `grenadiers.controls.v3`:

```json
{"version":3,"bindings":{"north":["ArrowUp","KeyW"],"east":["ArrowRight","KeyD"],"south":["ArrowDown","KeyS"],"west":["ArrowLeft","KeyA"],"bomb":["Space"]}}
```

Reads require exactly five actions, one or two allowed codes per action, globally unique codes
and at most 2,048 raw characters. A valid v3 value wins. Otherwise, a valid v2 preference migrates
by dropping only Equipment while preserving all five remaining bindings, including a custom E
binding. Valid v1 preferences remain supported. Older stored values remain for rollback.
If no stored version is valid, malformed JSON, extra/missing actions, unsupported/duplicate codes
or unavailable storage select immutable defaults. Writes are best-effort. Sessions, capabilities and room state are never stored
in this preference.

## Verification

Run `scripts/gates.sh web` and `scripts/gates.sh e2e` for parser, migration, localization,
reconfiguration and scenario checks. Against the built application, run
`E2E_BASE_URL=http://127.0.0.1:8097 npm --prefix e2e run test:controls`.
The browser scenario remaps through the interface, reloads preferences, checks native movement
and Bomb behavior, restores defaults, verifies that default E sends no Equipment action, and
checks accessibility plus desktop and 320-pixel Polish containment. Traces remain disabled
because admission traffic contains private resume capabilities.
