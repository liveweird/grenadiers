import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import test from "node:test";

import { captureStartupFileSnapshot, collectSample, FILE_SNAPSHOT_ELAPSED_MS,
  MAX_MAPPING_BYTES, MAX_MAPPING_ENTRIES, MAX_MAPPING_GROUPS, MAX_PROCESSES,
  parseMemAvailable, parsePressure, parseProcIo, parseProcStat, parseProcStatusMemory }
  from "./ci-visibility-startup-observer.mjs";

const observer = new URL("./ci-visibility-startup-observer.mjs", import.meta.url);

function statLine({ pid, ppid, pgid = ppid, starttime, state = "S", threads = 1,
  minorFaults = 5, majorFaults = 6, userTicks = 2, systemTicks = 3, rssPages = 4 }) {
  const fields = Array(22).fill("0");
  fields[0] = state; fields[1] = String(ppid); fields[2] = String(pgid);
  fields[7] = String(minorFaults); fields[9] = String(majorFaults);
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
  fs.writeFileSync(path.join(directory, "io"),
    "syscr: 7\nsyscw: 8\nread_bytes: 9\nwrite_bytes: 10\n");
  fs.writeFileSync(path.join(directory, "status"),
    "VmRSS:\t11 kB\nRssAnon:\t12 kB\nRssFile:\t13 kB\nRssShmem:\t14 kB\nVmSwap:\t15 kB\n");
  fs.symlinkSync(stderrTarget ?? path.join(directory, "stderr.log"), path.join(directory, "fd", "2"));
  if (!stderrTarget) fs.writeFileSync(path.join(directory, "stderr.log"), "");
}

function writeFileMappings(root, pid, executable, extraPath = null) {
  const directory = path.join(root, String(pid));
  fs.symlinkSync(executable, path.join(directory, "exe"));
  const lines = [
    `1000-2000 r-xp 00000000 08:01 41 ${executable}`,
    `2000-3000 r--p 00001000 08:01 41 ${executable}`,
  ];
  if (extraPath) lines.push(`3000-4000 r--p 00000000 08:02 42 ${extraPath}`);
  fs.writeFileSync(path.join(directory, "maps"), `${lines.join("\n")}\n`);
  fs.writeFileSync(path.join(directory, "smaps"), `${lines.map((line, index) =>
    `${line}\nRss: ${index + 2} kB\nReferenced: ${index + 1} kB`).join("\n")}\n`);
}

function executableExpectation(executable) {
  const stat = fs.statSync(executable, { bigint: true });
  return { canonicalPathSHA256: createHash("sha256").update(executable).digest("hex"),
    device: String(stat.dev), inode: String(stat.ino), size: Number(stat.size) };
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
    state: "R", threads: 4, minorFaults: 6, majorFaults: 7,
    userTicks: 8, systemTicks: 9, rssPages: 10 })), {
    pid: 7, state: "R", ppid: 3, pgid: 2, userTicks: 8, systemTicks: 9,
    minorFaults: 6, majorFaults: 7, threads: 4, starttime: 99, rssPages: 10,
  });
});

test("parses selected process and bounded host counters at their documented positions", () => {
  assert.deepEqual(parseProcIo("syscr: 3\nsyscw: 4\nread_bytes: 5\nwrite_bytes: 6\n"),
    { readCalls: 3, writeCalls: 4, readBytes: 5, writeBytes: 6 });
  assert.deepEqual(parseProcStatusMemory(
    "VmRSS: 7 kB\nRssAnon: 8 kB\nRssFile: 9 kB\nRssShmem: 10 kB\nVmSwap: 11 kB\n"),
  { residentKb: 7, anonymousKb: 8, fileKb: 9, sharedKb: 10, swapKb: 11 });
  assert.deepEqual(parsePressure(
    "some avg10=0.00 avg60=0.00 avg300=0.00 total=12\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=13\n"),
  { someTotalUs: 12, fullTotalUs: 13 });
  assert.equal(parseMemAvailable("MemTotal: 99 kB\nMemAvailable: 14 kB\n"), 14);
});

