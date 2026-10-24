#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { extractPrefixedJson, PLAN_PREFIX, validatePlan } from "./ci-server-mask-plan.mjs";

export function generatePlan(root, source, output) {
  if (!/^[0-9a-f]{40}$/.test(source)) throw new Error("source must be a 40-hex commit");
  const result = spawnSync("cargo", ["test", "-p", "grenadiers-server",
    "game::rooms::replay::item_mask_plan::tests::export_checked_plan_for_ci",
    "--locked", "--", "--exact", "--show-output"], {
    cwd: root, encoding: "utf8", env: { ...process.env,
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_SOURCE: source,
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_EXPORT_PLAN: "1",
    },
  });
  process.stdout.write(result.stdout); process.stderr.write(result.stderr);
  if (result.status !== 0) throw new Error(`source-derived plan command exited ${result.status}`);
  const plan = validatePlan(extractPrefixedJson(result.stdout, PLAN_PREFIX), source);
  fs.writeFileSync(output, `${JSON.stringify(plan, null, 2)}\n`, { flag: "wx" });
  return plan;
}

function main() {
  const [root, source, output] = process.argv.slice(2);
  if (!root || !output || !path.isAbsolute(root) || !path.isAbsolute(output)) {
    throw new Error("usage: local-server-mask-plan.mjs <absolute-root> <source> <absolute-output>");
  }
  generatePlan(root, source, output);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) main();
