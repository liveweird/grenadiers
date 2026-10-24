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

test("coverage export creates its project-relative parent with an external Cargo target", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "grenadiers-coverage-export-"));
  const externalTarget = fs.mkdtempSync(path.join(os.tmpdir(), "grenadiers-cargo-target-"));
  const cargoLog = path.join(os.tmpdir(), `grenadiers-cargo-args-${process.pid}.log`);
  try {
    fs.mkdirSync(path.join(root, "scripts"));
    fs.mkdirSync(path.join(root, "bin"));
    const gates = fs.readFileSync(new URL("./gates.sh", import.meta.url), "utf8");
    const setup = "  mkdir -p target\n";
    assert.equal(gates.split(setup).length - 1, 1);
    fs.writeFileSync(path.join(root, "scripts", "gates.sh"), gates);
    fs.writeFileSync(path.join(root, "scripts", "gates-without-output-parent.sh"),
      gates.replace(setup, ""));
    fs.copyFileSync(new URL("./ci-server-mask-gate-context.sh", import.meta.url),
      path.join(root, "scripts", "ci-server-mask-gate-context.sh"));
    fs.writeFileSync(path.join(root, "Cargo.lock"), "# fixture lock\n");
    fs.writeFileSync(path.join(root, ".gitignore"), "target/\n");
    const cargo = `#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" > "$STUB_CARGO_LOG"
[ "$CARGO_TARGET_DIR" = "$EXPECTED_CARGO_TARGET_DIR" ]
[ "$*" = "llvm-cov --workspace --locked --fail-under-lines 80 --lcov --output-path target/coverage.lcov -- --show-output" ]
if [ ! -d target ]; then
  echo 'target output parent missing' >&2
  exit 73
fi
printf 'TN:\n' > target/coverage.lcov
`;
    fs.writeFileSync(path.join(root, "bin", "cargo"), cargo, { mode: 0o755 });
    assert.equal(spawnSync("git", ["init", "-q"], { cwd: root }).status, 0);
    spawnSync("git", ["config", "user.name", "Test"], { cwd: root });
    spawnSync("git", ["config", "user.email", "test@example.invalid"], { cwd: root });
    assert.equal(spawnSync("git", ["add", "."], { cwd: root }).status, 0);
    assert.equal(spawnSync("git", ["commit", "-qm", "fixture"], { cwd: root }).status, 0);
    const env = { ...process.env, PATH: `${path.join(root, "bin")}:${process.env.PATH}`,
      CARGO_TARGET_DIR: externalTarget, EXPECTED_CARGO_TARGET_DIR: externalTarget,
      STUB_CARGO_LOG: cargoLog };
    for (const name of ["PARTITION_INDEX", "PARTITION_COUNT", "REPORT_DIR", "PLAN_FILE", "CI_SOURCE"])
      delete env[`GRENADIERS_SERVER_REPLAY_ITEM_MASK_${name}`];

    const old = spawnSync("bash", ["scripts/gates-without-output-parent.sh", "coverage"],
      { cwd: root, env, encoding: "utf8" });
    assert.equal(old.status, 73);
    assert.match(old.stderr, /target output parent missing/);
    assert.equal(fs.existsSync(path.join(root, "target", "coverage.lcov")), false);

    const fixed = spawnSync("bash", ["scripts/gates.sh", "coverage"],
      { cwd: root, env, encoding: "utf8" });
    assert.equal(fixed.status, 0, fixed.stderr);
    assert.equal(fs.readFileSync(path.join(root, "target", "coverage.lcov"), "utf8"), "TN:\n");
    assert.equal(fs.readFileSync(cargoLog, "utf8").trim(),
      "llvm-cov --workspace --locked --fail-under-lines 80 --lcov --output-path target/coverage.lcov -- --show-output");
    assert.equal(spawnSync("git", ["status", "--porcelain"],
      { cwd: root, encoding: "utf8" }).stdout, "");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
    fs.rmSync(externalTarget, { recursive: true, force: true });
    fs.rmSync(cargoLog, { force: true });
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
  const partition = workflow.slice(
    workflow.indexOf("\n  server-mask-partition:\n"), workflow.indexOf("\n  server:\n"),
  );
  const aggregate = workflow.slice(workflow.indexOf("\n  server:\n"), workflow.indexOf("\n  security:\n"));
  const outcome = aggregate.indexOf("Require successful server mask jobs");
  const checkout = aggregate.indexOf("actions/checkout@");
  assert.ok(outcome > 0 && outcome < checkout);
  assert.match(aggregate, /needs\.server-mask-plan\.result \}\}' = 'success'/);
  assert.match(aggregate, /needs\.server-mask-partition\.result \}\}' = 'success'/);
  assert.match(aggregate, /actions\/setup-node@820762786026740c76f36085b0efc47a31fe5020/);
  assert.match(aggregate, /node-version-file: \.nvmrc/);
  assert.match(aggregate, /Verify aggregate source/);
  const pinnedDownload = /actions\/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c/g;
  assert.equal((partition.match(pinnedDownload) ?? []).length, 1);
  assert.equal((aggregate.match(pinnedDownload) ?? []).length, 2);
  assert.doesNotMatch(aggregate, /continue-on-error/);
  const expectedArtifactPath = "path: ${{ runner.temp }}/server-mask-artifacts${{ fromJSON(needs.server-mask-plan.outputs.matrix).include[0].count == 1 && format('/server-mask-partition-{0}-0', needs.server-mask-plan.outputs.source) || '' }}";
  const artifactPaths = aggregate.split("\n").map((line) => line.trim())
    .filter((line) => line.startsWith("path:") && line.includes("server-mask-artifacts"));
  assert.deepEqual(artifactPaths, [expectedArtifactPath]);
  const artifactDownload = aggregate.slice(aggregate.indexOf("pattern: server-mask-partition-*"),
    aggregate.indexOf("- name: Validate complete server mask evidence"));
  assert.match(artifactDownload, /merge-multiple: false/);
});

