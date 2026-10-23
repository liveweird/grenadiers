import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

export const SAMPLE_INTERVAL_MS = 250;
export const MAX_SAMPLES = 400;
export const MAX_PROCESSES = 64;
const MAX_SCAN_ENTRIES = 4096;
const VISIBILITY_ARGS = [
  "--no-sandbox", "--disable-dev-shm-usage", "--no-first-run",
  "--no-default-browser-check", "--enable-gpu", "--use-gl=angle",
  "--use-angle=swiftshader", "--remote-debugging-port=0",
];

function safeInteger(value) {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed >= 0 ? parsed : null;
}

export function parseProcStat(text) {
  const close = text.lastIndexOf(") ");
  if (close < 1) return null;
  const pid = safeInteger(text.slice(0, text.indexOf(" ")));
  const fields = text.slice(close + 2).trim().split(/\s+/);
  if (pid === null || fields.length < 22) return null;
  const values = {
    pid, state: fields[0], ppid: safeInteger(fields[1]), pgid: safeInteger(fields[2]),
    minorFaults: safeInteger(fields[7]), majorFaults: safeInteger(fields[9]),
    userTicks: safeInteger(fields[11]), systemTicks: safeInteger(fields[12]),
    threads: safeInteger(fields[17]), starttime: safeInteger(fields[19]),
    rssPages: safeInteger(fields[21]),
  };
  if (Object.values(values).some((value) => value === null)) return null;
  return values;
}

function parseNamedIntegers(text, names, suffix = "") {
  const values = {};
  for (const line of text.trim().split("\n")) {
    const match = line.match(/^([A-Za-z_]+):\s*(\d+)(?:\s+([A-Za-z]+))?$/);
    if (!match || !names.has(match[1]) || (suffix && match[3] !== suffix)) continue;
    const value = safeInteger(match[2]);
    if (value !== null) values[names.get(match[1])] = value;
  }
  return values;
}

export function parseProcIo(text) {
  return parseNamedIntegers(text, new Map([
    ["syscr", "readCalls"], ["syscw", "writeCalls"],
    ["read_bytes", "readBytes"], ["write_bytes", "writeBytes"],
  ]));
}

export function parseProcStatusMemory(text) {
  return parseNamedIntegers(text, new Map([
    ["VmRSS", "residentKb"], ["RssAnon", "anonymousKb"],
    ["RssFile", "fileKb"], ["RssShmem", "sharedKb"], ["VmSwap", "swapKb"],
  ]), "kB");
}

export function parsePressure(text) {
  const values = {};
  for (const line of text.trim().split("\n")) {
    const match = line.match(/^(some|full)\s+.*\btotal=(\d+)(?:\s|$)/);
    const total = match ? safeInteger(match[2]) : null;
    if (match && total !== null) values[`${match[1]}TotalUs`] = total;
  }
  return values;
}

export function parseMemAvailable(text) {
  const match = text.match(/^MemAvailable:\s*(\d+)\s+kB$/m);
  return match ? safeInteger(match[1]) : null;
}

function boundedRead(file, limit) {
  const fd = fs.openSync(file, "r");
  try {
    const buffer = Buffer.alloc(limit + 1);
    const size = fs.readSync(fd, buffer, 0, buffer.length, 0);
    if (size > limit) return { value: null, gap: "oversize" };
    return { value: buffer.subarray(0, size), gap: null };
  } finally {
    fs.closeSync(fd);
  }
}

function readOptional(file, limit) {
  try {
    return boundedRead(file, limit);
  } catch (error) {
    return { value: null, gap: error?.code === "ENOENT" ? "missing" : "unreadable" };
  }
}

function parsedOptional(file, limit, parser, requiredKeys, label, missingFields) {
  const read = readOptional(file, limit);
  if (read.value === null) {
    missingFields.push(`${label}:${read.gap}`);
    return {};
  }
  const parsed = parser(read.value.toString("utf8"));
  for (const key of requiredKeys) {
    if (!(key in parsed)) missingFields.push(`${label}.${key}:missing-or-invalid`);
  }
  return parsed;
}

