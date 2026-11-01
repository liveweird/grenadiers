# CPU intent and developer diagnostics

Live CPU27/28 retain CPU26's persistent objective model and checked spike-pit execution. A
subject is an exact exposed
pickup (cell and kind), destructible block, or visible opponent seat. Firing/approach cells can
change without changing that subject. When no productive subject is available, the CPU explicitly
explores, waits, or survives instead of pretending to have an attack target.

## Commitment and execution

A valid subject survives route, pulse and contact-proof invalidation. Those events withdraw all
movement/bomb authority; the next action still needs a fresh certificate. World changes clear
intent. Stale/cancelled results cannot replace accepted intent or restore execution authority.

The default minimum commitment is two seconds. Afterward, a different objective needs a material
utility improvement to interrupt it. Completion, disappearance, lost visibility and immediate
danger can interrupt sooner. A three-second progress deadline and a two-second failed-target
cooldown prevent endless retry; real movement toward the prior approach point counts, including
within-cell alignment. Moving a waypoint alone does not count as progress.

Phases distinguish approach, planting, escape, waiting, re-engagement and evasion. Confirmed own
bombs and their relevant flame window suspend progress deadlines. Waiting stays stopped unless
safety requires evasion. Pickup pursuit cannot plant an unrelated useful bomb; farming/combat
placement must still threaten the selected exact subject and pass the existing escape/contact
proofs. Admission rechecks the target against fresh visible facts. Easy retains its slower cadence.

Reaching an approach tile does not imply a safe bomb is available. If planting is refused,
These policies attempt a certified movement pulse and temporarily defers that exact subject instead
of repeatedly stopping beside it. Safety and contact proofs remain mandatory; a genuinely
trapped CPU may still have to stop. CPU25 and CPU24 retain their archived decision behavior.

## Inspecting a match

The server operator must explicitly enable diagnostics:

```sh
CPU_DEBUG_ENABLED=true docker compose up -d --build app
```

Open `http://localhost:8085/?cpuDebug=1`, start a match with CPUs, and expand **CPU intent
diagnostics**. Each available row shows objective, subject, approach cell, phase, observation
tick and latest switch reason. English and Polish are supported. Normal URLs show no panel and
make no diagnostic requests. Changing the server flag requires a restart, ending in-memory matches.

The flag defaults to false, including Compose and Kubernetes. It grants the read-only view to
connected human room participants; it does not introduce developer accounts or a spectator mode.
The authenticated `POST /api/v1/game/cpu-debug` contract is separate from gameplay19 and the
anonymous diagnostic socket. Its [schema](../../api/cpu-debug.schema.json) and
[OpenAPI](../../api/openapi.json) are authoritative.

A resume capability stays private in an Authorization header; reading does not resume, rotate,
replace or mutate the seat. Requests are bounded to two per second per peer, four concurrent
queries and a separate four-entry owner queue. No seeds, RNG, routes, certificates or private
world identities are exposed. Hidden CPUs, unavailable/hidden targets and bomb-blind views yield
fully unavailable rows. The browser rechecks its latest recipient view and clears old admission
responses on reconnect. An unavailable row is a privacy/availability result, not proof of no intent.

See [CPU boundaries](multiplayer-cpu.md), [security](../security.md) and [testing](../testing.md).