test("attributes matching executable mappings, aggregates residency, and reports later deltas", () => {
  withProc((root) => {
    const browserRoot = path.join(root, "browser");
    fs.mkdirSync(browserRoot);
    const executable = path.join(browserRoot, "chrome");
    fs.writeFileSync(executable, "browser-binary");
    const privatePath = path.join(root, "grenadiers-visibility-secret", "cache.bin");
    writeProcess(root, { pid: 101, ppid: 1, starttime: 11 });
    writeFileMappings(root, 101, executable, privatePath);
    const opened = [];
    const fsApi = Object.create(fs);
    fsApi.openSync = (file, ...args) => { opened.push(file); return fs.openSync(file, ...args); };
    const first = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      elapsedMs: 10, expectedExecutable: executableExpectation(executable), fsApi });
    assert.equal(first.executable.readinessMatch, true);
    assert.deepEqual(first.mappings[0], {
      device: "08:01", inode: "41", mappedBytes: 8192,
      rssBytes: 5 * 1024, referencedBytes: 3 * 1024,
      permissions: ["r--p", "r-xp"],
      path: { pathSHA256: createHash("sha256").update(executable).digest("hex"),
        kind: "browser", browserRelativePath: "chrome" },
    });
    assert.equal(JSON.stringify(first).includes("grenadiers-visibility-secret"), false);
    assert(opened.every((file) => file.startsWith(path.join(root, "101"))));
    assert.equal(opened.includes(executable), false);

    const smaps = fs.readFileSync(path.join(root, "101", "smaps"), "utf8")
      .replace("Rss: 2 kB", "Rss: 7 kB").replace("Referenced: 1 kB", "Referenced: 4 kB");
    fs.writeFileSync(path.join(root, "101", "smaps"), smaps);
    const second = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      elapsedMs: FILE_SNAPSHOT_ELAPSED_MS, expectedExecutable: executableExpectation(executable),
      baseline: first });
    assert.deepEqual(second.mappings[0].deltaFromFirst,
      { mappedBytes: 0, rssBytes: 5 * 1024, referencedBytes: 3 * 1024 });
    const mismatch = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      expectedExecutable: { ...executableExpectation(executable), inode: "999" } });
    assert.equal(mismatch.executable.readinessMatch, false);
    assert.equal(mismatch.executable.identityMatchesReadiness, false);
    assert.equal(mismatch.executable.pathMatchesReadiness, true);
    assert.equal(mismatch.mappings[0].path.kind, "private-or-unknown");
    assert.equal("browserRelativePath" in mismatch.mappings[0].path, false);
  });
});

