import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('endure_compiler', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)

class EndureCompilerTests(unittest.TestCase):
    def test_complete_fixed_bodies_and_exact_nonflying_factories(self):
        snapshot = json.loads((g.ROOT / 'data/standard/cards.json').read_text())
        target = json.loads((g.ROOT / 'data/targets/standard-2026-10-15.json').read_text())
        rust, report = g.compile_catalogue(snapshot, target)
        manifest = json.loads((g.ROOT / 'docs/endure-admissions.json').read_text())
        supported = {c['name'] for c in report['cards'] if c['implemented']}
        self.assertEqual(len(manifest['cards']), 8)
        for row in manifest['cards']:
            self.assertIn(row['name'], supported)
            original = next(c for c in snapshot['cards'] if c['name'] == row['name'])
            self.assertEqual((row['oracle_text'], row['mana_cost']), (original['oracle_text'], original['mana_cost']))
            self.assertIsNotNone(g.endure_body(original))
        for row in manifest['gated']:
            self.assertNotIn(row['name'], supported)
        for amount in [1, 2, 3]:
            self.assertIn(f'Endure {{ amount: {amount}, token: Card(', rust)
        self.assertIn('MayPayMana { cost: ManaCost { colored: [1, 0, 0, 0, 0, 0], generic: 1 }, skip: 1 }', rust)
        self.assertEqual(report['token_definitions'], 80)
        # Components are u32; a total above u32 is still legal when each distinct
        # component fits. Only overflow within a component must be rejected.
        card = next(c for c in snapshot['cards'] if c['name'] == 'Fortress Kin-Guard')
        boundary = dict(card, mana_cost='{4294967295}{W}')
        one = dict(cards=[boundary], card_count=1)
        one['cards_sha256'] = hashlib.sha256(json.dumps(one['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        _, boundary_report = g.compile_catalogue(one, dict(banned_card_names=[]))
        self.assertTrue(next(c for c in boundary_report['cards'] if c['name'] == card['name'])['implemented'])


    def test_unknown_residuals_numeric_and_composed_payment_bodies_remain_closed(self):
        snapshot = json.loads((g.ROOT / 'data/standard/cards.json').read_text())
        rows = {c['name']: c for c in snapshot['cards']}
        manifest = json.loads((g.ROOT / 'docs/endure-admissions.json').read_text())
        evidence = rows['Analyze the Pollen']['oracle_text'].splitlines()[0] + '\n'
        for row in manifest['cards']:
            card = rows[row['name']]
            mutants = [dict(card, oracle_text=card['oracle_text'] + '\nDiscover 3.'), dict(card, mana_cost='{X}' + card['mana_cost'])]
            for prefix in ['Convoke\n', 'Bargain\n', 'Kicker {1}\n', 'Offspring {1}\n', evidence]:
                mutants.append(dict(card, oracle_text=prefix + card['oracle_text']))
            mutants += [dict(card, mana_cost=cost) for cost in ['{Y}', '{W/P}', '{Q}', '{4294967296}', '{4294967295}{1}{W}']]
            for mutant in mutants:
                with self.subTest(name=card['name'], cost=mutant['mana_cost'], text=mutant['oracle_text']):
                    one = dict(cards=[mutant], card_count=1)
                    one['cards_sha256'] = hashlib.sha256(json.dumps(one['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                    _, report = g.compile_catalogue(one, dict(banned_card_names=[]))
                    self.assertFalse(next(c for c in report['cards'] if c['name'] == card['name'])['implemented'])
