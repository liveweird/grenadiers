import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

export const PLAN_PREFIX = "GRENADIERS_SERVER_REPLAY_ITEM_MASK_PLAN=";

const hex40 = /^[0-9a-f]{40}$/;
const planKeys = ["capacity", "catalog", "partitionCount", "ranges", "schemaVersion", "source", "totalCardinality"];
const rangeKeys = ["endOrdinalExclusive", "index", "startOrdinal"];

function hasExactKeys(value, keys) {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    && JSON.stringify(Object.keys(value).sort()) === JSON.stringify(keys);
}

export function validatePlan(plan, source) {
  if (!hasExactKeys(plan, planKeys) || plan.schemaVersion !== 1 || !hex40.test(source) || plan.source !== source) {
    throw new Error("plan schema/source mismatch");
  }
  const { catalog, capacity, totalCardinality, partitionCount, ranges } = plan;
  if (!Array.isArray(catalog) || catalog.length === 0 || catalog.length > 32
      || !Number.isSafeInteger(capacity) || capacity < 1
      || !Number.isSafeInteger(totalCardinality) || totalCardinality < 1
      || !Number.isSafeInteger(partitionCount) || partitionCount < 1) {
    throw new Error("invalid catalog or capacity");
  }
  const seen = new Set();
  for (const [index, bit] of catalog.entries()) {
    if (!Number.isSafeInteger(bit) || bit <= 0 || bit > 0x80000000) {
      throw new Error("catalog entries must be ascending distinct positive u32 single bits");
    }
    const wide = BigInt(bit);
    if ((wide & (wide - 1n)) !== 0n || seen.has(bit)
        || (index > 0 && catalog[index - 1] >= bit)) {
      throw new Error("catalog entries must be ascending distinct positive u32 single bits");
    }
    seen.add(bit);
  }
  if (totalCardinality !== 2 ** catalog.length || partitionCount !== Math.ceil(totalCardinality / capacity)) {
    throw new Error("plan cardinality/count mismatch");
  }
  if (!Array.isArray(ranges) || ranges.length !== partitionCount) throw new Error("range count mismatch");
  let cursor = 0;
  for (const [index, range] of ranges.entries()) {
    const end = Math.min(cursor + capacity, totalCardinality);
    if (!hasExactKeys(range, rangeKeys) || !Number.isSafeInteger(range.index)
        || !Number.isSafeInteger(range.startOrdinal) || !Number.isSafeInteger(range.endOrdinalExclusive)
        || range.index !== index || range.startOrdinal !== cursor || range.endOrdinalExclusive !== end) {
      throw new Error("ranges must be ordered, disjoint, and complete");
    }
    cursor = end;
  }
  if (cursor !== totalCardinality) throw new Error("ranges do not cover cardinality");
  return plan;
}

export function extractPrefixedJson(output, prefix) {
  const lines = output.split(/\r?\n/).filter((line) => line.startsWith(prefix));
  if (lines.length !== 1) throw new Error(`expected exactly one ${prefix} line, got ${lines.length}`);
  return JSON.parse(lines[0].slice(prefix.length));
}

function main() {
  const source = process.argv[2];
  if (!hex40.test(source ?? "")) throw new Error("usage: ci-server-mask-plan.mjs <40hex-source>");
  const test = "game::rooms::replay::item_mask_plan::tests::export_checked_plan_for_ci";
  const result = spawnSync("cargo", ["test", "-p", "grenadiers-server", test, "--locked", "--", "--exact", "--show-output"], {
    encoding: "utf8", env: { ...process.env,
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_SOURCE: source,
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_EXPORT_PLAN: "1",
    },
  });
  process.stdout.write(result.stdout); process.stderr.write(result.stderr);
  if (result.status !== 0) process.exit(result.status ?? 1);
  const plan = validatePlan(extractPrefixedJson(result.stdout, PLAN_PREFIX), source);
  fs.writeFileSync("server-mask-plan.json", `${JSON.stringify(plan, null, 2)}\n`);
  const matrix = { include: plan.ranges.map((range) => ({ index: range.index, count: plan.partitionCount })) };
  if (process.env.GITHUB_OUTPUT) fs.appendFileSync(process.env.GITHUB_OUTPUT, `matrix=${JSON.stringify(matrix)}\n`);
  else process.stdout.write(`${JSON.stringify(matrix)}\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) main();
