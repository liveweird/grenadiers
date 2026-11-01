#!/usr/bin/env python3
"""Compile once per profile, then run exact server-mask ranges in parallel."""
from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import os
import re
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any, Callable

ROOT = Path(__file__).resolve().parent.parent
COVERAGE_TARGET = ROOT / "target/grenadiers-local-server-coverage"
HEAVY_TEST = "game::rooms::replay::settings_tests::expanded_item_masks_and_disabled_preset_replay_deterministically"
ISOLATED_TESTS = (
    "game::rooms::roster_owner::spike_cpu_acceptance_tests::real_worker_tests::real_workers_record_cpu26_progress_and_scheduling_with_bounded_plans",
    "game::rooms::roster_owner::spike_cpu_acceptance_tests::real_worker_tests::real_workers_record_cpu27_progress_and_scheduling_with_bounded_plans",
    "game::rooms::roster_owner::hard_episode_tests::native_hard_v57_seed73_progresses_with_two_workers_at_production_pacing",
    "game::rooms::roster_owner::teleporter_owner_tests::native_worker_completes_collection_stop_and_productive_follow_up",
)
INSTRUMENTATION_PROBE_TEST = "tests::blank_database_url_disables_postgres"
SUMMARY_PREFIX = "GRENADIERS_SERVER_REPLAY_ITEM_MASK_SUMMARY="
PLAN_KEYS = {"schemaVersion", "source", "catalog", "capacity", "totalCardinality", "partitionCount", "ranges"}
RANGE_KEYS = {"index", "startOrdinal", "endOrdinalExclusive"}


def run(command: list[str], *, env: dict[str, str] | None = None,
        capture: bool = False, merge_stderr: bool = True) -> subprocess.CompletedProcess[str]:
    print("+", shlex.join(command), flush=True)
    return subprocess.run(command, cwd=ROOT, env=env, check=True, text=True,
                          stdout=subprocess.PIPE if capture else None,
                          stderr=subprocess.STDOUT if capture and merge_stderr else None)


def git(*args: str) -> str:
    return run(["git", *args], capture=True).stdout.strip()


def validate_plan(plan: Any, source: str) -> dict[str, Any]:
    if type(plan) is not dict or set(plan) != PLAN_KEYS or plan["schemaVersion"] != 1 \
            or plan["source"] != source or re.fullmatch(r"[0-9a-f]{40}", source) is None:
        raise ValueError("plan schema/source mismatch")
    catalog, capacity, total, count = (plan[key] for key in
                                       ("catalog", "capacity", "totalCardinality", "partitionCount"))
    if type(catalog) is not list or not catalog or len(catalog) > 32 \
            or any(type(bit) is not int or bit <= 0 or bit & (bit - 1) for bit in catalog) \
            or catalog != sorted(set(catalog)) or type(capacity) is not int or capacity < 1 \
            or type(total) is not int or total != 2 ** len(catalog) \
            or type(count) is not int or count != (total + capacity - 1) // capacity:
        raise ValueError("plan catalog/cardinality mismatch")
    ranges, cursor = plan["ranges"], 0
    if type(ranges) is not list or len(ranges) != count:
        raise ValueError("plan range count mismatch")
    for index, item in enumerate(ranges):
        end = min(cursor + capacity, total)
        if type(item) is not dict or set(item) != RANGE_KEYS \
                or any(type(item[key]) is not int for key in RANGE_KEYS) \
                or item != {"index": index, "startOrdinal": cursor, "endOrdinalExclusive": end}:
            raise ValueError("plan ranges are not exact, ordered, and complete")
        cursor = end
    if cursor != total:
        raise ValueError("plan ranges do not cover the catalog")
    return plan


def schedule(plan: dict[str, Any], workers: int) -> list[list[dict[str, int]]]:
    if type(workers) is not int or not 1 <= workers <= 8:
        raise ValueError("workers must be an integer from 1 through 8")
    ranges = plan["ranges"]
    buckets = [[] for _ in range(min(workers, len(ranges)))]
    for index, item in enumerate(ranges):
        buckets[index % len(buckets)].append(item)
    flattened = [item["index"] for bucket in buckets for item in bucket]
    if sorted(flattened) != list(range(plan["partitionCount"])) or len(flattened) != len(set(flattened)):
        raise ValueError("scheduled ranges are missing or duplicated")
    return buckets


