import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import test from "node:test";

import { collectSample, MAX_PROCESSES, parseProcStat } from "./ci-visibility-startup-observer.mjs";

const observer = new URL("./ci-visibility-startup-observer.mjs", import.meta.url);

function statLine({ pid, ppid, pgid = ppid, starttime, state = "S", threads = 1,
  userTicks = 2, systemTicks = 3, rssPages = 4 }) {
  const fields = Array(22).fill("0");
  fields[0] = state; fields[1] = String(ppid); fields[2] = String(pgid);
  fields[11] = String(userTicks); fields[12] = String(systemTicks);
  fields[17] = String(threads); fields[19] = String(starttime); fields[21] = String(rssPages);
  return `${pid} (fixture process) ${fields.join(" ")}\n`;
}

function writeProcess(root, process, argv = ["node"], stderrTarget = null) {
  const directory = path.join(root, String(process.pid));
  fs.mkdirSync(path.join(directory, "fd"), { recursive: true });
  fs.writeFileSync(path.join(directory, "stat"), statLine(process));
  fs.writeFileSync(path.join(directory, "cmdline"), `${argv.join("\0")}\0`);
  fs.writeFileSync(path.join(directory, "wchan"), "do_wait\n");
  fs.symlinkSync(stderrTarget ?? path.join(directory, "stderr.log"), path.join(directory, "fd", "2"));
  if (!stderrTarget) fs.writeFileSync(path.join(directory, "stderr.log"), "");
}

function withProc(assertion) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "visibility-observer-proc-"));
  try { return assertion(root); }
  finally { fs.rmSync(root, { recursive: true, force: true }); }
}

async function waitFor(predicate, description, timeoutMs = 3000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  throw new Error(`timed out waiting for ${description}`);
}

function processExists(pid) {
  try { process.kill(pid, 0); return true; }
  catch (error) {
    if (error?.code === "ESRCH") return false;
    throw error;
  }
}

test("parses parent, group, CPU, thread, RSS and starttime fields", () => {
  assert.deepEqual(parseProcStat(statLine({ pid: 7, ppid: 3, pgid: 2, starttime: 99,
    state: "R", threads: 4, userTicks: 8, systemTicks: 9, rssPages: 10 })), {
    pid: 7, state: "R", ppid: 3, pgid: 2, userTicks: 8, systemTicks: 9,
    threads: 4, starttime: 99, rssPages: 10,
  });
});

test("keeps only the owned ancestry and exposes no argv, profile path, endpoint, or environment", () => {
  withProc((root) => {
    writeProcess(root, { pid: 100, ppid: 1, starttime: 10 });
    const secret = path.join(root, "grenadiers-visibility-secret-profile");
    fs.mkdirSync(secret);
    fs.writeFileSync(path.join(secret, "DevToolsActivePort"), "secret-endpoint\n");
    writeProcess(root, { pid: 101, ppid: 100, starttime: 11 }, ["chromium",
      "--no-sandbox", "--disable-dev-shm-usage", "--no-first-run",
      "--no-default-browser-check", "--enable-gpu", "--use-gl=angle",
      "--use-angle=swiftshader", "--remote-debugging-port=0",
      `--user-data-dir=${secret}`, "about:blank"]);
    writeProcess(root, { pid: 200, ppid: 1, starttime: 12 }, ["unrelated", "secret"]);
    const sample = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10,
      tempRoot: root });
    assert.deepEqual(sample.processes.map(({ pid }) => pid), [100, 101]);
    assert.equal(sample.processes[1].visibilityArgvContract, "exact");
    assert.equal(sample.processes[1].visibilityProfileUnique, true);
    assert.deepEqual(sample.processes[1].devToolsActivePort,
      { exists: true, type: "regular-file", size: 16 });
    const secondProfile = path.join(root, "grenadiers-visibility-second-profile");
    const repeated = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10,
      tempRoot: root, knownProfiles: new Set([secondProfile]) });
    assert.equal(repeated.processes[1].visibilityProfileUnique, false);
    assert.equal(repeated.processes[1].visibilityChromium, false);
    const serialized = JSON.stringify(sample);
    for (const forbidden of [secret, secondProfile, "secret-endpoint", "cmdline", "environment",
      "about:blank"])
      assert.equal(serialized.includes(forbidden), false);
  });
});

