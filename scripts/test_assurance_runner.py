"""Fast infrastructure contracts; never allocate to provoke a cgroup kill."""
import contextlib
import errno
import io
import json
import os
import shutil
from pathlib import Path
import stat
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
    def test_every_scenario_file_has_one_cargo_target(self):
        tests = ROOT/'tests'
        metadata = json.loads(subprocess.check_output(
            ['cargo', 'metadata', '--no-deps', '--format-version=1'], cwd=ROOT))
        package = next(p for p in metadata['packages'] if Path(p['manifest_path']) == ROOT/'Cargo.toml')
        targets = {t['name']: Path(t['src_path']) for t in package['targets'] if 'test' in t['kind']}
        registered = list(targets.values())
        expected = set(tests.glob('*.rs'))
        for name, entry in native.scenario_suites(tests):
            self.assertEqual(targets.get(name), entry)
            registered.extend(native.scenario_files(entry))
            if entry.parent != tests:
                expected.update(entry.parent.glob('*.rs'))
        self.assertCountEqual(registered, expected)

    def test_grouped_case_filter_searches_only_original_case_names(self):
        names = ['beam_engine::cast_beam', 'beam_engine::cast', 'other::cast_beam']
        selected = native.selected_case_batches(names, {'beam_engine'}, 'beam')
        case = 'beam_engine::cast_beam'
        self.assertEqual(list(selected), [([case], ['--exact', case])])
        self.assertEqual(list(native.selected_case_batches(names, {'beam_engine'}, 'beam_engine')), [])
        many = ['beam::' + 'x'*200 + str(i) for i in range(150)]
        batches = list(native.selected_case_batches(many + ['other::x'], {'beam'}, 'x'))
        self.assertGreater(len(batches), 1)
        self.assertEqual([name for cases, _ in batches for name in cases], many)
        self.assertTrue(all(args == ['--exact', *cases] for cases, args in batches))

    def test_real_guarded_native_batches_link_run_and_retire(self):
        cache = Path(os.environ.get('XDG_CACHE_HOME', Path.home()/'.cache'))
        directory = Path(tempfile.mkdtemp(prefix='rustmage-batch-probe-', dir=cache)).resolve()
        passed = False
        try:
            root = directory/'fixture'
            (root/'tests').mkdir(parents=True)
            (root/'Cargo.toml').write_text(
                '[package]\nname="rustmage"\nversion="0.0.0"\nedition="2021"\n'
                '[lib]\npath="lib.rs"\ntest=false\n'
                '[profile.test]\ndebug=false\nincremental=false\n')
            (root/'lib.rs').write_text('pub fn answer() -> u8 { 42 }\n')
            first = root/'tests/group_second'
            first.mkdir()
            (first/'main.rs').write_text('mod alpha;\nmod also_second;\n')
            case = '#[test] fn actual_case() { assert_eq!(rustmage::answer(), 42); }\n'
            for path in (first/'alpha.rs', first/'also_second.rs', root/'tests/second.rs'):
                path.write_text(case)
            target = directory/'target'
            report = directory/'native.json'
            wrapper = os.environ.get('RUSTC_WRAPPER', str(ROOT/'scripts/rustc_guard.py'))
            arguments = ['run_native_tests.py', '--target-dir', str(target), '--report', str(report),
                         '--retire-binaries', '--batch-size', '1', '--compile-timeout', '20',
                         '--suite-timeout', '10']
            with patch.object(native, 'ROOT', root), patch.object(sys, 'argv', arguments), \
                    patch.object(native.signal, 'signal'), \
                    patch.dict(os.environ, {'RUSTC_WRAPPER': wrapper}), \
                    contextlib.redirect_stdout(io.StringIO()):
                status = native.main()
            result = json.loads(report.read_text())
            self.assertEqual(status, 0, str(directory)+'\n'+json.dumps(result))
            self.assertEqual(len(result['compile_batches']), 2)
            self.assertEqual(result['expected_tests'], 3)
            self.assertEqual(result['retired_native_test_binaries'], 2)
            self.assertTrue(all(not Path(row['binary']).exists() for row in result['binaries']))
            self.assertTrue(list((target/'debug/deps').glob('librustmage*.rlib')))
            second_compile = Path(result['compile_batches'][1]['log']).read_text().splitlines()
            libraries = [json.loads(line) for line in second_compile if line.startswith('{')]
            self.assertTrue(any(row.get('fresh') and 'lib' in row.get('target', {}).get('kind', [])
                                for row in libraries), 'second batch rebuilt its shared library')
            # The glob also happens to match the suite name. Only its matching
            # former target and the separate standalone target were requested.
            # The case substring need not start at the module boundary.
            focused = ['run_native_tests.py', '--target-dir', str(target), '--report', str(report),
                       '--test-target', '*second', '--filter', 'tual_']
            with patch.object(native, 'ROOT', root), patch.object(sys, 'argv', focused), \
                    patch.object(native.signal, 'signal'), \
                    patch.dict(os.environ, {'RUSTC_WRAPPER': wrapper}), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(native.main(), 0)
            self.assertCountEqual([row['test'] for row in json.loads(report.read_text())['tests']],
                                  ['actual_case', 'also_second::actual_case'])
            passed = True
        finally:
            if passed:
                shutil.rmtree(directory)

    def run_batched_fixture(self, *, fail_execution=False, omit_target=False):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            target = directory/'target'
            report = directory/'reports/native.json'
            dependency = target/'debug/deps/libengine.rlib'
            release = target/'release/library.snapshot'
            release.parent.mkdir(parents=True)
            release.write_bytes(b'preserve production output')
            builds, executed, binaries = [], [], []

            def command_fixture(command, *, env, log, timeout):
                if command[:2] == ['cargo', 'metadata']:
                    log.write_text(json.dumps({'packages': [{
                        'manifest_path': str(native.ROOT/'Cargo.toml'),
                        'targets': [{'kind': ['test'], 'name': name}
                                    for name in ['third', 'first', 'second']],
                    }]}))
                elif command[:2] == ['cargo', 'test']:
                    # Earlier completed batches must be gone before the next link.
                    self.assertTrue(all(not path.exists() for path in binaries))
                    names = [command[i+1] for i, arg in enumerate(command) if arg == '--test']
                    builds.append((names, timeout))
                    dependency.parent.mkdir(parents=True, exist_ok=True)
                    dependency.write_bytes(b'reuse engine')
                    artifacts = []
                    for name in names[:-1] if omit_target else names:
                        binary = dependency.parent/(name + ('.exe' if os.name == 'nt' else ''))
                        binary.write_bytes(b'test executable')
                        binaries.append(binary)
                        artifacts.append(dict(reason='compiler-artifact', executable=str(binary),
                                              profile=dict(test=True), target=dict(kind=['test'], name=name)))
                    log.write_text(''.join(json.dumps(row)+'\n' for row in artifacts))
                else:
                    log.write_text('fixture_case: test\n')
                return dict(exit=0, seconds=2, log=str(log),
                            timed_out=False, orphaned_processes=False)

            def execute_fixture(command, *, env, log, timeout, test_timeout):
                executed.append(Path(command[0]).stem)
                status = 'FAILED' if fail_execution else 'ok'
                log.write_text(f'test fixture_case ... {status}\n'
                               + ('test result: FAILED. 0 passed; 1 failed; 0 ignored;\n'
                                  if fail_execution else
                                  'test result: ok. 1 passed; 0 failed; 0 ignored;\n'))
                return dict(exit=int(fail_execution), seconds=0.001, log=str(log),
                            timed_out=False, orphaned_processes=False)

            arguments = ['run_native_tests.py', '--target-dir', str(target), '--report', str(report),
                         '--retire-binaries', '--batch-size', '2', '--compile-timeout', '10']
            with patch.object(native, 'managed_context', return_value={'backend': 'fixture'}), \
                    patch.object(native, 'active_peers', return_value=set()), \
                    patch.object(native.signal, 'signal'), \
                    patch.object(native, 'run_command', side_effect=command_fixture), \
                    patch.object(native, 'run_suite', side_effect=execute_fixture), \
                    patch.object(sys, 'argv', arguments), contextlib.redirect_stdout(io.StringIO()), \
                    contextlib.redirect_stderr(io.StringIO()):
                status = native.main()
            result = json.loads(report.read_text())
            self.assertTrue(result['terminal'])
            self.assertEqual(dependency.read_bytes(), b'reuse engine')
            self.assertEqual(release.read_bytes(), b'preserve production output')
            if fail_execution or omit_target:
                self.assertNotEqual(status, 0)
                self.assertFalse(result['success'])
                self.assertEqual(len(builds), 1)
                self.assertTrue(all(path.exists() for path in binaries))
                self.assertEqual(result['retired_native_test_binaries'], 0)
            else:
                self.assertEqual(status, 0)
                self.assertTrue(result['success'])
                self.assertEqual([names for names, _ in builds], [['first', 'second'], ['third']])
                self.assertEqual(executed, ['first', 'second', 'third'])
                self.assertEqual(result['expected_tests'], 3)
                self.assertEqual(len(result['tests']), 3)
                self.assertEqual(result['retired_native_test_binaries'], 3)
                self.assertTrue(all(not path.exists() for path in binaries))
                # Discovery and every batch share one compile budget.
                self.assertEqual([timeout for _, timeout in builds], [8, 6])

    def test_native_batches_execute_every_target_and_reclaim_before_next_compile(self):
        self.run_batched_fixture()

    def test_failed_native_batch_preserves_outputs_and_stops_before_next_compile(self):
        self.run_batched_fixture(fail_execution=True)

    def test_native_batches_reject_incomplete_compiler_target_lists(self):
        self.run_batched_fixture(omit_target=True)

    def test_native_retirement_preserves_outputs_while_processes_are_active(self):
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary).resolve()/'target'
            binary = target/'debug/deps'/('fixture.exe' if os.name == 'nt' else 'fixture')
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b'active test')
            with patch.object(native, 'active_peers', return_value={12345}):
                with self.assertRaisesRegex(RuntimeError, 'processes remain active'):
                    native.retire_test_binaries([binary], target)
            self.assertEqual(binary.read_bytes(), b'active test')

    def test_native_listing_repairs_missing_owner_execute_bit(self):
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary)/'target'
            binary = target/'debug/deps/fixture'
            binary.parent.mkdir(parents=True)
            log = Path(temporary)/'listing.log'
            if os.name == 'nt':
                denied = PermissionError(errno.EACCES, 'Permission denied', str(binary))
                with patch.object(native, 'run_command', side_effect=denied) as command:
                    with self.assertRaises(PermissionError):
                        native.list_compiled_tests(
                            binary, target=target, filter_name='', env=os.environ.copy(), log=log)
                self.assertEqual(command.call_count, 1)
                return
            binary.write_text('#!/bin/sh\nprintf "fixture_case: test\\n"\n')
            binary.chmod(0o600)
            with patch.object(native, 'group_members', return_value=set()):
                result = native.list_compiled_tests(
                    binary, target=target, filter_name='', env=os.environ.copy(), log=log)
            self.assertEqual(result['exit'], 0)
            self.assertTrue(binary.stat().st_mode & stat.S_IXUSR)
            self.assertIn('fixture_case: test', log.read_text())

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
            self.assertEqual(result['timeout_scope'], 'case')

    def test_suite_deadline_is_distinct_from_the_active_case_deadline(self):
        with tempfile.TemporaryDirectory() as temporary:
            code = "import time; print('test still_running ... ',end='',flush=True); time.sleep(20)"
            result = run_suite([sys.executable, '-c', code], env=os.environ,
                               log=Path(temporary)/'suite.log', timeout=2, test_timeout=10)
            self.assertEqual(result['exit'], 124)
            self.assertTrue(result['timed_out'])
            self.assertEqual(result['timed_out_case'], 'still_running')
            self.assertEqual(result['timeout_scope'], 'suite')
            self.assertFalse(result['orphaned_processes'])

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