def parallel_map(buckets: list[list[dict[str, int]]],
                 worker: Callable[[int, list[dict[str, int]]], Any]) -> list[Any]:
    results: list[Any] = [None] * len(buckets)
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(buckets)) as pool:
        futures = {pool.submit(worker, index, bucket): index for index, bucket in enumerate(buckets)}
        for future in concurrent.futures.as_completed(futures):
            results[futures[future]] = future.result()
    return results


def parse_coverage_environment(output: str,
                               base: dict[str, str] | None = None) -> dict[str, str]:
    env = dict(base or os.environ)
    for line in output.splitlines():
        if line.startswith("export "):
            assignment = shlex.split(line[7:])
            if len(assignment) != 1 or "=" not in assignment[0]: raise ValueError("unsafe llvm-cov environment")
            key, value = assignment[0].split("=", 1); env[key] = value
    return env


def coverage_base_environment(target_dir: Path = COVERAGE_TARGET) -> dict[str, str]:
    env = dict(os.environ)
    coverage_keys = {"LLVM_PROFILE_FILE", "LLVM_PROFILE_FILE_NAME", "CARGO_LLVM_COV",
                     "CARGO_LLVM_COV_SHOW_ENV", "CARGO_LLVM_COV_TARGET_DIR",
                     "CARGO_LLVM_COV_BUILD_DIR", "__CARGO_LLVM_COV_RUSTC_WRAPPER",
                     "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
                     "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES"}
    coverage_wrapper = Path(env.get("RUSTC_WRAPPER", "")).name == "cargo-llvm-cov" \
        or any(key.startswith("__CARGO_LLVM_COV_") for key in env)
    for key in coverage_keys: env.pop(key, None)
    if coverage_wrapper: env.pop("RUSTC_WRAPPER", None)
    env["CARGO_TARGET_DIR"] = str(target_dir.resolve())
    return env


def coverage_environment(target_dir: Path = COVERAGE_TARGET) -> dict[str, str]:
    base = coverage_base_environment(target_dir)
    output = run(["cargo", "llvm-cov", "show-env", "--sh"], env=base, capture=True).stdout
    return parse_coverage_environment(output, base)


def workspace_packages() -> list[str]:
    output = run(["cargo", "metadata", "--locked", "--format-version=1", "--no-deps"],
                 capture=True).stdout
    metadata = json.loads(output)
    members = metadata.get("workspace_members")
    packages = metadata.get("packages")
    if type(members) is not list or not members or type(packages) is not list:
        raise ValueError("cargo workspace metadata missing")
    by_id = {package.get("id"): package.get("name") for package in packages
             if type(package) is dict}
    names = [by_id.get(member) for member in members]
    if any(type(name) is not str or not name for name in names) or len(names) != len(set(names)):
        raise ValueError("cargo workspace package selection invalid")
    return sorted(names)


def preflight_coverage_target(target_dir: Path = COVERAGE_TARGET) -> None:
    """Make Cargo itself validate and clean the existing private coverage cache."""
    target_dir = target_dir.resolve()
    command = ["cargo", "clean", "--target-dir", str(target_dir)]
    for package in workspace_packages():
        command.extend(["--package", package])
    run(command, env=coverage_base_environment(target_dir))


def coverage_exclusion_regex() -> str:
    lines = (ROOT / "scripts/coverage-exclusions.txt").read_text().splitlines()
    if len(lines) != 1 or not lines[0]:
        raise ValueError("coverage exclusions must contain exactly one non-empty regex")
    return lines[0]


