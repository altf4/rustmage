"""Intrinsic reminders authenticate their own semantic production and position."""
import copy
import json
import unittest
from scripts import test_legacy_metadata as legacy
from scripts import generate_catalogue as g


class IntrinsicReminderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}

    def admitted(self, card):
        return legacy.LegacyMetadataTests().definition(card)[1]['implemented']

    def test_real_complete_intrinsic_bodies_preserve_renamed_programs(self):
        for name in ["Healer's Hawk", 'Serra Angel', 'Swiftblade Vindicator',
                     'Thornweald Archer', 'Vampire Nighthawk', 'Chitinous Graspling',
                     'Raging Redcap', 'Brazen Scourge', 'Magnigoth Sentry']:
            card = copy.deepcopy(self.cards[name])
            self.assertTrue(self.admitted(card), name)
            card['name'] = 'Independent Intrinsic Subject'
            self.assertTrue(self.admitted(card), name)

    def test_unknown_parentheticals_gate_all_reserved_names_and_generic_fallback(self):
        for name, _, _, _ in [*g.LEGACY, ('Independent Intrinsic Subject', None, None, None)]:
            for body in ['Flying (You win the game.)', 'Flying\n(You win the game.)',
                         '(You win the game.)', 'Fly(You win the game.)ing',
                         '((You win the game.))', '(Flying (You win the game.))',
                         'Flying ((You win the game.))', 'Flying (', 'Flying )',
                         'Flying (You win the game.) )']:
                card = legacy.LegacyMetadataTests.creature(name)
                card['oracle_text'] = body
                with self.subTest(name=name, body=body):
                    self.assertFalse(self.admitted(card))

    def test_known_reminder_cannot_be_stolen_moved_nested_or_mutated(self):
        reach = 'This creature can block creatures with flying.'
        vigilance = "Attacking doesn't cause this creature to tap."
        for body in [f'Flying ({reach})', f'Reach ({vigilance})', f'({reach})\nReach',
                     f'Reach\n({reach})', f'Reach (({reach}))',
                     f'Re({reach})ach', f'Reach ({reach} You win the game.)',
                     f'Reach ({reach.replace("can block", "cannot block")})',
                     f'Reach ({reach}) ({reach})',
                     f'Reach, flying ({reach})']:
            card = legacy.LegacyMetadataTests.creature('Independent Intrinsic Subject')
            card['oracle_text'] = body
            with self.subTest(body=body):
                self.assertFalse(self.admitted(card))
        card['oracle_text'] = f'Flying, reach ({reach})'
        self.assertTrue(self.admitted(card))

    def test_every_accepted_intrinsic_keyword_rejects_injected_rule(self):
        for keyword in g.INTRINSIC_KEYWORDS:
            card = legacy.LegacyMetadataTests.creature('Independent Intrinsic Subject')
            card['oracle_text'] = keyword
            self.assertTrue(self.admitted(card), keyword)
            for suffix in [' (You win the game.)', '\n(Draw nineteen cards.)']:
                with self.subTest(keyword=keyword, suffix=suffix):
                    self.assertFalse(self.admitted(dict(card, oracle_text=keyword+suffix)))

    def test_real_basic_and_dual_intrinsic_mana_uses_actual_types(self):
        for name in ['Plains', 'Island', 'Swamp', 'Mountain', 'Forest']:
            card = copy.deepcopy(self.cards[name])
            self.assertTrue(self.admitted(card))
            for text in [card['oracle_text']+' (You win the game.)',
                         '('+card['oracle_text']+')', '(You win the game.)',
                         card['oracle_text'].replace('Add', 'A(You win the game.)dd')]:
                self.assertFalse(self.admitted(dict(card, oracle_text=text)), (name,text))
        card = copy.deepcopy(self.cards['Forest'])
        card.update(name='Independent Dual', type_line='Land — Island Forest',
                    oracle_text='({T}: Add {G} or {U}.)')
        self.assertTrue(self.admitted(card))
        for text in ['({T}: Add {G} or {B}.)', '({T}: Add {G}{U}.)',
                     '({T}: Add {G} or {U}. You win the game.)']:
            self.assertFalse(self.admitted(dict(card, oracle_text=text)), text)
        self.assertFalse(self.admitted(dict(card, type_line='Land — Forest')))

    def test_reminder_cannot_supply_missing_rules_or_change_real_keyword_body(self):
        for keyword in ['Flying','Reach','Vigilance']:
            card = legacy.LegacyMetadataTests.creature('Independent Intrinsic Subject')
            for body in [f'({keyword})', f'(({keyword}))', f'Flying ({keyword})']:
                self.assertFalse(self.admitted(dict(card, oracle_text=body)), body)
        card['oracle_text'] = 'Reach (This creature can block creatures with flying.)'
        rust, coverage = legacy.LegacyMetadataTests.compile([card])
        self.assertTrue(next(r['implemented'] for r in coverage['cards'] if r['name']==card['name']))
        self.assertIn('keywords: Keywords(2)', rust)
        self.assertNotIn('keywords: Keywords(3)', rust)


if __name__ == '__main__':
    unittest.main()
