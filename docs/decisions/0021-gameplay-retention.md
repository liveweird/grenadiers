# ADR 0021: Live gameplay support and historical retention

Status: accepted; records the current support boundary and prerequisites for cleanup.

## Context

Only gameplay v11 is served, but earlier wire schemas and browser applications remain in the
repository. Some code under historical Rust and browser namespaces is shared by the live path.
Wire compatibility and private replay compatibility have different identities and consumers;
deleting whole version directories would erase useful assertions and live dependencies.

## Decision

### One live network version

The server supports exactly v11. Earlier and future protocol versions receive incompatibility
before credential binding. Upgrade server and browser together and reload existing tabs.
Retaining old declarations does not promise that historical clients can join the current server.
Simultaneous support for multiple live versions requires a separate compatibility decision.

### Explicit archived contracts

Retain gameplay v2 through v10 schemas, generated declarations, conformance fixtures and unique
version-specific rejection checks. V2 uses `api/gameplay.schema.json`; v3 through v11 use
`api/gameplay-vN.schema.json`. The shared generation inventory must reject unknown, missing,
malformed and mismatched contracts rather than silently treating a new file as supported.

Archived catalogs and constraints keep their version-specific meaning. A new item does not
automatically expand every historical schema. Existing shared implementations may evolve behind
version-specific adapters, provided each retained adapter's contract tests continue to pass.

### Independent private replay compatibility

Preserve the replay runner's accepted combinations of journal format, lifecycle, rules and
generator versions. Journals record semantic operations and private seeds; they are process-local
and currently have no durable serialization or public export contract. A wire-version cleanup
does not authorize changing generation streams, old rule semantics or replay acceptance.

Production journal allocation/recording is a separate operational decision. Removing the old
browser UI neither changes recording nor creates a replay migration requirement.

## Cleanup and acceptance

Unused historical application code may be removed after checking repository consumers and
moving any unique regression assertions to the current implementation. Keep the corresponding
archived schema and conformance tests. In particular, preserve checks of the actual socket URL,
session request/abort behavior, native-fetch receiver binding and credential storage/redaction.

Before retiring adapters, extract helpers used by the live path into a neutral shared module,
initially retaining compatibility re-exports. Preserve checked projection, recipient privacy,
atomic encoding, frame limits and version-specific rejection behavior. No complete historical
Rust version directory is selected for deletion by this decision.

Run affected contract generation/drift checks and behavioral tests, followed by the applicable
local gates. Keep exhaustive acceptance, coverage floors and failure visibility unchanged.
Historical UI removal does not require replay or Rust suites when those inputs are unchanged.
Record separate source changes and checks for later adapter retirement; this ADR is not evidence
that those future cleanups have already passed.

## Consequences

The current client and shared validation can be simplified without promising old playable UIs.
Archived contract coverage remains until a later documented retirement decision identifies
which guarantees replace it. Durable replay export or external historical replay consumers
would need explicit format, migration and retention requirements before changing this boundary.