def select_binary(release: bool, env: dict[str, str] | None = None) -> Path:
    command = ["cargo", "test", "-p", "grenadiers-server", "--lib", "--locked", "--no-run",
               "--message-format=json"]
    if release: command.insert(6, "--release")
    messages = run(command, env=env, capture=True, merge_stderr=False).stdout
    selected = subprocess.run(["node", "scripts/measure-game-artifact.mjs", "select"], cwd=ROOT,
                              input=messages, text=True, check=True, stdout=subprocess.PIPE).stdout.strip()
    binary = Path(selected).resolve()
    if binary.is_symlink() or not binary.is_file(): raise ValueError("server test binary missing")
    listed = run([str(binary), "--list"], env=env, capture=True).stdout.splitlines()
    if any(sum(line == f"{test}: test" for line in listed) != 1
           for test in (HEAVY_TEST, *ISOLATED_TESTS, INSTRUMENTATION_PROBE_TEST)):
        raise ValueError("exact server tests missing or duplicated")
    return binary


def execute_ordinary_and_isolated(profile: str, binary: Path, *, release: bool = False,
                                  env: dict[str, str] | None = None,
                                  raw_dir: Path | None = None) -> dict[str, Any]:
    ordinary = ["cargo", "test", "--workspace", "--locked"]
    if release: ordinary.insert(3, "--release")
    skips = [value for test in (HEAVY_TEST, *ISOLATED_TESTS) for value in ("--skip", test)]
    run([*ordinary, "--", *skips, "--show-output"], env=env)
    before = set(raw_dir.glob("*.profraw")) if raw_dir else set()
    if raw_dir and not before:
        raise ValueError("ordinary instrumented coverage produced no raw profiles")
    run([str(binary), *ISOLATED_TESTS, "--exact", "--show-output", "--test-threads=1"], env=env)
    created = sorted(set(raw_dir.glob("*.profraw")) - before) if raw_dir else []
    if raw_dir and not created:
        raise ValueError("isolated coverage test produced no raw profile")
    result = {"names": list(ISOLATED_TESTS), "profile": profile, "result": "completed",
              "executionsPerTest": 1, "concurrentOrdinaryOrMaskWork": False,
              "rawProfileCount": len(created)}
    return result


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def file_identity(path: Path, require_executable: bool = True) -> dict[str, Any]:
    canonical = path.resolve()
    if path.is_symlink() or not canonical.is_file() \
            or (require_executable and not os.access(canonical, os.X_OK)):
        raise ValueError("workspace test executable missing or unsafe")
    return {"path": str(canonical), "sha256": file_sha256(canonical),
            "mode": stat.S_IMODE(canonical.stat().st_mode)}


def capture_workspace_executables(env: dict[str, str]) -> tuple[list[dict[str, Any]], Path]:
    """Capture every executable emitted by one complete instrumented workspace test build."""
    target_text = env.get("CARGO_LLVM_COV_TARGET_DIR")
    if target_text is None or not Path(target_text).is_absolute():
        raise ValueError("cargo-llvm-cov target directory unavailable")
    coverage_target = Path(target_text).resolve()
    messages = run(["cargo", "test", "--workspace", "--locked", "--no-run",
                    "--message-format=json"], env=env, capture=True, merge_stderr=False).stdout
    artifacts: list[dict[str, Any]] = []
    build_finished: list[bool] = []
    seen: set[Path] = set()
    for line in messages.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError("malformed cargo workspace artifact stream") from error
        if message.get("reason") == "build-finished":
            build_finished.append(message.get("success"))
            continue
        if message.get("reason") != "compiler-artifact" or not message.get("executable"):
            continue
        profile, target = message.get("profile"), message.get("target")
        if type(profile) is not dict or profile.get("test") is not True or type(target) is not dict:
            raise ValueError("malformed cargo workspace test artifact")
        reported_path = Path(message["executable"])
        identity = file_identity(reported_path)
        path = Path(identity["path"])
        try:
            path.relative_to(coverage_target)
        except ValueError as error:
            raise ValueError("workspace test executable outside coverage target") from error
        if path in seen:
            raise ValueError("duplicate cargo workspace test executable")
        seen.add(path)
        identity.update({"packageId": message.get("package_id"), "targetName": target.get("name"),
                         "targetKind": target.get("kind")})
        if type(identity["packageId"]) is not str or not identity["packageId"] \
                or type(identity["targetName"]) is not str or not identity["targetName"] \
                or type(identity["targetKind"]) is not list or not identity["targetKind"] \
                or any(type(kind) is not str for kind in identity["targetKind"]):
            raise ValueError("incomplete cargo workspace test artifact")
        artifacts.append(identity)
    if build_finished != [True]:
        raise ValueError("cargo workspace artifact stream did not finish exactly once")
    if not artifacts:
        raise ValueError("cargo workspace emitted no test executables")

    heavy: list[Path] = []
    for artifact in artifacts:
        binary = Path(artifact["path"])
        listed = run([str(binary), "--list"], env=env, capture=True).stdout.splitlines()
        has_heavy = sum(line == f"{HEAVY_TEST}: test" for line in listed)
        isolated_counts = [sum(line == f"{test}: test" for line in listed)
                           for test in ISOLATED_TESTS]
        has_probe = sum(line == f"{INSTRUMENTATION_PROBE_TEST}: test" for line in listed)
        if has_heavy or any(isolated_counts) or has_probe:
            if has_heavy != 1 or any(count != 1 for count in isolated_counts) or has_probe != 1:
                raise ValueError("exact server tests missing or duplicated")
            heavy.append(binary)
    if len(heavy) != 1:
        raise ValueError("instrumented heavy server test executable selection is not exact")
    return artifacts, heavy[0]


