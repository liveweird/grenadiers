import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

const fullPath = new URL("../.github/workflows/ci.yml", import.meta.url);
const platformPath = new URL("../.github/workflows/platform.yml", import.meta.url);
const fixturePath = new URL("../e2e/compose/fixture.yaml", import.meta.url);
const exhaustive = "expanded_item_masks_and_disabled_preset_replay_deterministically";
const rustCache = "Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3";
const failureArtifactPaths = [
  "e2e/playwright-report/",
  "e2e/test-results/",
  "e2e/.playwright/art-atlas-test-results/",
  "e2e/.playwright/atlas-gameplay-test-results/",
  "e2e/.playwright/terrain-pixel-test-results/",
  "e2e/playwright-lairs-report/",
  "e2e/lairs-test-results/",
  "e2e/playwright-pickup-report/",
  "e2e/playwright-heart-report/",
  "e2e/playwright-shield-report/",
  "e2e/playwright-golden-flame-report/",
  "e2e/playwright-short-fuse-report/",
  "e2e/playwright-short-flame-report/",
  "e2e/playwright-long-fuse-report/",
  "e2e/playwright-reverse-controls-report/",
  "e2e/playwright-diarrhea-report/",
  "e2e/playwright-slow-foot-report/",
  "e2e/playwright-piercing-bomb-report/",
  "e2e/playwright-power-bomb-report/",
  "e2e/playwright-remote-bomb-report/",
  "e2e/playwright-landmine-report/",
  "e2e/playwright-invisibility-cloak-report/",
  "e2e/playwright-stopwatch-report/",
  "e2e/playwright-invisible-bombs-report/",
  "e2e/playwright-teleporter-report/",
  "e2e/playwright-wall-pass-report/",
  "e2e/playwright-cpu-item-report/",
  "e2e/cpu-item-test-results/",
  "e2e/playwright-kick-report/",
  "e2e/playwright-punch-report/",
  "e2e/punch-test-results/",
  "e2e/playwright-bracelet-report/",
  "e2e/bracelet-test-results/",
  "e2e/playwright-oil-barrel-report/",
  "e2e/oil-barrel-test-results/",
  "e2e/playwright-match-report/",
  "e2e/playwright-reconnect-report/",
  "e2e/playwright-controls-report/",
  "e2e/playwright-audio-report/",
  "${{ runner.temp }}/browser-file-readiness.json",
  "${{ runner.temp }}/visibility-startup-observer.jsonl",
  "${{ runner.temp }}/visibility-xvfb.log",
];
const fixtures = {
  "bracelet": "arena-v2-bracelet.seed",
  "diarrhea": "arena-v2-diarrhea.seed",
  "golden-flame": "arena-v2-golden-flame.seed",
  "heart": "arena-v2-heart.seed",
  "invisibility-cloak": "arena-v2-invisibility-cloak.seed",
  "invisible-bombs": "arena-v2-invisible-bombs.seed",
  "kick": "arena-v2-kick.seed",
  "landmine": "arena-v2-landmine.seed",
  "lairs": "arena-v3-lairs.seed",
  "long-fuse": "arena-v2-long-fuse.seed",
  "matches": "arena-v1-matches.seed",
  "oil-barrel": "arena-v2-oil-barrel.seed",
  "pickups": "arena-v3-pickups.seed",
  "piercing-bomb": "arena-v2-piercing-bomb.seed",
  "power-bomb": "arena-v2-power-bomb.seed",
  "punch": "arena-v2-punch.seed",
  "remote-bomb": "arena-v2-remote-bomb.seed",
  "reverse-controls": "arena-v2-reverse-controls.seed",
  "shield": "arena-v2-shield.seed",
  "short-flame": "arena-v2-short-flame.seed",
  "short-fuse": "arena-v2-short-fuse.seed",
  "slow-foot": "arena-v2-slow-foot.seed",
  "stopwatch": "arena-v2-stopwatch.seed",
  "teleporter": "arena-v2-teleporter.seed",
  "wall-pass": "arena-v2-wall-pass.seed",
  "cpu-item": "arena-v2-cpu-wall-pass.seed",
};

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
  const artifactPathBlock = failureArtifact.match(/^          path: \|\n((?:^ {12}.+\n)+)/m);
  assert.ok(artifactPathBlock, "missing failure artifact path block");
  const artifactPaths = artifactPathBlock[1].trim().split("\n")
    .map((line) => line.trim());
  assert.deepEqual(artifactPaths, failureArtifactPaths);
  assert.equal((workflow.match(/for _attempt in \$\(seq 1 60\); do/g) ?? []).length, Object.keys(fixtures).length + 1);
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

