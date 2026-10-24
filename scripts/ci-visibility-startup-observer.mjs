import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

export const SAMPLE_INTERVAL_MS = 250;
export const MAX_SAMPLES = 400;
export const MAX_PROCESSES = 64;
export const FILE_SNAPSHOT_ELAPSED_MS = 8000;
export const MAX_MAPPING_BYTES = 4 * 1024 * 1024;
export const MAX_MAPPING_ENTRIES = 4096;
export const MAX_MAPPING_GROUPS = 512;
const MAX_SCAN_ENTRIES = 4096;
const VISIBILITY_ARGS = [
  "--no-sandbox", "--disable-dev-shm-usage", "--no-first-run",
  "--no-default-browser-check", "--enable-gpu", "--use-gl=angle",
  "--use-angle=swiftshader", "--remote-debugging-port=0",
];

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function diagnosticIdentity(stat) {
  const size = Number(stat.size);
  if (!Number.isSafeInteger(size) || size < 0) return null;
  return { device: String(stat.dev), inode: String(stat.ino), size };
}

function sameIdentity(left, right) {
  return left && right && left.device === right.device && left.inode === right.inode
    && left.size === right.size;
}

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

function readMetadataPrefix(file, limit, fsApi) {
  let descriptor;
  let bytesRead = 0;
  try {
    descriptor = fsApi.openSync(file, "r");
    const buffer = Buffer.alloc(limit + 1);
    let eof = false;
    while (bytesRead < buffer.length) {
      const bytes = fsApi.readSync(descriptor, buffer, bytesRead,
        buffer.length - bytesRead, bytesRead);
      if (!Number.isSafeInteger(bytes) || bytes < 0
        || bytes > buffer.length - bytesRead)
        throw new Error("invalid metadata read length");
      if (bytes === 0) { eof = true; break; }
      bytesRead += bytes;
    }
    const truncated = bytesRead > limit;
    return { text: buffer.subarray(0, Math.min(bytesRead, limit)).toString("utf8"),
      bytesRead, truncated, status: truncated ? "truncated" : eof ? "complete" : "unreadable" };
  } catch (error) {
    return { text: "", bytesRead, truncated: false,
      status: error?.code === "ENOENT" ? "missing" : "unreadable" };
  } finally {
    if (descriptor !== undefined) fsApi.closeSync(descriptor);
  }
}

function parseMappingHeader(line) {
  const match = line.match(/^([0-9a-f]+)-([0-9a-f]+)\s+(\S{4})\s+[0-9a-f]+\s+(\S+)\s+(\d+)(?:\s+(.*))?$/i);
  if (!match) return null;
  const start = Number.parseInt(match[1], 16);
  const end = Number.parseInt(match[2], 16);
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || end < start
    || !/^\d+$/.test(match[5])) return null;
  return { mappedBytes: end - start, permissions: match[3], device: match[4],
    inode: BigInt(match[5]).toString(), pathname: (match[6] ?? "").trim() };
}

function safeMappingPath(pathname, executablePath, browserRoot) {
  const normalized = pathname.endsWith(" (deleted)")
    ? pathname.slice(0, -" (deleted)".length) : pathname;
  const result = { pathSHA256: sha256(normalized), kind: "private-or-unknown" };
  if (executablePath && browserRoot && path.resolve(normalized) === normalized) {
    const relative = path.relative(browserRoot, normalized);
    if (relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative))
      return { ...result, kind: "browser", browserRelativePath: relative
        .split(path.sep).join("/") };
  }
  for (const prefix of ["/usr/", "/lib/", "/lib64/"]) {
    if (normalized.startsWith(prefix) && path.resolve(normalized) === normalized)
      return { ...result, kind: "system", systemRelativePath: normalized.slice(1) };
  }
  return result;
}

