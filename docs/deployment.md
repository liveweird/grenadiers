# Local deployment

Grenadiers is packaged as one image: the Rust process serves both the API and the built React
application on port 8085. Active matches will live in that process, so the initial deployment
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
