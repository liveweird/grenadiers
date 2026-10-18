# Catalog-mask batching plan — preparatory work for Teleporter42

This is a test-only plan, not an implementation. The architect inspected committed v41 at `/tmp/grenadiers-invisible-bombs`, identified by root as `aac12e2030a0506832cc6a22c6cf2db4c268f058`. No files, benchmarks, tests, builds or native sessions were run by that planning task. Root persisted the returned plan.

The authoritative passing baseline is `/tmp/grenadiers-v0341-web-mask-final.log`: 65,545 tests, comprising 65,536 real masks and nine guards; Vitest duration 2.37 seconds. `/tmp/grenadiers-v0341-web-mask-cost.log` records 2.59 seconds real and maximum RSS **793,378,816 bytes**. The duplicate-fixture reporter's approximately 3.8 GB is failure-path evidence, not a passing baseline. No improvement is claimed before measurement.

## Smallest change

Replace eager per-mask test registration with bounded named range tests. Each range iterates masks lazily and executes the existing assertion unchanged for every mask.

Current allocation sites are `web/src/game/v9/validation.test.ts` (`it.each(Array.from(...))`, 65,536 cases) and `web/src/game/v8/validation.test.ts` (`subsets = Array.from(...)`, 16,384 archived cases). Limit implementation ownership to:

1. New `web/src/test/catalogMasks.ts`: preflight, ranges and bounded assertion execution.
2. New `web/src/test/catalogMasks.test.ts`: coverage and failure-continuation tests.
3. Those two existing validation files: replace only mask-table registration.
4. Future v10/v11 validation wrappers: adopt the helper when introduced.

`src/test/**` is already excluded from production coverage in `web/vite.config.ts`. Do not change exclusions, thresholds, runner settings or production modules. Leave efficient Rust mask loops, generation and replay tests untouched. This can be a preparatory commit on the Teleporter42 branch without renumbering releases or changing protocol/schema/DTO bytes.

## Validate the catalog before discovery

At inspected commit `aac12e2`, the registered length/uniqueness `it` did not run before eager `Array.from` discovery. Root subsequently had the author implement actual discovery-time preflight and a negative enumerator-not-called test; independent review cleared that minimal fix in commit `a2c6c0c783437786752f7e5b3af4712ec39c6ee3`. Its final focused suite has 65,546 tests. Preserve that protection in the batching implementation.

Inspect the small catalog before registering ranges. Pin expected count and canonical order independently for each protocol; never derive archived catalogs from the active catalog. Invalid fixtures must produce a clear failing catalog guard and no purported mask matrix; retain other malformed/boundary tests where possible. Do not generate millions of misleading cases. Preserve existing registered length/uniqueness assertions and add independent canonical-order checking.

Keep the current dense-mask interpretation:

```text
enabledItems = supported.filter(
  (_, index) => (mask & (1 << index)) !== 0
)
```

These are catalog indices, not sparse domain item bits. The planned 22 entries remain within this bitwise expression's range.

## Lazy, complete ranges

Use half-open `[start, end)` ranges covering `[0, 2 ** itemCount)`. Construct enabled items and a fresh snapshot inside the loop and release them after each assertion. Do not cache masks, messages, results or successful assertions.

Start benchmarking with **1,024 masks per range**, subject to measured adoption:

| Catalog | Logical masks | Candidate range tests |
| --- | ---: | ---: |
| Archived v8: 14 items | 16,384 | 16 |
| Archived v9: 16 items | 65,536 | 64 |
| Initial v10: 17 items | 131,072 | 128 |
| Final archived v10: 21 items | 2,097,152 | 2,048 |
| v11: 22 items | 4,194,304 | 4,096 |

At the final stage, the four retained catalogs total **6,373,376 logical validations** across 6,224 candidate range tests. Registration decreases; validation is not sampled or reduced.

Keep each version's fixture construction and exact assertion:

```text
expect(validateVersion(message))
  .toEqual({ valid: true, value: message })
```

Do not substitute a boolean check, stub or conditional assertion.

## Failure behavior and diagnostics

A plain assertion loop would stop on its first failed mask, skipping checks formerly represented by independent tests. Catch failures around each unchanged mask callback, count them and continue through every remaining mask in that range. Fail the range after completing it if any callback failed. Other ranges continue under existing runner behavior; add no bail, retry or concurrency setting.

Retain bounded diagnostic strings rather than Error objects, snapshots or an unbounded AggregateError. Each failed range reports protocol/catalog, exact range, attempted/failed counts, first failed mask in decimal/hex, its canonical enabled-item list, a length-limited assertion message and the number of omitted additional details. Throw a compact error after all checks. Every failed range stays visibly failed while reporter storage is bounded per range. Diagnosis can reproduce a reported mask/range without changing gate coverage.

## Acceptance and measurement

Prove helper mechanics independently:

- Ranges begin at zero, are contiguous/disjoint and end at the exact total, including a short final range.
- Successful execution visits every index exactly once; check progression/count without a full-mask visited array.
- Exhaustively compare 16-item mappings with the existing filter expression for all 65,536 indices.
- Empty, singleton and full masks preserve canonical ordering.
- Duplicate, missing, reordered and enlarged catalogs fail preflight.
- Several deliberate failures still allow every later index to execute; reported counts and bounded diagnostics remain correct.
- A failed range does not suppress another range.

Retain all existing malformed, boundary, privacy and archive-policy tests. When production fixtures migrate to v10, archived v9 needs a pinned v41 fixture as part of that migration; the helper must not depend on current production fixtures.

Measure the same passing focused v9 suite before/after the candidate using the recorded timing method. Record logical count, registered count, wall time, maximum RSS and actual exit status. Exercise a bounded synthetic failure workload to verify continuation and diagnostic size.

Independent review must verify assertion equivalence and full coverage before adoption. Choose range size or later process sharding from actual measurements, without invented gains, sampling, removed assertions, coverage-floor changes or timeout increases. A lower Vitest test count must always be accompanied by the unchanged logical-mask count.

No product question blocks this plan. Range size remains a measured decision; exhaustive coverage remains required.
