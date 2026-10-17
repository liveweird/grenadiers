import assert from "node:assert/strict";
import test from "node:test";

import { attachIdentity, selectServerLibtest } from "./measure-game-artifact.mjs";

const artifact = (overrides = {}) => ({
  reason: "compiler-artifact",
  target: { name: "grenadiers_server", kind: ["lib"] },
  profile: { test: true },
  executable: "/current/grenadiers_server-exact",
  ...overrides,
});

test("selects the current Cargo libtest artifact instead of stale-looking candidates", () => {
  const stream = [
    artifact({ executable: null }),
    artifact({ target: { name: "grenadiers_server", kind: ["bin"] }, executable: "/stale/bin" }),
    artifact({ profile: { test: false }, executable: "/stale/release-lib" }),
    artifact(),
  ]
    .map(JSON.stringify)
    .join("\n");

  assert.equal(selectServerLibtest(stream), "/current/grenadiers_server-exact");
});

test("rejects ambiguous Cargo libtest artifacts", () => {
  const stream = [artifact(), artifact({ executable: "/other/current" })]
    .map(JSON.stringify)
    .join("\n");
  assert.throws(() => selectServerLibtest(stream), /exactly one/);
});

test("attaches validated source and binary identity", () => {
  const source = attachIdentity(
    { name: "owner-1" },
    {
      commit: "a".repeat(40),
      state: "clean",
      binarySha256: "b".repeat(64),
      imageIdentity: "native-macos",
    },
  );
  assert.equal(source.source.commit, "a".repeat(40));
  assert.equal(source.source.binarySha256, "b".repeat(64));
});