function hostMetrics(procRoot) {
  const missingFields = [];
  const meminfo = readOptional(path.join(procRoot, "meminfo"), 16384);
  let memAvailableKb = null;
  if (meminfo.value === null) missingFields.push(`memAvailableKb:${meminfo.gap}`);
  else {
    memAvailableKb = parseMemAvailable(meminfo.value.toString("utf8"));
    if (memAvailableKb === null) missingFields.push("memAvailableKb:missing-or-invalid");
  }
  const pressure = {};
  for (const resource of ["io", "memory"]) {
    pressure[resource] = parsedOptional(path.join(procRoot, "pressure", resource), 4096,
      parsePressure, ["someTotalUs", "fullTotalUs"], `pressure.${resource}`, missingFields);
  }
  return { memAvailableKb, pressure, missingFields };
}

function stderrKind(procDirectory) {
  try {
    const stat = fs.statSync(path.join(procDirectory, "fd", "2"));
    if (stat.isFIFO()) return "pipe";
    if (stat.isSocket()) return "socket";
    if (stat.isCharacterDevice()) return "character-device";
    if (stat.isFile()) return "regular-file";
    if (stat.isDirectory()) return "directory";
    return "other";
  } catch (error) {
    return error?.code === "ENOENT" ? "missing" : "unreadable";
  }
}

function devtoolsState(profile) {
  try {
    const stat = fs.lstatSync(path.join(profile, "DevToolsActivePort"));
    const type = stat.isFile() ? "regular-file" : stat.isDirectory() ? "directory"
      : stat.isSymbolicLink() ? "symlink" : "other";
    return { exists: true, type, size: Number.isSafeInteger(stat.size) ? stat.size : null };
  } catch (error) {
    if (error?.code === "ENOENT") return { exists: false, type: "missing", size: null };
    return { exists: null, type: "unreadable", size: null };
  }
}

function visibilityProfile(argv, tempRoot) {
  if (argv.length !== VISIBILITY_ARGS.length + 3) return null;
  if (!/^(chrome|chromium|chromium-browser)$/.test(path.basename(argv[0]))) return null;
  if (!VISIBILITY_ARGS.every((value, index) => argv[index + 1] === value)) return null;
  const profileArgument = argv[VISIBILITY_ARGS.length + 1];
  if (!profileArgument.startsWith("--user-data-dir=")) return null;
  const profile = profileArgument.slice("--user-data-dir=".length);
  if (path.dirname(profile) !== path.resolve(tempRoot)
    || !/^grenadiers-visibility-[A-Za-z0-9_-]{6,}$/.test(path.basename(profile))) return null;
  if (argv.at(-1) !== "about:blank") return null;
  return profile;
}

function processIdentity(process) {
  return `${process.pid}:${process.starttime}`;
}

function isOwned(process, processes, rootIdentity) {
  let current = process;
  const seen = new Set();
  for (let depth = 0; depth < 64; depth += 1) {
    const identity = processIdentity(current);
    if (identity === rootIdentity) return true;
    if (seen.has(identity) || current.ppid === 0) return false;
    seen.add(identity);
    current = processes.get(current.ppid);
    if (!current) return false;
  }
  return false;
}

function withObserverState(result, profiles) {
  Object.defineProperty(result, "observedProfiles", { value: profiles });
  return result;
}

