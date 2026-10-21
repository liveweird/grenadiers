import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { extractPrefixedJson, validatePlan } from "./ci-server-mask-plan.mjs";
import { validateSummary } from "./ci-server-mask-summary.mjs";
import { validateAggregate } from "./ci-server-mask-aggregate.mjs";

const source = "a".repeat(40);
const plan = {
  schemaVersion: 1,
  source,
  catalog: [1, 4, 16],
  capacity: 3,
  totalCardinality: 8,
  partitionCount: 3,
  ranges: [
    { index: 0, startOrdinal: 0, endOrdinalExclusive: 3 },
    { index: 1, startOrdinal: 3, endOrdinalExclusive: 6 },
    { index: 2, startOrdinal: 6, endOrdinalExclusive: 8 },
  ],
};

const summary = (index, profile, overrides = {}) => ({
  schemaVersion: 1,
  source,
  profile,
  selection: "partition",
  partitionIndex: index,
  partitionCount: 3,
  startOrdinal: plan.ranges[index].startOrdinal,
  endOrdinalExclusive: plan.ranges[index].endOrdinalExclusive,
  observedCount: plan.ranges[index].endOrdinalExclusive - plan.ranges[index].startOrdinal,
  totalCardinality: 8,
  ...overrides,
});

test("validates a complete sparse ordinal plan", () => {
  assert.deepEqual(validatePlan(structuredClone(plan), source), plan);
  for (const mutation of [
    { ranges: [plan.ranges[0], plan.ranges[2], plan.ranges[1]] },
    { partitionCount: 2 },
    { totalCardinality: 7 },
    { catalog: [1, 4, 4] },
    { catalog: [4, 1, 16] },
    { catalog: [1, 4, 2 ** 32] },
    { source: "b".repeat(40) },
    { extra: true },
    { ranges: [{ ...plan.ranges[0], extra: true }, ...plan.ranges.slice(1)] },
  ]) assert.throws(() => validatePlan({ ...plan, ...mutation }, source));
});

test("extracts exactly one controlled prefixed JSON line", () => {
  assert.deepEqual(extractPrefixedJson(`noise\nPREFIX={"ok":true}\n`, "PREFIX="), { ok: true });
  assert.throws(() => extractPrefixedJson("noise", "PREFIX="));
  assert.throws(() => extractPrefixedJson("PREFIX={}\nPREFIX={}", "PREFIX="));
});

test("rejects wrong source, profile, count, range, and observed completion", () => {
  assert.equal(validateSummary(summary(1, "debug"), { plan, source, profile: "debug", index: 1 }).profile, "debug");
  for (const mutation of [
    { source: "b".repeat(40) }, { profile: "other" }, { partitionCount: 2 },
    { startOrdinal: 2 }, { endOrdinalExclusive: 5 }, { observedCount: 2 },
    { totalCardinality: 7 }, { partitionIndex: 0 }, { selection: "all" },
    { extra: true }, { observedCount: "3" },
  ]) assert.throws(() => validateSummary({ ...summary(1, "debug"), ...mutation }, { plan, source, profile: "debug", index: 1 }));
});

function writeAggregateFixture(root) {
  const planPath = path.join(root, "plan.json");
  const artifactRoot = path.join(root, "artifacts");
  fs.writeFileSync(planPath, `${JSON.stringify(plan)}\n`);
  fs.mkdirSync(artifactRoot);
  for (const range of plan.ranges) {
    const directory = path.join(artifactRoot, `server-mask-partition-${source}-${range.index}`);
    fs.mkdirSync(directory);
    for (const profile of ["debug", "release", "coverage"]) {
      fs.writeFileSync(path.join(directory, `${profile}.json`), `${JSON.stringify(summary(range.index, profile))}\n`);
    }
  }
  return { planPath, artifactRoot };
}

function runAggregate(planPath, artifactRoot) {
  return spawnSync(process.execPath, [new URL("./ci-server-mask-aggregate.mjs", import.meta.url).pathname,
    planPath, artifactRoot, source], { encoding: "utf8" });
}

