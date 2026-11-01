#!/usr/bin/env python3
"""Run ordinary workspace tests and the latency-sensitive native CPU cohort."""
from __future__ import annotations

import argparse
import importlib.util
from pathlib import Path
from types import ModuleType


def load_runner() -> ModuleType:
    path = Path(__file__).with_name("local-server-parallel.py")
    spec = importlib.util.spec_from_file_location("local_server_parallel", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("local server runner module is unavailable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main(argv: list[str] | None = None, runner: ModuleType | None = None) -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args(argv)
    selected = runner or load_runner()
    profile = "release" if args.release else "debug"
    binary = selected.select_binary(args.release)
    selected.execute_ordinary_and_isolated(
        profile, binary, release=args.release
    )


if __name__ == "__main__":
    main()