export function collectSample({ procRoot = "/proc", rootPid, rootStarttime,
  previousIdentities = new Set(), elapsedMs = 0, tempRoot = os.tmpdir(),
  knownProfiles = new Set(), beforeMetadata } = {}) {
  const gaps = [];
  let names;
  try {
    names = fs.readdirSync(procRoot).filter((name) => /^\d+$/.test(name));
  } catch {
    const host = hostMetrics(procRoot);
    return withObserverState({ elapsedMs, processes: [], host,
      gaps: [{ kind: "proc-unavailable" }], scanTruncated: false,
      processTruncated: false, identities: new Set() }, knownProfiles);
  }
  const host = hostMetrics(procRoot);
  const scanTruncated = names.length > MAX_SCAN_ENTRIES;
  names = names.slice(0, MAX_SCAN_ENTRIES);
  const processes = new Map();
  let unreadableStatCount = 0;
  for (const name of names) {
    const stat = readOptional(path.join(procRoot, name, "stat"), 4096);
    if (stat.value === null) { unreadableStatCount += 1; continue; }
    const parsed = parseProcStat(stat.value.toString("utf8"));
    if (parsed) processes.set(parsed.pid, parsed);
    else unreadableStatCount += 1;
  }
  const rootIdentity = `${rootPid}:${rootStarttime}`;
  const root = processes.get(rootPid);
  if (!root || processIdentity(root) !== rootIdentity) {
    gaps.push({ kind: root ? "root-pid-reused" : "root-disappeared",
      pid: rootPid, starttime: rootStarttime });
    return withObserverState({ elapsedMs, processes: [], host,
      gaps, unreadableStatCount, scanTruncated, processTruncated: false,
      identities: new Set() }, knownProfiles);
  }
  const owned = [...processes.values()].filter((item) => isOwned(item, processes, rootIdentity))
    .sort((left, right) => left.pid - right.pid);
  const processTruncated = owned.length > MAX_PROCESSES;
  const selected = owned.slice(0, MAX_PROCESSES);
  const candidates = [];
  const records = [];
  for (const item of selected) {
    const directory = path.join(procRoot, String(item.pid));
    const expectedIdentity = processIdentity(item);
    const identityBeforeRead = readOptional(path.join(directory, "stat"), 4096);
    const before = identityBeforeRead.value === null ? null
      : parseProcStat(identityBeforeRead.value.toString("utf8"));
    if (!before || processIdentity(before) !== expectedIdentity) {
      gaps.push({ kind: "owned-process-identity-changed", pid: item.pid,
        starttime: item.starttime, phase: "before-metadata" });
      continue;
    }
    beforeMetadata?.(item.pid);
    const missingFields = [];
    const wchanRead = readOptional(path.join(directory, "wchan"), 64);
    let wchan = null;
    if (wchanRead.value === null) missingFields.push(`wchan:${wchanRead.gap}`);
    else {
      const candidate = wchanRead.value.toString("utf8").trim();
      wchan = /^[A-Za-z0-9_.+-]{0,63}$/.test(candidate) ? candidate : "redacted";
    }
    const cmdlineRead = readOptional(path.join(directory, "cmdline"), 16384);
    let profile = null;
    if (cmdlineRead.value === null) missingFields.push(`argv:${cmdlineRead.gap}`);
    else {
      const argv = cmdlineRead.value.toString("utf8").split("\0").filter(Boolean);
      profile = visibilityProfile(argv, tempRoot);
    }
    const io = parsedOptional(path.join(directory, "io"), 4096, parseProcIo,
      ["readCalls", "writeCalls", "readBytes", "writeBytes"], "io", missingFields);
    const memoryKb = parsedOptional(path.join(directory, "status"), 16384,
      parseProcStatusMemory,
      ["residentKb", "anonymousKb", "fileKb", "sharedKb", "swapKb"],
      "memoryKb", missingFields);
    const devToolsActivePort = profile ? devtoolsState(profile) : null;
    const record = {
      pid: item.pid, ppid: item.ppid, pgid: item.pgid, starttime: item.starttime,
      state: item.state, cpuTicks: item.userTicks + item.systemTicks,
      minorFaults: item.minorFaults, majorFaults: item.majorFaults,
      threads: item.threads, rssPages: item.rssPages, io, memoryKb, wchan,
      stderrFdKind: stderrKind(directory), visibilityChromium: false,
      missingFields,
    };
    const identityAfterRead = readOptional(path.join(directory, "stat"), 4096);
    const after = identityAfterRead.value === null ? null
      : parseProcStat(identityAfterRead.value.toString("utf8"));
    if (!after || processIdentity(after) !== expectedIdentity) {
      gaps.push({ kind: "owned-process-identity-changed", pid: item.pid,
        starttime: item.starttime, phase: "after-metadata" });
      continue;
    }
    records.push(record);
    if (profile) candidates.push({ pid: item.pid, profile, devToolsActivePort });
  }
  const identities = new Set(records.map(processIdentity));
  for (const identity of previousIdentities) {
    if (!identities.has(identity)) {
      const [pid, starttime] = identity.split(":").map(Number);
      gaps.push({ kind: "owned-process-disappeared", pid, starttime });
    }
  }
  const uniqueProfiles = new Set(candidates.map(({ profile }) => profile));
  const observedProfiles = new Set([...knownProfiles, ...uniqueProfiles]);
  for (const record of records) {
    const candidate = candidates.find(({ pid }) => pid === record.pid);
    if (candidate) {
      record.visibilityArgvContract = "exact";
      record.visibilityProfileUnique = candidates.length === 1 && observedProfiles.size === 1;
      record.visibilityChromium = record.visibilityProfileUnique;
      record.devToolsActivePort = candidate.devToolsActivePort;
    }
  }
  return withObserverState({ elapsedMs, processes: records, host,
    gaps, unreadableStatCount, scanTruncated, processTruncated, identities }, observedProfiles);
}