def verify_file_identities(artifacts: list[dict[str, Any]]) -> None:
    if not artifacts:
        raise ValueError("workspace test executable inventory empty")
    for artifact in artifacts:
        current = file_identity(Path(artifact["path"]))
        if current["path"] != artifact["path"] or current["sha256"] != artifact["sha256"] \
                or current["mode"] != artifact["mode"]:
            raise ValueError("workspace test executable changed")


def verify_instrumented_binary(binary: Path, env: dict[str, str], raw_dir: Path) -> None:
    raw_dir.mkdir()
    probe_env = dict(env); probe_env["LLVM_PROFILE_FILE"] = str(raw_dir / "%p-%18m.profraw")
    before = hashlib.sha256(binary.read_bytes()).hexdigest()
    run([str(binary), INSTRUMENTATION_PROBE_TEST, "--exact", "--test-threads=1"], env=probe_env)
    if not list(raw_dir.glob("*.profraw")):
        raise ValueError("coverage binary instrumentation probe produced no raw profiles")
    if hashlib.sha256(binary.read_bytes()).hexdigest() != before:
        raise ValueError("coverage binary changed during instrumentation probe")


def verify_summary(output: str, plan: dict[str, Any], profile: str, index: int) -> None:
    rows = [json.loads(line[len(SUMMARY_PREFIX):]) for line in output.splitlines()
            if line.startswith(SUMMARY_PREFIX)]
    expected = plan["ranges"][index]
    if len(rows) != 1 or rows[0] != {"schemaVersion": 1, "source": plan["source"],
            "profile": profile, "selection": "partition", "partitionIndex": index,
            "partitionCount": plan["partitionCount"], "startOrdinal": expected["startOrdinal"],
            "endOrdinalExclusive": expected["endOrdinalExclusive"],
            "observedCount": expected["endOrdinalExclusive"] - expected["startOrdinal"],
            "totalCardinality": plan["totalCardinality"]}:
        raise ValueError("partition completion summary mismatch")