function summarizeMaps(read) {
  let entriesSeen = 0;
  let fileBackedEntries = 0;
  let malformedLines = 0;
  for (const line of read.text.split("\n")) {
    if (!line) continue;
    if (entriesSeen >= MAX_MAPPING_ENTRIES) break;
    entriesSeen += 1;
    const header = parseMappingHeader(line);
    if (!header) { malformedLines += 1; continue; }
    if (header.inode !== "0" && header.pathname && !header.pathname.startsWith("["))
      fileBackedEntries += 1;
  }
  return { status: read.status, bytesRead: read.bytesRead, truncated: read.truncated,
    entriesSeen, fileBackedEntries, malformedLines,
    entryLimitReached: entriesSeen >= MAX_MAPPING_ENTRIES && read.text.split("\n").length - 1 > entriesSeen };
}

function aggregateSmaps(read, executablePath, browserRoot) {
  const groups = new Map();
  let current = null;
  let entriesSeen = 0;
  let fileBackedEntries = 0;
  let malformedHeaders = 0;
  let entryLimitReached = false;
  let groupLimitReached = false;
  const commit = () => {
    if (!current || current.inode === "0" || !current.pathname
      || current.pathname.startsWith("[")) return;
    fileBackedEntries += 1;
    const key = `${current.device}:${current.inode}`;
    let group = groups.get(key);
    if (!group) {
      if (groups.size >= MAX_MAPPING_GROUPS) { groupLimitReached = true; return; }
      group = { device: current.device, inode: current.inode, mappedBytes: 0,
        rssBytes: 0, referencedBytes: 0, permissions: new Set(),
        rssComplete: true, referencedComplete: true,
        path: safeMappingPath(current.pathname, executablePath, browserRoot) };
      groups.set(key, group);
    }
    group.mappedBytes += current.mappedBytes;
    if (current.rssBytes === null) group.rssComplete = false;
    else group.rssBytes += current.rssBytes;
    if (current.referencedBytes === null) group.referencedComplete = false;
    else group.referencedBytes += current.referencedBytes;
    group.permissions.add(current.permissions);
  };
  for (const line of read.text.split("\n")) {
    const header = parseMappingHeader(line);
    if (header) {
      commit();
      if (entriesSeen >= MAX_MAPPING_ENTRIES) { entryLimitReached = true; current = null; break; }
      entriesSeen += 1;
      current = { ...header, rssBytes: null, referencedBytes: null };
      continue;
    }
    if (/^[0-9a-f]+-[0-9a-f]+\s/i.test(line)) { malformedHeaders += 1; current = null; continue; }
    if (!current) continue;
    const metric = line.match(/^(Rss|Referenced):\s+(\d+)\s+kB$/);
    if (!metric) continue;
    const value = safeInteger(metric[2]);
    const bytes = value === null ? null : value * 1024;
    if (Number.isSafeInteger(bytes))
      current[metric[1] === "Rss" ? "rssBytes" : "referencedBytes"] = bytes;
  }
  commit();
  const mappings = [...groups.values()].map(({ rssComplete, referencedComplete, ...group }) => ({
    ...group, rssBytes: rssComplete ? group.rssBytes : null,
    referencedBytes: referencedComplete ? group.referencedBytes : null,
    permissions: [...group.permissions].sort() }));
  return { summary: { status: read.status, bytesRead: read.bytesRead,
    truncated: read.truncated, entriesSeen, fileBackedEntries, malformedHeaders,
    entryLimitReached, groupLimitReached }, mappings };
}

function addSnapshotDeltas(mappings, baseline) {
  const first = new Map((baseline?.mappings ?? []).map((item) =>
    [`${item.device}:${item.inode}`, item]));
  for (const item of mappings) {
    const prior = first.get(`${item.device}:${item.inode}`);
    item.deltaFromFirst = prior ? {
      mappedBytes: item.mappedBytes - prior.mappedBytes,
      rssBytes: item.rssBytes === null || prior.rssBytes === null
        ? null : item.rssBytes - prior.rssBytes,
      referencedBytes: item.referencedBytes === null || prior.referencedBytes === null
        ? null : item.referencedBytes - prior.referencedBytes,
    } : null;
  }
}

