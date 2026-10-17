"""Whole effect identity survives extraction into different characteristic layers."""
import json
import re
import unittest
from scripts import generate_catalogue as g
from scripts.ability_layers import Layer, continuous_groups
from scripts.catalogue_preservation import canonical


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

    def test_every_existing_stat_and_keyword_helper_reconstructs_exactly_from_groups(self):
        cards = list(self.cards.values())
        physical = {c['oracle_id']: c['name'] for c in cards}
        catalogue = canonical((g.ROOT / 'src/generated/catalogue.rs').read_text(), physical)
        families = {'static_modifiers', 'keyword_grants', 'scaling_modifiers',
                    'enchanted_modifiers', 'counter_keyword_grants', 'dynamic_stats',
                    'base_stat_setter', 'attached_subtype', 'replacement_effects',
                    'cast_timing_grants', 'cost_reductions', 'target_cost_reduction',
                    'combat_restriction', 'entry_trigger_suppression', 'opponent_entry_tapped'}
        checked = 0
        for card in cards:
            key = card['name'] + '#0'
            expected = {fn: payload for (fn, definition), payload in catalogue['helpers'].items()
                        if definition == key and fn in families}
            if not expected:
                continue
            old = dict(g.TOKEN_DEFS)
            groups = continuous_groups(card, g)
            self.assertEqual(g.TOKEN_DEFS, old)
            rows = {}
            for group in groups:
                for fragment in group.fragments:
                    rows.setdefault(fragment.kind, []).append(fragment.payload)
            for family, payload in expected.items():
                with self.subTest(card=card['name'], family=family):
                    values = rows.get(family, [])
                    if family == 'attached_subtype':
                        self.assertEqual(len(values), 1)
                        actual = 'Some(' + json.dumps(values[0]) + '),'
                    elif family in ('entry_trigger_suppression', 'opponent_entry_tapped'):
                        self.assertEqual(len(values), 1)
                        actual = str(values[0]) + ','
                    elif family == 'target_cost_reduction':
                        self.assertEqual(len(values), 1)
                        actual = 'Some(&' + values[0] + '),'
                    elif family in ('dynamic_stats', 'base_stat_setter', 'combat_restriction'):
                        self.assertEqual(len(values), 1)
                        actual = 'Some(' + values[0] + '),'
                    else:
                        actual = '&[' + ', '.join(values) + '],'
                    def token(match):
                        key = '@token/' + match[1] + '#0'
                        self.assertIn(key, catalogue['definitions'])
                        return 'Card(' + json.dumps(key) + ')'
                    actual = re.sub(r'__TOKEN_([a-f0-9]+)__', token, actual)
                    self.assertEqual(actual, payload)
                    checked += 1
        # Initial merged100 corpus has221 such helper arms. New parent arms must
        # also reconstruct; this is not an exemption for future admissions.
        self.assertGreaterEqual(checked, 221)

    def test_actual_live_static_parent_keeps_archon_group_and_later_only_providers(self):
        for name, first in [('Maha, Its Feathers Night', Layer.BASE), ('Aettir and Priwen', Layer.BASE), ('Archon of the Wild Rose', Layer.ABILITY), ('Burden of Proof', Layer.BASE)]:
            with self.subTest(name=name):
                groups = self.groups(name)
                live = [x for x in groups if any(f.kind.startswith('live_static_') for f in x.fragments)]
                self.assertEqual(len(live), 1)
                self.assertEqual(live[0].first_layer, first)
                if name == 'Archon of the Wild Rose':
                    self.assertEqual({f.layer for f in live[0].fragments}, {Layer.ABILITY, Layer.BASE})
                if name == 'Burden of Proof':
                    self.assertEqual({f.layer for f in live[0].fragments}, {Layer.BASE, Layer.MODIFY, Layer.RULE})

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