def execute_profile(profile: str, binary: Path, plan: dict[str, Any], workers: int,
                    base_env: dict[str, str] | None = None, raw_root: Path | None = None) -> dict[str, Any]:
    started = time.monotonic(); binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    buckets = schedule(plan, workers)

    def worker(slot: int, items: list[dict[str, int]]) -> str | None:
        raw_dir = raw_root / f"worker-{slot}" if raw_root else None
        if raw_dir: raw_dir.mkdir(parents=True)
        for item in items:
            env = dict(base_env or os.environ)
            env.update(GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX=str(item["index"]),
                       GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_COUNT=str(plan["partitionCount"]),
                       GRENADIERS_SERVER_REPLAY_ITEM_MASK_SOURCE=plan["source"],
                       GRENADIERS_SERVER_REPLAY_ITEM_MASK_PROFILE=profile)
            if raw_dir: env["LLVM_PROFILE_FILE"] = str(raw_dir / "%p-%18m.profraw")
            output = run([str(binary), HEAVY_TEST, "--exact", "--show-output", "--test-threads=1"],
                         env=env, capture=True).stdout
            verify_summary(output, plan, profile, item["index"])
        if raw_dir and not list(raw_dir.glob("*.profraw")): raise ValueError("coverage worker produced no raw profiles")
        return str(raw_dir) if raw_dir else None

    raw_dirs = parallel_map(buckets, worker)
    if hashlib.sha256(binary.read_bytes()).hexdigest() != binary_hash: raise ValueError("test binary changed")
    result = {"result": "completed", "binary": str(binary), "binarySHA256": binary_hash,
              "partitions": plan["partitionCount"], "workers": len(buckets),
              "retries": 0, "durationSeconds": round(time.monotonic() - started, 3)}
    if raw_root: result["rawDirectories"] = raw_dirs
    return result


def with_report_profiles(target_dir: Path, raw_dirs: list[Path], owner: str,
                         report: Callable[[], Any]) -> tuple[Any, int]:
    """Expose one exact private profile set where cargo-llvm-cov 0.9.1 discovers it."""
    if not target_dir.is_absolute() or target_dir.is_symlink():
        raise ValueError("unsafe coverage report target/owner")
    target_dir = target_dir.resolve()
    if not target_dir.is_dir() \
            or re.fullmatch(r"[a-z0-9-]+", owner) is None:
        raise ValueError("unsafe coverage report target/owner")
    prefix = f"grenadiers-local-server-{owner}-"
    for stale in target_dir.glob("grenadiers-local-server-*.profraw"):
        if stale.is_symlink() or not stale.is_file(): raise ValueError("unsafe owned stale profile")
        stale.unlink()
    if list(target_dir.glob("*.profraw")):
        raise ValueError("unowned coverage profiles would contaminate report")
    staged: list[Path] = []
    try:
        for group, directory in enumerate(raw_dirs):
            if not directory.is_absolute() or directory.is_symlink() or not directory.is_dir():
                raise ValueError("unsafe coverage raw directory")
            directory = directory.resolve()
            files = sorted(directory.glob("*.profraw"))
            if not files: raise ValueError("coverage raw directory empty")
            for index, source in enumerate(files):
                if source.is_symlink() or not source.is_file() or source.resolve().parent != directory.resolve():
                    raise ValueError("unsafe coverage raw profile")
                destination = target_dir / f"{prefix}{group}-{index}.profraw"
                with destination.open("xb") as output_file, source.open("rb") as input_file:
                    shutil.copyfileobj(input_file, output_file)
                staged.append(destination)
        expected = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in staged}
        if set(target_dir.glob("*.profraw")) != set(expected):
            raise ValueError("coverage report profile discovery mismatch")
        result = report()
        if set(target_dir.glob("*.profraw")) != set(expected) or any(
                hashlib.sha256(path.read_bytes()).hexdigest() != digest
                for path, digest in expected.items()):
            raise ValueError("coverage report profile set changed during report")
        return result, len(staged)
    finally:
        for path in staged:
            path.unlink(missing_ok=True)


def stage_report_executables(source_target: Path, report_target: Path,
                             artifacts: list[dict[str, Any]]) -> list[dict[str, Any]]:
    source_target = source_target.resolve()
    if report_target.is_symlink() or not report_target.is_dir() or any(report_target.iterdir()):
        raise ValueError("coverage report target is not an owned empty directory")
    verify_file_identities(artifacts)
    staged: list[dict[str, Any]] = []
    for artifact in artifacts:
        source = Path(artifact["path"])
        try:
            relative = source.relative_to(source_target)
        except ValueError as error:
            raise ValueError("workspace test executable outside coverage target") from error
        destination = report_target / relative
        if destination.exists() or destination.is_symlink():
            raise ValueError("duplicate staged report executable")
        destination.parent.mkdir(parents=True, exist_ok=True)
        with source.open("rb") as input_file, destination.open("xb") as output_file:
            shutil.copyfileobj(input_file, output_file)
        destination.chmod(artifact["mode"])
        identity = file_identity(destination)
        if identity["sha256"] != artifact["sha256"] or identity["mode"] != artifact["mode"]:
            raise ValueError("staged report executable identity mismatch")
        staged.append(identity)
    return staged


