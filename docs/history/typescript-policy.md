# TypeScript option assessment — 26 October 2026

This read-only migration assessment used source `218b4ba2341d7d23ebc7a8c85b4a3d5517e9b0e7`.
The declared strict/exact-optional configuration passed its full web gate. Extra CLI flags below
were experiments, not enabled gates; diagnostics are migration size, not counts of runtime bugs.

| Proposed flag/project | Diagnostics |
| --- | ---: |
| `noUncheckedIndexedAccess`, application | 1,040: 1,035 test/mock and five production diagnostics |
| `noUncheckedIndexedAccess`, Node tooling | 0 |
| `noUncheckedIndexedAccess`, renderer diagnostics | 15, including four production diagnostics also reported above |
| `noUncheckedIndexedAccess`, E2E | 1,583 |
| `exactOptionalPropertyTypes`, E2E | 29 |

Commands used the installed project compiler with `--project`, `--noEmit`, `--pretty false` and
only the proposed extra flag. Raw commands, exit codes and output are retained locally under
`target/quality-review/typescript-assessment-*`.

## Production indexing disposition

Independent read-only review found no reachable defect in the five production diagnostics:

- `artAtlas/catalog.ts`: a 0–5 loop indexes exactly six locally constructed marker containers.
- `artAtlas/validate.ts`: both diagnostics refer to the same prior-frame lookup; the loop is
  bounded below the current existing index in dense checked JSON or push-built arrays.
- `terrainArt.ts`: both callers constrain the index modulo the dense four-entry floor table.
- `GameplaySettingsPanel.tsx`: the helper is called only for the four literal supported turning
  values, each an own key of the label record; schemas also close that set.

TypeScript does not retain these loop/modulo/record proofs in ordinary indexed-array types.
The `src/test/arena-pixi-mock.ts` diagnostic is counted as a test, not production code.

## Decision

Keep the existing strict and application/tooling exact-optional policy. Do not introduce broad
casts, disable assertions or exclude tests merely to enable unchecked-index or E2E optional flags.
The Node project has no migration diagnostics, but this assessment found no missing runtime check
that requires an additional project-wide gate. A later bounded adoption can use tuples/accessors
and explicit optional semantics where they improve the relevant code. This completes the option
assessment without manufacturing a new CPU prerequisite or a confirmed-defect claim.
