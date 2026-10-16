"""Closed intrinsic Mobilize declarations preserve every complete residual."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('mobilize_generate', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)
NAMES = ['Avenger of the Fallen', 'Dalkovan Packbeasts', 'Dragonback Lancer', 'Nightblade Brigade', 'Reigning Victor', 'Shock Brigade', 'Stadium Headliner', "Zurgo's Vanguard"]

class MobilizeCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.snapshot = json.loads((g.ROOT / 'data/standard/cards.json').read_text())
        cls.rows = {c['name']: c for c in cls.snapshot['cards']}
        cls.target = json.loads((g.ROOT / 'data/targets/standard-2026-10-15.json').read_text())

    def implemented(self, card):
        cards = [card]
        snapshot = dict(cards=cards, card_count=1, cards_sha256=hashlib.sha256(json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest())
        _, report = g.compile_catalogue(snapshot, dict(banned_card_names=[]))
        return next(c for c in report['cards'] if c['name'] == card['name'])['implemented']

    def test_eight_complete_bodies_and_all_victor_cost_options(self):
        _, report = g.compile_catalogue(self.snapshot, self.target)
        supported = {c['name'] for c in report['cards'] if c['implemented']}
        for name in NAMES:
            self.assertIn(name, supported)
            self.assertIsNotNone(g.mobilize_body(g.extract_stat_rules(self.rows[name])[0]), name)
        victor = self.rows['Reigning Victor']
        self.assertEqual(len(g.spell_cost_options(victor['mana_cost'])), 8)
        for name in ['Bone-Cairn Butcher', 'Venerated Stormsinger', 'Voice of Victory', "Zurgo, Thunder's Decree"]:
            self.assertNotIn(name, supported)

    def test_unknown_dynamic_counts_duplicate_declarations_and_residuals_reject(self):
        base = self.rows['Dalkovan Packbeasts']
        for replacement in ['Mobilize Y', 'Mobilize 65536', 'Mobilize -1', 'Mobilize X, where X is the number of lands you control.', 'Mobilize 3\nMobilize 2']:
            with self.subTest(replacement=replacement):
                self.assertFalse(self.implemented(dict(base, oracle_text=base['oracle_text'].replace('Mobilize 3', replacement))))
        for name in NAMES:
            self.assertFalse(self.implemented(dict(self.rows[name], oracle_text=self.rows[name]['oracle_text'] + '\nDiscover 3.')), name)
        for number in [0, 1, 65535]:
            self.assertIsNotNone(g.mobilize_body(dict(base, oracle_text=f'Mobilize {number}')))

    def test_printed_costs_are_bounded_without_rejecting_valid_widened_totals(self):
        base = self.rows['Dalkovan Packbeasts']
        for cost in ['{X}{W}', '{Q}', '{4294967296}', '{4294967295}{1}', '{2/Z}', '{W/P}', '', '{W}junk']:
            with self.subTest(cost=cost):
                self.assertFalse(self.implemented(dict(base, mana_cost=cost)))
        self.assertTrue(self.implemented(dict(base, mana_cost='{4294967295}{W}')))
        self.assertTrue(self.implemented(self.rows['Reigning Victor']))

    def test_cost_and_history_composites_never_revive_partial_bodies(self):
        declarations = [self.rows['Analyze the Pollen']['oracle_text'].splitlines()[0], self.rows['Candy Grapple']['oracle_text'].splitlines()[0], 'Saddle 1', 'Kicker {1}', 'Offspring {1}', 'Convoke', 'Cycling {1}', 'Flashback {1}']
        for name in NAMES:
            for declaration in declarations:
                with self.subTest(name=name, declaration=declaration):
                    self.assertFalse(self.implemented(dict(self.rows[name], oracle_text=declaration + '\n' + self.rows[name]['oracle_text'])))
