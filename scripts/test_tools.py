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
import card_coverage


class ToolContracts(unittest.TestCase):
    def test_feature_inventory_requires_real_rows_and_every_printed_paragraph(self):
        import json
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            def write(path, value):
                destination = root/path
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_text(json.dumps(value))
            write('data/catalogue-identities.json', dict(physical_count=1, cards=[
                dict(index=0, name='Fixture', oracle_id='oracle-fixture')]))
            write('data/standard/coverage.json', dict(cards=[dict(
                name='Fixture', implemented=True, definition='src/card_definitions/test/fixture.rs')]))
            write('data/standard/cards.json', dict(cards=[dict(
                name='Fixture', oracle_id='oracle-fixture', oracle_text='Draw a card.')]))
            write('src/card_definitions/test/fixture.json', dict(facts_sha256='pinned-facts'))
            write('src/testing/coverage/shared.json', {})
            proof = dict(case='native::draw', row='fixture/draw', outcomes=['draws one card'])
            entry = dict(catalogue_id=0, oracle_id='oracle-fixture', facts_sha256='pinned-facts',
                         choices_reviewed=True, features=[dict(id='draw', proof=proof, source=[
                             dict(face=0, paragraph=0, text='Draw a card.')])],
                         casting_routes=[dict(id='cast.normal', proof=proof)])
            write('src/testing/coverage/test.json', dict(schema=1, cards=[entry]))
            native = dict(success=True, tests=[dict(test='native::draw', status='ok')],
                          features=[dict(case='native::draw', row='fixture/draw')])
            self.assertTrue(card_coverage.check(root, native)['success'])
            native['features'] = []
            self.assertFalse(card_coverage.check(root, native)['success'])
            native['features'] = [dict(case='native::draw', row='fixture/draw')]
            entry['features'] = []
            write('src/testing/coverage/test.json', dict(schema=1, cards=[entry]))
            result = card_coverage.check(root, native)
            self.assertFalse(result['success'])
            self.assertTrue(any(error['reason'] == 'unmapped printed paragraph' for error in result['errors']))

            # A shared configuration needs all three independent witnesses:
            # its representative outcome, this member's binding, and its cast.
            family_id = 'plain_normal_spell_v1/draw/controller/1'
            write('src/testing/oracle_families.json', [dict(
                id=family_id, representative='Fixture',
                members=[dict(card='Fixture', catalogue_id=0)])])
            entry['features'] = [dict(id='draw', source=[
                dict(face=0, paragraph=0, text='Draw a card.')],
                proof=dict(family=family_id, member=0))]
            write('src/testing/coverage/test.json', dict(schema=1, cards=[entry]))
            gameplay = 'engine::testing::plain_families::representative_outcomes_and_each_members_normal_cast'
            binding = 'cards::coverage_bindings::oracle_families_bind_the_engine_programs'
            native['tests'] += [dict(test=gameplay, status='ok'), dict(test=binding, status='ok')]
            witnesses = [dict(case=gameplay, row='family/'+family_id),
                         dict(case=binding, row='card/0/binding/'+family_id),
                         dict(case=gameplay, row='card/0/cast/normal')]
            native['features'] += witnesses
            self.assertTrue(card_coverage.check(root, native)['success'])
            for missing in witnesses:
                native['features'].remove(missing)
                self.assertFalse(card_coverage.check(root, native)['success'])
                native['features'].append(missing)
            entry['features'][0]['proof']['member'] = 1
            write('src/testing/coverage/test.json', dict(schema=1, cards=[entry]))
            self.assertFalse(card_coverage.check(root, native)['success'])
            entry['features'][0]['proof'] = dict(all=[dict(family=family_id, member=0),
                                                     dict(shared='mechanic')])
            write('src/testing/coverage/shared.json', dict(mechanic=dict(
                case='native::mechanic', row='mechanic', outcomes=['shared rule outcome'])))
            write('src/testing/coverage/test.json', dict(schema=1, cards=[entry]))
            native['tests'].append(dict(test='native::mechanic', status='ok'))
            self.assertFalse(card_coverage.check(root, native)['success'])
            native['features'].append(dict(case='native::mechanic', row='mechanic'))
            self.assertTrue(card_coverage.check(root, native)['success'])

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