function executableMetadata(directory, fsApi) {
  try {
    const executablePath = fsApi.readlinkSync(path.join(directory, "exe"));
    const identity = diagnosticIdentity(fsApi.statSync(path.join(directory, "exe"), { bigint: true }));
    if (!identity) return { executablePath: null, identity: null, gap: "invalid-metadata" };
    return { executablePath, identity, canonicalPathSHA256: sha256(executablePath), gap: null };
  } catch (error) {
    return { executablePath: null, identity: null, canonicalPathSHA256: null,
      gap: error?.code === "ENOENT" ? "missing" : "unreadable" };
  }
}

function sameExecutableMetadata(left, right) {
  if (left.gap || right.gap) return left.gap === right.gap;
  return left.canonicalPathSHA256 === right.canonicalPathSHA256
    && sameIdentity(left.identity, right.identity);
}

function safeExecutableMetadata(metadata) {
  return metadata.gap ? { gap: metadata.gap }
    : { canonicalPathSHA256: metadata.canonicalPathSHA256, ...metadata.identity };
}

export function captureStartupFileSnapshot({ procRoot = "/proc", pid, starttime,
  elapsedMs = 0, expectedExecutable = null, baseline = null, fsApi = fs } = {}) {
  const directory = path.join(procRoot, String(pid));
  const statBefore = readMetadataPrefix(path.join(directory, "stat"), 4096, fsApi);
  const before = parseProcStat(statBefore.text);
  if (!before || before.starttime !== starttime || before.pid !== pid)
    return { gap: "process-identity-changed-before-snapshot", pid, starttime };
  const executableBefore = executableMetadata(directory, fsApi);
  const executable = executableBefore.gap === null ? {
    canonicalPathSHA256: executableBefore.canonicalPathSHA256, ...executableBefore.identity,
    identityMatchesReadiness: sameIdentity(executableBefore.identity, expectedExecutable),
    pathMatchesReadiness:
      executableBefore.canonicalPathSHA256 === expectedExecutable?.canonicalPathSHA256,
    readinessMatch: sameIdentity(executableBefore.identity, expectedExecutable)
      && executableBefore.canonicalPathSHA256 === expectedExecutable?.canonicalPathSHA256,
  } : { gap: executableBefore.gap };
  const browserRoot = executable.readinessMatch ? path.dirname(executableBefore.executablePath) : null;
  const mapsRead = readMetadataPrefix(path.join(directory, "maps"), MAX_MAPPING_BYTES, fsApi);
  const smapsRead = readMetadataPrefix(path.join(directory, "smaps"), MAX_MAPPING_BYTES, fsApi);
  const executableAfter = executableMetadata(directory, fsApi);
  if (!sameExecutableMetadata(executableBefore, executableAfter))
    return { gap: "executable-changed-during-snapshot", pid, starttime,
      executableBefore: safeExecutableMetadata(executableBefore),
      executableAfter: safeExecutableMetadata(executableAfter) };
  const mappingSource = smapsRead.status === "complete" || smapsRead.status === "truncated"
    ? "smaps" : "maps";
  const aggregated = aggregateSmaps(mappingSource === "smaps" ? smapsRead : mapsRead,
    executableBefore.executablePath, browserRoot);
  if (baseline) addSnapshotDeltas(aggregated.mappings, baseline);
  const statAfter = readMetadataPrefix(path.join(directory, "stat"), 4096, fsApi);
  const after = parseProcStat(statAfter.text);
  if (!after || after.starttime !== starttime || after.pid !== pid)
    return { gap: "process-identity-changed-after-snapshot", pid, starttime };
  return { elapsedMs, pid, starttime, executable,
    interpretation: "sampled-mapping-residency-not-file-io-causality",
    limits: { maxBytesPerProcFile: MAX_MAPPING_BYTES, maxEntries: MAX_MAPPING_ENTRIES,
      maxGroups: MAX_MAPPING_GROUPS }, maps: summarizeMaps(mapsRead),
    smaps: { status: smapsRead.status, bytesRead: smapsRead.bytesRead,
      truncated: smapsRead.truncated },
    aggregation: { source: mappingSource, ...aggregated.summary },
    mappingSource, mappings: aggregated.mappings };
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
  knownProfiles = new Set(), beforeMetadata, expectedExecutable = null,
  fileSnapshotState = null, snapshotFsApi = fs } = {}) {
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
  if (fileSnapshotState && candidates.length === 1 && observedProfiles.size === 1) {
    const candidate = candidates[0];
    const record = records.find(({ pid }) => pid === candidate.pid);
    const identity = record ? processIdentity(record) : null;
    const takeFirst = record && fileSnapshotState.count === 0;
    const takeSecond = record && fileSnapshotState.count === 1
      && elapsedMs >= FILE_SNAPSHOT_ELAPSED_MS
      && fileSnapshotState.browserIdentity === identity;
    if (takeFirst || takeSecond) {
      const snapshot = captureStartupFileSnapshot({ procRoot, pid: record.pid,
        starttime: record.starttime, elapsedMs, expectedExecutable,
        baseline: takeSecond ? fileSnapshotState.first : null, fsApi: snapshotFsApi });
      record.startupFileSnapshot = { ordinal: fileSnapshotState.count + 1, ...snapshot };
      fileSnapshotState.count += 1;
      if (takeFirst && !snapshot.gap) {
        fileSnapshotState.browserIdentity = identity;
        fileSnapshotState.first = snapshot;
      }
    }
  }
  return withObserverState({ elapsedMs, processes: records, host,
    gaps, unreadableStatCount, scanTruncated, processTruncated, identities }, observedProfiles);
}

