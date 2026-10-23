#!/usr/bin/env python3
"""Run the bounded public binding contract and retain per-test timing evidence.

The smoke/full names are compatibility aliases for the same small suite. Card
rules belong to the native suite; adding a Python card rollout increases JSON
and ctypes work without improving the native rule assertion.
"""
import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time
import unittest

if __package__:
    from .resource_limits import bounded_group
else:
    from resource_limits import bounded_group

ROOT = Path(__file__).resolve().parents[1]


class TimedResult(unittest.TextTestResult):
    def __init__(self, *args, events):
        super().__init__(*args)
        self.events = events

    def record(self, **data):
        self.events.write(json.dumps(data) + "\n")
        self.events.flush()

    def startTest(self, test):
        super().startTest(test)
        self.started = time.monotonic()
        self.failures_before = len(self.failures) + len(self.errors) + len(self.unexpectedSuccesses)
        self.record(event="start", test=test.id())

    def stopTest(self, test):
        failed = len(self.failures) + len(self.errors) + len(self.unexpectedSuccesses) > self.failures_before
        self.record(event="finish", test=test.id(), seconds=time.monotonic() - self.started, failed=failed)
        super().stopTest(test)


def write_report(path, data):
    pending = path.with_suffix(path.suffix + ".tmp")
    pending.write_text(json.dumps(data, indent=2) + "\n")
    pending.replace(path)


def run_worker(args, library):
    sys.path.insert(0, str(ROOT / "python"))
    started = time.monotonic()
    suite = unittest.TestLoader().loadTestsFromName("test_contract.BindingContractTests")
    with args.report.with_suffix(".jsonl").open("a") as events:
        result = unittest.TextTestRunner(
            verbosity=2, resultclass=lambda *values: TimedResult(*values, events=events)
        ).run(suite)
    elapsed = time.monotonic() - started
    over_budget = elapsed > args.runtime_budget
    success = result.wasSuccessful() and result.testsRun > 0 and not over_budget
    write_report(args.report, dict(
        suite="contract", requested_suite=args.suite, library=str(library), terminal=True,
        tests=result.testsRun, success=success, seconds=elapsed,
        failures=len(result.failures), errors=len(result.errors), skipped=len(result.skipped),
        runtime_budget_seconds=args.runtime_budget, budget_exceeded=over_budget,
        timeout_seconds=args.timeout_seconds,
    ))
    if over_budget:
        print(f"Binding contract exceeded {args.runtime_budget:g}s runtime budget ({elapsed:.3f}s)",
              file=sys.stderr)
    return 0 if success else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", choices=["contract", "smoke", "full"], default="contract")
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--runtime-budget", type=float, default=5.0,
                        help="Fail if suite runtime exceeds this many seconds (default: 5)")
    parser.add_argument("--timeout-seconds", type=float, default=15.0,
                        help="Terminate a stuck native test process after this wall time (default: 15)")
    parser.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if any(not math.isfinite(value) or value <= 0 for value in
           (args.runtime_budget, args.timeout_seconds)):
        parser.error("runtime budget and timeout must be positive finite seconds")
    if bounded_group() is None:
        os.execv(sys.executable, [sys.executable, str(ROOT / "scripts/resource_guard.py"), "--",
                                 sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    try:
        library = Path(os.environ["RUSTMAGE_LIBRARY"]).resolve(strict=True)
    except (KeyError, OSError) as error:
        parser.error(f"RUSTMAGE_LIBRARY must name the built shared library: {error}")
    os.environ["RUSTMAGE_LIBRARY"] = str(library)
    if args.worker:
        return run_worker(args, library)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.with_suffix(".jsonl").write_text("")
    initial = dict(suite="contract", requested_suite=args.suite, library=str(library),
                   terminal=False, success=False, runtime_budget_seconds=args.runtime_budget,
                   timeout_seconds=args.timeout_seconds)
    write_report(args.report, initial)
    started = time.monotonic()
    command = [sys.executable, str(Path(__file__).resolve()), "--worker", *sys.argv[1:]]
    try:
        result = subprocess.run(command, timeout=args.timeout_seconds,
                                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
    except subprocess.TimeoutExpired:
        write_report(args.report, dict(initial, terminal=True, seconds=time.monotonic() - started,
                                      timed_out=True, error="binding worker exceeded wall timeout"))
        print(f"Binding contract timed out after {args.timeout_seconds:g}s; see {args.report}",
              file=sys.stderr)
        return 1
    final = json.loads(args.report.read_text())
    if not final.get("terminal"):
        write_report(args.report, dict(initial, terminal=True, seconds=time.monotonic() - started,
                                      exit_code=result.returncode,
                                      error="binding worker exited without a terminal report"))
        return 1
    return 0 if result.returncode == 0 and final.get("success") else 1


if __name__ == "__main__":
    sys.exit(main())