def report_environment(env: dict[str, str], report_target: Path) -> dict[str, str]:
    private = dict(env); target = str(report_target.resolve())
    for key in ("CARGO_TARGET_DIR", "CARGO_LLVM_COV_TARGET_DIR",
                "CARGO_BUILD_BUILD_DIR", "CARGO_LLVM_COV_BUILD_DIR"):
        private[key] = target
    return private


def raw_profile_inventory(raw_dirs: list[Path]) -> list[dict[str, Any]]:
    inventory: list[dict[str, Any]] = []
    for directory in raw_dirs:
        if not directory.is_absolute() or directory.is_symlink() or not directory.is_dir():
            raise ValueError("unsafe coverage raw directory")
        files = sorted(directory.glob("*.profraw"))
        if not files:
            raise ValueError("coverage raw directory empty")
        inventory.append({"directory": str(directory.resolve()),
                          "files": [file_identity(path, require_executable=False) for path in files]})
    return inventory


def write_execution_proof(path: Path, source: str, tree: str, compiler: dict[str, str],
                          plan: dict[str, Any], profiles: dict[str, Any],
                          artifacts: list[dict[str, Any]], raw_dirs: list[Path]) -> None:
    proof = {"schemaVersion": 1, "source": source, "tree": tree, "compiler": compiler,
             "plan": plan, "completedProfiles": profiles, "workspaceExecutables": artifacts,
             "rawProfiles": raw_profile_inventory(raw_dirs)}
    path.write_text(json.dumps(proof, indent=2, sort_keys=True) + "\n")


def report_lcov(env: dict[str, str], raw_dirs: list[Path], output: Path,
                owner: str, artifacts: list[dict[str, Any]], report_target: Path) -> dict[str, Any]:
    target_text = env.get("CARGO_LLVM_COV_TARGET_DIR")
    if target_text is None or not Path(target_text).is_absolute():
        raise ValueError("cargo-llvm-cov target directory unavailable")
    staged: list[dict[str, Any]] = []
    owns_report_target = False

    def report() -> None:
        run(["cargo", "llvm-cov", "report", "--workspace", "--locked", "--fail-under-lines", "90",
             "--ignore-filename-regex", coverage_exclusion_regex(), "--lcov", "--output-path",
             str(output)], env=private_env)

    try:
        try:
            report_target.mkdir()
        except FileExistsError as error:
            raise ValueError("coverage report target must be fresh") from error
        owns_report_target = True
        staged = stage_report_executables(Path(target_text), report_target, artifacts)
        private_env = report_environment(env, report_target)
        _, count = with_report_profiles(report_target, raw_dirs, owner, report)
    finally:
        try:
            if staged:
                verify_file_identities(staged)
        finally:
            if owns_report_target:
                shutil.rmtree(report_target)
    if output.is_symlink() or not output.is_file() or output.stat().st_size == 0:
        raise ValueError("official LCOV report missing or empty")
    return {"lineFloor": 90, "rawProfiles": count, "reporter": "cargo-llvm-cov"}


