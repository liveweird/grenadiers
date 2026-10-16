# API guidelines

These stable IDs adapt Flow's rules to Grenadiers. `api/openapi.json` is authoritative for HTTP;
`api/realtime.schema.json` is authoritative for realtime envelopes. Implementation, contracts,
generated TypeScript, conformance tests, and docs change together.

- **API-VER-001:** HTTP operations live under `/api/v<major>/`. Welcome announces protocol
  version 1. A breaking realtime change requires an explicit version/migration decision.
- **API-STRUCT-001:** HTTP JSON success bodies are objects, with camelCase properties.
  Error responses use problem details (`type`, `title`, `status`, `detail`) and
  `application/problem+json`. The malformed upgrade's framework-generated plain-text 400
  is an explicit scaffold exception; transport control errors are WebSocket messages/close codes.
- **API-DOC-001:** Document actual methods, statuses, media types, and examples; do not list
  hypothetical errors. Keep OpenAPI 3.0 compatibility for the inherited code generation flow.
- **API-ERR-001:** Unknown API paths yield problem JSON, including when static SPA fallback is enabled.
  Dependency failure returns readiness 503, with liveness 200.
- **API-WS-001:** Require an exact allowed browser Origin before upgrading. Origin is not identity.
  HTTP documents the upgrade; JSON Schema validates traffic after it.
- **API-WS-002:** Inbound messages are tagged JSON objects with no extra fields. Ping sequence
  is an integer in `0..2147483647`. The server sends welcome, pong, or invalidMessage error.
  Malformed/unsupported input yields the error then policy close 1008; oversized frames close
  the connection at the WebSocket transport boundary and need not produce an application error.
- **API-WS-003:** Bound socket count, frames, messages, inactivity, and sends. Control ping/pong
  frames remain distinct from application ping/pong envelopes. Do not expose credentials in URLs.
- **API-STATE-001:** Sockets do not reserve player slots. `maxPlayers: 2` describes the future
  per-match capacity. Clients will submit inputs, never authoritative outcomes.

Spectral checks the machine-verifiable OpenAPI subset. Runtime conformance tests and browser
smoke tests cover the remaining behavior. No authentication, CRUD, pagination, or retry/idempotency
policy is claimed before those APIs exist.
