# Local deployment

Grenadiers is packaged as one image: the Rust process serves both the API and the built React
application on port 8085. Active matches live in that process, so the initial deployment
intentionally runs one replica with a `Recreate` update strategy. A restart ends active matches;
horizontal scaling and recovery require match routing and persistence designs first.

## Docker Compose

The image build uses the repository's reviewed `Cargo.lock` and pinned base-image digests:

```sh
docker compose up --build -d
curl --fail http://127.0.0.1:8085/api/v1/health
curl --fail http://127.0.0.1:8085/api/v1/ready
docker compose down
```

PostgreSQL is optional and disabled by default. The `persistence` Compose profile starts a local
database, but the app only uses it when `DATABASE_URL` is explicitly set. No application schema
exists yet.

## Local Kubernetes

Build the local image and make it available to the cluster:

```sh
docker build -t grenadiers-app:local .
kubectl kustomize k8s
kubectl apply -k k8s
kubectl -n grenadiers rollout restart deployment/grenadiers
kubectl -n grenadiers rollout status deployment/grenadiers
kubectl -n grenadiers port-forward service/grenadiers 8085:8085
```

OrbStack can use images from its Docker daemon directly. For kind, minikube, or another local
cluster, load `grenadiers-app:local` with that cluster's image-loader command before applying the
manifests. The explicit rollout restart recreates the Pod after reusing the mutable local image
tag; applying unchanged manifests alone would leave the previous binary running. Then open <http://localhost:8085> or call the health endpoints above. The WebSocket
origin allowlist accepts the `localhost:8085` and `127.0.0.1:8085` port-forward URLs.

A port-forward remains attached to the selected pod even when started through a Service.
After any rollout or fixture change, stop the old forward, wait for the replacement pod,
start a new forward, and require successful health/readiness responses before opening the
game or running browser tests. A healthy replacement does not repair an old forward.

The base manifests contain no credentials or PostgreSQL resources. If durable data is introduced,
supply `DATABASE_URL` from a Kubernetes Secret and deploy the database separately. Readiness has a
three-second probe timeout because a configured database check may take up to two seconds.

The manifests can be checked offline with `kubectl kustomize k8s`. A successful render validates
composition only; image startup, probes, WebSockets, and cluster policy still require a live deploy.

## Kubernetes schema guardrail

`scripts/gates.sh k8s` renders Kustomize and validates the rendered resources with
**kubeconform 0.8.0**, strict Kubernetes **1.33.0** schemas from a pinned schema-repository
commit. Install `kubectl`, kubeconform, and the web package's locked npm dependencies first.
The CI workflow downloads the Linux
kubeconform archive and verifies its SHA-256 inherited from Flow before running this gate.
The gate checks the same rendered YAML for one replica, the Recreate strategy, absence
of an autoscaler targeting the game Deployment, and consistent Service routing. It rejects
duplicate YAML keys. Local installations require the appropriate platform binary. Schema validation may require
network access to the pinned schemas; successful rendering alone does not pass this gate.

## Reproduce a generated pickup round

Default rooms draw fresh private OS randomness only at an actual round start. For isolated
reproduction, set `GAME_ROUND_SEED_FILE` to a readable file containing exactly 64 lowercase
hex characters, optionally followed by one LF. Startup reads once before listening and fails
on missing/unreadable/malformed/oversized input. The loaded seed repeats for every room/round;
file edits require restart. Keep ordinary deployments in the default random mode.

After the default browser gate, reuse the built image with the checked public fixture:

```sh
docker compose -f docker-compose.yaml -f docker-compose.pickups.yaml up -d --no-build --force-recreate app
curl --fail http://127.0.0.1:8085/api/v1/ready
E2E_PICKUP_FIXTURE=arena-v1-pickups npm --prefix e2e run test:pickups
docker compose -f docker-compose.yaml -f docker-compose.pickups.yaml down
```

The override mounts `core/tests/fixtures/arena-v1-pickups.seed` read-only. That fixture is
outside browser assets and the final runtime image. To reproduce in isolated Kubernetes,
create a temporary Secret from the same file, mount its key at `/run/grenadiers/round.seed`
with `defaultMode: 0444`, and set the environment variable to that path. UID 65532 must be
able to read it. Retain base probes, one replica, and all security restrictions; use the same
image digest, wait for rollout/readiness, port-forward, and run the same fixture command.
Remove the temporary Secret/namespace afterward. No seed is sent through an API or browser flag.