test("makes missing, malformed, truncated, and PID-reused snapshot metadata explicit", () => {
  withProc((root) => {
    const executable = path.join(root, "chrome");
    fs.writeFileSync(executable, "browser-binary");
    writeProcess(root, { pid: 101, ppid: 1, starttime: 11 });
    fs.symlinkSync(executable, path.join(root, "101", "exe"));
    fs.writeFileSync(path.join(root, "101", "maps"),
      `malformed\n${"x".repeat(MAX_MAPPING_BYTES + 1)}`);
    const incomplete = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      expectedExecutable: executableExpectation(executable) });
    assert.equal(incomplete.maps.status, "truncated");
    assert.equal(incomplete.maps.truncated, true);
    assert.equal(incomplete.maps.malformedLines > 0, true);
    assert.equal(incomplete.smaps.status, "missing");

    fs.writeFileSync(path.join(root, "101", "maps"),
      `1000-2000 r--p 00000000 08:03 77 /private/cache.bin\n`);
    const mapsFallback = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      expectedExecutable: executableExpectation(executable) });
    assert.equal(mapsFallback.mappingSource, "maps");
    assert.equal(mapsFallback.aggregation.source, "maps");
    assert.equal(mapsFallback.aggregation.groupLimitReached, false);
    assert.equal(mapsFallback.mappings[0].rssBytes, null);
    assert.equal(mapsFallback.mappings[0].referencedBytes, null);

    const manyMapGroups = Array.from({ length: MAX_MAPPING_GROUPS + 2 }, (_, index) =>
      `${(0x1000 + index * 0x1000).toString(16)}-${(0x2000 + index * 0x1000).toString(16)} r--p 00000000 08:04 ${2000 + index} /private/maps-only/${index}`);
    fs.writeFileSync(path.join(root, "101", "maps"), `${manyMapGroups.join("\n")}\n`);
    const mapsOnlyCapped = captureStartupFileSnapshot({ procRoot: root, pid: 101,
      starttime: 11, expectedExecutable: executableExpectation(executable) });
    assert.equal(mapsOnlyCapped.smaps.status, "missing");
    assert.equal(mapsOnlyCapped.aggregation.source, "maps");
    assert.equal(mapsOnlyCapped.aggregation.groupLimitReached, true);
    assert.equal(mapsOnlyCapped.mappings.length, MAX_MAPPING_GROUPS);

    const manyMappings = Array.from({ length: MAX_MAPPING_ENTRIES + 2 }, (_, index) =>
      `${(0x1000 + index * 0x1000).toString(16)}-${(0x2000 + index * 0x1000).toString(16)} r--p 00000000 08:03 ${1000 + index} /private/${index}`);
    fs.writeFileSync(path.join(root, "101", "maps"), `${manyMappings.join("\n")}\n`);
    fs.writeFileSync(path.join(root, "101", "smaps"),
      `${manyMappings.map((line) => `${line}\nRss: 1 kB\nReferenced: 1 kB`).join("\n")}\n`);
    const capped = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      expectedExecutable: executableExpectation(executable) });
    assert.equal(capped.maps.entryLimitReached, true);
    assert.equal(capped.aggregation.source, "smaps");
    assert.equal(capped.aggregation.entryLimitReached, true);
    assert.equal(capped.aggregation.groupLimitReached, true);
    assert.equal(capped.mappings.length, MAX_MAPPING_GROUPS);

    const replacement = path.join(root, "replacement-chrome");
    fs.writeFileSync(replacement, "replacement-browser-binary");
    let executableReads = 0;
    const execRaceFsApi = Object.create(fs);
    execRaceFsApi.readlinkSync = (...args) => {
      executableReads += 1;
      if (executableReads === 2) {
        fs.unlinkSync(path.join(root, "101", "exe"));
        fs.symlinkSync(replacement, path.join(root, "101", "exe"));
      }
      return fs.readlinkSync(...args);
    };
    const execRace = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      expectedExecutable: executableExpectation(executable), fsApi: execRaceFsApi });
    assert.equal(execRace.gap, "executable-changed-during-snapshot");
    assert.equal(execRace.executableBefore.canonicalPathSHA256,
      createHash("sha256").update(executable).digest("hex"));
    assert.equal(execRace.executableAfter.canonicalPathSHA256,
      createHash("sha256").update(replacement).digest("hex"));
    assert.equal("mappings" in execRace, false);

    const fsApi = Object.create(fs);
    fsApi.readlinkSync = (...args) => {
      const result = fs.readlinkSync(...args);
      fs.writeFileSync(path.join(root, "101", "stat"),
        statLine({ pid: 101, ppid: 1, starttime: 999 }));
      return result;
    };
    const reused = captureStartupFileSnapshot({ procRoot: root, pid: 101, starttime: 11,
      expectedExecutable: executableExpectation(executable), fsApi });
    assert.equal(reused.gap, "process-identity-changed-after-snapshot");
    assert.equal("mappings" in reused, false);
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
    assert.deepEqual(sample.processes[1].io,
      { readCalls: 7, writeCalls: 8, readBytes: 9, writeBytes: 10 });
    assert.deepEqual(sample.processes[1].memoryKb,
      { residentKb: 11, anonymousKb: 12, fileKb: 13, sharedKb: 14, swapKb: 15 });
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

