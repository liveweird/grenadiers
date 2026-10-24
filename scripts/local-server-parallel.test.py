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


def lcov(hit_a, hit_b, checksum="sum"):
    return module.parse_lcov(
        f"SF:{module.ROOT / 'server/src/lib.rs'}\nDA:1,{hit_a},{checksum}\nDA:2,{hit_b},{checksum}2\n"
        f"LF:2\nLH:{int(hit_a > 0) + int(hit_b > 0)}\nend_of_record\n")


union = module.union_lcov([lcov(1, 0), lcov(0, 1)], floor=50)
assert union["linesFound"] == 2 and union["linesHit"] == 2 and union["percent"] == 100
try:
    module.union_lcov([lcov(1, 0), lcov(0, 1, "changed")], floor=50)
except ValueError:
    pass
else:
    raise AssertionError("coverage checksum mismatch accepted")


with tempfile.TemporaryDirectory(prefix="grenadiers-llvm-cov-discovery-") as temporary:
    fixture = Path(temporary)
    (fixture / "src").mkdir()
    (fixture / "Cargo.toml").write_text(
        '[package]\nname = "llvm-cov-discovery-fixture"\nversion = "0.0.0"\nedition = "2021"\n'
    )
    (fixture / "src/lib.rs").write_text(
        "pub fn covered(value: bool) -> u8 { if value { 1 } else { 0 } }\n"
        "#[cfg(test)] mod tests { use super::*; #[test]\n"
        "fn blank_database_url_disables_postgres() { assert_eq!(covered(true), 1); } }\n"
    )
    manifest = str(fixture / "Cargo.toml")
    subprocess.run(["cargo", "test", "--manifest-path", manifest], check=True, cwd=fixture,
                   stdout=subprocess.DEVNULL)
    ordinary = [path for path in (fixture / "target/debug/deps").glob("llvm_cov_discovery_fixture-*")
                if path.is_file() and os.access(path, os.X_OK)]
    assert len(ordinary) == 1
    ordinary_hash = hashlib.sha256(ordinary[0].read_bytes()).hexdigest()
    coverage_target = fixture / "coverage-target"
    coverage_base = module.coverage_base_environment(coverage_target)
    shown = subprocess.run(["cargo", "llvm-cov", "--manifest-path", manifest, "show-env", "--sh"],
                           check=True, cwd=fixture, env=coverage_base, text=True,
                           stdout=subprocess.PIPE).stdout
    cov_env = module.parse_coverage_environment(shown, coverage_base)
    assert Path(cov_env["CARGO_LLVM_COV_TARGET_DIR"]).resolve() == coverage_target.resolve()
    subprocess.run(["cargo", "llvm-cov", "--manifest-path", manifest, "clean", "--workspace"],
                   check=True, cwd=fixture, env=cov_env, stdout=subprocess.DEVNULL)
    assert ordinary[0].is_file()
    assert hashlib.sha256(ordinary[0].read_bytes()).hexdigest() == ordinary_hash
    baseline = fixture / "baseline"; baseline.mkdir()
    baseline_env = dict(cov_env); baseline_env["LLVM_PROFILE_FILE"] = str(baseline / "%p-%18m.profraw")
    subprocess.run(["cargo", "test", "--manifest-path", manifest], check=True, cwd=fixture,
                   env=baseline_env, stdout=subprocess.DEVNULL)
    assert list(baseline.glob("*.profraw"))
    messages = subprocess.run(
        ["cargo", "test", "--manifest-path", manifest, "--no-run", "--message-format=json"],
        check=True, cwd=fixture, env=cov_env, text=True, stdout=subprocess.PIPE).stdout
    artifacts = [json.loads(line) for line in messages.splitlines() if line]
    binaries = [Path(item["executable"]) for item in artifacts
                if item.get("reason") == "compiler-artifact" and item.get("executable")]
    assert len(binaries) == 1
    assert coverage_target.resolve() in binaries[0].resolve().parents
    assert binaries[0].resolve() != ordinary[0].resolve()
    worker = fixture / "worker"
    module.verify_instrumented_binary(binaries[0], cov_env, worker)
    assert list(worker.glob("*.profraw"))
    target = Path(cov_env["CARGO_LLVM_COV_TARGET_DIR"]).resolve()
    report_path = fixture / "coverage.lcov"

    def actual_report():
        return subprocess.run(
            ["cargo", "llvm-cov", "report", "--manifest-path", manifest,
             "--lcov", "--output-path", str(report_path)], check=True, cwd=fixture,
            env=cov_env, stdout=subprocess.DEVNULL)

    _, profile_count = module.with_report_profiles(target, [baseline, worker], "actual-tool", actual_report)
    assert profile_count == len(list(baseline.glob("*.profraw"))) + len(list(worker.glob("*.profraw")))
    assert report_path.is_file()
    assert "SF:" in report_path.read_text() and not list(target.glob("*.profraw"))

    (target / "grenadiers-local-server-abandoned-0-0.profraw").write_bytes(b"owned stale")
    module.with_report_profiles(target, [baseline, worker], "stale-recovery", lambda: None)
    assert not list(target.glob("*.profraw"))

    foreign = target / "foreign.profraw"; foreign.write_bytes(b"not accepted")
    try:
        module.with_report_profiles(target, [baseline, worker], "contaminated", actual_report)
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

print("LOCAL_SERVER_PARALLEL_TESTS_PASS")
