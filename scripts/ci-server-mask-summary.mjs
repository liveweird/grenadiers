import fs from "node:fs";
import { pathToFileURL } from "node:url";
import { extractPrefixedJson } from "./ci-server-mask-plan.mjs";

export const SUMMARY_PREFIX = "GRENADIERS_SERVER_REPLAY_ITEM_MASK_SUMMARY=";
const profiles = new Set(["debug", "release", "coverage"]);
const summaryKeys = ["endOrdinalExclusive", "observedCount", "partitionCount", "partitionIndex",
  "profile", "schemaVersion", "selection", "source", "startOrdinal", "totalCardinality"];

export function validateSummary(summary, expected) {
  const range = expected.plan.ranges[expected.index];
  if (summary === null || typeof summary !== "object" || Array.isArray(summary)
      || JSON.stringify(Object.keys(summary).sort()) !== JSON.stringify(summaryKeys)
      || summary.schemaVersion !== 1 || summary.source !== expected.source || summary.profile !== expected.profile
      || !profiles.has(summary.profile) || summary.selection !== "partition"
      || !Number.isSafeInteger(summary.partitionIndex) || !Number.isSafeInteger(summary.partitionCount)
      || !Number.isSafeInteger(summary.startOrdinal) || !Number.isSafeInteger(summary.endOrdinalExclusive)
      || !Number.isSafeInteger(summary.observedCount) || !Number.isSafeInteger(summary.totalCardinality)
      || summary.partitionIndex !== expected.index || summary.partitionCount !== expected.plan.partitionCount
      || summary.startOrdinal !== range.startOrdinal || summary.endOrdinalExclusive !== range.endOrdinalExclusive
      || summary.observedCount !== range.endOrdinalExclusive - range.startOrdinal
      || summary.totalCardinality !== expected.plan.totalCardinality) {
    throw new Error("mask completion summary does not match checked plan/profile/source");
  }
  return summary;
}

function main() {
  const [logPath, planPath, source, profile, indexText, outputPath] = process.argv.slice(2);
  const plan = JSON.parse(fs.readFileSync(planPath, "utf8"));
  const summary = extractPrefixedJson(fs.readFileSync(logPath, "utf8"), SUMMARY_PREFIX);
  validateSummary(summary, { plan, source, profile, index: Number(indexText) });
  fs.writeFileSync(outputPath, `${JSON.stringify(summary, null, 2)}\n`);
  process.stdout.write(`validated server item-mask ${profile} summary for partition ${indexText}\n`);
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) main();
