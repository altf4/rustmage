"""Validator orchestration contracts; external commands are deterministic fixtures."""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import validate


class ValidatorTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.root = self.directory/'checkout'
        self.cache = self.directory/'cache'
        (self.root/'src/card_definitions').mkdir(parents=True)
        (self.root/'data/standard').mkdir(parents=True)
        (self.root/'data/standard/coverage.json').write_text(
            json.dumps({'implemented_standard_cards': 7}))
        self.source = self.root/'src/lib.rs'
        self.source.write_text('first source revision\n')
        self.commands = []
        self.command_timeouts = []
        self.python_libraries = []
        self.library_bytes = b'first compiled library'
        self.release_exit = 0
        self.on_command = lambda command: None
        self.active = set()
        self.previous_cwd = Path.cwd()
        self.addCleanup(os.chdir, self.previous_cwd)
        patches = [
            patch.object(validate, 'ROOT', self.root),
            patch.object(validate, 'bounded_group', return_value=Path('/fixture-cgroup')),
            patch.object(validate, 'active_peers', side_effect=lambda: self.active),
            patch.object(validate, 'source_files', side_effect=self.source_files),
            patch.object(validate, 'run_command', side_effect=self.run_command),
            patch.object(validate.subprocess, 'check_output', return_value='fixture-head\n'),
            patch.object(validate.signal, 'signal'),
            patch.object(validate.os, 'execv', side_effect=AssertionError('unexpected guard re-entry')),
            patch.dict(os.environ, {'XDG_CACHE_HOME': str(self.cache)}),
        ]
        for mocked in patches:
            mocked.start()
            self.addCleanup(mocked.stop)
        os.environ.pop('CARGO_TARGET_DIR', None)
        os.environ.pop('RUSTMAGE_LIBRARY', None)

    def source_files(self):
        return {'src/lib.rs': hashlib.sha256(self.source.read_bytes()).hexdigest()}

    def run_command(self, command, *, env, log, timeout):
        command = [str(argument) for argument in command]
        target = Path(env['CARGO_TARGET_DIR'])
        self.commands.append((command, dict(env)))
        self.command_timeouts.append(timeout)
        target.mkdir(parents=True, exist_ok=True)
        log = Path(log)
        log.parent.mkdir(parents=True, exist_ok=True)
        log.write_text('fixture command completed\n')
        status = 0
        if 'scripts/run_native_tests.py' in command:
            intermediate = target/'debug/incremental/fixture.o'
            intermediate.parent.mkdir(parents=True, exist_ok=True)
            intermediate.write_bytes(b'reusable compiled object')
        if command[:2] == ['cargo', 'build']:
            status = self.release_exit
            if status == 0:
                library = target/'release/librustmage.so'
                library.parent.mkdir(parents=True, exist_ok=True)
                library.write_bytes(self.library_bytes)
        if 'scripts/python_tests.py' in command:
            library = Path(env['RUSTMAGE_LIBRARY'])
            self.python_libraries.append((library, library.read_bytes()))
        if Path(command[0]).name == 'benchmark':
            log.write_text(json.dumps({'games': 2, 'games_per_second': 200}))
        self.on_command(command)
        return dict(command=command, exit=status, seconds=0.001, log=str(log),
                    timed_out=False, orphaned_processes=False)

    def reports(self):
        return set(self.cache.glob('rustmage-validation-*/report/result.json'))

    def invoke(self, *arguments):
        before = self.reports()
        first_command = len(self.commands)
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            status = validate.main(list(arguments))
        new_reports = self.reports() - before
        self.assertEqual(len(new_reports), 1)
        path = new_reports.pop()
        return status, path, json.loads(path.read_text()), self.commands[first_command:]

    def assert_snapshot(self, report_path, report, expected_bytes):
        library, consumed = self.python_libraries[-1]
        self.assertEqual(consumed, expected_bytes)
        self.assertTrue(library.is_relative_to(report_path.parent/'libraries'))
        self.assertFalse(library.is_symlink())
        self.assertEqual(library.read_bytes(), expected_bytes)
        self.assertEqual(report['shared_libraries'][str(library)],
                         hashlib.sha256(expected_bytes).hexdigest())
        mutable = Path(report['target'])/'release/librustmage.so'
        self.assertNotEqual(library.stat().st_ino, mutable.stat().st_ino)
        return library

    def test_repeated_runs_reuse_builds_rerun_checks_and_keep_independent_evidence(self):
        status, first_path, first, first_commands = self.invoke('--release', '--python', 'smoke')
        self.assertEqual(status, 0)
        self.assertTrue(first['success'])
        self.assertFalse(first['target_preexisting'])
        self.assertFalse(first['fresh_target'])
        target = Path(first['target'])
        self.assertTrue(target.is_relative_to(self.cache/'rustmage-targets'))
        original_snapshot = self.assert_snapshot(first_path, first, self.library_bytes)
        original_digest = first['shared_libraries'][str(original_snapshot)]
        object_file = target/'debug/incremental/fixture.o'
        self.assertEqual(object_file.read_bytes(), b'reusable compiled object')
        sentinel = target/'preserve-unrelated-build-output'
        sentinel.write_bytes(b'keep me')

        status, unchanged_path, unchanged, unchanged_commands = self.invoke(
            '--release', '--python', 'smoke')
        self.assertEqual(status, 0)
        self.assertEqual(first['target'], unchanged['target'])
        self.assertTrue(unchanged['target_preexisting'])
        self.assertNotEqual(first_path, unchanged_path)
        self.assertEqual(first['input_sha256'], unchanged['input_sha256'])
        self.assertEqual([phase['name'] for phase in first['phases']],
                         [phase['name'] for phase in unchanged['phases']])
        self.assertEqual(len(first_commands), len(unchanged_commands))
        self.assert_snapshot(unchanged_path, unchanged, self.library_bytes)

        self.source.write_text('second source revision\n')
        self.library_bytes = b'second compiled library'
        status, second_path, second, second_commands = self.invoke('--release', '--python', 'smoke')
        self.assertEqual(status, 0)
        self.assertEqual(first['target'], second['target'])
        self.assertTrue(second['target_preexisting'])
        self.assertNotEqual(first_path, second_path)
        self.assertNotEqual(first['input_sha256'], second['input_sha256'])
        self.assertTrue(first['input_sha256'])
        self.assertEqual(first['changed'], [])
        self.assertEqual(second['changed'], [])
        required_phases = {'source pins', 'format', 'card format', 'script contracts',
                           'clippy', 'native assurance', 'release build',
                           'simulation sample', 'Python assurance'}
        self.assertTrue(required_phases.issubset({phase['name'] for phase in first['phases']}))
        self.assertEqual([phase['name'] for phase in first['phases']],
                         [phase['name'] for phase in second['phases']])
        self.assertEqual(len(first_commands), len(second_commands))
        self.assertEqual(sentinel.read_bytes(), b'keep me')
        self.assertTrue(object_file.is_file())
        second_snapshot = self.assert_snapshot(second_path, second, self.library_bytes)
        self.assertNotEqual(original_snapshot, second_snapshot)
        self.assertEqual(hashlib.sha256(original_snapshot.read_bytes()).hexdigest(), original_digest)
        self.assertEqual(json.loads(first_path.read_text()), first)

    def test_native_compile_budget_fits_inside_parent_deadline(self):
        status, _, report, commands = self.invoke('--timeout', '1800')
        self.assertEqual(status, 0)
        index = next(i for i, (command, _) in enumerate(commands)
                     if 'scripts/run_native_tests.py' in command)
        command = commands[index][0]
        self.assertEqual(command[command.index('--compile-timeout') + 1], '900')
        self.assertEqual(command[command.index('--test-timeout') + 1], '45')
        self.assertEqual(command[command.index('--suite-timeout') + 1], '240')
        self.assertGreaterEqual(self.command_timeouts[index], 1140)
        self.assertLessEqual(self.command_timeouts[index], 1160)
        self.assertTrue(report['success'])

    def test_environment_target_is_reused_and_explicit_target_takes_precedence(self):
        environment_target = self.directory/'environment-target'
        environment_target.mkdir()
        sentinel = environment_target/'cached-object'
        sentinel.write_bytes(b'cached')
        os.environ['CARGO_TARGET_DIR'] = str(environment_target)
        status, _, report, _ = self.invoke()
        self.assertEqual(status, 0)
        self.assertEqual(Path(report['target']), environment_target)
        self.assertTrue(report['target_preexisting'])
        self.assertEqual(sentinel.read_bytes(), b'cached')
        explicit_target = self.directory/'explicit-target'
        status, _, report, commands = self.invoke('--target-dir', str(explicit_target))
        self.assertEqual(status, 0)
        self.assertEqual(Path(report['target']), explicit_target)
        self.assertTrue(all(Path(env['CARGO_TARGET_DIR']) == explicit_target for _, env in commands))

    def test_failed_release_never_publishes_or_consumes_a_stale_library(self):
        target = self.directory/'reused-target'
        stale = target/'release/librustmage.so'
        stale.parent.mkdir(parents=True)
        stale.write_bytes(b'stale library from an earlier build')
        self.release_exit = 9
        status, _, report, commands = self.invoke('--target-dir', str(target), '--python', 'smoke')
        self.assertNotEqual(status, 0)
        self.assertFalse(report['success'])
        self.assertEqual(report['shared_libraries'], {})
        self.assertEqual(self.python_libraries, [])
        self.assertFalse(any('scripts/python_tests.py' in command for command, _ in commands))
        self.assertEqual(stale.read_bytes(), b'stale library from an earlier build')
        self.assertTrue((target/'debug/incremental/fixture.o').is_file())

    def test_source_changes_during_a_run_invalidate_the_result(self):
        def change_source(command):
            if command[:2] == ['cargo', 'clippy']:
                self.source.write_text('changed while a consuming phase was running\n')
        self.on_command = change_source
        status, _, report, _ = self.invoke()
        self.assertNotEqual(status, 0)
        self.assertFalse(report['success'])
        self.assertEqual(report['changed'], ['src/lib.rs'])
        self.assertEqual(report['files_verified'], 1)

    def assert_rejected_before_execution(self, *arguments):
        before = len(self.commands)
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            try:
                status = validate.main(list(arguments))
            except SystemExit as error:
                status = error.code
        self.assertNotEqual(status, 0)
        self.assertEqual(len(self.commands), before)

    def test_source_tree_and_legacy_retained_targets_are_rejected_before_commands(self):
        self.assert_rejected_before_execution('--target-dir', str(self.root/'target'))
        alias = self.directory/'source-alias'
        alias.symlink_to(self.root, target_is_directory=True)
        self.assert_rejected_before_execution('--target-dir', str(alias/'target'))
        legacy = self.directory/'legacy-target'
        (legacy/'report').mkdir(parents=True)
        pinned = legacy/'release/librustmage.so'
        pinned.parent.mkdir()
        pinned_bytes = b'library pinned by a prior validation report'
        pinned.write_bytes(pinned_bytes)
        marker = legacy/'report/result.json'
        marker.write_text(json.dumps({'shared_libraries': {
            str(pinned): hashlib.sha256(pinned_bytes).hexdigest()}}))
        marker_bytes = marker.read_bytes()
        self.assert_rejected_before_execution('--target-dir', str(legacy))
        self.assertEqual(pinned.read_bytes(), pinned_bytes)
        self.assertFalse(pinned.is_symlink())
        self.assertEqual(marker.read_bytes(), marker_bytes)

    def test_fresh_target_is_disposable_after_snapshot_and_does_not_use_environment_target(self):
        reusable = self.directory/'environment-target'
        reusable.mkdir()
        sentinel = reusable/'cached-object'
        sentinel.write_bytes(b'keep cached build')
        os.environ['CARGO_TARGET_DIR'] = str(reusable)
        status, path, report, _ = self.invoke('--fresh', '--python', 'smoke')
        self.assertEqual(status, 0)
        self.assertTrue(report['fresh_target'])
        target = Path(report['target'])
        self.assertNotEqual(target, reusable)
        library, consumed = self.python_libraries[-1]
        self.assertEqual(consumed, self.library_bytes)
        self.assertTrue(library.is_relative_to(path.parent/'libraries'))
        self.assertEqual(library.read_bytes(), self.library_bytes)
        self.assertEqual(report['shared_libraries'][str(library)],
                         hashlib.sha256(self.library_bytes).hexdigest())
        self.assertEqual([entry for entry in target.rglob('*') if entry.is_file()], [])
        self.assertEqual(sentinel.read_bytes(), b'keep cached build')

    def test_fresh_and_explicit_target_are_mutually_exclusive(self):
        self.assert_rejected_before_execution('--fresh', '--target-dir', str(self.directory/'target'))

    def test_nonterminal_run_preserves_outputs_and_does_not_publish_snapshots(self):
        self.active = {12345}
        status, _, report, _ = self.invoke('--fresh', '--release')
        self.assertNotEqual(status, 0)
        self.assertFalse(report['success'])
        self.assertFalse(report['terminal'])
        self.assertEqual(report['active_processes'], [12345])
        self.assertEqual(report['shared_libraries'], {})
        target = Path(report['target'])
        self.assertEqual((target/'release/librustmage.so').read_bytes(), self.library_bytes)
        self.assertTrue((target/'debug/incremental/fixture.o').is_file())


if __name__ == '__main__':
    unittest.main(verbosity=2)