def execute_profiles(source: str, tree: str, compiler: dict[str, str], workers: int,
                     temp: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    plan_path = temp / "server-mask-plan.json"
    run(["node", "scripts/local-server-mask-plan.mjs", str(ROOT), source, str(plan_path)])
    plan = validate_plan(json.loads(plan_path.read_text()), source); profiles = {}
    for profile, release in (("debug", False), ("release", True)):
        binary = select_binary(release)
        isolated = execute_ordinary_and_isolated(profile, binary, release=release)
        profiles[profile] = execute_profile(profile, binary, plan, workers)
        profiles[profile]["isolatedTest"] = isolated
    cov_env = coverage_environment()
    artifacts, coverage_binary = capture_workspace_executables(cov_env)
    baseline = temp / "coverage-baseline"; baseline.mkdir()
    cov_env["LLVM_PROFILE_FILE"] = str(baseline / "%p-%18m.profraw")
    isolated = execute_ordinary_and_isolated("coverage", coverage_binary, env=cov_env,
                                             raw_dir=baseline)
    verify_instrumented_binary(coverage_binary, cov_env, temp / "coverage-probe")
    coverage_raw = temp / "coverage-workers"; coverage_raw.mkdir()
    profiles["coverage"] = execute_profile("coverage", coverage_binary, plan, workers,
                                            cov_env, coverage_raw)
    profiles["coverage"]["isolatedTest"] = isolated
    worker_dirs = [Path(path) for path in profiles["coverage"].pop("rawDirectories")]
    verify_file_identities(artifacts)
    report_raw_dirs = [baseline, *worker_dirs]
    write_execution_proof(temp / "execution-proof.json", source, tree, compiler, plan, profiles,
                          artifacts, report_raw_dirs)
    scratch_report = temp / "coverage.lcov"
    coverage = report_lcov(cov_env, report_raw_dirs, scratch_report, "all", artifacts,
                           temp / "coverage-report-target")
    coverage["output"] = "target/coverage.lcov"
    profiles["coverage"]["coverage"] = coverage
    return plan, profiles


def main() -> None:
    parser = argparse.ArgumentParser(); parser.add_argument("--workers", type=int, default=4)
    args = parser.parse_args(); started = time.monotonic()
    if not 1 <= args.workers <= 8: raise SystemExit("--workers must be from 1 through 8")
    source, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
    if git("status", "--porcelain=v1", "--untracked-files=all"):
        raise SystemExit("server-parallel requires a clean worktree; no clean result was produced")
    cargo_version = run(["cargo", "--version"], capture=True).stdout.strip()
    rustc_version = run(["rustc", "-Vv"], capture=True).stdout.strip()
    preflight_coverage_target()
    run(["bash", "-n", "scripts/measure-game.sh"])
    run(["node", "--test", "scripts/measure-game-artifact.test.mjs"])
    run(["node", "--test", "scripts/ci-server-mask-contract.test.mjs"])
    run(["cargo", "fmt", "--all", "--", "--check"])
    run(["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"])
    run(["cargo", "build", "--workspace", "--locked"])
    temp = Path(tempfile.mkdtemp(prefix="grenadiers-server-parallel-"))
    try:
        compiler = {"cargo": cargo_version, "rustc": rustc_version}
        plan, profiles = execute_profiles(source, tree, compiler, args.workers, temp)
        if git("rev-parse", "HEAD") != source or git("rev-parse", "HEAD^{tree}") != tree \
                or git("status", "--porcelain=v1", "--untracked-files=all") \
                or run(["cargo", "--version"], capture=True).stdout.strip() != cargo_version \
                or run(["rustc", "-Vv"], capture=True).stdout.strip() != rustc_version:
            raise SystemExit("source or compiler identity changed; no result was produced")
        final_path = ROOT / "target/coverage.lcov"; final_path.parent.mkdir(exist_ok=True)
        shutil.copyfile(temp / "coverage.lcov", final_path)
        summary = {"result": "completed-clean-local-verification", "source": source, "tree": tree,
                   "workers": args.workers, "retries": 0,
                   "compiler": compiler,
                   "catalogItems": len(plan["catalog"]), "maskCount": plan["totalCardinality"],
                   "partitionCount": plan["partitionCount"], "profiles": profiles,
                   "durationSeconds": round(time.monotonic() - started, 3)}
    except BaseException:
        print(f"LOCAL_SERVER_PARALLEL_RETAINED_SCRATCH={temp}", file=sys.stderr, flush=True)
        raise
    shutil.rmtree(temp)
    print("LOCAL_SERVER_PARALLEL_SUMMARY=" + json.dumps(summary, sort_keys=True))


if __name__ == "__main__": main()
