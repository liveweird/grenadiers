import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  ARCHIVE_FIRST_MIRRORS,
  RUNNER_MIRRORS,
  configureMirrorFile,
  prioritizeUbuntuArchive,
} from "./ci-browser-apt-mirror.mjs";

const upstreamMirrors = "http://azure.archive.ubuntu.com/ubuntu/\tpriority:1\n"
  + "https://archive.ubuntu.com/ubuntu/\tpriority:2\n"
  + "https://security.ubuntu.com/ubuntu/\tpriority:3\n";
const desiredMirrors = "http://azure.archive.ubuntu.com/ubuntu/\tpriority:2\n"
  + "https://archive.ubuntu.com/ubuntu/\tpriority:1\n"
  + "https://security.ubuntu.com/ubuntu/\tpriority:3\n";

function withMirrorFile(contents, assertion) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "grenadiers-apt-mirror-"));
  const file = path.join(root, "apt-mirrors.txt");
  try {
    fs.writeFileSync(file, contents, { mode: 0o640 });
    return assertion({ file, root });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

test("swaps explicit archive priorities atomically and preserves the file mode", () => {
  assert.equal(RUNNER_MIRRORS, upstreamMirrors);
  assert.equal(ARCHIVE_FIRST_MIRRORS, desiredMirrors);
  withMirrorFile(upstreamMirrors, ({ file, root }) => {
    assert.equal(configureMirrorFile(file), true);
    assert.equal(fs.readFileSync(file, "utf8"), desiredMirrors);
    assert.equal(fs.statSync(file).mode & 0o777, 0o640);
    assert.deepEqual(fs.readdirSync(root), ["apt-mirrors.txt"]);
    assert.match(fs.readFileSync(file, "utf8"), /archive\.ubuntu\.com\/ubuntu\/\tpriority:1/);
    assert.match(fs.readFileSync(file, "utf8"), /azure\.archive\.ubuntu\.com\/ubuntu\/\tpriority:2/);
  });
});

test("accepts the exact desired state without rewriting it", () => {
  withMirrorFile(desiredMirrors, ({ file }) => {
    const before = fs.statSync(file);
    assert.equal(configureMirrorFile(file), false);
    const after = fs.statSync(file);
    assert.equal(after.ino, before.ino);
    assert.equal(after.mtimeMs, before.mtimeMs);
    assert.equal(fs.readFileSync(file, "utf8"), desiredMirrors);
  });
});

test("rejects missing, duplicate, unknown, reordered, or reprioritized configuration before writing", () => {
  const invalid = [
    upstreamMirrors.split("\n").slice(0, 2).join("\n") + "\n",
    upstreamMirrors.replace(
      "https://security.ubuntu.com/ubuntu/\tpriority:3",
      "http://azure.archive.ubuntu.com/ubuntu/\tpriority:1",
    ),
    upstreamMirrors.replace("https://security.ubuntu.com/ubuntu/", "https://example.invalid/ubuntu/"),
    upstreamMirrors.replace("priority:3", "priority:4"),
    upstreamMirrors.split("\n").slice(0, -1).reverse().join("\n") + "\n",
    upstreamMirrors + "https://extra.invalid/ubuntu/\tpriority:4\n",
    upstreamMirrors.replaceAll("\tpriority:", " priority:"),
  ];
  for (const contents of invalid) {
    withMirrorFile(contents, ({ file, root }) => {
      const before = fs.readFileSync(file);
      assert.throws(() => configureMirrorFile(file), /unexpected Ubuntu runner/);
      assert.deepEqual(fs.readFileSync(file), before);
      assert.deepEqual(fs.readdirSync(root), ["apt-mirrors.txt"]);
    });
  }
});

test("rejects a symlink without changing its target", () => {
  withMirrorFile(upstreamMirrors, ({ file, root }) => {
    const link = path.join(root, "linked-mirrors.txt");
    fs.symlinkSync(file, link);
    assert.throws(() => configureMirrorFile(link), /regular file/);
    assert.equal(fs.readFileSync(file, "utf8"), upstreamMirrors);
  });
});

test("workflow tests and configures mirrors before the unchanged Playwright install", () => {
  const workflow = fs.readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
  const browser = workflow.slice(workflow.indexOf("\n  browser:\n"));
  const regression = browser.indexOf("node --test scripts/ci-browser-apt-mirror.test.mjs");
  const configure = browser.indexOf("sudo \"$(command -v node)\" scripts/ci-browser-apt-mirror.mjs /etc/apt/apt-mirrors.txt");
  const install = browser.indexOf("npm --prefix e2e exec -- playwright install --with-deps chromium");
  assert.ok(regression > 0 && regression < configure && configure < install);
  assert.equal((browser.match(/playwright install --with-deps chromium/g) ?? []).length, 1);
  assert.match(browser, /matrix:\n\s+shard: \[items, classic\]/);
  assert.match(browser, /timeout-minutes: 20/);
  assert.equal(prioritizeUbuntuArchive(upstreamMirrors), desiredMirrors);
});
