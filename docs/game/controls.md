# Configurable keyboard controls

The active browser keeps keyboard preferences locally. Defaults remain the arrow keys and
W A S D for movement, with Space for bombs and E for Punch equipment. **Keyboard controls** opens a
localized dialog outside the arena. Choosing **Change** for North, East, South, West, Bomb, or Equipment replaces that
action's aliases with one physical key. **Restore defaults** restores every original alias.

The capture boundary accepts only ArrowUp/Right/Down/Left, physical KeyA–Z, Digit0–9, Space,
and Enter. Modifier chords, modifier-only keys, browser-reserved keys, and auto-repeat do not
change a binding. Tab keeps normal focus navigation. Escape cancels an active capture; after
capture ends, Escape dismisses the dialog normally. A key already assigned to another action
produces an inline error and leaves the mapping unchanged.

## Input lifecycle

Opening settings disables arena input and immediately publishes a movement stop if one is held.
The authoritative match clock continues. Changing a mapping clears movement through the old
callback before installing the new mapping. A physical key held across that change remains
blocked until its native key-up, so input is neither replayed nor buffered. Bomb and Equipment
actions each remain one per fresh physical press and share the server's manual-action sequence.
Existing focus, blur, document visibility, room/round identity, periodic movement
refresh, and unmount cleanup apply with custom mappings as they do with defaults.

Arena ownership stays scoped to its focusable region. Unrelated keys and keys from editable
descendants remain native. Omitting the optional arena binding property selects the stable
default mapping for older presentation callers.

## Local preference boundary

The browser stores the current non-secret preference under `grenadiers.controls.v2`:

```json
{"version":2,"bindings":{"north":["ArrowUp","KeyW"],"east":["ArrowRight","KeyD"],"south":["ArrowDown","KeyS"],"west":["ArrowLeft","KeyA"],"bomb":["Space"],"equipment":["KeyE"]}}
```

Reads require the exact six action names, version 2, one or two allowed codes per action, globally
unique codes, and at most 2,048 raw characters. A valid frozen v1 preference migrates all five
bindings and assigns Equipment to the first free key in E, F, Q, then the remaining supported-key
order; the v1 value remains for rollback. Invalid JSON, extra or missing actions, duplicates,
unsupported codes, and unavailable storage all select immutable defaults. Storage writes are
best-effort. Seat capabilities, sessions, room state, and other game data never enter this preference.

## Verification

Unit tests cover parser corruption, exact v1 migration and unavailable storage, localized capture,
duplicate and Escape behavior, reset and reload persistence, remapped movement/Bomb/Equipment
behavior, repeats, and held-input release during reconfiguration. Run the pinned web and E2E static gates:

```sh
scripts/gates.sh web
scripts/gates.sh e2e
```

With the built application and native server available, run the dedicated browser scenario:

```sh
E2E_BASE_URL=http://127.0.0.1:8097 npm --prefix e2e run test:controls
```

It remaps through the interface, reloads the preference, uses real native keys in a live round,
proves the old key is inactive and the new key moves, places one bomb per press, restores defaults,
and checks Axe plus desktop and 320-pixel Polish containment. Playwright traces remain disabled
because v4 admission traffic contains private resume capabilities.

Reviewed [desktop and 320-pixel Polish screenshots](../screenshots/controls.md) show the actual interface.
