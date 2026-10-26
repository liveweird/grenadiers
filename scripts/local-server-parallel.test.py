#!/usr/bin/env python3
import hashlib
import importlib.util
import json
import os
import subprocess
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("local_server_parallel", HERE / "local-server-parallel.py")
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)


with tempfile.TemporaryDirectory(prefix="grenadiers-cargo-json-stream-") as temporary:
    binary = Path(temporary) / "server-libtest"
    binary.write_text("fixture executable")
    cargo_message = json.dumps({"reason": "compiler-artifact",
        "target": {"name": "grenadiers_server", "kind": ["lib"]},
        "profile": {"test": True}, "executable": str(binary)}) + "\n"
    original_run = module.run

    def fake_run(command, **kwargs):
        if command[0] == "cargo":
            assert kwargs.get("capture") is True and kwargs.get("merge_stderr") is False
            return subprocess.CompletedProcess(command, 0, stdout=cargo_message)
        if command[0] == str(binary.resolve()):
            return subprocess.CompletedProcess(command, 0, stdout=
                f"{module.HEAVY_TEST}: test\n{module.INSTRUMENTATION_PROBE_TEST}: test\n")
        raise AssertionError(f"unexpected command: {command}")

    module.run = fake_run
    try:
        assert module.select_binary(False) == binary.resolve()
    finally:
        module.run = original_run

    inherited = {"CARGO_LLVM_COV": "1", "CARGO_LLVM_COV_TARGET_DIR": "/stale-target",
        "__CARGO_LLVM_COV_RUSTC_WRAPPER": "1", "LLVM_PROFILE_FILE": "/stale.profraw",
        "RUSTC_WRAPPER": "/tool/cargo-llvm-cov", "RUSTFLAGS": "-C debuginfo=1"}
    previous = {key: os.environ.get(key) for key in inherited}
    try:
        os.environ.update(inherited)
        base = module.coverage_base_environment(Path(temporary) / "isolated-coverage")
        assert not any(key in base for key in inherited if key != "RUSTFLAGS")
        assert base["RUSTFLAGS"] == inherited["RUSTFLAGS"]
        assert base["CARGO_TARGET_DIR"].endswith("/isolated-coverage")
    finally:
        for key, value in previous.items():
            if value is None: os.environ.pop(key, None)
            else: os.environ[key] = value

with tempfile.TemporaryDirectory(prefix="grenadiers-workspace-artifacts-") as temporary:
    root = Path(temporary)
    binaries = [root / name for name in ("core-libtest", "server-libtest", "server-main-test")]
    for binary in binaries:
        binary.write_text("fixture executable")
        binary.chmod(0o755)
    messages = []
    for index, binary in enumerate(binaries):
        messages.append(json.dumps({"reason": "compiler-artifact", "package_id": f"pkg-{index}",
            "target": {"name": binary.name, "kind": ["lib" if index < 2 else "bin"]},
            "profile": {"test": True}, "executable": str(binary)}))
    messages.append(json.dumps({"reason": "build-finished", "success": True}))
    original_run = module.run

    def fake_workspace_run(command, **kwargs):
        if command[:3] == ["cargo", "test", "--workspace"]:
            return subprocess.CompletedProcess(command, 0, stdout="\n".join(messages) + "\n")
        if command[0] == str(binaries[1].resolve()):
            return subprocess.CompletedProcess(command, 0, stdout=
                f"{module.HEAVY_TEST}: test\n{module.INSTRUMENTATION_PROBE_TEST}: test\n")
        if command[0] in {str(binaries[0].resolve()), str(binaries[2].resolve())}:
            return subprocess.CompletedProcess(command, 0, stdout="unrelated: test\n")
        raise AssertionError(f"unexpected command: {command}")

    module.run = fake_workspace_run
    try:
        artifacts, selected_binary = module.capture_workspace_executables({})
        assert selected_binary == binaries[1].resolve() and len(artifacts) == 3
        assert {item["targetKind"][0] for item in artifacts} == {"lib", "bin"}
        empty_messages = messages
        messages = [json.dumps({"reason": "build-finished", "success": True})]
        try:
            module.capture_workspace_executables({})
        except ValueError as error:
            assert "no test executables" in str(error)
        else:
            raise AssertionError("empty workspace executable selection accepted")
        messages = ["not-json"]
        try:
            module.capture_workspace_executables({})
        except ValueError as error:
            assert "malformed" in str(error)
        else:
            raise AssertionError("malformed workspace executable selection accepted")
        messages = empty_messages
    finally:
        module.run = original_run

progress = "import sys; print('{\"reason\":\"build-finished\",\"success\":true}'); print('Finished test profile', file=sys.stderr)"
captured = module.run(["python3", "-c", progress], capture=True, merge_stderr=False)
assert json.loads(captured.stdout)["success"] is True and "Finished" not in captured.stdout