export function validateSharedFixture(workflow, fixture) {
  assert.equal(fixture, `# Shared operator reproduction mode for deterministic browser acceptance.
services:
  app:
    environment:
      GAME_ROUND_SEED_FILE: /run/grenadiers/round.seed
    volumes:
      - type: bind
        source: '\${GRENADIERS_FIXTURE_SEED_FILE:?Set the exact fixture seed file}'
        target: /run/grenadiers/round.seed
        read_only: true
        bind:
          create_host_path: false
`);
  for (const [feature, seed] of Object.entries(fixtures)) {
    assert.ok(fs.existsSync(new URL(`../core/tests/fixtures/${seed}`, import.meta.url)),
      `missing seed ${seed}`);
    assert.ok(!fs.existsSync(new URL(`../docker-compose.${feature}.yaml`, import.meta.url)),
      `obsolete per-feature override docker-compose.${feature}.yaml remains`);
    const invocation = `GRENADIERS_FIXTURE_SEED_FILE=./core/tests/fixtures/${seed} docker compose ` +
      "-f docker-compose.yaml -f e2e/compose/fixture.yaml up -d --no-build --force-recreate app";
    assert.equal(workflow.split(invocation).length - 1, 1,
      `expected one shared fixture invocation for ${seed}`);
  }
  assert.equal(workflow.split("-f e2e/compose/fixture.yaml up").length - 1,
    Object.keys(fixtures).length);
  // Seed filenames retain their historical names; live item selectors bind generator3.
  assert.doesNotMatch(workflow, /E2E_[A-Z_]+_FIXTURE=arena-v2-/);
  assert.doesNotMatch(workflow, /docker compose -f docker-compose\.yaml -f docker-compose\.[\w-]+\.yaml/);
  assert.match(workflow, /name: Tear down isolated stack\n        if: always\(\)\n        run: docker compose down --volumes --remove-orphans/);
}

test("full verification remains complete and manual-only", () => {
  validateFullWorkflow(fs.readFileSync(fullPath, "utf8"));
});

test("automatic Linux workflow closes only the Rust execution platform gap", () => {
  validatePlatformWorkflow(fs.readFileSync(platformPath, "utf8"));
});

test("browser fixtures use one required host-selected read-only seed override", () => {
  validateSharedFixture(
    fs.readFileSync(fullPath, "utf8"),
    fs.readFileSync(fixturePath, "utf8"),
  );
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

test("failure artifact paths reject broad parents and hidden subtree patterns", () => {
  const full = fs.readFileSync(fullPath, "utf8");
  const firstPath = "          path: |\n            e2e/playwright-report/";

  for (const parentPath of ["e2e/", "e2e/*", "e2e/**"]) {
    assert.throws(() => validateFullWorkflow(full.replace(
      firstPath, `          path: |\n            ${parentPath}\n            e2e/playwright-report/`)));
    assert.throws(() => validateFullWorkflow(full.replace(
      "e2e/playwright-report/", parentPath)));
  }
  for (const hiddenSubtree of ["e2e/.playwright/*", "e2e/.playwright/**"]) {
    assert.throws(() => validateFullWorkflow(full.replace(
      "e2e/.playwright/terrain-pixel-test-results/", hiddenSubtree)));
  }
  for (const rootGlob of ["*", "**"]) {
    assert.throws(() => validateFullWorkflow(full.replace(
      "e2e/playwright-report/", rootGlob)));
  }
});

test("fixture contracts reject missing selectors, reversed files, and selector-dependent cleanup", () => {
  const workflow = fs.readFileSync(fullPath, "utf8");
  const fixture = fs.readFileSync(fixturePath, "utf8");
  assert.throws(() => validateSharedFixture(
    workflow.replace("E2E_SHIELD_FIXTURE=arena-v3-shield", "E2E_SHIELD_FIXTURE=arena-v2-shield"),
    fixture,
  ));
  assert.throws(() => validateSharedFixture(
    workflow.replace("GRENADIERS_FIXTURE_SEED_FILE=./core/tests/fixtures/arena-v3-pickups.seed ", ""),
    fixture,
  ));
  assert.throws(() => validateSharedFixture(
    workflow.replace("-f docker-compose.yaml -f e2e/compose/fixture.yaml",
      "-f e2e/compose/fixture.yaml -f docker-compose.yaml"),
    fixture,
  ));
  assert.throws(() => validateSharedFixture(
    workflow.replace(
      "name: Tear down isolated stack\n        if: always()\n        run: docker compose down --volumes --remove-orphans",
      "name: Tear down isolated stack\n        if: always()\n        run: docker compose -f docker-compose.yaml -f e2e/compose/fixture.yaml down"),
    fixture,
  ));
});