test("every opt-in browser scenario runs only in the item shard", () => {
  const workflow = fs.readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
  const steps = workflow.split(/(?=^      - name: )/m)
    .filter((step) => step.startsWith("      - name: Verify opt-in "));
  assert.ok(steps.length > 0, "expected opt-in browser scenarios");
  for (const step of steps) {
    const header = step.slice(0, step.indexOf("        run:"));
    assert.match(header, /^        if: matrix\.shard == 'items'$/m, step.split("\n")[0]);
  }
});

function planForPartitions(partitionCount) {
  const capacity = { 1: 8, 2: 4, 3: 3 }[partitionCount];
  const ranges = Array.from({ length: partitionCount }, (_, index) => ({
    index,
    startOrdinal: index * capacity,
    endOrdinalExclusive: Math.min((index + 1) * capacity, plan.totalCardinality),
  }));
  return { ...structuredClone(plan), capacity, partitionCount, ranges };
}

function summaryFor(fixturePlan, index, profile, overrides = {}) {
  const range = fixturePlan.ranges[index];
  return { schemaVersion: 1, source, profile, selection: "partition", partitionIndex: index,
    partitionCount: fixturePlan.partitionCount, startOrdinal: range.startOrdinal,
    endOrdinalExclusive: range.endOrdinalExclusive,
    observedCount: range.endOrdinalExclusive - range.startOrdinal,
    totalCardinality: fixturePlan.totalCardinality, ...overrides };
}

function writeOfficialActionLayout(root, partitionCount, { oldSingleRoot = false,
  matchedIndexes = Array.from({ length: partitionCount }, (_, index) => index) } = {}) {
  const fixturePlan = planForPartitions(partitionCount);
  const planPath = path.join(root, "plan.json");
  const artifactRoot = path.join(root, "artifacts");
  fs.writeFileSync(planPath, `${JSON.stringify(fixturePlan)}\n`);
  fs.mkdirSync(artifactRoot);
  const resolvedPath = partitionCount === 1 && !oldSingleRoot
    ? partitionDirectory(artifactRoot, 0) : artifactRoot;
  for (const index of matchedIndexes) {
    const range = fixturePlan.ranges[index];
    const artifactName = `server-mask-partition-${source}-${range.index}`;
    const directory = matchedIndexes.length === 1 ? resolvedPath : path.join(resolvedPath, artifactName);
    fs.mkdirSync(directory, { recursive: true });
    for (const profile of ["debug", "release", "coverage"]) {
      fs.writeFileSync(path.join(directory, `${profile}.json`),
        `${JSON.stringify(summaryFor(fixturePlan, range.index, profile))}\n`);
    }
  }
  return { fixturePlan, planPath, artifactRoot };
}

