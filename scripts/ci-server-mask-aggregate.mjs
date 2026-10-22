import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { validatePlan } from "./ci-server-mask-plan.mjs";
import { validateSummary } from "./ci-server-mask-summary.mjs";

const profiles = ["debug", "release", "coverage"];

export function validateAggregate(plan, source, artifacts) {
  validatePlan(plan, source);
  const expectedPaths = new Set();
  for (const range of plan.ranges) for (const profile of profiles) {
    const relative = `server-mask-partition-${source}-${range.index}/${profile}.json`;
    expectedPaths.add(relative);
    const summary = artifacts.get(relative);
    if (!summary) throw new Error(`missing ${relative}`);
    validateSummary(summary, { plan, source, profile, index: range.index });
  }
  if (artifacts.size !== expectedPaths.size || [...artifacts.keys()].some((key) => !expectedPaths.has(key))) {
    throw new Error("unexpected or duplicate server-mask artifact paths");
  }
}

function main() {
  const [planPath, artifactRoot, source] = process.argv.slice(2);
  const plan = JSON.parse(fs.readFileSync(planPath, "utf8"));
  const artifacts = new Map();
  validatePlan(plan, source);
  const expectedDirectories = new Set(plan.ranges.map((range) =>
    `server-mask-partition-${source}-${range.index}`));
  const rootEntries = fs.readdirSync(artifactRoot, { withFileTypes: true });
  if (rootEntries.length !== expectedDirectories.size) throw new Error("artifact directory count mismatch");
  for (const entry of rootEntries) {
    if (!entry.isDirectory() || entry.isSymbolicLink() || !expectedDirectories.has(entry.name)) {
      throw new Error(`unexpected artifact root entry ${entry.name}`);
    }
    const directory = entry.name;
    const full = path.join(artifactRoot, directory);
    const files = fs.readdirSync(full, { withFileTypes: true });
    if (files.length !== profiles.length) throw new Error(`artifact file count mismatch in ${directory}`);
    for (const fileEntry of files) {
      if (!fileEntry.isFile() || fileEntry.isSymbolicLink()
          || !profiles.map((profile) => `${profile}.json`).includes(fileEntry.name)) {
        throw new Error(`unexpected nested artifact entry ${directory}/${fileEntry.name}`);
      }
      const entryName = fileEntry.name;
      const relative = `${directory}/${entryName}`;
      if (artifacts.has(relative)) throw new Error(`duplicate artifact path ${relative}`);
      artifacts.set(relative, JSON.parse(fs.readFileSync(path.join(full, entryName), "utf8")));
    }
  }
  validateAggregate(plan, source, artifacts);
  process.stdout.write(`validated ${plan.partitionCount} server mask partitions and ${artifacts.size} profile summaries for ${source}\n`);
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) main();
