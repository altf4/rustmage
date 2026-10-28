"""Fast infrastructure contracts; never allocate to provoke a cgroup kill."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from run_native_tests import active_peers, descendants, process_running, run_command
from resource_limits import MAX_MEMORY, bounded_group
from test_validate import ValidatorTests

ROOT = Path(__file__).resolve().parents[1]


class AssuranceRunnerTests(unittest.TestCase):
    def test_process_exit_during_proc_read_is_treated_as_gone(self):
        # Linux can return ESRCH after opening procfs when the task exits before
        # the read; ENOENT is also possible if it exits before the open.
        for error in (FileNotFoundError, ProcessLookupError):
            with self.subTest(error=error), patch.object(Path, 'read_text', side_effect=error):
                self.assertFalse(process_running(123456))
                self.assertEqual(descendants(123456), set())
                with patch('run_native_tests.group_members', return_value=set()):
                    self.assertEqual(active_peers(), set())

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


if __name__ == '__main__':
    suite = unittest.TestSuite([
        unittest.defaultTestLoader.loadTestsFromTestCase(AssuranceRunnerTests),
        unittest.defaultTestLoader.loadTestsFromTestCase(ValidatorTests),
    ])
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() and result.testsRun > 0 and not result.skipped else 1)
