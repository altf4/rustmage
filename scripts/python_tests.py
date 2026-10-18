#!/usr/bin/env python3
"""Run existing binding tests serially and keep timings even if a run is interrupted."""
import argparse
import json
import os
from pathlib import Path
import sys
import time
import unittest
if __package__:
    from .resource_limits import bounded_group
else:
    from resource_limits import bounded_group

ROOT = Path(__file__).resolve().parents[1]
SMOKE = [
    'test_schema_version.SchemaVersionTests',
    'test_binding.BindingTests.test_input_and_lifecycle',
    'test_binding.BindingTests.test_standard_admission_distinguishes_legality_and_rule_coverage',
    'test_binding.BindingTests.test_decoded_nul_in_invalid_card_is_an_error_without_aborting_host',
    'test_binding.BindingTests.test_private_views_clone_restore_and_stale_indices',
    'test_binding.BindingTests.test_complete_selfplay_through_shared_library',
]


class TimedResult(unittest.TextTestResult):
    def __init__(self, *args, events):
        super().__init__(*args)
        self.events = events

    def record(self, data):
        self.events.write(json.dumps(data) + '\n')
        self.events.flush()

    def startTest(self, test):
        super().startTest(test)
        self.started = time.monotonic()
        self.failures_before = len(self.failures) + len(self.errors) + len(self.unexpectedSuccesses)
        self.record(dict(event='start', test=test.id()))

    def stopTest(self, test):
        self.record(dict(event='finish', test=test.id(), seconds=time.monotonic()-self.started,
                         failed=len(self.failures)+len(self.errors)+len(self.unexpectedSuccesses) > self.failures_before))
        super().stopTest(test)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite', choices=['smoke', 'full'], default='smoke')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if bounded_group() is None:
        os.execv(sys.executable, [sys.executable, str(ROOT/'scripts/resource_guard.py'), '--',
                                 sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    library = Path(os.environ['RUSTMAGE_LIBRARY']).resolve(strict=True)
    os.environ['RUSTMAGE_LIBRARY'] = str(library)
    sys.path.insert(0, str(ROOT/'python'))
    loader = unittest.TestLoader()
    suite = (loader.discover(str(ROOT/'python')) if args.suite == 'full'
             else loader.loadTestsFromNames(SMOKE))
    args.report.parent.mkdir(parents=True, exist_ok=True)
    start = time.monotonic()
    with args.report.with_suffix('.jsonl').open('w') as events:
        result = unittest.TextTestRunner(verbosity=2, resultclass=lambda *a: TimedResult(*a, events=events)).run(suite)
    args.report.write_text(json.dumps(dict(suite=args.suite, library=str(library),
        tests=result.testsRun, success=result.wasSuccessful(), seconds=time.monotonic()-start,
        failures=len(result.failures), errors=len(result.errors), skipped=len(result.skipped)), indent=2)+'\n')
    return 0 if result.wasSuccessful() else 1


if __name__ == '__main__':
    sys.exit(main())
