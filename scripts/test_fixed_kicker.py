"""Closed complete-body kicker compiler and canonical preservation boundaries."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('kicker_generate', Path(__file__).with_name('generate_catalogue.py'))
generate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generate)


class FixedKickerCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.snapshot = json.loads((generate.ROOT / 'data/standard/cards.json').read_text())
        cls.facts = {c['name']: c for c in cls.snapshot['cards']}
        cls.target = json.loads((generate.ROOT / 'data/targets/standard-2026-10-15.json').read_text())
        cls.manifest = json.loads((generate.ROOT / 'docs/fixed-kicker-admissions.json').read_text())

    def test_ten_complete_bodies_and_exact_single_operations(self):
        self.assertEqual(len(self.manifest['cards']), 10)
        for row in self.manifest['cards']:
            with self.subTest(card=row['name']):
                card = self.facts[row['name']]
                self.assertEqual(card['oracle_text'], row['oracle_text'])
                stripped, cost = generate.extract_kicker(card)
                self.assertIsNotNone(cost)
                result = generate.fixed_kicker_body(stripped)
                self.assertIsNotNone(result)
                if row['name'] in ('Burst Lightning', 'Firebending Lesson'):
                    self.assertEqual(result[3][1][0], 'IfKicked { skip: 2 }')
                    self.assertEqual(result[3][1][2], 'Skip(1)')
                    self.assertEqual(len(result[3][1]), 4)
                if row['name'] == 'Gnarlid Colony':
                    self.assertEqual(result[2], [])
                    self.assertEqual(result[4:], (2, True))
                if row['name'] == 'Grow from the Ashes':
                    program = result[3][1]
                    self.assertEqual(len(program), 4)
                    self.assertIn('maximum: 2', program[1])
                    self.assertIn('maximum: 1', program[3])
                    self.assertNotIn('tapped: true', str(program))
                if row['name'] == "Zuko's Conviction":
                    self.assertEqual(result[3][1], ['IfKicked { skip: 2 }', 'ReturnToBattlefieldTapped', 'Skip(1)', 'ReturnToHand'])

    def test_fixed_cost_declaration_rejects_variable_repeat_duplicate_and_bad_reminders(self):
        card = self.facts['Burst Lightning']
        for line in ['Multikicker {4}', 'Kicker {X}', 'Kicker {2/R}', 'Kicker {R/P}', 'Kicker {4} and/or {U}', 'Kicker {4} (You may pay an additional {3} as you cast this spell.)', 'Kicker {4294967296}', 'Kicker {4}\nKicker {4}', 'Kicker {4} (When you do, draw a card.)']:
            changed = dict(card, oracle_text=line + '\n' + card['oracle_text'].split('\n', 1)[1])
            self.assertIsNone(generate.extract_kicker(changed)[1], line)
        for cost in ['{0}', '{3}{U}{U}', '{C}']:
            changed = dict(card, oracle_text='Kicker ' + cost + '\n' + card['oracle_text'].split('\n', 1)[1])
            self.assertIsNotNone(generate.extract_kicker(changed)[1], cost)

    def test_unknown_residuals_and_changed_scope_are_closed(self):
        for row in self.manifest['cards']:
            card, _ = generate.extract_kicker(self.facts[row['name']])
            for suffix in ['\nDiscover 3.', '\nCreate a token copy of this creature.', '\nGain control of target permanent.']:
                self.assertIsNone(generate.fixed_kicker_body(dict(card, oracle_text=card['oracle_text'] + suffix)))
        card, _ = generate.extract_kicker(self.facts['Gnarlid Colony'])
        for body in [card['oracle_text'].replace('two', 'three'), card['oracle_text'].replace('you control', 'on the battlefield'), card['oracle_text'].replace('a +1/+1 counter', 'a counter')]:
            self.assertIsNone(generate.fixed_kicker_body(dict(card, oracle_text=body)))

    def test_all_complete_bodies_admit_and_every_unknown_extra_gates(self):
        source, coverage = generate.compile_catalogue(self.snapshot, self.target)
        names = {c['name'] for c in self.manifest['cards']}
        supported = {c['name'] for c in coverage['cards'] if c['implemented']}
        self.assertTrue(names <= supported)
        self.assertFalse({c['name'] for c in self.manifest['gated_cards']} & supported)
        self.assertIn('pub(super) fn kicker_cost', source)
        # Fixed-copy machinery must not silently omit the extra immutable grants.
        cards = [dict(c, oracle_text='Offspring {2}\n' + c['oracle_text']) if c['name'] == 'Gnarlid Colony' else c for c in self.snapshot['cards']]
        combined = dict(self.snapshot, cards=cards, cards_sha256=hashlib.sha256(json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest())
        _, declined = generate.compile_catalogue(combined, self.target)
        self.assertFalse(next(c['implemented'] for c in declined['cards'] if c['name'] == 'Gnarlid Colony'))
        cards = [dict(c, oracle_text=c['oracle_text'] + '\nDiscover 3.') if c['name'] in names else c for c in self.snapshot['cards']]
        changed = dict(self.snapshot, cards=cards, cards_sha256=hashlib.sha256(json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest())
        _, invalid = generate.compile_catalogue(changed, self.target)
        self.assertFalse(names & {c['name'] for c in invalid['cards'] if c['implemented']})


if __name__ == '__main__':
    unittest.main()