test("rejects root PID reuse and records owned-process disappearance", () => {
  withProc((root) => {
    writeProcess(root, { pid: 100, ppid: 1, starttime: 20 });
    const reused = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10 });
    assert.deepEqual(reused.processes, []);
    assert.equal(reused.gaps[0].kind, "root-pid-reused");
    fs.rmSync(path.join(root, "100"), { recursive: true });
    writeProcess(root, { pid: 100, ppid: 1, starttime: 10 });
    const missing = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10,
      previousIdentities: new Set(["100:10", "101:11"]) });
    assert.deepEqual(missing.gaps, [{ kind: "owned-process-disappeared", pid: 101, starttime: 11 }]);
  });
});

test("omits metadata when an owned PID is reused during collection", () => {
  withProc((root) => {
    writeProcess(root, { pid: 100, ppid: 1, starttime: 10 });
    const originalProfile = path.join(root, "grenadiers-visibility-original-profile");
    fs.mkdirSync(originalProfile);
    writeProcess(root, { pid: 101, ppid: 100, starttime: 11 }, ["chromium",
      "--no-sandbox", "--disable-dev-shm-usage", "--no-first-run",
      "--no-default-browser-check", "--enable-gpu", "--use-gl=angle",
      "--use-angle=swiftshader", "--remote-debugging-port=0",
      `--user-data-dir=${originalProfile}`, "about:blank"]);
    const unrelatedProfile = path.join(root, "grenadiers-visibility-unrelated-secret");
    fs.mkdirSync(unrelatedProfile);
    fs.writeFileSync(path.join(unrelatedProfile, "DevToolsActivePort"), "unrelated-endpoint\n");
    const sample = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10,
      tempRoot: root, beforeMetadata(pid) {
        if (pid !== 101) return;
        fs.writeFileSync(path.join(root, "101", "stat"), statLine({
          pid: 101, ppid: 1, starttime: 999,
        }));
        fs.writeFileSync(path.join(root, "101", "cmdline"),
          `chromium\0--user-data-dir=${unrelatedProfile}\0unrelated-secret\0`);
        fs.writeFileSync(path.join(root, "101", "wchan"), "unrelated_secret\n");
      } });
    assert.deepEqual(sample.processes.map(({ pid }) => pid), [100]);
    assert.deepEqual(sample.gaps, [{ kind: "owned-process-identity-changed", pid: 101,
      starttime: 11, phase: "after-metadata" }]);
    const serialized = JSON.stringify(sample);
    for (const forbidden of [unrelatedProfile, "unrelated-endpoint", "unrelated_secret"])
      assert.equal(serialized.includes(forbidden), false);
  });
});

test("bounds owned process records and reports truncation", () => {
  withProc((root) => {
    writeProcess(root, { pid: 100, ppid: 1, starttime: 10 });
    for (let index = 0; index < MAX_PROCESSES + 4; index += 1)
      writeProcess(root, { pid: 101 + index, ppid: 100, starttime: 20 + index });
    const sample = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10 });
    assert.equal(sample.processes.length, MAX_PROCESSES);
    assert.equal(sample.processTruncated, true);
  });
});

