# ADR 0022: Build-time validation for live browser messages

Status: accepted for v0.3.51. CSP enforcement is a later change.

## Context

The browser's gameplay-v11 and realtime diagnostic consumers compile fixed JSON Schemas with
Ajv at module load. Compilation constructs JavaScript functions dynamically, preventing a
restrictive script policy. The schemas are repository inputs; clients do not need to compile
new schemas supplied by users.

Pixi also constructs functions dynamically, and Mantine uses style elements and attributes.
Precompiling validators addresses one prerequisite; it does not establish that enforcing CSP
is already compatible with the complete application.

## Decision

Generate standalone ESM for the two production validation routes. Preserve the existing
`allErrors: true` behavior and compilation boundaries: gameplay registers its schema and
resolves `$id#/$defs/GameServerMessage`; realtime compiles its root schema directly. Schema
constraints, wire types and subsequent semantic/recipient checks do not change.

Use pinned Ajv's standalone CommonJS output, then bundle it in memory with the existing pinned
Vite Node API. Direct Ajv ESM output still contains CommonJS helper calls for these schemas;
handwritten rewriting would be fragile. Require one ESM chunk without external or dynamic
imports, executable `require`, dynamic function construction or evaluation. Helper metadata
strings are data and must not be mistaken for executable calls.

Load the entry and its three installed runtime helpers through stable virtual module IDs,
and bind the bundler working directory explicitly. Reject unexpected physical module IDs.
This prevents generated region comments from depending on checkout depth or invocation
directory; compare complete artifacts across both variations and the pinned Linux build.

Generated declarations describe the interface actually supplied: a callable type guard with
optional nullable validation errors. They must not claim the standalone function has every
property of Ajv's compiler-side `ValidateFunction`.

Keep archived validators, their distinct catalogs and Ajv development tooling under
[ADR 0021](0021-gameplay-retention.md). The active consumers import generated functions and
their types; they no longer import the compiler into the production dependency graph.

## Generation and verification

`gen:validators` writes reviewed code and declarations. `check:validators` regenerates both in
memory and compares exact bytes. Include checking before the normal production build, retaining
the existing compressed-bundle guard afterward. Generated output contains no timestamps or
checkout-dependent paths and introduces no new package dependency.

Compare generated and compiler-backed results for every live message variant and representative
malformed nested data, bounds, Unicode lengths, duplicate arrays and extra/missing fields.
Preserve complete ordered error arrays, including paths, keywords and parameters. Test
invalid/valid/invalid transitions and retained error snapshots. Existing exhaustive and
semantic/privacy tests continue through the same consumer boundary; avoid adding a duplicate
exhaustive parity loop.

Check artifact/declaration drift, TypeScript narrowing, Vitest and Vite imports, the production
module graph, and the measured bundle budget. Run focused checks during development and one
integrated web gate after independent review. Narrow generated-file tooling exclusions must
leave handwritten tests/helpers in scope.

## Consequences and follow-up

Browsers validate the same untrusted data without runtime schema compilation. Regeneration is
required when a live schema or its pinned generation tools change. The production code remains
coupled to its reviewed generated artifacts, so drift is a build failure.

Next, verify Pixi's static compatibility module and actual WebGL/context restoration, Mantine
styles, same-origin HTTP/WS and HTTPS/WSS, and diagnostic entry points under an enforced policy.
Do not derive CSP permissions from untrusted forwarding or Host headers. A report-only policy
is an intermediate state with an explicit enforcement criterion, rather than completion.

References: [Ajv standalone validation](https://ajv.js.org/standalone.html),
[Mantine provider](https://mantine.dev/theming/mantine-provider/).