function rootStarttime(procRoot, pid) {
  const read = readOptional(path.join(procRoot, String(pid), "stat"), 4096);
  return read.value === null ? null : parseProcStat(read.value.toString("utf8"))?.starttime ?? null;
}

export function runObserved(command, args, { procRoot = "/proc", output } = {}) {
  if (!command || !output) throw new Error("observer requires a command and output path");
  const fd = fs.openSync(output, "wx", 0o600);
  const write = (record) => fs.writeSync(fd, `${JSON.stringify(record)}\n`);
  const started = process.hrtime.bigint();
  const isolatedGroup = process.platform !== "win32";
  const child = spawn(command, args, { env: process.env, stdio: "inherit",
    detached: isolatedGroup });
  let samples = 0;
  let previousIdentities = new Set();
  let knownProfiles = new Set();
  let finished = false;
  let forwardedSignal = null;
  const forwardingHandlers = new Map();
  const removeForwardingHandlers = () => {
    for (const [signal, handler] of forwardingHandlers) process.off(signal, handler);
    forwardingHandlers.clear();
  };
  const forward = (signal) => {
    if (finished || forwardedSignal !== null || !child.pid) return;
    forwardedSignal = signal;
    write({ kind: "signal-forwarded", signal });
    try {
      if (isolatedGroup) process.kill(-child.pid, signal);
      else child.kill(signal);
    } catch (error) {
      if (error?.code !== "ESRCH") throw error;
    }
  };
  for (const signal of ["SIGHUP", "SIGINT", "SIGTERM"]) {
    const handler = () => forward(signal);
    forwardingHandlers.set(signal, handler);
    process.on(signal, handler);
  }
  const starttime = child.pid ? rootStarttime(procRoot, child.pid) : null;
  write({ schemaVersion: 1, kind: "visibility-startup-process-observation",
    sampleIntervalMs: SAMPLE_INTERVAL_MS, maxSamples: MAX_SAMPLES,
    maxProcessesPerSample: MAX_PROCESSES, childPid: child.pid ?? null,
    childStarttime: starttime, commandArgumentCount: args.length,
    atomicTimeoutWitness: false, interpretation: "sampled-non-atomic-diagnostics-only",
    startupGaps: starttime === null ? ["child-starttime-unavailable"] : [] });
  const sample = () => {
    if (finished || samples >= MAX_SAMPLES || !child.pid || starttime === null) return;
    const elapsedMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
    const result = collectSample({ procRoot, rootPid: child.pid, rootStarttime: starttime,
      previousIdentities, elapsedMs, knownProfiles });
    previousIdentities = result.identities;
    knownProfiles = result.observedProfiles;
    delete result.identities;
    write({ kind: "sample", ...result });
    samples += 1;
  };
  sample();
  const interval = setInterval(sample, SAMPLE_INTERVAL_MS);
  interval.unref();
  const finish = (code, signal, spawnError = null) => {
    if (finished) return;
    finished = true;
    clearInterval(interval);
    removeForwardingHandlers();
    write({ kind: "complete", samples, sampleLimitReached: samples >= MAX_SAMPLES,
      visibilityProfilesObserved: knownProfiles.size,
      visibilityProfileUnique: knownProfiles.size === 1,
      exitCode: code, signal, forwardedSignal, spawnError });
    fs.closeSync(fd);
    const terminationSignal = forwardedSignal ?? signal;
    if (terminationSignal) process.kill(process.pid, terminationSignal);
    else process.exitCode = code ?? (spawnError === "ENOENT" ? 127 : 126);
  };
  child.once("error", (error) => finish(null, null, error?.code ?? "spawn-error"));
  child.once("exit", (code, signal) => finish(code, signal));
  return child;
}

function main() {
  const separator = process.argv.indexOf("--");
  if (separator < 0 || separator === process.argv.length - 1) {
    throw new Error("usage: ci-visibility-startup-observer.mjs -- command [args...]");
  }
  const runnerTemp = process.env.RUNNER_TEMP;
  if (!runnerTemp) throw new Error("RUNNER_TEMP is required");
  const [command, ...args] = process.argv.slice(separator + 1);
  runObserved(command, args, { output: path.join(runnerTemp, "visibility-startup-observer.jsonl") });
}

if (import.meta.url === `file://${process.argv[1]}`) main();