def plan(count):
    capacity = 8
    requested_total = count * capacity
    catalog = [1 << index for index in range(requested_total.bit_length() - 1)]
    total = 2 ** len(catalog); count = (total + capacity - 1) // capacity
    return module.validate_plan({"schemaVersion": 1, "source": "a" * 40, "catalog": catalog,
        "capacity": capacity, "totalCardinality": total, "partitionCount": count,
        "ranges": [{"index": index, "startOrdinal": index * capacity,
                    "endOrdinalExclusive": min((index + 1) * capacity, total)} for index in range(count)]},
        "a" * 40)


for count in (1, 4, 8):
    value = plan(count); buckets = module.schedule(value, 4)
    indexes = [item["index"] for bucket in buckets for item in bucket]
    assert sorted(indexes) == list(range(value["partitionCount"]))
    assert len(indexes) == len(set(indexes)) and all(buckets)

try:
    bad = plan(4); bad["ranges"][1]["startOrdinal"] += 1
    module.validate_plan(bad, "a" * 40)
except ValueError:
    pass
else:
    raise AssertionError("gapped range accepted")
try:
    bad = plan(4); bad["ranges"][1] = dict(bad["ranges"][0], index=1)
    module.validate_plan(bad, "a" * 40)
except ValueError:
    pass
else:
    raise AssertionError("duplicated range accepted")

single = plan(1); selected = single["ranges"][0]
zero = {"schemaVersion": 1, "source": single["source"], "profile": "debug",
        "selection": "partition", "partitionIndex": 0, "partitionCount": 1,
        "startOrdinal": selected["startOrdinal"], "endOrdinalExclusive": selected["endOrdinalExclusive"],
        "observedCount": 0, "totalCardinality": single["totalCardinality"]}
try:
    module.verify_summary(module.SUMMARY_PREFIX + __import__("json").dumps(zero), single, "debug", 0)
except ValueError:
    pass
else:
    raise AssertionError("zero-selected partition accepted")

executed = []
assert module.parallel_map([[{"index": 0}], [{"index": 1}]],
                           lambda slot, items: executed.append((slot, items[0]["index"]))) == [None, None]
assert sorted(executed) == [(0, 0), (1, 1)]
try:
    module.parallel_map([[{"index": 0}], [{"index": 1}]],
                        lambda slot, _items: (_ for _ in ()).throw(RuntimeError("child failed"))
                        if slot == 1 else slot)
except RuntimeError as error:
    assert str(error) == "child failed"
else:
    raise AssertionError("worker failure not propagated")


