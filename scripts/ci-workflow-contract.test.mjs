import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

const fullPath = new URL("../.github/workflows/ci.yml", import.meta.url);
const platformPath = new URL("../.github/workflows/platform.yml", import.meta.url);
const exhaustive = "expanded_item_masks_and_disabled_preset_replay_deterministically";
const rustCache = "Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3";

function trigger(workflow) {
  return workflow.slice(workflow.indexOf("on:\n"), workflow.indexOf("permissions:\n"));
}

export function validateFullWorkflow(workflow) {
  assert.equal(trigger(workflow), "on:\n  workflow_dispatch:\n");
  assert.doesNotMatch(trigger(workflow), /push:|pull_request:/);
  for (const job of ["web", "server-mask-plan", "server-mask-partition", "server",
    "security", "browser-image", "browser", "kubernetes"]) {
    assert.match(workflow, new RegExp(`^  ${job}:$`, "m"));
  }
  assert.match(workflow, /scripts\/gates\.sh server/);
  assert.match(workflow, /scripts\/gates\.sh coverage/);
  assert.match(workflow, /scripts\/ci-server-mask-aggregate\.mjs/);
  assert.equal(workflow.split(rustCache).length - 1, 2);
  assert.match(workflow, /key: llvm-cov-0\.9\.1\n[\s\S]*?cache-targets: true\n[\s\S]*?cache-workspace-crates: false\n[\s\S]*?cache-on-failure: false\n[\s\S]*?cache-bin: true\n[\s\S]*?save-if: \$\{\{ matrix\.index == 0 \}\}/);
  assert.match(workflow, /key: cargo-deny-0\.20\.2\n[\s\S]*?cache-targets: false\n[\s\S]*?cache-workspace-crates: false\n[\s\S]*?cache-on-failure: false\n[\s\S]*?cache-bin: true/);
  assert.match(workflow, /installed_version="\$\(cargo llvm-cov --version 2>\/dev\/null \|\| true\)"[\s\S]*?if \[ "\$installed_version" != 'cargo-llvm-cov 0\.9\.1' \]; then[\s\S]*?cargo install cargo-llvm-cov --version 0\.9\.1 --locked[\s\S]*?test "\$\(cargo llvm-cov --version\)" = 'cargo-llvm-cov 0\.9\.1'/);
  assert.match(workflow, /installed_version="\$\(cargo deny --version 2>\/dev\/null \|\| true\)"[\s\S]*?if \[ "\$installed_version" != 'cargo-deny 0\.20\.2' \]; then[\s\S]*?cargo install cargo-deny --version 0\.20\.2 --locked[\s\S]*?test "\$\(cargo deny --version\)" = 'cargo-deny 0\.20\.2'/);
  assert.doesNotMatch(workflow, /cache-hit/);

  const atlas = workflow.indexOf("scripts/gates.sh atlas-gameplay");
  const terrain = workflow.indexOf("npm --prefix e2e run test:terrain-pixels");
  const nextItemScenario = workflow.indexOf("Verify deterministic pickups", terrain);
  assert.ok(atlas < terrain && terrain < nextItemScenario);
  const failureArtifact = workflow.slice(
    workflow.indexOf("name: playwright-report-${{ matrix.shard }}"),
    workflow.indexOf("\n\n  kubernetes:"),
  );
  assert.match(failureArtifact, /include-hidden-files: true/);
  const hiddenArtifactPaths = [...failureArtifact.matchAll(/^\s+(e2e\/\.playwright\/.*)$/gm)]
    .map(([, path]) => path);
  assert.deepEqual(hiddenArtifactPaths, [
    "e2e/.playwright/art-atlas-test-results/",
    "e2e/.playwright/atlas-gameplay-test-results/",
    "e2e/.playwright/terrain-pixel-test-results/",
  ]);
  assert.equal((workflow.match(/for _attempt in \$\(seq 1 60\); do/g) ?? []).length, 25);
  assert.doesNotMatch(workflow, /for attempt in \$\(seq 1 60\); do/);
}

