import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

export const MAX_FILES = 4096;
export const MAX_ENTRIES = 8192;
export const MAX_BYTES = 2 * 1024 * 1024 * 1024;
export const READ_BUFFER_BYTES = 64 * 1024;

function inside(root, candidate) {
  const relative = path.relative(root, candidate);
  return relative !== "" && relative !== ".." && !relative.startsWith(`..${path.sep}`)
    && !path.isAbsolute(relative);
}

function stableFileIdentity(stat) {
  return [stat.dev, stat.ino, stat.size, stat.mtimeNs, stat.ctimeNs].map(String).join(":");
}

function assertConfined(root, candidate, fsApi) {
  const resolved = fsApi.realpathSync(candidate);
  if (resolved !== root && !inside(root, resolved))
    throw new Error("browser distribution path escaped its root");
}

function readStableFile(file, expected, buffer, fsApi) {
  const flags = fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW ?? 0);
  const descriptor = fsApi.openSync(file, flags);
  const digest = createHash("sha256");
  let bytes = 0;
  try {
    const opened = fsApi.fstatSync(descriptor, { bigint: true });
    if (!opened.isFile() || stableFileIdentity(opened) !== stableFileIdentity(expected))
      throw new Error("browser distribution file changed before reading");
    while (bytes < Number(expected.size)) {
      const wanted = Math.min(buffer.length, Number(expected.size) - bytes);
      const count = fsApi.readSync(descriptor, buffer, 0, wanted, bytes);
      if (count === 0) throw new Error("browser distribution file was truncated while reading");
      digest.update(buffer.subarray(0, count));
      bytes += count;
    }
    if (fsApi.readSync(descriptor, buffer, 0, 1, bytes) !== 0)
      throw new Error("browser distribution file grew while reading");
    const completed = fsApi.fstatSync(descriptor, { bigint: true });
    if (stableFileIdentity(completed) !== stableFileIdentity(opened))
      throw new Error("browser distribution file changed while reading");
  } finally {
    fsApi.closeSync(descriptor);
  }
  return { bytes, sha256: digest.digest("hex") };
}

export function warmBrowserFiles({ executablePath, outputPath, fsApi = fs,
  maxFiles = MAX_FILES, maxEntries = MAX_ENTRIES, maxBytes = MAX_BYTES } = {}) {
  if (!executablePath || !outputPath) throw new Error("executablePath and outputPath are required");
  const executable = path.resolve(executablePath);
  const executableEntry = fsApi.lstatSync(executable, { bigint: true });
  if (executableEntry.isSymbolicLink() || !executableEntry.isFile())
    throw new Error("pinned browser executable must be a regular file, not a symlink");
  if (fsApi.realpathSync(executable) !== executable)
    throw new Error("pinned browser executable path must already be canonical");
  const root = path.dirname(executable);
  const rootEntry = fsApi.lstatSync(root, { bigint: true });
  if (rootEntry.isSymbolicLink() || !rootEntry.isDirectory()
    || fsApi.realpathSync(root) !== root)
    throw new Error("pinned browser distribution root must be a canonical directory");

  const pending = [root];
  const files = [];
  let declaredBytes = 0;
  let entryCount = 0;
  let directoryCount = 0;
  while (pending.length > 0) {
    const directory = pending.pop();
    directoryCount += 1;
    assertConfined(root, path.join(directory, "."), fsApi);
    const handle = fsApi.opendirSync(directory);
    try {
      let dirent;
      while ((dirent = handle.readSync()) !== null) {
        entryCount += 1;
        if (entryCount > maxEntries)
          throw new Error("browser distribution entry limit exceeded");
        const candidate = path.join(directory, dirent.name);
        if (!inside(root, candidate))
          throw new Error("browser distribution entry escaped its root");
        const entry = fsApi.lstatSync(candidate, { bigint: true });
        if (entry.isSymbolicLink()) throw new Error("browser distribution contains a symlink");
        assertConfined(root, candidate, fsApi);
        if (entry.isDirectory()) pending.push(candidate);
        else if (entry.isFile()) {
          files.push({ absolute: candidate,
            relative: path.relative(root, candidate).split(path.sep).join("/"), stat: entry });
          declaredBytes += Number(entry.size);
          if (files.length > maxFiles) throw new Error("browser distribution file limit exceeded");
          if (!Number.isSafeInteger(declaredBytes) || declaredBytes > maxBytes)
            throw new Error("browser distribution byte limit exceeded");
        } else throw new Error("browser distribution contains a non-file entry");
      }
    } finally {
      handle.closeSync();
    }
  }
  files.sort((left, right) => left.relative.localeCompare(right.relative));
  const buffer = Buffer.allocUnsafe(READ_BUFFER_BYTES);
  const started = process.hrtime.bigint();
  const inventory = [];
  const aggregate = createHash("sha256");
  let bytesRead = 0;
  for (const file of files) {
    const result = readStableFile(file.absolute, file.stat, buffer, fsApi);
    bytesRead += result.bytes;
    aggregate.update(`${file.relative}\0${result.bytes}\0${result.sha256}\n`);
    inventory.push({ path: file.relative, ...result });
  }
  if (bytesRead !== declaredBytes) throw new Error("browser distribution byte count changed");
  const record = {
    schemaVersion: 1,
    kind: "chromium-distribution-file-readiness",
    result: "PASS",
    interpretation: "bounded-page-cache-pre-read-not-root-cause-proof",
    directoryWalkAssumption: "completed-installation-remains-unchanged-during-walk",
    limits: { maxFiles, maxEntries, maxBytes, readBufferBytes: READ_BUFFER_BYTES },
    entryCount,
    directoryCount,
    fileCount: inventory.length,
    bytesRead,
    inventorySHA256: aggregate.digest("hex"),
    elapsedMs: Number((process.hrtime.bigint() - started) / 1_000_000n),
    files: inventory,
  };
  fsApi.writeFileSync(outputPath, `${JSON.stringify(record, null, 2)}\n`, { flag: "wx", mode: 0o600 });
  return record;
}

function pinnedExecutable() {
  const packagePath = path.resolve("e2e/package.json");
  const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8"));
  const expectedVersion = packageJson.devDependencies?.["@playwright/test"];
  if (!/^\d+\.\d+\.\d+$/.test(expectedVersion ?? ""))
    throw new Error("e2e must pin an exact @playwright/test version");
  const requireFromE2E = createRequire(packagePath);
  const installedPackage = requireFromE2E("@playwright/test/package.json");
  if (installedPackage.version !== expectedVersion)
    throw new Error("installed Playwright version does not match the pinned e2e dependency");
  return requireFromE2E("@playwright/test").chromium.executablePath();
}

function main() {
  if (!process.env.RUNNER_TEMP) throw new Error("RUNNER_TEMP is required");
  const record = warmBrowserFiles({ executablePath: pinnedExecutable(),
    outputPath: path.join(process.env.RUNNER_TEMP, "browser-file-readiness.json") });
  console.log(`BROWSER_FILE_READINESS result=${record.result} files=${record.fileCount} bytes=${record.bytesRead} sha256=${record.inventorySHA256} elapsedMs=${record.elapsedMs}`);
}

if (import.meta.url === `file://${process.argv[1]}`) main();
