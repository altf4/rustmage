"""Small core contracts for the bounded build/test tools, without allocation probes."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from resource_limits import MAX_MEMORY, bounded_group
from run_native_tests import process_running, run_command, suite_results
from validate import sha, snapshot_library


class ToolContracts(unittest.TestCase):
    def test_builds_are_bounded_and_a_forged_flag_cannot_bypass_the_wrapper(self):
        group = bounded_group()
        self.assertIsNotNone(group, 'use scripts/resource_guard.py')
        self.assertEqual(int((group/'memory.max').read_text()), MAX_MEMORY)
        parent = next(p for p in group.parents if p.name == 'rustmage-builds.slice')
        self.assertEqual(int((parent/'memory.max').read_text()), MAX_MEMORY)
        self.assertEqual((group/'memory.swap.max').read_text().strip(), '0')
        self.assertEqual(os.environ['CARGO_BUILD_JOBS'], '1')
        self.assertEqual(os.environ['RUST_TEST_THREADS'], '1')
        probe = "import sys,runpy; sys.path.insert(0,'scripts'); import resource_limits; resource_limits.bounded_group=lambda:None; sys.argv=['rustc_guard','rustc','--version']; runpy.run_path('scripts/rustc_guard.py',run_name='__main__')"
        result = subprocess.run([sys.executable, '-c', probe], env=dict(os.environ, RUSTMAGE_RESOURCE_GUARD='1'),
                                capture_output=True, text=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('bounded runner', result.stderr)

    def test_native_results_require_every_selected_case_to_execute(self):
        output = 'test mechanics::flying ... ok\ntest cards::smaug ... ok\n' \
                 'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n'
        names = ['mechanics::flying', 'cards::smaug']
        self.assertTrue(suite_results(names, output)[1])
        self.assertFalse(suite_results(names, output.replace('test cards::smaug ... ok\n', ''))[1])
        self.assertFalse(suite_results(names, output.replace('smaug ... ok', 'smaug ... ignored'))[1])
        self.assertFalse(suite_results([], '')[1])

    def test_timeout_reaps_a_nested_session_before_returning(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            pidfile = directory/'child.pid'
            code = ("import pathlib,subprocess,sys,time; "
                    "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(20)'],start_new_session=True); "
                    f"pathlib.Path({str(pidfile)!r}).write_text(str(p.pid)); time.sleep(20)")
            result = run_command([sys.executable, '-c', code], env=os.environ,
                                 log=directory/'timeout.log', timeout=0.5)
            self.assertEqual(result['exit'], 124)
            self.assertFalse(process_running(int(pidfile.read_text())))

    def test_debug_and_release_library_copies_survive_later_builds(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for profile in ('debug', 'release'):
                source = directory/'target'/profile/'librustmage.so'
                source.parent.mkdir(parents=True)
                source.write_bytes(profile.encode())
                report = directory/profile
                report.mkdir()
                library, digest = snapshot_library(directory/'target', report, profile)
                source.write_bytes(b'next build')
                self.assertEqual(library.read_bytes(), profile.encode())
                self.assertEqual(sha(library), digest)


if __name__ == '__main__':
    unittest.main(verbosity=2)