with tempfile.TemporaryDirectory(prefix="grenadiers-llvm-cov-discovery-") as temporary:
    fixture_root = Path(temporary)
    fixture = fixture_root / "current"
    stale_fixture = fixture_root / "stale"
    for workspace in (fixture, stale_fixture):
        (workspace / "src").mkdir(parents=True)
        (workspace / "Cargo.toml").write_text(
            '[package]\nname = "llvm-cov-discovery-fixture"\nversion = "0.0.0"\nedition = "2021"\n'
        )
    (fixture / "src/lib.rs").write_text(
        "pub fn covered() -> u8 { 1 }\n"
        "#[cfg(test)] mod test_support;\n"
        "#[cfg(test)] mod tests { use super::*;\n"
        "#[test] fn heavy() { assert_eq!(covered(), 1); }\n"
        "#[test] fn blank_database_url_disables_postgres() { "
        "assert_eq!(test_support::helper(), 7); } }\n"
    )
    (fixture / "src/main.rs").write_text(
        "fn value() -> u8 { 3 }\nfn main() { assert_eq!(value(), 3); }\n"
        "#[cfg(test)] mod tests { #[test] fn main_is_covered() { super::main(); } }\n"
    )
    (fixture / "src/test_support.rs").write_text("pub fn helper() -> u8 { 7 }\n")
    (fixture / "scripts").mkdir()
    (fixture / "scripts/coverage-exclusions.txt").write_text(
        "(^|/)([^/]*_)?test_support\\.rs$\n")
    (stale_fixture / "src/lib.rs").write_text(
        "\n".join(f"pub fn stale_{index}(value: bool) -> u8 {{ if value {{ 1 }} else {{ 0 }} }}"
                  for index in range(64)) + "\n"
    )
    with (stale_fixture / "Cargo.toml").open("a") as manifest_file:
        manifest_file.write("[features]\nstale = []\n")

    original_root = module.ROOT
    original_heavy = module.HEAVY_TEST
    original_probe = module.INSTRUMENTATION_PROBE_TEST
    module.ROOT = fixture
    module.HEAVY_TEST = "tests::heavy"
    module.INSTRUMENTATION_PROBE_TEST = "tests::blank_database_url_disables_postgres"
    try:
        manifest = str(fixture / "Cargo.toml")
        subprocess.run(["cargo", "test", "--manifest-path", manifest], check=True, cwd=fixture,
                       stdout=subprocess.DEVNULL)
        ordinary = [path for path in (fixture / "target/debug/deps").iterdir()
                    if path.is_file() and os.access(path, os.X_OK)]
        assert len(ordinary) == 2
        ordinary_hashes = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in ordinary}

        invalid_target = fixture_root / "invalid-target"
        invalid_target.mkdir()
        (invalid_target / "untrusted-object").write_text("must survive")
        try:
            module.preflight_coverage_target(invalid_target)
        except subprocess.CalledProcessError:
            pass
        else:
            raise AssertionError("invalid existing coverage cache accepted")
        assert (invalid_target / "untrusted-object").read_text() == "must survive"
        absent_target = fixture_root / "absent-target"
        module.preflight_coverage_target(absent_target)

        coverage_target = fixture_root / "shared-coverage-target"
        cov_env = module.coverage_environment(coverage_target)
        assert Path(cov_env["CARGO_LLVM_COV_TARGET_DIR"]).resolve() == coverage_target.resolve()

        stale_messages = subprocess.run(
            ["cargo", "test", "--manifest-path", str(stale_fixture / "Cargo.toml"),
             "--features", "stale", "--no-run", "--message-format=json"],
            check=True, cwd=stale_fixture, env=cov_env,
            text=True, stdout=subprocess.PIPE).stdout
        stale_binaries = {Path(item["executable"]).resolve()
                          for item in map(json.loads, stale_messages.splitlines())
                          if item.get("reason") == "compiler-artifact" and item.get("executable")
                          and item.get("profile", {}).get("test") is True}
        assert stale_binaries

        artifacts, coverage_binary = module.capture_workspace_executables(cov_env)
        current_binaries = {Path(item["path"]) for item in artifacts}
        assert len(current_binaries) == 2
        assert {item["targetKind"][0] for item in artifacts} == {"lib", "bin"}
        assert stale_binaries.isdisjoint(current_binaries)
        assert all(path.is_file() for path in stale_binaries)
        discovered = {path.resolve() for path in coverage_target.rglob("*")
                      if path.is_file() and os.access(path, os.X_OK)}
        assert stale_binaries < discovered and current_binaries <= discovered

        baseline = fixture_root / "baseline"; baseline.mkdir()
        baseline_env = dict(cov_env)
        baseline_env["LLVM_PROFILE_FILE"] = str(baseline / "%p-%18m.profraw")
        subprocess.run(["cargo", "test", "--manifest-path", manifest], check=True, cwd=fixture,
                       env=baseline_env, stdout=subprocess.DEVNULL)
        assert list(baseline.glob("*.profraw"))

        worker = fixture_root / "worker"
        module.verify_instrumented_binary(coverage_binary, cov_env, worker)
        assert list(worker.glob("*.profraw"))
        target = Path(cov_env["CARGO_LLVM_COV_TARGET_DIR"]).resolve()
        for incidental_profile in target.glob("*.profraw"):
            incidental_profile.unlink()
        report_path = fixture_root / "coverage.lcov"

        def contaminated_report():
            return subprocess.run(
                ["cargo", "llvm-cov", "report", "--manifest-path", manifest,
                 "--fail-under-lines", "90", "--ignore-filename-regex",
                 module.coverage_exclusion_regex(), "--lcov", "--output-path", str(report_path)],
                check=False, cwd=fixture, env=cov_env, stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL)

        contaminated, profile_count = module.with_report_profiles(
            target, [baseline, worker], "stale-object-proof", contaminated_report)
        assert contaminated.returncode != 0

        report_target = fixture_root / "private-report-target"
        official = module.report_lcov(cov_env, [baseline, worker],
                                      fixture_root / "production.lcov", "production",
                                      artifacts, report_target)
        assert official["lineFloor"] == 90 and official["rawProfiles"] == profile_count
        assert official["reporter"] == "cargo-llvm-cov" and not report_target.exists()
        official_sources = [line[3:] for line in (fixture_root / "production.lcov").read_text().splitlines()
                            if line.startswith("SF:")]
        assert any(Path(source).name == "lib.rs" for source in official_sources)
        assert any(Path(source).name == "main.rs" for source in official_sources)
        assert not any(Path(source).name == "test_support.rs" for source in official_sources)

        for path, digest in ordinary_hashes.items():
            assert path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest() == digest

        proof_path = fixture_root / "execution-proof.json"
        proof_plan = plan(1)
        proof_profiles = {"coverage": {"result": "completed"}}
        module.write_execution_proof(
            proof_path, "a" * 40, "b" * 40, {"cargo": "cargo fixture", "rustc": "rustc fixture"},
            proof_plan, proof_profiles, artifacts, [baseline, worker])
        proof = json.loads(proof_path.read_text())
        assert proof["source"] == "a" * 40 and proof["tree"] == "b" * 40
        assert proof["completedProfiles"] == proof_profiles
        assert len(proof["workspaceExecutables"]) == 2 and len(proof["rawProfiles"]) == 2

        json_path = fixture_root / "coverage.json"
        def json_report():
            return subprocess.run(
                ["cargo", "llvm-cov", "report", "--manifest-path", manifest, "--json",
                 "--summary-only", "--output-path", str(json_path)], check=True, cwd=fixture,
                env=cov_env, stdout=subprocess.DEVNULL)
        module.with_report_profiles(target, [baseline, worker], "actual-json", json_report)
        llvm_json = json.loads(json_path.read_text())
        assert set(llvm_json) == {"cargo_llvm_cov", "data", "type", "version"}
        assert llvm_json["type"] == "llvm.coverage.json.export" and len(llvm_json["data"]) == 1

        def impossible_floor():
            return subprocess.run(
                ["cargo", "llvm-cov", "report", "--manifest-path", manifest,
                 "--fail-under-lines", "101", "--ignore-filename-regex",
                 module.coverage_exclusion_regex(), "--lcov", "--output-path",
                 str(fixture_root / "impossible.lcov")],
                check=False, cwd=fixture, env=cov_env, stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL)
        failed, _ = module.with_report_profiles(
            target, [baseline, worker], "floor-rejection", impossible_floor)
        assert failed.returncode != 0

        (target / "grenadiers-local-server-abandoned-0-0.profraw").write_bytes(b"owned stale")
        module.with_report_profiles(target, [baseline, worker], "stale-recovery", lambda: None)
        assert not list(target.glob("*.profraw"))

        foreign = target / "foreign.profraw"; foreign.write_bytes(b"not accepted")
        try:
            module.with_report_profiles(target, [baseline, worker], "contaminated", lambda: None)
        except ValueError as error:
            assert "contaminate" in str(error)
        else:
            raise AssertionError("unowned report profile accepted")
        foreign.unlink()

        late = target / "late.profraw"
        try:
            module.with_report_profiles(
                target, [baseline, worker], "late-change", lambda: late.write_bytes(b"late profile"))
        except ValueError as error:
            assert "changed during report" in str(error)
        else:
            raise AssertionError("late report profile accepted")
        late.unlink()

        outside_artifact = module.file_identity(ordinary[0])
        malformed_target = fixture_root / "malformed-report-target"
        try:
            module.report_lcov(cov_env, [baseline, worker], fixture_root / "malformed.lcov",
                               "malformed", [outside_artifact], malformed_target)
        except ValueError as error:
            assert "outside coverage target" in str(error)
        else:
            raise AssertionError("out-of-target report executable accepted")
        assert not malformed_target.exists()

        cleanup_target = fixture_root / "failed-report-target"
        original_run = module.run
        def failing_report_run(command, **kwargs):
            if command[:3] == ["cargo", "llvm-cov", "report"]:
                for key in ("CARGO_TARGET_DIR", "CARGO_LLVM_COV_TARGET_DIR",
                            "CARGO_BUILD_BUILD_DIR", "CARGO_LLVM_COV_BUILD_DIR"):
                    assert Path(kwargs["env"][key]).resolve() == cleanup_target.resolve()
                relative = Path(artifacts[0]["path"]).relative_to(target)
                staged_binary = cleanup_target / relative
                staged_binary.write_bytes(staged_binary.read_bytes() + b"report mutation")
                return subprocess.CompletedProcess(command, 0)
            return original_run(command, **kwargs)
        module.run = failing_report_run
        try:
            try:
                module.report_lcov(cov_env, [baseline, worker], fixture_root / "failed.lcov",
                                   "failed", artifacts, cleanup_target)
            except ValueError as error:
                assert "changed" in str(error)
            else:
                raise AssertionError("staged report executable mutation accepted")
            assert not cleanup_target.exists()
        finally:
            module.run = original_run

        mutated = Path(artifacts[0]["path"])
        original_bytes = mutated.read_bytes()
        mutated.write_bytes(original_bytes + b"mutation")
        try:
            module.verify_file_identities(artifacts)
        except ValueError as error:
            assert "changed" in str(error)
        else:
            raise AssertionError("workspace executable mutation accepted")
        mutated.write_bytes(original_bytes)
        mutated.chmod(artifacts[0]["mode"])
        module.verify_file_identities(artifacts)
    finally:
        module.ROOT = original_root
        module.HEAVY_TEST = original_heavy
        module.INSTRUMENTATION_PROBE_TEST = original_probe
print("LOCAL_SERVER_PARALLEL_TESTS_PASS")
