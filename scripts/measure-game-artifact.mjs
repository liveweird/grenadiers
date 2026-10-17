import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const COMMIT_PATTERN = /^[0-9a-f]{40}$/;
const SHA256_PATTERN = /^[0-9a-f]{64}$/;

export function selectServerLibtest(lines) {
  const artifacts = lines
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map((line) => JSON.parse(line))
    .filter(
      (message) =>
        message.reason === "compiler-artifact" &&
        message.target?.name === "grenadiers_server" &&
        message.target?.kind?.includes("lib") &&
        message.profile?.test === true &&
        typeof message.executable === "string",
    );

  if (artifacts.length !== 1) {
    throw new Error(
      `expected exactly one grenadiers_server libtest artifact, found ${artifacts.length}`,
    );
  }
  return artifacts[0].executable;
}

export function attachIdentity(record, identity) {
  if (!COMMIT_PATTERN.test(identity.commit)) {
    throw new Error("source commit must be exactly 40 lowercase hexadecimal characters");
  }
  if (!new Set(["clean", "dirty", "export"]).has(identity.state)) {
    throw new Error("source state must be clean, dirty, or export");
  }
  if (!SHA256_PATTERN.test(identity.binarySha256)) {
    throw new Error("binary SHA-256 must be exactly 64 lowercase hexadecimal characters");
  }
  if (identity.imageIdentity.length === 0) {
    throw new Error("image identity must not be empty");
  }
  return { ...record, source: identity };
}

function identityFromEnvironment() {
  return {
    commit: process.env.MEASUREMENT_COMMIT ?? "",
    state: process.env.MEASUREMENT_SOURCE_STATE ?? "",
    binarySha256: process.env.MEASUREMENT_BINARY_SHA256 ?? "",
    imageIdentity: process.env.MEASUREMENT_IMAGE_IDENTITY ?? "",
  };
}

function main() {
  const command = process.argv[2];
  const input = readFileSync(0, "utf8");
  if (command === "select") {
    process.stdout.write(`${selectServerLibtest(input)}\n`);
    return;
  }
  if (command === "identify") {
    const record = JSON.parse(input);
    process.stdout.write(`${JSON.stringify(attachIdentity(record, identityFromEnvironment()))}\n`);
    return;
  }
  throw new Error("usage: measure-game-artifact.mjs select|identify");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
