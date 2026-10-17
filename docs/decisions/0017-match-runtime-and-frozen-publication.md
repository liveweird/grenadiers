# ADR 0017: Authoritative match runtime and frozen publication

Status: dormant simulation adapter implemented; bounded owner and activation follow.

## Decision

Compose the actual pure match lifecycle and simulation rather than synthesize match facts
from the single-round owner. Retain immutable construction/publication profiles and draw
fresh private entropy only at actual starts. Pass each lifecycle assignment into world
construction; the active v2 adapter explicitly retains its first-round assignment.

Freeze physics during Result and MatchResult while the room clock continues. A completed
snapshot must match its own completed-round epoch, end tick, and result. Unfinished terminal
physics keeps no round result and freezes at the independent match outcome tick. Clear the
world at an inter-round countdown, so no old state is published against a new command epoch.

Keep at most three reports and flush on finality; holds contain no repeated events. Check
seat/epoch/phase before touching input high-water state. Rematch resets world/input state
while transport sequence and command epoch remain independently owned. Preserve frame caps,
fallible projection, and private generation records. Construction failure interrupts truthfully.

## Staging

Compile the adapter only with tests until the bounded owner and browser can activate together.
No alternate endpoint, feature flag, fabricated round result, or duplicate authentication is
introduced. Owner timing/departure/shutdown semantics are a separate reviewed increment.
