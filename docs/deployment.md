# Local deployment

Grenadiers is packaged as one image: the Rust process serves both the API and the built React
application on port 8085. Active matches live in that process, so the initial deployment
intentionally runs one replica with a `Recreate` update strategy. A restart ends active matches;
horizontal scaling and recovery require match routing and persistence designs first.

Deploy the current server and matching web assets together, then reload connected browsers.
Gameplay v11 has an exact catalog of 22 item types: 21 collectible pickup types plus the Oil Barrel
arena object. It publishes materialized Oil state while concealed and pending locations, generation
data, and typed exit grace remain private. Gameplay-v10 clients fail version negotiation before
credential lookup. Roll back the server and web assets as one paired image and reload browsers;
mixing either side across gameplay versions is unsupported. No storage migration is required;
active matches remain process-local and end on rollout or rollback.

## Peer identity behind a proxy

Direct local deployment needs no trusted proxy configuration. `TRUSTED_PROXY_IPS` defaults
empty in Compose and Kubernetes. For a real reverse proxy, set only its exact immediate
transport IP literals and make it overwrite `X-Forwarded-For` with one client IP. Do not append
forwarding chains or accept a client-supplied value. Restrict backend access to that proxy;
choose addresses from the actual container/cluster topology, not an assumed loopback address.
See [identity and quotas](security.md#peer-identity-and-admission) for strict parsing and limits.

TLS termination, proxy addresses and ingress connection/header/idle limits remain deployment
choices. Test the paired image through the actual proxy before public exposure. An older image
lacks these peer protections even if the same environment variable is present; restrict exposure
when rolling back. Process restarts clear quotas as well as matches and capabilities.

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
exists yet. The app container has a 256 MiB memory ceiling and a 128-process PID ceiling. These
are local operational defaults, not measured hosting capacity. The optional PostgreSQL service
continues to use the upstream image's resource profile.

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

The committed `grenadiers-app:local` tag is a developer-cluster default. Deployment overlays for
another environment supply that environment's exact accepted image reference; an immutable
registry digest records the precise paired server/web image when one is available. Registry
publication and authentication remain environment-specific operations.

A port-forward remains attached to the selected pod even when started through a Service.
After any rollout or fixture change, stop the old forward, wait for the replacement pod,
start a new forward, and require successful health/readiness responses before opening the
game or running browser tests. A healthy replacement does not repair an old forward.

The base manifests contain no credentials or PostgreSQL resources. If durable data is introduced,
supply `DATABASE_URL` from a Kubernetes Secret and deploy the database separately. Readiness has a
three-second probe timeout because a configured database check may take up to two seconds.
The app requests 50 millicores, 64 MiB memory, and 16 MiB ephemeral storage, with limits of
500 millicores, 256 MiB memory, and 64 MiB ephemeral storage. The ephemeral-storage values are
explicit local scheduling and eviction budgets, not storage capacity or performance guarantees.
Kubernetes does not define a portable per-Pod PID field; cluster operators apply kubelet/runtime
PID policy separately when needed.

The manifests can be checked offline with `kubectl kustomize k8s`. A successful render validates
composition only; image startup, probes, WebSockets, and cluster policy still require a live deploy.

## Kubernetes schema guardrail

`scripts/gates.sh k8s` renders Kustomize and validates the rendered resources with
**kubeconform 0.8.0**, strict Kubernetes **1.33.0** schemas from a pinned schema-repository
commit. Install `kubectl`, kubeconform, and the web package's locked npm dependencies first.
The CI workflow downloads the Linux
kubeconform archive and verifies its SHA-256 inherited from Flow before running this gate.
The gate checks the same rendered YAML for one replica, the Recreate strategy, absence
of an autoscaler targeting the game Deployment, consistent Service routing, the committed
restricted Pod Security namespace labels, effective non-root UID/GID and restricted container
security context, disabled service-account token automounting, and positive CPU, memory, and
ephemeral-storage requests and limits. Container overrides cannot weaken the inherited identity or
seccomp settings, and capabilities cannot be added after dropping `ALL`. It rejects duplicate YAML
keys. Resource quantities may be adjusted by deployment overlays as long as they remain present and
positive. Local installations require the appropriate platform binary. Schema validation may require
network access to the pinned schemas; successful rendering alone does not pass this gate.

The archived feature rollout notes retain their original release versions and catalog counts.
The current reproduction commands below use the checked-out paired image; deployments use
[gameplay-v11](game/protocol-v11.md) and its current catalog, even for an older optional item.

## Fixture-backed browser acceptance

This section is the current operator procedure. It describes configured reproduction routes, not
whether a candidate, image, commit, or release passed them. Use [testing](testing.md) for suite
scope and [verification](verification.md) for dated execution evidence. The former per-feature
release and candidate narrative is retained as a [historical record](history/deployment.md).

### Seed contract and one-image workflow

Default rooms draw fresh private OS randomness only when a round starts. Fixture reproduction sets
`GAME_ROUND_SEED_FILE` to a readable file containing exactly 64 lowercase hexadecimal characters,
optionally followed by one LF. Startup reads it once before listening and fails on a missing,
unreadable, malformed, or oversized file. The loaded seed repeats for every room and round; edits
take effect only after a process restart. Keep ordinary deployments in random mode.

Build the paired server/web image once, prove the ordinary route first, and reuse that image for
each fixture. Fixture changes force-recreate the process and therefore end all live matches and
invalidate in-memory capabilities on port 8085.

```sh
docker compose up --build -d app
curl --fail http://127.0.0.1:8085/api/v1/health
curl --fail http://127.0.0.1:8085/api/v1/ready
npm --prefix e2e test
```

For example, run the generated-pickup fixture below. For another row, use that row's exact seed
filename and acknowledgement command. Keep `docker-compose.yaml` first: Compose resolves relative
bind sources from the first file, so reversing the files changes their meaning.

```sh
chmod 444 core/tests/fixtures/arena-v3-pickups.seed
GRENADIERS_FIXTURE_SEED_FILE=./core/tests/fixtures/arena-v3-pickups.seed \
  docker compose -f docker-compose.yaml -f e2e/compose/fixture.yaml \
  up -d --no-build --force-recreate app
curl --fail http://127.0.0.1:8085/api/v1/health
curl --fail http://127.0.0.1:8085/api/v1/ready
E2E_PICKUP_FIXTURE=arena-v3-pickups npm --prefix e2e run test:pickups
docker compose -f docker-compose.yaml down
```

The 24 former `docker-compose.<feature>.yaml` files have been replaced by the single
[`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml) override. The old operator form was
`docker compose -f docker-compose.yaml -f docker-compose.<feature>.yaml up ...`; the new form is
`GRENADIERS_FIXTURE_SEED_FILE=./core/tests/fixtures/<exact-name>.seed docker compose -f docker-compose.yaml -f e2e/compose/fixture.yaml up ...`.
The selector is required and is expanded by Compose on the host; it is not a container variable or
browser API. Use either an absolute host file path or a relative path prefixed with `./`. Compose
interprets that relative path from the first, root `docker-compose.yaml`; keep the base file first.
An unset or empty selector fails Compose configuration. The explicit bind contract also disables
host-path creation, so a nonexistent source is rejected before the application starts and is not
silently created as a directory. The independent `E2E_*_FIXTURE` acknowledgement still makes a
mismatched or forgotten fixture fail instead of skip and does not select the server seed.

The shared override mounts the selected tracked seed at `/run/grenadiers/round.seed` read-only and
sets `GAME_ROUND_SEED_FILE` to that path. Keep the file external to the runtime image and browser
assets; never send it through an API or browser flag. Preserve the base read-only container,
dropped capabilities, `no-new-privileges`, one-process topology, and stop grace. Always tear down
with the base-only `docker compose -f docker-compose.yaml down` command so cleanup cannot be blocked
by an unset selector.

The base Compose origin allowlist permits `http://localhost:8085` and
`http://127.0.0.1:8085`. If the browser uses another origin, set `ALLOWED_ORIGINS` to that exact
origin before recreating the app and set `E2E_BASE_URL` consistently; do not weaken the allowlist.

After fixture work, remove the override and restore the same image in ordinary random mode. This
restart also ends fixture matches and capabilities. Require fresh health/readiness before use.

```sh
docker compose up -d --no-build --force-recreate app
curl --fail http://127.0.0.1:8085/api/v1/health
curl --fail http://127.0.0.1:8085/api/v1/ready
```

For isolated Kubernetes reproduction, create a temporary Secret from the row's same seed file.
Mount its `round.seed` key at `/run/grenadiers/round.seed` with `defaultMode: 0444`, set
`GAME_ROUND_SEED_FILE` to that path, and ensure UID 65532 can read it. Retain the base probes,
Recreate strategy, one replica, security context, Service, and exact paired image digest. After
a Secret content change, explicitly restart the Deployment: the process reads its seed only at
startup. Applying a seed environment/volume change to the Pod template triggers a rollout itself.
After changing Secret contents, run:

```sh
kubectl -n grenadiers rollout restart deployment/grenadiers
kubectl -n grenadiers rollout status deployment/grenadiers
```

Wait for the replacement Pod to become ready. After every rollout or old-forward failure, stop
the old port-forward, start a fresh `service/grenadiers 8085:8085` forward to the replacement Pod,
and require successful health/readiness. Remove the temporary Secret or namespace and remove
the seed env/volume patch to
restore ordinary random mode. A healthy replacement never repairs an old port-forward.

On Linux, the headed visibility command is
`E2E_MATCH_FIXTURE=arena-v1-matches xvfb-run -a npm --prefix e2e run test:hardening`.
On macOS, use the table's command without the wrapper. See the
[natural visibility setup](../e2e/README.md#natural-headed-visibility).

### Fixture index

New live rooms enable all 22 configurable items. Deterministic item scenarios use the real
settings UI: **All off**, enable the required item(s), then **Apply rules**; classic fixtures
select **Classic settings** and apply. The seed mount does not choose the enabled item catalog.
The table retains historical fragment names containing `opt-in` for incoming links.

All seed paths and the shared Compose mount below are checked repository files. Follow the linked scenario
for privacy, pixel/compositor, localization, timing-sensitive capture, and allowed evidence rules;
do not replace those observations with state injection, direct gameplay messages, retries, or seed
searches. Where a scenario names visual proofs, preserve its English desktop and Polish 320-pixel
reduced-motion captures, exact filenames, and capture timing.

| Feature | Shared override and exact read-only seed | Required acknowledgement and command | Scenario-specific contract |
| --- | --- | --- | --- |
| <a id="reproduce-a-generated-pickup-round"></a>Generated pickups | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v3-pickups.seed`](../core/tests/fixtures/arena-v3-pickups.seed) | `E2E_PICKUP_FIXTURE=arena-v3-pickups npm --prefix e2e run test:pickups` | [Generated pickup scenario](../e2e/scenarios/pickups.md): native collection, both HUDs, captured ranges, screenshots/checkpoints and redacted summaries; per-key DOM/compositor capture stays disabled because its overhead distorts real-time escape. |
| Lairs | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v3-lairs.seed`](../core/tests/fixtures/arena-v3-lairs.seed) | `E2E_LAIRS_FIXTURE=arena-v3-lairs npm --prefix e2e run test:lairs` | [Lair scenario](../e2e/scenarios/lairs.md): clear spawn exits, real pursuit, independent caps, privacy, rendering and reconnect/rematch. |
| <a id="reproduce-heart-collection-and-protection"></a>Heart | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-heart.seed`](../core/tests/fixtures/arena-v2-heart.seed) | `E2E_HEART_FIXTURE=arena-v3-heart npm --prefix e2e run test:heart` | [Heart scenario](../e2e/scenarios/heart.md): explicit Heart-only collection, one protected hit, later lethal hit, localized views. |
| <a id="reproduce-opt-in-shield-protection"></a>Shield | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-shield.seed`](../core/tests/fixtures/arena-v2-shield.seed) | `E2E_SHIELD_FIXTURE=arena-v3-shield npm --prefix e2e run test:shield` | [Shield scenario](../e2e/scenarios/shield.md): explicit Shield-only configuration, protection expiry, later lethal hit, localized views. |
| <a id="reproduce-a-complete-classic-match"></a>Classic match fixture | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v1-matches.seed`](../core/tests/fixtures/arena-v1-matches.seed) | `E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:matches`<br>`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:reconnect`<br>`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:latency`<br>`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:hardening` | [Match](../e2e/scenarios/matches.md), [reconnect](../e2e/scenarios/reconnect.md), [latency](../e2e/scenarios/websocket-latency.md), and [headed visibility](../e2e/scenarios/browser-hardening.md); Classic settings retain the fixture's generator identity. |
| <a id="reproduce-opt-in-golden-flame-range"></a>Golden Flame | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-golden-flame.seed`](../core/tests/fixtures/arena-v2-golden-flame.seed) | `E2E_GOLDEN_FLAME_FIXTURE=arena-v3-golden-flame npm --prefix e2e run test:golden-flame` | [Golden Flame scenario](../e2e/scenarios/golden-flame.md): captured range, reconnect deadline, exact expiry, and base range. |
| <a id="reproduce-opt-in-short-fuse-timing"></a>Short Fuse | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-short-fuse.seed`](../core/tests/fixtures/arena-v2-short-fuse.seed) | `E2E_SHORT_FUSE_FIXTURE=arena-v3-short-fuse npm --prefix e2e run test:short-fuse` | [Short Fuse scenario](../e2e/scenarios/short-fuse.md): selected base fuse, captured shortened fuse, reconnect, expiry, and ordinary timing. |
| <a id="reproduce-opt-in-short-flame-range"></a>Short Flame | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-short-flame.seed`](../core/tests/fixtures/arena-v2-short-flame.seed) | `E2E_SHORT_FLAME_FIXTURE=arena-v3-short-flame npm --prefix e2e run test:short-flame` | [Short Flame scenario](../e2e/scenarios/short-flame.md): real range-one fire boundary, reconnect, exact expiry, and restored damage. |
| <a id="reproduce-opt-in-long-fuse-timing"></a>Long Fuse | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-long-fuse.seed`](../core/tests/fixtures/arena-v2-long-fuse.seed) | `E2E_LONG_FUSE_FIXTURE=arena-v3-long-fuse npm --prefix e2e run test:long-fuse` | [Long Fuse scenario](../e2e/scenarios/long-fuse.md): retained placement history, zero-valid IDs, exact expiry, and restored timing. |
| <a id="reproduce-opt-in-reverse-controls-movement"></a>Reverse Controls | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-reverse-controls.seed`](../core/tests/fixtures/arena-v2-reverse-controls.seed) | `E2E_REVERSE_CONTROLS_FIXTURE=arena-v3-reverse-controls npm --prefix e2e run test:reverse-controls` | [Reverse Controls scenario](../e2e/scenarios/reverse-controls.md): one held raw key through collection, reverse travel, exact expiry, and restoration. |
| <a id="reproduce-opt-in-diarrhea-automatic-placement"></a>Diarrhea | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-diarrhea.seed`](../core/tests/fixtures/arena-v2-diarrhea.seed) | `E2E_DIARRHEA_FIXTURE=arena-v3-diarrhea npm --prefix e2e run test:diarrhea` | [Diarrhea scenario](../e2e/scenarios/diarrhea.md): automatic next-tick placement without another browser bomb action, death, and clean round. |
| <a id="reproduce-opt-in-slow-foot-movement"></a>Slow foot | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-slow-foot.seed`](../core/tests/fixtures/arena-v2-slow-foot.seed) | `E2E_SLOW_FOOT_FIXTURE=arena-v3-slow-foot npm --prefix e2e run test:slow-foot` | [Slow-foot scenario](../e2e/scenarios/slow-foot.md): authoritative unclamped half/base-rate windows, exact timers, and key release. |
| <a id="reproduce-opt-in-piercing-bomb-penetration"></a>Piercing Bomb | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-piercing-bomb.seed`](../core/tests/fixtures/arena-v2-piercing-bomb.seed) | `E2E_PIERCING_BOMB_FIXTURE=arena-v3-piercing-bomb npm --prefix e2e run test:piercing-bomb` | [Piercing Bomb scenario](../e2e/scenarios/piercing-bomb.md): localized armed views, two burned blocks, charge consumption, ordinary successor. |
| <a id="reproduce-opt-in-power-bomb-hard-wall-traversal"></a>Power Bomb | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-power-bomb.seed`](../core/tests/fixtures/arena-v2-power-bomb.seed) | `E2E_POWER_BOMB_FIXTURE=arena-v3-power-bomb npm --prefix e2e run test:power-bomb` | [Power Bomb scenario](../e2e/scenarios/power-bomb.md): hard-wall traversal, safe footprint boundary, consumption, and fresh ordinary placement. |
| <a id="reproduce-opt-in-remote-bomb-manual-detonation"></a>Remote Bomb | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-remote-bomb.seed`](../core/tests/fixtures/arena-v2-remote-bomb.seed) | `E2E_REMOTE_BOMB_FIXTURE=arena-v3-remote-bomb npm --prefix e2e run test:remote-bomb` | [Remote Bomb scenario](../e2e/scenarios/remote-bomb.md): same ID beyond ordinary fuse, manual detonation without placement, ordinary successor. |
| <a id="reproduce-opt-in-landmine-contact-detonation"></a>Landmine | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-landmine.seed`](../core/tests/fixtures/arena-v2-landmine.seed) | `E2E_LANDMINE_FIXTURE=arena-v3-landmine npm --prefix e2e run test:landmine` | [Landmine scenario](../e2e/scenarios/landmine.md): safe overlap exit, same-ID persistence, keyboard contact, and two-recipient detonation parity. |
| <a id="reproduce-opt-in-invisibility-cloak-recipient-privacy"></a>Invisibility Cloak | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-invisibility-cloak.seed`](../core/tests/fixtures/arena-v2-invisibility-cloak.seed) | `E2E_INVISIBILITY_CLOAK_FIXTURE=arena-v3-invisibility-cloak npm --prefix e2e run test:invisibility-cloak` | [Cloak privacy scenario](../e2e/scenarios/invisibility-cloak.md): recipient-local actor/protocol and compositor pixels, reconnect without replay/refresh, exact reveal. |
| <a id="reproduce-opt-in-stopwatch-action-and-fuse-pausing"></a>Stopwatch | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-stopwatch.seed`](../core/tests/fixtures/arena-v2-stopwatch.seed) | `E2E_STOPWATCH_FIXTURE=arena-v3-stopwatch npm --prefix e2e run test:stopwatch` | [Stopwatch scenario](../e2e/scenarios/stopwatch.md): two holder windows, blocked actions, thaw, paused fuse, natural detonation. |
| <a id="reproduce-opt-in-invisible-bombs-privacy"></a>Invisible Bombs | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-invisible-bombs.seed`](../core/tests/fixtures/arena-v2-invisible-bombs.seed) | `E2E_INVISIBLE_BOMBS_FIXTURE=arena-v3-invisible-bombs npm --prefix e2e run test:invisible-bombs` | [Invisible Bombs privacy scenario](../e2e/scenarios/invisible-bombs.md): recipient-local live pixels/events, truthful public state, reconnect boundary, expiry, and reveal. |
| <a id="reproduce-opt-in-teleporter-swaps"></a>Teleporter | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-teleporter.seed`](../core/tests/fixtures/arena-v2-teleporter.seed) | `E2E_TELEPORTER_FIXTURE=arena-v3-teleporter npm --prefix e2e run test:teleporter` | [Teleporter scenario](../e2e/scenarios/teleporter.md): safe swap/marker, fresh reconnect state, and unsafe consumption without movement. |
| <a id="reproduce-opt-in-wall-pass-movement"></a>Wall Pass | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-wall-pass.seed`](../core/tests/fixtures/arena-v2-wall-pass.seed) | `E2E_WALL_PASS_FIXTURE=arena-v3-wall-pass npm --prefix e2e run test:wall-pass` | [Wall Pass scenario](../e2e/scenarios/wall-pass.md): opaque-block passage/pixels, rejected allocation, reconnect inside, exit, and EN/PL reduced-motion captures. |
| Kick | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-kick.seed`](../core/tests/fixtures/arena-v2-kick.seed) | `E2E_KICK_FIXTURE=arena-v3-kick npm --prefix e2e run test:kick` | [Kick scenario](../e2e/scenarios/kick.md): ordinary native input, retained pre-input timeline, same bomb identity/fuse, no injection or direct gameplay commands. |
| Punch | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-punch.seed`](../core/tests/fixtures/arena-v2-punch.seed) | `E2E_PUNCH_FIXTURE=arena-v3-punch npm --prefix e2e run test:punch` | [Punch scenario](../e2e/scenarios/punch.md): remapped Equipment key, wrapped bomb move, public parity, safe escape, and natural fuse. |
| Bracelet | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-bracelet.seed`](../core/tests/fixtures/arena-v2-bracelet.seed) | `E2E_BRACELET_FIXTURE=arena-v3-bracelet npm --prefix e2e run test:bracelet` | [Bracelet scenario](../e2e/scenarios/bracelet.md): exact one-tile shove with no victim command, Equipment action, event, or private marker; localized captures. |
| <a id="reproduce-opt-in-oil-barrel-reveal-and-persistent-fire"></a>Oil Barrel | [`e2e/compose/fixture.yaml`](../e2e/compose/fixture.yaml)<br>[`arena-v2-oil-barrel.seed`](../core/tests/fixtures/arena-v2-oil-barrel.seed) | `E2E_OIL_BARREL_FIXTURE=arena-v3-oil-barrel npm --prefix e2e run test:oil-barrel` | [Oil Barrel scenario](../e2e/scenarios/oil-barrel.md): pending privacy, unlit collision pixels, ignition/escape, finite and persistent fire, reconnect, and localized rendering. |

## Graceful stop

SIGTERM/Ctrl+C signal unfinished rooms with ServerStopped while preserving already-final match
outcomes. One five-second asynchronous budget starts when the signal is observed and covers HTTP
serving plus upgraded game/diagnostic sessions and room cleanup concurrently. Final publication
and close are best effort within that budget. On expiry the executable logs a warning and
returns, allowing Tokio runtime teardown to release remaining asynchronous I/O. Keep Compose's ten-second
and Kubernetes' fifteen-second stop grace periods. Transport reconnect reserves a seat for 600 authoritative ticks while this process remains alive.
A process restart still discards matches and capabilities; this is not restart recovery.
