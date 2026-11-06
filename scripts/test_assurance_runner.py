"""Fast infrastructure contracts; never allocate to provoke a cgroup kill."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import run_native_tests as native

from run_native_tests import active_peers, descendants, process_running, run_command, run_suite, suite_results
from resource_limits import MAX_MEMORY, bounded_group, managed_context
from test_portable_build import PortableBuildTests
from test_validate import ValidatorTests
from test_build_cache import CacheInputTests
from test_workspace_catalogue import WorkspaceCatalogueTests
from test_oracle_facts import OracleFactsTests, CatalogueFactsTests

ROOT = Path(__file__).resolve().parents[1]


class AssuranceRunnerTests(unittest.TestCase):
    def test_guard_starts_without_unowned_peers(self):
        peers = active_peers()
        self.assertFalse(peers, native.process_details(peers))

    def test_native_cli_retains_distinct_evidence_for_reports_sharing_a_parent(self):
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ):
            directory = Path(temporary).resolve()
            target = directory/'target'
            report_parent = directory/'reports'
            binary = target/'debug/deps/fixture'
            consumed = []
            os.environ.pop('RUSTMAGE_TDM_EVIDENCE_DIR', None)

            def run_fixture(command, *, env, log, timeout):
                if command[0] == 'cargo':
                    log.write_text(json.dumps(dict(reason='compiler-artifact',
                        executable=str(binary), profile=dict(test=True),
                        target=dict(kind=['test']))) + '\n')
                else:
                    log.write_text('fixture_case: test\n')
                return dict(exit=0, seconds=0.001, log=str(log),
                            timed_out=False, orphaned_processes=False)

            def execute_fixture(command, *, env, log, timeout, test_timeout):
                evidence = Path(env['RUSTMAGE_TDM_EVIDENCE_DIR'])
                evidence.mkdir(parents=True, exist_ok=True)
                (evidence/'receipt.json').write_text(f'receipt-{len(consumed)}')
                consumed.append(evidence)
                log.write_text('test fixture_case ... ok\n'
                               'test result: ok. 1 passed; 0 failed; 0 ignored;\n')
                return dict(exit=0, seconds=0.001, log=str(log),
                            timed_out=False, orphaned_processes=False)

            with patch.object(native, 'managed_context', return_value={'backend': 'fixture'}), \
                    patch.object(native, 'active_peers', return_value=set()), \
                    patch.object(native.signal, 'signal'), \
                    patch.object(native, 'run_command', side_effect=run_fixture), \
                    patch.object(native, 'run_suite', side_effect=execute_fixture), \
                    contextlib.redirect_stdout(io.StringIO()):
                for index in range(2):
                    report = report_parent/f'native-{index}.json'
                    arguments = ['run_native_tests.py', '--target-dir', str(target),
                                 '--report', str(report)]
                    with patch.object(sys, 'argv', arguments):
                        self.assertEqual(native.main(), 0)
                    result = json.loads(report.read_text())
                    self.assertEqual(result['game_evidence_directory'], str(consumed[-1]))
                    self.assertTrue(consumed[-1].is_relative_to(report_parent))
                    self.assertFalse(consumed[-1].is_relative_to(target))
                self.assertNotEqual(consumed[0], consumed[1])
                for index, evidence in enumerate(consumed):
                    self.assertEqual((evidence/'receipt.json').read_text(), f'receipt-{index}')

                explicit = directory/'explicit-evidence'
                explicit.mkdir()
                sentinel = explicit/'old-receipt.json'
                sentinel.write_text('preserve previous evidence')
                os.environ['RUSTMAGE_TDM_EVIDENCE_DIR'] = str(explicit)
                with patch.object(sys, 'argv', arguments):
                    self.assertEqual(native.main(), 0)
                self.assertEqual(consumed[-1], explicit)
                self.assertEqual(sentinel.read_text(), 'preserve previous evidence')

    def test_suite_requires_every_selected_case_to_execute_and_pass(self):
        names = ['first', 'second']
        passing = ('test first ... ok\ntest second ... ok\n'
                   'test result: ok. 2 passed; 0 failed; 0 ignored;\n')
        self.assertTrue(suite_results(names, passing)[1])
        self.assertFalse(suite_results(names, passing.replace('second ... ok', 'second ... ignored'))[1])
        self.assertFalse(suite_results(names, passing.replace('test second ... ok\n', ''))[1])

    def test_serial_suite_keeps_a_per_case_timeout(self):
        with tempfile.TemporaryDirectory() as temporary:
            code = "import time; print('test blocked ... ',end='',flush=True); time.sleep(20)"
            result = run_suite([sys.executable, '-c', code], env=os.environ,
                               log=Path(temporary)/'suite.log', timeout=3, test_timeout=0.1)
            self.assertEqual(result['exit'], 124)
            self.assertEqual(result['timed_out_case'], 'blocked')

    def test_process_exit_during_proc_read_is_treated_as_gone(self):
        # Linux can return ESRCH after opening procfs when the task exits before
        # the read; ENOENT is also possible if it exits before the open.
        if sys.platform != 'linux':
            self.assertFalse(process_running(999999999))
            self.assertEqual(descendants(999999999), set())
            return
        for error in (FileNotFoundError, ProcessLookupError):
            with self.subTest(error=error), patch.object(Path, 'read_text', side_effect=error):
                self.assertFalse(process_running(123456))
                self.assertEqual(descendants(123456), set())
                with patch('run_native_tests.group_members', return_value=set()):
                    self.assertEqual(active_peers(), set())

    def test_limits_are_enforced_by_the_running_backend(self):
        context = managed_context()
        self.assertIsNotNone(context, 'run through scripts/resource_guard.py')
        self.assertEqual(context['memory_max_bytes'], MAX_MEMORY)
        self.assertEqual(os.environ['CARGO_BUILD_JOBS'], '1')
        self.assertEqual(os.environ['RUST_TEST_THREADS'], '1')
        if context['backend'] != 'cgroup':
            self.assertEqual(context['memory_limit_kind'], 'sampled-rss')
            self.assertIn(os.getpid(), context['members'])
            return
        group = bounded_group()
        self.assertIsNotNone(group, 'run through scripts/resource_guard.py')
        self.assertEqual(int((group/'memory.max').read_text()), MAX_MEMORY)
        parent = next(p for p in group.parents if p.name == 'rustmage-builds.slice')
        self.assertEqual(int((parent/'memory.max').read_text()), MAX_MEMORY)
        self.assertEqual((group/'memory.swap.max').read_text().strip(), '0')
        self.assertEqual(os.environ['CARGO_BUILD_JOBS'], '1')
        self.assertEqual(os.environ['RUST_TEST_THREADS'], '1')

    def test_wrapper_rejects_an_unbounded_host_even_with_a_forged_flag(self):
        code = "import sys; sys.path.insert(0, 'scripts'); import resource_limits,runpy; resource_limits.bounded_group=lambda:None; sys.argv=['scripts/rustc_guard.py','rustc','--version']; runpy.run_path('scripts/rustc_guard.py',run_name='__main__')"
        result = subprocess.run([sys.executable, '-c', code], cwd=ROOT,
                                env=dict({k: v for k, v in os.environ.items() if not k.startswith('RUSTMAGE_GUARD_')}, RUSTMAGE_RESOURCE_GUARD='1'),
                                capture_output=True, text=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('bounded runner', result.stderr)

    def test_timeout_stops_and_reaps_a_nested_session_before_returning(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            pidfile = directory/'child.pid'
            # A new session is deliberate: killpg alone would miss this child.
            code = ("import pathlib,subprocess,sys,time; "
                    "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(20)'],start_new_session=True); "
                    f"pathlib.Path({str(pidfile)!r}).write_text(str(p.pid)); time.sleep(20)")
            result = run_command([sys.executable, '-c', code], env=os.environ,
                                 log=directory/'command.log', timeout=0.3)
            self.assertEqual(result['exit'], 124)
            self.assertTrue(result['timed_out'])
            self.assertFalse(process_running(int(pidfile.read_text())))

    def test_normal_nonzero_exit_is_not_mistaken_for_a_timeout(self):
        with tempfile.TemporaryDirectory() as temporary:
            result = run_command([sys.executable, '-c', 'raise SystemExit(7)'], env=os.environ,
                                 log=Path(temporary)/'command.log', timeout=2)
            self.assertEqual(result['exit'], 7)
            self.assertFalse(result['timed_out'])

    def test_normal_exit_with_detached_child_fails_and_reaps_the_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            pidfile = directory/'child.pid'
            code = ("import pathlib,subprocess,sys; "
                    "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(20)'],start_new_session=True); "
                    f"pathlib.Path({str(pidfile)!r}).write_text(str(p.pid))")
            result = run_command([sys.executable, '-c', code], env=os.environ,
                                 log=directory/'command.log', timeout=2)
            self.assertNotEqual(result['exit'], 0)
            self.assertTrue(result['orphaned_processes'])
            self.assertFalse(process_running(int(pidfile.read_text())))

    def test_timeout_reaps_a_child_created_by_the_termination_handler(self):
        if os.name == 'nt':
            # TerminateProcess does not deliver a catchable SIGTERM on Windows.
            return
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            pidfile = directory/'late.pid'
            code = ("import pathlib,signal,subprocess,sys,time\n"
                    "def terminate(*args):\n"
                    " p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(20)'],start_new_session=True)\n"
                    f" pathlib.Path({str(pidfile)!r}).write_text(str(p.pid))\n"
                    " raise SystemExit(0)\n"
                    "signal.signal(signal.SIGTERM,terminate)\ntime.sleep(20)\n")
            result = run_command([sys.executable, '-c', code], env=os.environ,
                                 log=directory/'command.log', timeout=0.3)
            self.assertEqual(result['exit'], 124)
            self.assertTrue(pidfile.exists())
            self.assertFalse(process_running(int(pidfile.read_text())))


if __name__ == '__main__':
    suite = unittest.TestSuite([
        unittest.defaultTestLoader.loadTestsFromTestCase(AssuranceRunnerTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(PortableBuildTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(ValidatorTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(CacheInputTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(WorkspaceCatalogueTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(OracleFactsTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(CatalogueFactsTests),
    ])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() and result.testsRun > 0 and not result.skipped else 1)
