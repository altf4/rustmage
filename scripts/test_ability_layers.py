"""Whole effect identity survives extraction into different characteristic layers."""
import json
import unittest
from scripts import generate_catalogue as g
from scripts.ability_layers import Layer, continuous_groups


class AbilityLayerIdentityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((g.ROOT / 'data/standard/cards.json').read_text())['cards']}

    def groups(self, name):
        old = dict(g.TOKEN_DEFS)
        result = continuous_groups(self.cards[name], g)
        self.assertEqual(g.TOKEN_DEFS, old, 'metadata projection changed token factory registry')
        return result

    def test_job_type_keywords_trigger_and_stats_remain_one_earlier_started_effect(self):
        for name in ["Ninja's Blades", "Samurai's Katana", "White Mage's Staff", "Black Mage's Rod", "Paladin's Arms"]:
            with self.subTest(name=name):
                groups = self.groups(name)
                group = next(x for x in groups if any(f.kind == 'attached_subtype' for f in x.fragments))
                self.assertEqual(group.first_layer, Layer.TYPE)
                self.assertTrue(any(f.layer == Layer.ABILITY for f in group.fragments))
                self.assertTrue(any(f.layer == Layer.MODIFY for f in group.fragments))
                self.assertIn('in addition to its other types', group.raw)
                self.assertFalse(any('Equip' in x.raw and 'Equipped' not in x.raw for x in groups))

    def test_separate_earlier_and_later_abilities_are_not_grouped_by_physical_card(self):
        card = dict(self.cards['Bear Cub'], oracle_text='Changeling\nThis creature gets +2/+2 as long as you control three or more lands.\nAs long as you control three or more lands, this creature has flying.')
        groups = continuous_groups(card, g)
        self.assertEqual(len(groups), 3)
        self.assertEqual({x.line for x in groups}, {0, 1, 2})
        self.assertEqual(groups[0].first_layer, Layer.TYPE)
        self.assertEqual(groups[1].first_layer, Layer.MODIFY)
        self.assertEqual(groups[2].first_layer, Layer.ABILITY)

    def test_combined_conditional_clause_keeps_its_identity_through_synthetic_residual(self):
        card = dict(self.cards['Bear Cub'], oracle_text='As long as you control three or more lands, this creature gets +2/+2 and has flying.')
        groups = continuous_groups(card, g)
        self.assertEqual(len(groups), 1)
        self.assertEqual({f.layer for f in groups[0].fragments}, {Layer.ABILITY, Layer.MODIFY})
        self.assertEqual(groups[0].raw, card['oracle_text'])

    def test_attachment_stats_and_granted_ward_share_one_group_with_original_raw(self):
        for name in ['Lavaspur Boots', 'Super Strength', 'Ethereal Armor']:
            with self.subTest(name=name):
                groups = self.groups(name)
                self.assertEqual(len(groups), 1)
                self.assertEqual(groups[0].first_layer, Layer.ABILITY)
                self.assertTrue(any(f.layer == Layer.MODIFY for f in groups[0].fragments))
                if name != 'Ethereal Armor':
                    self.assertTrue(any(f.kind == 'attachment_trigger' for f in groups[0].fragments))
                self.assertIn(groups[0].raw, self.cards[name]['oracle_text'])


if __name__ == '__main__':
    unittest.main()
