"""Fast infrastructure contracts; never allocate to provoke a cgroup kill."""
import os
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from run_native_tests import NativeProgress, process_running, run_command, run_test_suite
from resource_limits import MAX_MEMORY, bounded_group

ROOT = Path(__file__).resolve().parents[1]


class AssuranceRunnerTests(unittest.TestCase):
    def test_limits_are_enforced_by_the_running_cgroup(self):
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
                                env=dict(os.environ, RUSTMAGE_RESOURCE_GUARD='1'),
                                capture_output=True, text=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('bounded runner', result.stderr)

    def test_timeout_stops_and_reaps_a_nested_session_before_returning(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
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
            directory = Path(temporary)
            pidfile = directory/'child.pid'
            code = ("import pathlib,subprocess,sys; "
                    "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(20)'],start_new_session=True); "
                    f"pathlib.Path({str(pidfile)!r}).write_text(str(p.pid))")
            result = run_command([sys.executable, '-c', code], env=os.environ,
                                 log=directory/'command.log', timeout=2)
            self.assertNotEqual(result['exit'], 0)
            self.assertTrue(result['orphaned_processes'])
            self.assertFalse(process_running(int(pidfile.read_text())))


class NativeStreamingTests(unittest.TestCase):
    def test_partial_start_without_newline_and_split_results(self):
        progress = NativeProgress(['a', 'b'], io.StringIO(), 'suite.log')
        progress.feed(b'\nrunning 2 tests\ntest a ..', 1)
        self.assertIsNone(progress.active)
        progress.feed(b'. ', 2)
        self.assertEqual(progress.active, ('a', 2))
        progress.feed(b'o', 3)
        self.assertEqual(len(progress.tests), 0)
        progress.feed(b'k\ntest b ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n', 4)
        self.assertTrue(progress.complete())
        self.assertEqual([t['test'] for t in progress.tests], ['a', 'b'])

    def test_reordered_duplicate_unknown_and_ignored_results_fail_closed(self):
        for text in [b'test b ... ok\n', b'test unknown ... ok\n',
                     b'test a ... ignored\n', b'test a ... ok\ntest a ... ok\n']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                NativeProgress(['a', 'b'], io.StringIO(), 'suite.log').feed(text, 1)

    def test_missing_failed_filtered_or_extra_results_cannot_pass(self):
        for text in [b'test a ... ', b'test a ... FAILED\n',
                     b'test a ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s\n',
                     b'test a ... ok\ntest unknown ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n']:
            with self.subTest(text=text):
                progress = NativeProgress(['a'], io.StringIO(), 'suite.log')
                progress.feed(text, 1)
                self.assertFalse(progress.complete())

    def run_fake(self, code, names=('a',), test_timeout=0.2, suite_timeout=2):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        path = Path(temporary.name)/'suite.log'
        result = run_test_suite([sys.executable, '-u', '-c', code], names=list(names),
                                env=os.environ, log=path, events=io.StringIO(),
                                test_timeout=test_timeout, suite_timeout=suite_timeout)
        return result, path

    def test_streaming_complete_exact_success_and_nonzero_early_exit(self):
        output = 'test a ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'
        result, log = self.run_fake(f'print({output!r},end="",flush=True)')
        self.assertTrue(result['success'])
        self.assertIn('test result: ok.', log.read_text())
        result, _ = self.run_fake('print("test a ... ",end="",flush=True); raise SystemExit(7)')
        self.assertFalse(result['success'])
        self.assertEqual(result['exit'], 7)
        self.assertFalse(result.get('timed_out', False))

    def test_silent_active_test_is_killed_at_per_test_deadline(self):
        result, _ = self.run_fake('import time; print("test a ... ",end="",flush=True); time.sleep(20)')
        self.assertFalse(result['success'])
        self.assertTrue(result['timed_out'])
        self.assertFalse(result['budget_exceeded'])
        self.assertEqual(result['tests'][0]['test'], 'a')
        self.assertEqual(result['tests'][0]['exit'], 124)
        self.assertLess(result['execution_seconds'], 2)

    def test_global_deadline_is_independent_of_per_test_deadline(self):
        result, _ = self.run_fake('import time; print("test a ... ",end="",flush=True); time.sleep(20)',
                                  test_timeout=2, suite_timeout=0.2)
        self.assertFalse(result['success'])
        self.assertTrue(result['budget_exceeded'])
        self.assertLess(result['execution_seconds'], 2)

    def test_streaming_timeout_and_success_reap_detached_children(self):
        for timeout in [False, True]:
            with self.subTest(timeout=timeout), tempfile.TemporaryDirectory() as folder:
                pidfile = Path(folder)/'pid'
                code = ('import pathlib,subprocess,sys,time; '
                        'p=subprocess.Popen([sys.executable,"-c","import time; time.sleep(20)"],start_new_session=True); '
                        f'pathlib.Path({str(pidfile)!r}).write_text(str(p.pid)); '
                        'print("test a ... ",end="",flush=True); ')
                if timeout:
                    code += 'time.sleep(20)'
                else:
                    code += 'print("ok\\n\\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s",flush=True)'
                result, _ = self.run_fake(code)
                self.assertFalse(result['success'])
                self.assertFalse(process_running(int(pidfile.read_text())))


if __name__ == '__main__':
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() and result.testsRun > 0 and not result.skipped else 1)