export function validatePlatformWorkflow(workflow) {
  const event = trigger(workflow);
  assert.match(event, /^on:\n  pull_request:\n/m);
  assert.match(event, /^  workflow_dispatch:$/m);
  assert.doesNotMatch(event, /^  push:$/m);
  for (const path of ["Cargo.lock", "Cargo.toml", ".cargo/**", "rust-toolchain.toml", "api/**",
    "core/**", "server/**", "scripts/gates.sh", "scripts/ci-server-mask-*.mjs"]) {
    assert.ok(event.includes(`      - ${path}\n`), `missing path filter ${path}`);
  }
  assert.match(workflow, /runs-on: ubuntu-24\.04/);
  assert.match(workflow, /ref: \$\{\{ github\.event\.pull_request\.head\.sha \|\| github\.sha \}\}/);
  assert.match(workflow, /test "\$\(git rev-parse HEAD\)" = "\$EXPECTED_SOURCE"/);
  assert.match(workflow,
    /Swatinem\/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2\.9\.3/);
  assert.match(workflow, /cache-targets: true/);
  assert.match(workflow, /cache-workspace-crates: false/);
  assert.match(workflow, /cache-on-failure: false/);
  assert.doesNotMatch(workflow, /upload-artifact|download-artifact/);

  const commands = [...workflow.matchAll(/^        run: >-\n((?:^          .+\n?)+)/gm)]
    .map(([, lines]) => lines.trim().split("\n").map((line) => line.trim()).join(" "))
    .filter((command) => command.startsWith("cargo test "));
  assert.deepEqual(commands, [
    `cargo test --workspace --locked -- --skip ${exhaustive}`,
    `cargo test --workspace --release --locked -- --skip ${exhaustive}`,
  ]);
  assert.equal((workflow.match(new RegExp(`--skip ${exhaustive}`, "g")) ?? []).length, 2);
  assert.doesNotMatch(workflow, /--skip checked_enabled_items_catalog_is_authoritative/);
  assert.doesNotMatch(workflow, /--ignored|--include-ignored/);
}

test("full verification remains complete and manual-only", () => {
  validateFullWorkflow(fs.readFileSync(fullPath, "utf8"));
});

test("automatic Linux workflow closes only the Rust execution platform gap", () => {
  validatePlatformWorkflow(fs.readFileSync(platformPath, "utf8"));
});

test("contracts reject automatic full verification and weakened mask coverage", () => {
  const full = fs.readFileSync(fullPath, "utf8");
  assert.throws(() => validateFullWorkflow(full.replace("  workflow_dispatch:\n", "  push:\n")));
  assert.throws(() => validateFullWorkflow(full.replace("scripts/gates.sh coverage", "true")));
  assert.throws(() => validateFullWorkflow(full.replace("cache-bin: true", "cache-bin: false")));
  assert.throws(() => validateFullWorkflow(full.replace(
    "          npm --prefix e2e run test:terrain-pixels\n", "")));
  assert.throws(() => validateFullWorkflow(full.replace("          include-hidden-files: true\n", "")));
  assert.throws(() => validateFullWorkflow(full.replace(
    "e2e/.playwright/terrain-pixel-test-results/", "e2e/.playwright/*")));
  assert.throws(() => validateFullWorkflow(full.replace(
    "e2e/.playwright/terrain-pixel-test-results/", "e2e/.playwright/**")));
  assert.throws(() => validateFullWorkflow(full.replace(
    "installed_version=\"$(cargo llvm-cov --version 2>/dev/null || true)\"",
    "installed_version='cargo-llvm-cov 0.9.1'")));

  const platform = fs.readFileSync(platformPath, "utf8");
  assert.throws(() => validatePlatformWorkflow(platform.replace("  workflow_dispatch:\n", "  push:\n")));
  assert.throws(() => validatePlatformWorkflow(platform.replace(
    `--skip ${exhaustive}`, "--skip checked_enabled_items_catalog_is_authoritative")));
  assert.throws(() => validatePlatformWorkflow(platform.replace("cargo test --workspace --release", "true")));
  assert.throws(() => validatePlatformWorkflow(platform.replace(
    `--skip ${exhaustive}`, `--skip ${exhaustive}\n          --skip rejects_invalid_catalogs_and_cardinality_overflow`)));
  assert.throws(() => validatePlatformWorkflow(platform.replace(
    "cargo test --workspace --locked --\n", "cargo test --workspace --locked --\n          replay\n")));
});
