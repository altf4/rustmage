"""Small process tests: verify enforced limits without deliberately killing processes."""
import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from scripts.resource_limits import bounded_group

ROOT = Path(__file__).resolve().parents[1]
GUARD = ROOT/'scripts/resource_guard.py'

class ResourceGuardTests(unittest.TestCase):
    def run_guard(self, command, **kwargs):
        return subprocess.run([sys.executable, str(GUARD), '--', *command], cwd=ROOT,
                              text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, **kwargs)

    def test_actual_limits_and_build_environment(self):
        code = "import json,os;from pathlib import Path;p=Path('/sys/fs/cgroup')/Path('/proc/self/cgroup').read_text().strip().split('::')[1].lstrip('/');print(json.dumps([int((p/'memory.max').read_text()),int((p.parent/'memory.max').read_text()),(p/'memory.swap.max').read_text().strip(),os.environ['CARGO_BUILD_JOBS'],os.environ['RUST_TEST_THREADS'],os.environ['CARGO_TARGET_DIR']]))"
        result = self.run_guard([sys.executable, '-c', code], env=dict(os.environ, CARGO_TARGET_DIR='/a/custom/target'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), [6*1024**3, 6*1024**3, '0', '1', '1', '/a/custom/target'])

    @unittest.skipIf(bounded_group() is not None, 'Launcher rejection requires an unbounded Python host')
    def test_compilation_cannot_be_enabled_by_a_fake_environment_flag(self):
        result = subprocess.run([sys.executable, str(ROOT/'scripts/rustc_guard.py'), 'rustc', '--version'],
                                env=dict(os.environ, RUSTMAGE_RESOURCE_GUARD='1'), capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('bounded runner', result.stderr)

    @unittest.skipIf(bounded_group() is not None, 'Cross-unit scheduling requires an unbounded Python host')
    def test_two_units_serialize_and_nested_runner_does_not_deadlock(self):
        with tempfile.TemporaryDirectory() as directory:
            def run(i):
                path=Path(directory)/str(i)
                code=f"import json,time;from pathlib import Path;s=time.monotonic();time.sleep(.1);Path({str(path)!r}).write_text(json.dumps([s,time.monotonic()]))"
                result=self.run_guard([sys.executable, str(GUARD), '--', sys.executable, '-c', code], timeout=15, env=dict(os.environ, XDG_CACHE_HOME=str(Path(directory)/('cache'+str(i)))))
                self.assertEqual(result.returncode, 0, result.stderr)
                return json.loads(path.read_text())
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                intervals=sorted(pool.map(run, [0,1]))
            self.assertLessEqual(intervals[0][1], intervals[1][0])

if __name__ == '__main__':
    unittest.main()