## Reproduce Heart collection and protection

Use the same built image and the explicit Heart fixture:

```sh
chmod 444 core/tests/fixtures/arena-v2-heart.seed
docker compose -f docker-compose.yaml -f docker-compose.heart.yaml up -d --no-build --force-recreate app
curl --fail http://127.0.0.1:8085/api/v1/ready
E2E_HEART_FIXTURE=arena-v2-heart npm --prefix e2e run test:heart
docker compose -f docker-compose.yaml -f docker-compose.heart.yaml down
```

Wait for readiness before running the scenario. It retains the default Heart-enabled settings,
reveals and collects a generated Heart through ordinary controls, absorbs one blast, then proves
the charge is gone with a later lethal blast. Kubernetes uses the same isolated Secret procedure
and a fresh port-forward after every rollout. Keep this seed outside the final runtime image.

## Reproduce opt-in Shield protection

Use the same built image with the separate Shield fixture:

```sh
chmod 444 core/tests/fixtures/arena-v2-shield.seed
docker compose -f docker-compose.yaml -f docker-compose.shield.yaml up -d --no-build --force-recreate app
curl --fail http://127.0.0.1:8085/api/v1/ready
E2E_SHIELD_FIXTURE=arena-v2-shield npm --prefix e2e run test:shield
docker compose -f docker-compose.yaml -f docker-compose.shield.yaml down
```

Wait for readiness after recreating the service. The scenario enables Shield through the real
lobby, reveals and collects the generated pickup, survives a blast, waits for expiry, then
proves a later blast is lethal. Mount fixtures externally and read-only; keep seeds out of images.

## Reproduce a complete classic match

The existing seeded scenarios select the Classic settings preset before readiness, keeping
their original generated layouts while the room retains Expanded-v5/generator-v2 identity.

Reuse the same image after default and pickup acceptance, with the tracked open-corridor
fixture. This is the same strict read-once seed-file mechanism, with a different public seed.

```sh
docker compose -f docker-compose.yaml -f docker-compose.matches.yaml up -d --no-build --force-recreate app
curl --fail http://127.0.0.1:8085/api/v1/ready
E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:matches
E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:reconnect
docker compose -f docker-compose.yaml -f docker-compose.matches.yaml down
```

Poll readiness before tests. The fixture requires explicit acknowledgement and proves a real
draw, three wins, resets, and rematch over two existing sockets. The separate reconnect suite
checks a real transport loss, full-world restore and action high-water, connected takeover,
uniform rejection, and expiry after eleven real seconds. Kubernetes reproduction uses
the same temporary read-only 0444 Secret procedure with `arena-v1-matches.seed`. The seed is
never packaged into the runtime image or sent to a browser.

## Graceful stop

SIGTERM/Ctrl+C signal unfinished rooms with ServerStopped while preserving already-final match
outcomes. The process explicitly waits up to five seconds for upgraded game/diagnostic
sessions and room cleanup; final publication/close is best effort. Keep Compose's ten-second
and Kubernetes' fifteen-second stop grace periods. Transport reconnect reserves a seat for 600 authoritative ticks while this process remains alive.
A process restart still discards matches and capabilities; this is not restart recovery.

## Reproduce opt-in Golden Flame range

Use the same built image with the separate production fixture:

```sh
chmod 444 core/tests/fixtures/arena-v2-golden-flame.seed
docker compose -f docker-compose.yaml -f docker-compose.golden-flame.yaml up -d --no-build --force-recreate app
# Wait for /api/v1/ready before running the scenario.
E2E_GOLDEN_FLAME_FIXTURE=arena-v2-golden-flame npm --prefix e2e run test:golden-flame
docker compose -f docker-compose.yaml -f docker-compose.golden-flame.yaml down
```

The scenario enables Golden Flame through shared settings, reveals and collects its generated
pickup, escapes a radius-eight blast, resumes with the same deadline and verifies base range
after exact expiry. The seed file stays external to the image and must remain read-only.

## Reproduce opt-in Short Fuse timing

