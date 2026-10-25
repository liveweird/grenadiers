# ADR 0024: Bounded peer admission and explicit proxy identity

Status: v0.3.53 candidate; foundations reviewed, caller acceptance pending.

## Context

Process-global limits bound memory and aggregate load but allow one network peer to consume the
whole budget. Origin is a browser boundary, not client identity. A reverse proxy also hides the
client transport address unless the backend explicitly trusts its forwarding behavior.

## Decision

Add peer budgets before existing global backstops, keeping separate gameplay and diagnostic
registries. Normalize mapped IPv4 addresses; use IPv4 /32 and native IPv6 /64 quota keys, while
comparing full normalized addresses for proxy trust. Treat these keys as admission identities,
not authentication or user accounts. Configure no trusted proxies by default.

Support a small explicit immediate-proxy contract: up to 32 exact IP literals and exactly one
overwritten `X-Forwarded-For` client literal. Reject malformed trusted forwarding; ignore
untrusted forwarding. Fail closed when transport metadata is unavailable. Multi-hop chains and
CIDR trust need actual deployment requirements rather than automatic private-network trust.

Bound each registry to 4,096 keys; evict only inactive, permit-free entries after 60 seconds.
Clamp captured time and throttle bounded sweeps. Hold typed RAII socket/admission permits;
count issued and claimed sessions against the minting peer under the existing session mutex.
Claiming from another peer remains valid bearer authentication. Quotas never bind a resume
capability to an address. The [security guide](../security.md#peer-identity-and-admission) owns
the engineering defaults and operator configuration.

Keep existing problem responses, retry headers, correlated non-closing room throttles,
per-socket flood closure and gameplay v11. Aggregate denial counters use finite operation/reason
labels and exclude addresses, credentials, room identifiers and arbitrary request data.

## Acceptance and limits

Verify trust parsing, per-peer independence with global headroom, cardinality/eviction clocks,
concurrent quotas, cancellation/expiry/drop release and cross-peer claiming. Real listeners must
prove ConnectInfo propagation; router tests supply explicit mock metadata. Capture logs with
sentinel inputs to establish telemetry privacy. Retain global saturation and resume regressions,
then verify normal two-player browser flows through Compose and Kubernetes.

IP/NAT grouping trades finer fairness for bounded state and resistance to source-port/IPv6
interface rotation. Public hosting still requires actual proxy addresses, overwrite behavior,
backend reachability restrictions, TLS and ingress limits. Restart clears process-local counters,
quotas and matches; rollback to an older image removes these protections. No storage migration
or durable identity system is introduced.
