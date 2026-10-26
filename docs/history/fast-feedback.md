# Fast-feedback evidence

This historical measurement records focused feedback at source
`f4fd65220cf4fbdc7322fe54bca36af2c5abd288` on 2026-10-26. It does not replace
the repository acceptance gates. The exhaustive Rust profiles, shuffled web gate,
coverage execution and their existing floors remain unchanged.

## Method and environment

The measurement used Node 24.21.0, npm 11.19.0, rustc 1.99.1
(`58cf8f9c3 2026-10-13`) and Cargo 1.99.1 on the local macOS worktree. Web
dependencies came from the existing root `web/node_modules` through an ignored
worktree symlink. Rust used only the worktree-private
`target/fast-feedback-cargo`; it never wrote to the root target directory.

The private Rust cache began as an APFS copy-on-write clone of the existing
`test-fixtures/target/debug` dependency cache. The copied Cargo tag was
byte-identical but inode-independent, and its rustc fingerprint matched the
active toolchain. The fault-validation phase then compiled `grenadiers-core` in
that cache. The quiet timings below therefore describe a dependency-loaded,
precompiled-core edit-feedback loop. They are neither cold-build measurements
nor first-ever command executions.

## Quiet focused timings

The coordinator held other builds, tests and browser work idle. Each command ran
once as the first measured batch and exactly once more, immediately afterward in
the same order. The results were not selected from repeated trials. Wall time is
the external `/usr/bin/time -p` `real` value.

| Scope and command | Assertions executed | First measured | Immediate warm repeat |
| --- | ---: | ---: | ---: |
| From `web/`: `npm test -- src/game/validation/expanded-client-conformance.test.ts src/game/validation/expanded-server-conformance.test.ts src/game/client/client.test.ts src/game/client/client-settings.test.ts src/game/client/locale.test.tsx src/App.test.tsx src/game/arena/GameArena.test.tsx` | 57 tests in 7 files | 1.45 s | 1.40 s |
| `CARGO_TARGET_DIR=<worktree>/target/fast-feedback-cargo cargo test -p grenadiers-core 'protocol::expanded_client_'` | 11 passed; 944 filtered | 0.09 s | 0.08 s |
| `CARGO_TARGET_DIR=<worktree>/target/fast-feedback-cargo cargo test -p grenadiers-core 'protocol::expanded_server_contract_tests'` | 4 passed; 951 filtered | 0.09 s | 0.09 s |
| `CARGO_TARGET_DIR=<worktree>/target/fast-feedback-cargo cargo test -p grenadiers-core 'admission_tests'` | 6 passed; 949 filtered | 0.08 s | 0.07 s |

The admission filter covers the `grenadiers-core` v8 protocol admission module;
it is not evidence for the server match-owner admission tests. Summing the four
sequential command times gives 1.71 seconds for the first measured batch and
1.64 seconds for the immediate warm repeat. These narrow checks are intended for
early fault feedback only.

## Loaded fault-detection validation

Before the quiet timing window, four intentional faults ran one at a time while
other repository work was active. This phase validated detection and prepared
the private cache; its elapsed times are not benchmark results. Every mutated
file was snapshotted, restored byte-for-byte, checked by SHA-256 and followed by
the same healthy focused test. All four faults compiled or transformed
successfully and were killed by an existing assertion rather than a compiler
error or timeout.

| Intentional fault | Existing assertion that killed it | Loaded-phase result |
| --- | --- | --- |
| Invert `GameClient.acceptLeft` sequence correlation from mismatch to equality | `App game flow > creates a room, reports player readiness, and clears only the correlated leave` | The unmatched acknowledgement incorrectly removed the room; failure at `App.test.tsx:174`. Mutant Vitest duration 148 ms; restored test passed in 520 ms. |
| Weaken client recipient uniqueness from `Set.size === values.length` to `<=` | `expanded gameplay server conformance > exercises live v11 recipient privacy and identity checks after generic validation` | A duplicate actor identity was accepted (`true` instead of `false`) at line 112. Mutant duration 8 ms; restored test passed in 287 ms. |
| Make the locale formatter return constant `"0"` | `game locale access > matches the legacy formatter for every current English and Polish catalog template` | The first checked template produced `"0"` instead of `"Round wins for <value>"` at line 93. Mutant duration 5 ms; restored test passed in 192 ms. The test traverses all 44 current EN/PL templates when healthy. |
| Remove only `deny_unknown_fields` from the v8 client-message enum | `protocol::expanded_client_mutation_tests::every_corpus_variant_rejects_unknown_fields_and_wrong_field_types` | Compilation succeeded in 12.23 s, then the assertion rejected the survivor under case `authenticate-protocol-minimum`. The restored source recompiled in 4.14 s and the focused test passed (954 filtered). |

No mutant survived. No application source, test assertion, framework, dependency,
version, threshold, exclusion or retry policy changed as part of this evidence.