test("runs the child once and preserves a nonzero exit without retrying", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "visibility-observer-exit-"));
  try {
    const marker = path.join(root, "marker");
    const child = `require('node:fs').appendFileSync(${JSON.stringify(marker)}, 'x');process.exit(7)`;
    const result = spawnSync(process.execPath, [observer.pathname, "--", process.execPath, "-e", child],
      { env: { ...process.env, RUNNER_TEMP: root } });
    assert.equal(result.status, 7);
    assert.equal(fs.readFileSync(marker, "utf8"), "x");
    const lines = fs.readFileSync(path.join(root, "visibility-startup-observer.jsonl"), "utf8")
      .trim().split("\n").map(JSON.parse);
    assert.equal(lines.at(-1).exitCode, 7);
    assert.equal(lines.filter(({ kind }) => kind === "complete").length, 1);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("preserves a child terminating signal", { skip: process.platform === "win32" }, () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "visibility-observer-signal-"));
  try {
    const result = spawnSync(process.execPath, [observer.pathname, "--", process.execPath, "-e",
      "process.kill(process.pid, 'SIGTERM')"], { env: { ...process.env, RUNNER_TEMP: root } });
    assert.equal(result.status, null);
    assert.equal(result.signal, "SIGTERM");
    const lines = fs.readFileSync(path.join(root, "visibility-startup-observer.jsonl"), "utf8")
      .trim().split("\n").map(JSON.parse);
    assert.equal(lines.at(-1).signal, "SIGTERM");
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("forwards a wrapper signal to its command group and waits for child completion",
  { skip: process.platform === "win32" }, async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "visibility-observer-forward-"));
    const invocation = path.join(root, "invocation");
    const childReady = path.join(root, "child-ready");
    const grandchildReady = path.join(root, "grandchild-ready");
    const signals = path.join(root, "signals");
    const grandchildScript = path.join(root, "grandchild.cjs");
    const childScript = path.join(root, "child.cjs");
    let wrapper;
    let childPid;
    let grandchildPid;
    try {
      fs.writeFileSync(grandchildScript, [
        "const fs = require('node:fs');",
        "process.on('SIGTERM', () => { fs.appendFileSync(process.env.SIGNALS, 'grandchild\\n'); process.exit(0); });",
        "fs.writeFileSync(process.env.GRANDCHILD_READY, String(process.pid));",
        "setInterval(() => {}, 1000);",
      ].join("\n"));
      fs.writeFileSync(childScript, [
        "const fs = require('node:fs');",
        "const { spawn } = require('node:child_process');",
        "fs.appendFileSync(process.env.INVOCATION, 'x');",
        "process.on('SIGTERM', () => { fs.appendFileSync(process.env.SIGNALS, 'child\\n'); setTimeout(() => process.exit(0), 100); });",
        "spawn(process.execPath, [process.env.GRANDCHILD_SCRIPT], { stdio: 'ignore' });",
        "fs.writeFileSync(process.env.CHILD_READY, String(process.pid));",
        "setInterval(() => {}, 1000);",
      ].join("\n"));
      wrapper = spawn(process.execPath,
        [observer.pathname, "--", process.execPath, childScript], {
          env: { ...process.env, RUNNER_TEMP: root, INVOCATION: invocation, CHILD_READY: childReady,
            GRANDCHILD_READY: grandchildReady, GRANDCHILD_SCRIPT: grandchildScript, SIGNALS: signals },
          stdio: "ignore",
        });
      await waitFor(() => fs.existsSync(childReady) && fs.existsSync(grandchildReady),
        "child and grandchild readiness");
      childPid = Number(fs.readFileSync(childReady, "utf8"));
      grandchildPid = Number(fs.readFileSync(grandchildReady, "utf8"));
      process.kill(wrapper.pid, "SIGTERM");
      const result = await new Promise((resolve, reject) => {
        wrapper.once("error", reject);
        wrapper.once("exit", (code, signal) => resolve({ code, signal }));
      });
      assert.deepEqual(result, { code: null, signal: "SIGTERM" });
      assert.equal(fs.readFileSync(invocation, "utf8"), "x");
      assert.deepEqual(fs.readFileSync(signals, "utf8").trim().split("\n").sort(),
        ["child", "grandchild"]);
      await waitFor(() => !processExists(childPid) && !processExists(grandchildPid),
        "command group exit");
      const lines = fs.readFileSync(path.join(root, "visibility-startup-observer.jsonl"), "utf8")
        .trim().split("\n").map(JSON.parse);
      assert.deepEqual(lines.filter(({ kind }) => kind === "signal-forwarded"),
        [{ kind: "signal-forwarded", signal: "SIGTERM" }]);
      assert.equal(lines.at(-1).forwardedSignal, "SIGTERM");
      assert.equal(lines.at(-1).exitCode, 0);
    } finally {
      for (const pid of [grandchildPid, childPid, wrapper?.pid]) {
        if (!pid) continue;
        try { process.kill(pid, "SIGKILL"); } catch (error) {
          if (error?.code !== "ESRCH") throw error;
        }
      }
      fs.rmSync(root, { recursive: true, force: true });
    }
  });

test("workflow wraps only hardening and retains observer and Xvfb diagnostics on failure", () => {
  const workflow = fs.readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
  assert.match(workflow, /E2E_MATCH_FIXTURE=arena-v1-matches xvfb-run -a -e "\$RUNNER_TEMP\/visibility-xvfb\.log" node scripts\/ci-visibility-startup-observer\.mjs -- npm --prefix e2e run test:hardening/);
  assert.equal((workflow.match(/ci-visibility-startup-observer\.mjs --/g) ?? []).length, 1);
  assert.equal((workflow.match(/node --test scripts\/ci-visibility-startup-observer\.test\.mjs/g)
    ?? []).length, 1);
  assert.match(workflow, /name: Check visibility startup observer contract\n\s+if: matrix\.shard == 'classic'/);
  assert.match(workflow, /xvfb-run -a npm --prefix e2e run test:audio/);
  assert.match(workflow, /\$\{\{ runner\.temp \}\}\/visibility-startup-observer\.jsonl/);
  assert.match(workflow, /\$\{\{ runner\.temp \}\}\/visibility-xvfb\.log/);
  assert.match(workflow, /matrix:\n\s+shard: \[items, classic\]/);
  assert.match(workflow, /timeout-minutes: 20/);
});