Use the paired candidate image with its separate production-seed fixture:

```sh
chmod 444 core/tests/fixtures/arena-v2-short-fuse.seed
docker compose -f docker-compose.yaml -f docker-compose.short-fuse.yaml up -d --no-build --force-recreate app
# Wait for /api/v1/ready to return 200 before running the scenario.
E2E_SHORT_FUSE_FIXTURE=arena-v2-short-fuse npm --prefix e2e run test:short-fuse
docker compose -f docker-compose.yaml -f docker-compose.short-fuse.yaml down
```

The scenario enables only Short Fuse and a five-second base fuse through the actual shared
settings, reveals and collects the pickup, checks shortened deadlines and detonation on both
peers, reconnects, and checks expiry followed by ordinary timing. The fixture remains external;
never bake it into the application image. Equivalent Kubernetes acceptance mounts the same
mode-0444 seed and runs the scenario through the Service port-forward. Recorded acceptance
status remains in [verification](verification.md); these reproduction commands do not imply a pass.

## Reproduce opt-in Short Flame range

Use the paired candidate image with its separate external production seed:

```sh
chmod 444 core/tests/fixtures/arena-v2-short-flame.seed
docker compose -f docker-compose.yaml -f docker-compose.short-flame.yaml up -d --no-build --force-recreate app
# Wait for /api/v1/ready to return 200 before running the scenario.
E2E_SHORT_FLAME_FIXTURE=arena-v2-short-flame npm --prefix e2e run test:short-flame
docker compose -f docker-compose.yaml -f docker-compose.short-flame.yaml down
```

The real settings panel enables only Short Flame. The scenario reveals and collects its
pickup, observes an actual range-one blast in the cleared corridor, reconnects without
extending the timer, and verifies base range after expiry. Equivalent Kubernetes acceptance
mounts the same mode-0444 seed and runs through the Service port-forward. Never bake the seed
into the application image. Consult [verification](verification.md) for actual candidate results.

## Reproduce opt-in Long Fuse timing

Corrected source `e0ffd635f9aa0b25c1f2dc00a5d20d8fc00be831` passes fresh 32/32 Compose and
32/32 Kubernetes browser cases, with wrapper exit zero and no retries or carried results.
See [verification](verification.md) for the exact image, sealed evidence, earlier failed
attempts, independent review and final CI/publication status.

Use the paired candidate image with its separate external production seed:

```sh
chmod 444 core/tests/fixtures/arena-v2-long-fuse.seed
docker compose -f docker-compose.yaml -f docker-compose.long-fuse.yaml up -d --no-build --force-recreate app
# Wait for /api/v1/ready to return 200 before running the scenario.
E2E_LONG_FUSE_FIXTURE=arena-v2-long-fuse npm --prefix e2e run test:long-fuse
docker compose -f docker-compose.yaml -f docker-compose.long-fuse.yaml down
```

The real settings panel enables only Long Fuse with a 120-tick base. The scenario follows the
ordinary generated pickup path, proves each emitted manual bomb action against retained placement
history, observes a captured 240-tick bomb on both peers, crosses the exact 600-tick expiry, and
verifies restored 120-tick timing. The first two escapes must retain the same running round and a
living collector. Bomb identifiers are numeric and zero is valid; the placement poll therefore
requires a defined ID rather than a truthy ID.

The earlier source-`39e764a9a20fc0c25e663fd4921491c5268c8358` attempt passed 31 Compose
cases before this truthiness defect rejected observed bomb ID `0`; Kubernetes was not started.
It is sealed at `/tmp/grenadiers-v0330-ready-native-acceptance` with 14 reports, 37 PNGs,
an unchanged 988-entry preparation-manifest SHA-256 of
`9e5b2ddc43d44f8fac51d78d071093c56ba93de5388673e5626636b61243bb95`, and a
1,380-entry final-manifest SHA-256 of
`0c782387e843cf3c9e8e40a22e9c767e35741c91a8ee8cba5dd3dd21a4aaad86`. It has no
accepted or carried matrix. The preceding source-`04b89a6` attempt lacked retained command/frame
history, so its similar symptom remains historically unproven. Fresh full acceptance is required
after this bounded driver correction. Never bake the seed into the image.

## Reproduce opt-in Reverse Controls movement