test("aggregate CLI rejects root files, empty directories, and symlinks", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "grenadiers-mask-artifacts-"));
  try {
    const { planPath, artifactRoot } = writeAggregateFixture(root);
    assert.equal(runAggregate(planPath, artifactRoot).status, 0);
    const rootFile = path.join(artifactRoot, "unexpected.json");
    fs.writeFileSync(rootFile, "{}\n");
    assert.notEqual(runAggregate(planPath, artifactRoot).status, 0);
    fs.rmSync(rootFile);
    const empty = path.join(artifactRoot, "empty");
    fs.mkdirSync(empty);
    assert.notEqual(runAggregate(planPath, artifactRoot).status, 0);
    fs.rmdirSync(empty);
    const link = path.join(artifactRoot, "linked");
    fs.symlinkSync(path.join(artifactRoot, `server-mask-partition-${source}-0`), link);
    assert.notEqual(runAggregate(planPath, artifactRoot).status, 0);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function runGateContext(root, extraEnv = {}) {
  const env = { ...process.env, ...extraEnv };
  for (const name of ["PARTITION_INDEX", "PARTITION_COUNT", "REPORT_DIR", "PLAN_FILE", "CI_SOURCE"]) {
    const full = `GRENADIERS_SERVER_REPLAY_ITEM_MASK_${name}`;
    if (!(full in extraEnv)) delete env[full];
  }
  const helper = new URL("./ci-server-mask-gate-context.sh", import.meta.url).pathname;
  return spawnSync("bash", ["-c", 'source "$1"; server_mask_resolve_context "$2" || exit $?; printf "%s" "$server_mask_source"',
    "gate-context", helper, root], { encoding: "utf8", env });
}

test("gate context rejects inherited repositories and partial selectors", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "grenadiers-mask-git-"));
  try {
    assert.equal(spawnSync("git", ["init", "-q"], { cwd: root }).status, 0);
    spawnSync("git", ["config", "user.name", "Test"], { cwd: root });
    spawnSync("git", ["config", "user.email", "test@example.invalid"], { cwd: root });
    fs.writeFileSync(path.join(root, "tracked"), "ok\n");
    fs.writeFileSync(path.join(root, ".gitignore"), "ignored/\n");
    spawnSync("git", ["add", "tracked", ".gitignore"], { cwd: root });
    assert.equal(spawnSync("git", ["commit", "-qm", "fixture"], { cwd: root }).status, 0);
    const head = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).stdout.trim();
    assert.equal(runGateContext(root).stdout, head);
    const planFile = path.join(os.tmpdir(), `grenadiers-mask-plan-${process.pid}.json`);
    fs.writeFileSync(planFile, "{}\n");
    const full = { GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX: "0",
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_COUNT: "1",
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_REPORT_DIR: path.join(root, "report"),
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_PLAN_FILE: planFile,
      GRENADIERS_SERVER_REPLAY_ITEM_MASK_CI_SOURCE: head };
    assert.equal(runGateContext(root, full).status, 0);
    assert.notEqual(runGateContext(root, { GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX: "0" }).status, 0);
    assert.notEqual(runGateContext(root, { ...full, GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_COUNT: "" }).status, 0);
    const archive = path.join(root, "ignored", "archive");
    fs.mkdirSync(archive, { recursive: true });
    const inherited = runGateContext(archive);
    assert.equal(inherited.status, 0);
    assert.equal(inherited.stdout, "");
    fs.rmSync(planFile);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("aggregate requires every partition profile and no extras", () => {
  const artifacts = new Map();
  for (const range of plan.ranges) for (const profile of ["debug", "release", "coverage"]) {
    artifacts.set(`server-mask-partition-${source}-${range.index}/${profile}.json`, summary(range.index, profile));
  }
  assert.doesNotThrow(() => validateAggregate(plan, source, artifacts));
  const missing = new Map(artifacts); missing.delete(`server-mask-partition-${source}-1/coverage.json`);
  assert.throws(() => validateAggregate(plan, source, missing), /missing/);
  const extra = new Map(artifacts); extra.set("unexpected/debug.json", summary(0, "debug"));
  assert.throws(() => validateAggregate(plan, source, extra), /unexpected/);
});

test("workflow aggregate fails closed before pinned runtime and artifact validation", () => {
  const workflow = fs.readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
  const aggregate = workflow.slice(workflow.indexOf("\n  server:\n"), workflow.indexOf("\n  security:\n"));
  const outcome = aggregate.indexOf("Require successful server mask jobs");
  const checkout = aggregate.indexOf("actions/checkout@");
  assert.ok(outcome > 0 && outcome < checkout);
  assert.match(aggregate, /needs\.server-mask-plan\.result \}\}' = 'success'/);
  assert.match(aggregate, /needs\.server-mask-partition\.result \}\}' = 'success'/);
  assert.match(aggregate, /actions\/setup-node@820762786026740c76f36085b0efc47a31fe5020/);
  assert.match(aggregate, /node-version-file: \.nvmrc/);
  assert.match(aggregate, /Verify aggregate source/);
  assert.equal((workflow.match(/actions\/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c/g) ?? []).length, 3);
  assert.doesNotMatch(aggregate, /continue-on-error/);
});