function withOfficialActionLayout(partitionCount, mutate, options = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "grenadiers-action-layout-"));
  try {
    const fixture = writeOfficialActionLayout(root, partitionCount, options);
    mutate?.(fixture);
    return runAggregate(fixture.planPath, fixture.artifactRoot);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function partitionDirectory(artifactRoot, index) {
  return path.join(artifactRoot, `server-mask-partition-${source}-${index}`);
}

function mutateSummary(file, mutation) {
  const value = JSON.parse(fs.readFileSync(file, "utf8"));
  fs.writeFileSync(file, `${JSON.stringify({ ...value, ...mutation })}\n`);
}

test("aggregate workflow path exactly preserves the official single-artifact directory", () => {
  assert.notEqual(withOfficialActionLayout(1, undefined, { oldSingleRoot: true }).status, 0);
  assert.equal(withOfficialActionLayout(1).status, 0);
});

test("aggregate CLI accepts official multi-artifact layouts for two and three partitions", () => {
  for (const count of [2, 3]) assert.equal(withOfficialActionLayout(count).status, 0);
});

test("aggregate CLI rejects malformed official-action artifact layouts", () => {
  assert.notEqual(withOfficialActionLayout(2, undefined, { matchedIndexes: [0] }).status, 0);
  assert.notEqual(withOfficialActionLayout(3, undefined, { matchedIndexes: [0] }).status, 0);
  assert.notEqual(withOfficialActionLayout(3, undefined, { matchedIndexes: [0, 1] }).status, 0);
  const failures = [
    ({ artifactRoot }) => fs.rmSync(partitionDirectory(artifactRoot, 1), { recursive: true }),
    ({ artifactRoot }) => fs.mkdirSync(partitionDirectory(artifactRoot, 3)),
    ({ artifactRoot }) => mutateSummary(path.join(partitionDirectory(artifactRoot, 0), "debug.json"),
      { source: "b".repeat(40) }),
    ({ artifactRoot }) => mutateSummary(path.join(partitionDirectory(artifactRoot, 0), "release.json"),
      { partitionIndex: 1 }),
    ({ artifactRoot }) => mutateSummary(path.join(partitionDirectory(artifactRoot, 0), "coverage.json"),
      { partitionCount: 2 }),
    ({ artifactRoot }) => mutateSummary(path.join(partitionDirectory(artifactRoot, 1), "debug.json"),
      { partitionIndex: 0 }),
    ({ artifactRoot }) => mutateSummary(path.join(partitionDirectory(artifactRoot, 1), "release.json"),
      { observedCount: 2 }),
    ({ artifactRoot }) => fs.rmSync(path.join(partitionDirectory(artifactRoot, 1), "coverage.json")),
    ({ artifactRoot, fixturePlan }) => fs.writeFileSync(
      path.join(partitionDirectory(artifactRoot, 1), "extra.json"),
      `${JSON.stringify(summaryFor(fixturePlan, 1, "coverage"))}\n`),
    ({ artifactRoot }) => fs.writeFileSync(path.join(artifactRoot, "unexpected.json"), "{}\n"),
    ({ artifactRoot }) => fs.symlinkSync(partitionDirectory(artifactRoot, 0), path.join(artifactRoot, "linked")),
    ({ artifactRoot }) => {
      const directory = partitionDirectory(artifactRoot, 0);
      for (const profile of ["debug", "release", "coverage"]) {
        fs.renameSync(path.join(directory, `${profile}.json`), path.join(artifactRoot, `${profile}.json`));
      }
      fs.rmdirSync(directory);
    },
  ];
  for (const mutate of failures) assert.notEqual(withOfficialActionLayout(3, mutate).status, 0);
});