Use the paired candidate image with its separate external production seed:

```sh
chmod 444 core/tests/fixtures/arena-v2-reverse-controls.seed
docker compose -f docker-compose.yaml -f docker-compose.reverse-controls.yaml up -d --no-build --force-recreate app
# Wait for /api/v1/ready to return 200 before running the scenario.
E2E_REVERSE_CONTROLS_FIXTURE=arena-v2-reverse-controls npm --prefix e2e run test:reverse-controls
docker compose -f docker-compose.yaml -f docker-compose.reverse-controls.yaml down
```

The real settings panel enables only Reverse Controls. The scenario holds a raw east key through
ordinary approach, collection, reversed travel to the left wall, exact expiry and restored east
movement. Both peers check the authoritative timer. Keep the seed external and read-only.

## Reproduce opt-in Diarrhea automatic placement

Use the already built candidate image and the pinned production seed:

```sh
chmod 444 core/tests/fixtures/arena-v2-diarrhea.seed
docker compose -f docker-compose.yaml -f docker-compose.diarrhea.yaml up -d --no-build --force-recreate app
E2E_DIARRHEA_FIXTURE=arena-v2-diarrhea npm --prefix e2e run test:diarrhea
docker compose -f docker-compose.yaml -f docker-compose.diarrhea.yaml down
```

Deploy the v0.3.32 server and web assets together and reload connected browsers. The exact v8
`availableItems` array grows to eight entries, so older v8 web clients reject the room even when
Diarrhea is disabled. No storage migration is required; active matches remain process-local.

## Reproduce opt-in Slow-foot movement

Use the already built candidate image and the pinned production seed:

```sh
chmod 444 core/tests/fixtures/arena-v2-slow-foot.seed
docker compose -f docker-compose.yaml -f docker-compose.slow-foot.yaml up -d --no-build --force-recreate app
E2E_SLOW_FOOT_FIXTURE=arena-v2-slow-foot npm --prefix e2e run test:slow-foot
docker compose -f docker-compose.yaml -f docker-compose.slow-foot.yaml down
```

The scenario enables only Slow foot, reveals and collects one generated stack, measures half-rate
and restored base-rate movement in unclamped corridor windows, and checks both peers' exact timer.
Deploy the v0.3.34 server and web assets together and reload connected browsers. The exact v8
`availableItems` array grows to ten entries, so older v8 clients reject the room even when Piercing
Bomb is disabled. No storage migration is required; active matches remain process-local.

## Reproduce opt-in Piercing Bomb penetration

```sh
chmod 444 core/tests/fixtures/arena-v2-piercing-bomb.seed
docker compose -f docker-compose.yaml -f docker-compose.piercing-bomb.yaml up -d --no-build --force-recreate app
E2E_PIERCING_BOMB_FIXTURE=arena-v2-piercing-bomb npm --prefix e2e run test:piercing-bomb
docker compose -f docker-compose.yaml -f docker-compose.piercing-bomb.yaml down
```

The scenario enables only Piercing Bomb, reveals the deterministic charge, captures localized
armed views, crosses two consecutive blocks with one manual bomb, and verifies the next manual
bomb is ordinary.

Deploy the v0.3.35 server and web assets together and reload connected browsers. The exact v8
`availableItems` array grows to eleven entries, so older v8 clients reject the room even when
Power Bomb is disabled. No storage migration is required; active matches remain process-local.

## Reproduce opt-in Power Bomb hard-wall traversal

```sh
chmod 444 core/tests/fixtures/arena-v2-power-bomb.seed
docker compose -f docker-compose.yaml -f docker-compose.power-bomb.yaml up -d --no-build --force-recreate app
E2E_POWER_BOMB_FIXTURE=arena-v2-power-bomb npm --prefix e2e run test:power-bomb
docker compose -f docker-compose.yaml -f docker-compose.power-bomb.yaml down
```

The scenario enables only Power Bomb and reveals the deterministic charge at (3,1). A manual
bomb planted at (1,2) burns and destroys the block at (3,2) beyond the retained, unburned hard
wall at (2,2), leaves the escape cell (3,1) safe, and consumes the charge. Both peers must then
agree on a fresh ordinary manual placement. Native execution remains a candidate gate.
