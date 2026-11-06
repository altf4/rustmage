"""Portable build contracts without allocation probes or compiler invocations."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch, Mock

import build_platform
import resource_limits
from resource_limits import MAX_MEMORY, managed_context


class PortableBuildTests(unittest.TestCase):
    def test_native_artifact_names(self):
        for platform, library, executable in [
                ('linux', 'librustmage.so', 'benchmark'),
                ('darwin', 'librustmage.dylib', 'benchmark'),
                ('win32', 'rustmage.dll', 'benchmark.exe')]:
            self.assertEqual(build_platform.library_name(platform), library)
            self.assertEqual(build_platform.executable_name('benchmark', platform), executable)

    def test_windows_native_launcher_preserves_long_unicode_arguments(self):
        if sys.platform != 'win32':
            return
        launcher = Path(os.environ['RUSTC_WRAPPER'])
        self.assertEqual(launcher.suffix, '.exe')
        with tempfile.TemporaryDirectory(prefix='compiler arguments ') as temporary:
            script = Path(temporary)/'echo arguments.py'
            script.write_text('import json,sys; print(json.dumps(sys.argv[1:]))\n', encoding='utf-8')
            arguments = [f'--extern=crate_{index}=C:/long build cache/カード {index}.rlib' for index in range(220)]
            self.assertGreater(sum(map(len, arguments)), 8191)
            result = subprocess.run([str(launcher), *arguments],
                                    env=dict(os.environ, RUSTMAGE_RUSTC_GUARD=str(script)),
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout), arguments)

    def test_non_linux_does_not_read_procfs(self):
        with patch.object(resource_limits.sys, 'platform', 'darwin'), \
                patch.object(Path, 'read_text', side_effect=AssertionError('procfs read')):
            self.assertIsNone(resource_limits.bounded_group())

    def test_format_batches_preserve_all_paths_under_windows_limit(self):
        paths = [f'C:/a long checkout/カード {index:05d}.rs' for index in range(6000)]
        batches = list(build_platform.argument_batches(paths))
        self.assertGreater(len(batches), 1)
        self.assertEqual([p for batch in batches for p in batch], paths)
        for batch in batches:
            self.assertLessEqual(sum(len(p.encode('utf-16-le')) // 2 + 3 for p in batch), 16000)

    def test_allocated_bytes_without_unix_blocks(self):
        with patch.object(Path, 'stat', return_value=type('Stat', (), {'st_size': 123})()):
            self.assertEqual(build_platform.allocated_bytes('file'), 123)

    def test_nested_guard_reuses_live_supervisor_without_locking_again(self):
        self.assertIsNotNone(managed_context())
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary)/'nested.json'
            command = [sys.executable, 'scripts/resource_guard.py', '--report', str(report),
                       '--', sys.executable, '-c', 'raise SystemExit(7)']
            result = subprocess.run(command, capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 7, result.stderr)
            self.assertEqual(json.loads(report.read_text())['exit_code'], 7)

    def test_synthetic_memory_sample_detects_budget_without_allocating(self):
        if managed_context()['backend'] == 'cgroup':
            # psutil is optional for the Linux cgroup backend.
            return
        import portable_guard
        process = Mock()
        process.pid = 123
        process.create_time.return_value = 1.0
        process.children.return_value = []
        process.memory_info.return_value.rss = MAX_MEMORY + 1
        with patch.object(portable_guard, 'marked_processes', return_value=[process]), \
                patch.object(portable_guard, 'running', return_value=True):
            supervisor = portable_guard.Supervisor('fixture-token')
            supervisor.sample()
        self.assertTrue(supervisor.exceeded)
        self.assertEqual(supervisor.peak, MAX_MEMORY + 1)

    def test_stale_supervisor_cannot_admit_a_compiler(self):
        with patch.object(resource_limits, 'bounded_group', return_value=None), \
                patch.dict(os.environ, {'RUSTMAGE_GUARD_PORT': '0', 'RUSTMAGE_GUARD_TOKEN': 'forged'}):
            self.assertIsNone(managed_context())

    def test_shared_slot_serializes_independent_processes(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            code = ("import sys,time,json; from pathlib import Path; "
                    "sys.path.insert(0,'scripts'); import resource_guard; "
                    "resource_guard.Path.home=classmethod(lambda cls:Path(sys.argv[1])); "
                    "\nwith resource_guard.shared_build_slot():\n"
                    " start=time.monotonic(); time.sleep(0.15)\n"
                    " Path(sys.argv[2]).write_text(json.dumps([start,time.monotonic()]))\n")
            processes = [subprocess.Popen([sys.executable, '-c', code, str(directory),
                                          str(directory/f'{index}.json')],
                                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                         for index in range(2)]
            try:
                for process in processes:
                    self.assertEqual(process.wait(timeout=10), 0)
            finally:
                for process in processes:
                    if process.poll() is None:
                        process.kill()
                        process.wait()
            intervals = sorted(json.loads((directory/f'{index}.json').read_text()) for index in range(2))
            self.assertGreaterEqual(intervals[1][0], intervals[0][1])

    def test_telemetry_failure_stops_worker_and_keeps_failure_report(self):
        if managed_context()['backend'] == 'cgroup':
            return
        import portable_guard
        from build_platform import cargo_environment
        created = []
        popen = subprocess.Popen

        def record(*args, **kwargs):
            process = popen(*args, **kwargs)
            created.append(process)
            return process

        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary)/'failure.json'
            with patch.object(portable_guard.subprocess, 'Popen', side_effect=record), \
                    patch.object(portable_guard.Supervisor, 'sample',
                                 side_effect=portable_guard.psutil.AccessDenied(123)):
                code = portable_guard.run([sys.executable, '-c', 'import time; time.sleep(20)'],
                                          report, cargo_environment())
            self.assertNotEqual(code, 0)
            self.assertTrue(created)
            self.assertTrue(all(p.poll() is not None for p in created))
            result = json.loads(report.read_text())
            self.assertTrue(result['terminal'])
            self.assertTrue(result['error'])

    def test_inaccessible_status_is_not_mistaken_for_a_terminated_process(self):
        if managed_context()['backend'] == 'cgroup':
            return
        import portable_guard
        process = Mock()
        process.is_running.return_value = True
        process.status.side_effect = portable_guard.psutil.AccessDenied(123)
        self.assertTrue(portable_guard.running(process))

    def test_one_denied_signal_does_not_prevent_cleanup_of_other_children(self):
        if managed_context()['backend'] == 'cgroup':
            return
        import portable_guard
        denied, accessible = Mock(), Mock()
        denied.terminate.side_effect = portable_guard.psutil.AccessDenied(123)
        denied.kill.side_effect = portable_guard.psutil.AccessDenied(123)
        with patch.object(portable_guard.psutil, 'wait_procs', return_value=([], [denied])), \
                patch.object(portable_guard, 'running', return_value=True):
            with self.assertRaisesRegex(RuntimeError, 'remain active'):
                portable_guard.stop_processes([denied, accessible])
        accessible.terminate.assert_called_once()
        denied.kill.assert_called_once()


if __name__ == '__main__':
    unittest.main(verbosity=2)
