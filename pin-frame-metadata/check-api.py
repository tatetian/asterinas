#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0

"""Check the safe allocator and reject each metadata escape route separately."""

import os
from pathlib import Path
import subprocess
import sys


def main():
    directory = Path(__file__).resolve().parent
    evidence = directory / "evidence"
    evidence.mkdir(exist_ok=True)
    command = [os.environ.get("CARGO_OSDK", "cargo-osdk"), "osdk", "check", "--lib"]
    environment = dict(os.environ, CARGO_TERM_COLOR="never")
    environment.setdefault("CARGO_TARGET_DIR", str(directory.parent / "target"))
    environment.setdefault("CARGO_INCREMENTAL", "0")
    cases = [
        ("safe-allocator", None, None),
        ("direct-swap", "direct-swap", "E0596"),
        ("whole-link-swap", "whole-link-swap", "E0277"),
        ("cursor-swap", "cursor-swap", "E0596"),
        ("dynamic-downcast", "dynamic-downcast", "E0596"),
        ("unpinned-access", "unpinned-access", "E0277"),
        ("replacement", "replacement", "E0277"),
        ("premature-teardown", "premature-teardown", "E0133"),
    ]
    for name, feature, error in cases:
        arguments = command + (["--features", feature] if feature else [])
        result = subprocess.run(
            arguments,
            cwd=directory / "api-tests",
            env=environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        (evidence / f"api-{name}.log").write_text(result.stdout)
        passed = result.returncode == 0 if error is None else (
            result.returncode != 0 and f"error[{error}]" in result.stdout
            and "src/lib.rs:" in result.stdout
            and ("DerefMut" in result.stdout or "PhantomPinned" in result.stdout
                 or (feature == "replacement" and "Default" in result.stdout)
                 or (feature == "premature-teardown" and "on_drop" in result.stdout))
        )
        print(f"{'PASS' if passed else 'FAIL'} {name}", flush=True)
        if not passed:
            print(result.stdout, file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