function rootStarttime(procRoot, pid) {
  const read = readOptional(path.join(procRoot, String(pid), "stat"), 4096);
  return read.value === null ? null : parseProcStat(read.value.toString("utf8"))?.starttime ?? null;
}

function expectedExecutableFromReadiness(readinessPath) {
  if (!readinessPath) return { executable: null, gap: "path-not-configured" };
  const read = readOptional(readinessPath, 4 * 1024 * 1024);
  if (read.value === null) return { executable: null, gap: read.gap };
  try {
    const parsed = JSON.parse(read.value.toString("utf8"));
    const executable = parsed?.executable;
    if (parsed?.result !== "PASS" || !/^[a-f0-9]{64}$/.test(executable?.canonicalPathSHA256)
      || !/^\d+$/.test(executable?.device) || !/^\d+$/.test(executable?.inode)
      || !Number.isSafeInteger(executable?.size) || executable.size < 0
      || !/^[a-f0-9]{64}$/.test(executable?.contentSHA256))
      return { executable: null, gap: "invalid" };
    return { executable, gap: null };
  } catch {
    return { executable: null, gap: "malformed" };
  }
}

export function runObserved(command, args, { procRoot = "/proc", output,
  readinessPath = process.env.RUNNER_TEMP
    ? path.join(process.env.RUNNER_TEMP, "browser-file-readiness.json") : null } = {}) {
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
  const fileSnapshotState = { count: 0, browserIdentity: null, first: null };
  const readiness = expectedExecutableFromReadiness(readinessPath);
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
    startupFileSnapshots: { maximum: 2, laterSnapshotElapsedMs: FILE_SNAPSHOT_ELAPSED_MS,
      backingFileContentsRead: false, readinessExecutableGap: readiness.gap },
    startupGaps: starttime === null ? ["child-starttime-unavailable"] : [] });
  const sample = () => {
    if (finished || samples >= MAX_SAMPLES || !child.pid || starttime === null) return;
    const elapsedMs = Number((process.hrtime.bigint() - started) / 1_000_000n);
    const result = collectSample({ procRoot, rootPid: child.pid, rootStarttime: starttime,
      previousIdentities, elapsedMs, knownProfiles, expectedExecutable: readiness.executable,
      fileSnapshotState });
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
      startupFileSnapshots: fileSnapshotState.count,
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
