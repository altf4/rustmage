"""Explicit metadata determines programs independently of reserved fixture names."""
import hashlib
import json
import unittest
from scripts import generate_catalogue as g


class LegacyMetadataTests(unittest.TestCase):
    @staticmethod
    def compile(cards):
        payload = json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
        return g.compile_catalogue(dict(cards=cards, card_count=len(cards),
            cards_sha256=hashlib.sha256(payload).hexdigest()), dict(banned_card_names=[]))

    @staticmethod
    def creature(name):
        return dict(name=name, oracle_id='test-metadata', layout='normal',
                    mana_cost='{2}{U}', type_line='Creature — Dragon', colors=['U'],
                    keywords=['Flying'], power='4', toughness='3', oracle_text='Flying')

    def definition(self, card):
        rust, coverage = self.compile([card])
        row = next(row for row in coverage['cards'] if row['name'] == card['name'])
        line = next(line for line in rust.splitlines()
                    if line.startswith('    Definition {') and f'name: {json.dumps(card["name"])}' in line)
        return line, row

    def test_all_reserved_names_compile_supplied_creature_semantics(self):
        for name, _, _, _ in g.LEGACY:
            with self.subTest(name=name):
                line, row = self.definition(self.creature(name))
                self.assertTrue(row['implemented'])
                for fragment in ['Effect::Creature(4, 3)', 'keywords: Keywords(1)',
                                 'kind: 1, basic: false, mana_color: None',
                                 'colored: [0, 1, 0, 0, 0, 0], generic: 2']:
                    self.assertIn(fragment, line)
                self.assertNotEqual(row['implementation'], 'explicit')

    def test_all_reserved_names_compile_supplied_spell_kind_target_amount_and_cost(self):
        for name, _, _, _ in g.LEGACY:
            with self.subTest(name=name):
                card = self.creature(name)
                card.update(type_line='Sorcery', mana_cost='{3}{R}{R}', colors=['R'],
                            keywords=[], oracle_text=f'{name} deals 7 damage to any target.')
                line, row = self.definition(card)
                self.assertTrue(row['implemented'])
                for fragment in ['Effect::Spell(', 'TargetRule::Any', 'Instruction::Damage(7)',
                                 'kind: 4, basic: false, mana_color: None',
                                 'colored: [0, 0, 0, 2, 0, 0], generic: 3']:
                    self.assertIn(fragment, line)

    def test_all_reserved_names_reject_unknown_body_and_malformed_shape(self):
        for name, _, _, _ in g.LEGACY:
            base = self.creature(name)
            mutations = [dict(oracle_text='Discover 4.'),
                         dict(oracle_text='Flying\nDiscover 4.'),
                         dict(oracle_text='Flying\nYou win the game.'),
                         dict(layout='unknown'),
                         dict(type_line='Creature Instant — Dragon'),
                         dict(type_line='Unrecognized — Dragon'),
                         dict(mana_cost='{Y}'), dict(mana_cost=''),
                         dict(mana_cost='{4294967296}'), dict(power='no'),
                         dict(toughness='unknown')]
            with self.subTest(name=name, invalid_text=None):
                with self.assertRaises((TypeError, ValueError)):
                    self.definition(dict(base, oracle_text=None))
            missing = dict(base)
            del missing['oracle_text']
            self.assertFalse(self.definition(missing)[1]['implemented'])
            for change in mutations:
                with self.subTest(name=name, change=change):
                    self.assertFalse(self.definition(dict(base, **change))[1]['implemented'])

    def test_explicit_mana_body_changes_are_not_overridden_by_fixture_color(self):
        for name in ['Llanowar Elves', 'Forest', 'Plains']:
            card = self.creature(name)
            card.update(power='1', toughness='1', keywords=[], oracle_text='{T}: Add {U}.')
            line, row = self.definition(card)
            self.assertTrue(row['implemented'])
            self.assertIn('mana_color: None', line)
            self.assertIn('ManaProduction::Fixed([0, 1, 0, 0, 0, 0])', line)
            self.assertIn('kind: 1, basic: false', line)
        # Even the old color only receives the compact route after the complete
        # generic compiler accepts the body; an extra unknown clause gates it.
        card.update(name='Llanowar Elves', oracle_text='{T}: Add {G}.')
        self.assertIn('mana_color: Some(4)', self.definition(card)[0])
        haste = dict(card, oracle_text='Haste\n{T}: Add {G}.', keywords=['Haste'])
        line, row = self.definition(haste)
        self.assertTrue(row['implemented'])
        self.assertIn('keywords: Keywords(4)', line)
        self.assertIn('mana_color: Some(4)', line)
        conditional = dict(card, oracle_text='{T}, Pay 1 life: Add {G}.')
        line, row = self.definition(conditional)
        self.assertTrue(row['implemented'])
        self.assertIn('mana_color: None', line)
        self.assertIn('life: 1', line)
        card['oracle_text'] += '\nDiscover 4.'
        self.assertFalse(self.definition(card)[1]['implemented'])

    def test_full_land_metadata_not_name_determines_basic_status_and_mana(self):
        for name, _, _, _ in g.LEGACY:
            card = dict(name=name, oracle_id='land', layout='normal', colors=[], keywords=[],
                        mana_cost='', type_line='Basic Land — Island', oracle_text='({T}: Add {U}.)')
            line, row = self.definition(card)
            self.assertTrue(row['implemented'])
            self.assertIn('Effect::Land', line)
            self.assertIn('kind: 2, basic: true', line)
            self.assertIn('deck_limit: None', line)
            # A genuine intrinsic reminder must match the supplied subtype.
            card['oracle_text'] = '({T}: Add {G}.)'
            self.assertFalse(self.definition(card)[1]['implemented'])

    def test_only_absent_fixture_identities_receive_fallback(self):
        rust, coverage = self.compile([])
        self.assertEqual(coverage['implemented_total'], 14)
        self.assertTrue(all(row['implementation'] == 'explicit' for row in coverage['cards']))
        original = next(card for card in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']
                        if card['name'] == 'Llanowar Elves')
        for change in [dict(oracle_text=''), dict(oracle_text='Flying'), dict(power='2')]:
            card = dict(original, **change)
            line, row = self.definition(card)
            self.assertTrue(row['implemented'])
            if 'oracle_text' in change:
                self.assertIn('mana_color: None', line)
                self.assertNotIn('Instruction::AddMana', line)
            else:
                self.assertIn('Effect::Creature(2, 1)', line)
                self.assertIn('mana_color: None', line)
                self.assertIn('Instruction::AddMana', line)


if __name__ == '__main__':
    unittest.main()
