import fs from "node:fs";
import path from "node:path";
import { randomUUID } from "node:crypto";
import { pathToFileURL } from "node:url";

export const RUNNER_MIRRORS = [
  "http://azure.archive.ubuntu.com/ubuntu/\tpriority:1",
  "https://archive.ubuntu.com/ubuntu/\tpriority:2",
  "https://security.ubuntu.com/ubuntu/\tpriority:3",
].join("\n") + "\n";

export const ARCHIVE_FIRST_MIRRORS = [
  "http://azure.archive.ubuntu.com/ubuntu/\tpriority:2",
  "https://archive.ubuntu.com/ubuntu/\tpriority:1",
  "https://security.ubuntu.com/ubuntu/\tpriority:3",
].join("\n") + "\n";

export function prioritizeUbuntuArchive(contents) {
  if (contents === ARCHIVE_FIRST_MIRRORS) return contents;
  if (contents === RUNNER_MIRRORS) return ARCHIVE_FIRST_MIRRORS;
  throw new Error("unexpected Ubuntu runner APT mirror configuration");
}

export function configureMirrorFile(filePath) {
  const stat = fs.lstatSync(filePath);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error("APT mirror configuration must be a regular file");
  }
  const current = fs.readFileSync(filePath, "utf8");
  const desired = prioritizeUbuntuArchive(current);
  if (desired === current) return false;

  const temporary = path.join(path.dirname(filePath), `.${path.basename(filePath)}.${process.pid}.${randomUUID()}`);
  let descriptor;
  try {
    descriptor = fs.openSync(temporary, "wx", stat.mode & 0o777);
    fs.writeFileSync(descriptor, desired, "utf8");
    fs.fchmodSync(descriptor, stat.mode & 0o777);
    fs.fsyncSync(descriptor);
    fs.closeSync(descriptor);
    descriptor = undefined;
    fs.renameSync(temporary, filePath);
  } catch (error) {
    if (descriptor !== undefined) fs.closeSync(descriptor);
    try {
      fs.unlinkSync(temporary);
    } catch (cleanupError) {
      if (cleanupError.code !== "ENOENT") throw cleanupError;
    }
    throw error;
  }
  return true;
}

function main() {
  if (process.argv.length !== 3) {
    throw new Error("usage: ci-browser-apt-mirror.mjs <apt-mirrors-file>");
  }
  const changed = configureMirrorFile(process.argv[2]);
  process.stdout.write(changed ? "prioritized Ubuntu archive mirror\n" : "Ubuntu archive mirror already prioritized\n");
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) main();
