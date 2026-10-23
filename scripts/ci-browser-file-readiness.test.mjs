import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { MAX_BYTES, READ_BUFFER_BYTES, warmBrowserFiles }
  from "./ci-browser-file-readiness.mjs";

function fixture() {
  const parent = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "browser-readiness-")));
  const root = path.join(parent, "chrome-linux64");
  fs.mkdirSync(path.join(root, "locales"), { recursive: true });
  const executable = path.join(root, "chrome");
  fs.writeFileSync(executable, "chromium-binary");
  fs.writeFileSync(path.join(root, "resources.pak"), "resources");
  fs.writeFileSync(path.join(root, "locales", "en-US.pak"), "locale");
  return { parent, root, executable, output: path.join(parent, "readiness.json") };
}

function cleanup(parent) {
  fs.rmSync(parent, { recursive: true, force: true });
}

test("streams every regular distribution file once into a closed inventory", () => {
  const current = fixture();
  try {
    let largestRead = 0;
    const fsApi = Object.create(fs);
    fsApi.readSync = (fd, buffer, offset, length, position) => {
      largestRead = Math.max(largestRead, length);
      return fs.readSync(fd, buffer, offset, Math.min(length, 3), position);
    };
    const record = warmBrowserFiles({ executablePath: current.executable,
      outputPath: current.output, fsApi });
    assert.equal(record.result, "PASS");
    assert.equal(record.fileCount, 3);
    assert.deepEqual(record.files.map(({ path: relative }) => relative),
      ["chrome", "locales/en-US.pak", "resources.pak"]);
    assert.equal(record.bytesRead, "chromium-binarylocaleresources".length);
    assert(largestRead <= READ_BUFFER_BYTES);
    assert.deepEqual(JSON.parse(fs.readFileSync(current.output, "utf8")), record);
  } finally { cleanup(current.parent); }
});

test("fails closed on missing files, read errors, truncation, and changing output", () => {
  const missing = fixture();
  try {
    assert.throws(() => warmBrowserFiles({ executablePath: path.join(missing.root, "missing"),
      outputPath: missing.output }), /ENOENT/);
  } finally { cleanup(missing.parent); }

  const unreadable = fixture();
  try {
    const fsApi = Object.create(fs);
    fsApi.readSync = () => { throw new Error("fixture read error"); };
    assert.throws(() => warmBrowserFiles({ executablePath: unreadable.executable,
      outputPath: unreadable.output, fsApi }), /fixture read error/);
    assert.equal(fs.existsSync(unreadable.output), false);
  } finally { cleanup(unreadable.parent); }

  const truncated = fixture();
  try {
    const fsApi = Object.create(fs);
    let changed = false;
    fsApi.readSync = (fd, buffer, offset, length, position) => {
      if (!changed) { fs.truncateSync(truncated.executable, 0); changed = true; }
      return fs.readSync(fd, buffer, offset, length, position);
    };
    assert.throws(() => warmBrowserFiles({ executablePath: truncated.executable,
      outputPath: truncated.output, fsApi }), /truncated/);
  } finally { cleanup(truncated.parent); }

  const existing = fixture();
  try {
    fs.writeFileSync(existing.output, "existing");
    assert.throws(() => warmBrowserFiles({ executablePath: existing.executable,
      outputPath: existing.output }), /EEXIST/);
    assert.equal(fs.readFileSync(existing.output, "utf8"), "existing");
  } finally { cleanup(existing.parent); }
});

test("rejects symlinks and paths escaping the exact browser distribution", () => {
  const current = fixture();
  try {
    const outside = path.join(current.parent, "outside");
    fs.writeFileSync(outside, "secret");
    fs.symlinkSync(outside, path.join(current.root, "escaped"));
    assert.throws(() => warmBrowserFiles({ executablePath: current.executable,
      outputPath: current.output }), /symlink/);
    assert.equal(fs.existsSync(current.output), false);
  } finally { cleanup(current.parent); }

  const linkedExecutable = fixture();
  try {
    const link = path.join(linkedExecutable.parent, "chrome-link");
    fs.symlinkSync(linkedExecutable.executable, link);
    assert.throws(() => warmBrowserFiles({ executablePath: link,
      outputPath: linkedExecutable.output }), /symlink/);
  } finally { cleanup(linkedExecutable.parent); }
});

test("enforces file and byte limits before allocating or reading file contents", () => {
  const files = fixture();
  try {
    assert.throws(() => warmBrowserFiles({ executablePath: files.executable,
      outputPath: files.output, maxFiles: 2 }), /file limit/);
  } finally { cleanup(files.parent); }

  const bytes = fixture();
  try {
    fs.truncateSync(path.join(bytes.root, "resources.pak"), 1024);
    assert.throws(() => warmBrowserFiles({ executablePath: bytes.executable,
      outputPath: bytes.output, maxBytes: 100 }), /byte limit/);
    assert(MAX_BYTES > READ_BUFFER_BYTES);
  } finally { cleanup(bytes.parent); }
});

test("streams directory entries and bounds empty directories as well as files", () => {
  const complete = fixture();
  try {
    fs.mkdirSync(path.join(complete.root, "empty"));
    const fsApi = Object.create(fs);
    fsApi.readdirSync = () => { throw new Error("must not materialize directory names"); };
    const record = warmBrowserFiles({ executablePath: complete.executable,
      outputPath: complete.output, fsApi, maxEntries: 5 });
    assert.equal(record.fileCount, 3);
    assert.equal(record.entryCount, 5);
    assert.equal(record.directoryCount, 3);
    assert.equal(record.directoryWalkAssumption,
      "completed-installation-remains-unchanged-during-walk");
  } finally { cleanup(complete.parent); }

  const bounded = fixture();
  try {
    fs.mkdirSync(path.join(bounded.root, "empty"));
    assert.throws(() => warmBrowserFiles({ executablePath: bounded.executable,
      outputPath: bounded.output, maxEntries: 4 }), /entry limit/);
    assert.equal(fs.existsSync(bounded.output), false);
  } finally { cleanup(bounded.parent); }
});

test("workflow pre-reads once after latency and immediately before observed hardening", () => {
  const workflow = fs.readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
  const latency = workflow.indexOf("npm --prefix e2e run test:latency");
  const readiness = workflow.indexOf("node scripts/ci-browser-file-readiness.mjs");
  const hardening = workflow.indexOf("node scripts/ci-visibility-startup-observer.mjs -- npm --prefix e2e run test:hardening");
  assert(latency >= 0 && latency < readiness && readiness < hardening);
  assert.equal((workflow.match(/node scripts\/ci-browser-file-readiness\.mjs/g) ?? []).length, 1);
  assert.equal((workflow.match(/node --test scripts\/ci-browser-file-readiness\.test\.mjs/g)
    ?? []).length, 1);
  assert.equal((workflow.match(/test:hardening/g) ?? []).length, 1);
  assert.match(workflow, /\$\{\{ runner\.temp \}\}\/browser-file-readiness\.json/);
});