test("takes at most two file snapshots on existing sample ticks for the same unique browser", () => {
  withProc((root) => {
    const executable = path.join(root, "chrome");
    fs.writeFileSync(executable, "browser-binary");
    writeProcess(root, { pid: 100, ppid: 1, starttime: 10 });
    const profile = path.join(root, "grenadiers-visibility-snapshot-profile");
    fs.mkdirSync(profile);
    writeProcess(root, { pid: 101, ppid: 100, starttime: 11 }, ["chromium",
      "--no-sandbox", "--disable-dev-shm-usage", "--no-first-run",
      "--no-default-browser-check", "--enable-gpu", "--use-gl=angle",
      "--use-angle=swiftshader", "--remote-debugging-port=0",
      `--user-data-dir=${profile}`, "about:blank"]);
    writeFileMappings(root, 101, executable);
    const state = { count: 0, browserIdentity: null, first: null };
    const options = { procRoot: root, rootPid: 100, rootStarttime: 10, tempRoot: root,
      expectedExecutable: executableExpectation(executable), fileSnapshotState: state };
    const first = collectSample({ ...options, elapsedMs: 0 });
    assert.equal(first.processes[1].startupFileSnapshot.ordinal, 1);
    const early = collectSample({ ...options, elapsedMs: FILE_SNAPSHOT_ELAPSED_MS - 1 });
    assert.equal("startupFileSnapshot" in early.processes[1], false);
    const later = collectSample({ ...options, elapsedMs: FILE_SNAPSHOT_ELAPSED_MS });
    assert.equal(later.processes[1].startupFileSnapshot.ordinal, 2);
    const after = collectSample({ ...options, elapsedMs: FILE_SNAPSHOT_ELAPSED_MS + 1000 });
    assert.equal("startupFileSnapshot" in after.processes[1], false);
    assert.equal(state.count, 2);
  });
});

test("marks unavailable process and host counters as gaps instead of zero", () => {
  withProc((root) => {
    writeProcess(root, { pid: 100, ppid: 1, starttime: 10 });
    fs.rmSync(path.join(root, "100", "io"));
    fs.rmSync(path.join(root, "100", "status"));
    const sample = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10 });
    assert.deepEqual(sample.processes[0].io, {});
    assert.deepEqual(sample.processes[0].memoryKb, {});
    assert(sample.processes[0].missingFields.includes("io:missing"));
    assert(sample.processes[0].missingFields.includes("memoryKb:missing"));
    assert.equal(sample.host.memAvailableKb, null);
    assert(sample.host.missingFields.includes("memAvailableKb:missing"));
    assert(sample.host.missingFields.includes("pressure.io:missing"));
    assert(sample.host.missingFields.includes("pressure.memory:missing"));
  });
});

test("keeps partial second-sample counters raw and never fabricates missing zeros", () => {
  withProc((root) => {
    writeProcess(root, { pid: 100, ppid: 1, starttime: 10,
      minorFaults: 20, majorFaults: 2 });
    const first = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10 });
    fs.writeFileSync(path.join(root, "100", "stat"), statLine({
      pid: 100, ppid: 1, starttime: 10, minorFaults: 23, majorFaults: 3,
    }));
    fs.writeFileSync(path.join(root, "100", "io"), "syscr: 9\n");
    const second = collectSample({ procRoot: root, rootPid: 100, rootStarttime: 10,
      previousIdentities: first.identities });
    assert.equal(second.processes[0].minorFaults, 23);
    assert.equal(second.processes[0].majorFaults, 3);
    assert.deepEqual(second.processes[0].io, { readCalls: 9 });
    assert(second.processes[0].missingFields.includes("io.writeCalls:missing-or-invalid"));
    assert(second.processes[0].missingFields.includes("io.readBytes:missing-or-invalid"));
    assert(second.processes[0].missingFields.includes("io.writeBytes:missing-or-invalid"));
    assert.equal(JSON.stringify(second).includes('"readBytes":0'), false);
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
    assert.equal(lines.at(-1).startupFileSnapshots, 0);
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
  assert.match(workflow, /name: Check visibility startup diagnostics contracts\n\s+if: matrix\.shard == 'classic'/);
  assert.match(workflow, /xvfb-run -a npm --prefix e2e run test:audio/);
  assert.match(workflow, /\$\{\{ runner\.temp \}\}\/visibility-startup-observer\.jsonl/);
  assert.match(workflow, /\$\{\{ runner\.temp \}\}\/visibility-xvfb\.log/);
  assert.match(workflow, /matrix:\n\s+shard: \[items, classic\]/);
  assert.match(workflow, /timeout-minutes: 20/);
});
