"""Complete semantic Role bodies, generic predicates and raw text authentication."""
import copy
import json
import unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata

ENCHANTED = ['A Tale for the Ages', 'Syr Armont, the Redeemer']

class RoleSemanticCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}

    def test_complete_return_binds_successful_entry_and_authenticates_young_hero_reminder(self):
        original=self.cards['Return Triumphant']
        before,row=metadata.LegacyMetadataTests().definition(original)
        self.assertTrue(row['implemented'])
        self.assertIn('Instruction::ReturnToBattlefieldWithRole',before)
        self.assertIn('zone: TargetZone::Graveyard, kinds: 1, relation: 0',before)
        self.assertIn('max_mana: 3',before)
        renamed=dict(original,name='Independent Returned Subject')
        after,row=metadata.LegacyMetadataTests().definition(renamed)
        self.assertTrue(row['implemented'])
        self.assertEqual(before.replace(json.dumps(original['name']),json.dumps(renamed['name'])),after)
        for text in [original['oracle_text']+'\nDiscover 4.',
                     original['oracle_text']+' (Draw seven cards.)',
                     original['oracle_text'].replace('toughness is 3 or less','toughness is 4 or less'),
                     original['oracle_text'].replace('Young Hero Role','Wicked Role'),
                     original['oracle_text'].replace('attached to it.','attached to it. (Discover 4.)'),
                     original['oracle_text'].replace('Role token attached','Role token (Draw seven cards.) attached')]:
            self.assertFalse(metadata.LegacyMetadataTests().definition(dict(original,oracle_text=text))[1]['implemented'],text)

    def test_complete_enchanted_bodies_are_generic_and_preserve_full_residual(self):
        for name in ENCHANTED:
            original=self.cards[name]
            before,row=metadata.LegacyMetadataTests().definition(original)
            self.assertTrue(row['implemented'])
            self.assertIn('EnchantedModifier',metadata.LegacyMetadataTests.compile([original])[0])
            card=copy.deepcopy(original);renamed='Independent Enchanted Subject'
            card['oracle_text']=card['oracle_text'].replace(name,renamed).replace(name.split(',')[0],renamed)
            card['name']=renamed
            after,row=metadata.LegacyMetadataTests().definition(card)
            self.assertTrue(row['implemented'])
            self.assertEqual(before.replace(json.dumps(name),json.dumps(renamed)),after)
            if name.startswith('Syr'):
                self.assertIn('TriggerCondition::SelfEnters',before)
                self.assertIn('Instruction::CreateRole',before)
                self.assertIn('other: true',before)
                self.assertIn('relation: 0',before)

    def test_enchanted_predicate_cannot_hide_unknown_reminders_or_executable_tails(self):
        for name in ENCHANTED:
            original=self.cards[name]
            for tail in ['\nDiscover 4.',' (Draw seven cards.)','\n(Draw seven cards.)',' ((Draw seven cards.))',' (',' )']:
                card=dict(original,oracle_text=original['oracle_text']+tail)
                self.assertFalse(metadata.LegacyMetadataTests().definition(card)[1]['implemented'],(name,tail))
            for reminder in ['(Draw seven cards.)','((Draw seven cards.))']:
                card=dict(original,oracle_text=original['oracle_text'].replace('Enchanted creatures','Enchanted '+reminder+' creatures'))
                self.assertFalse(metadata.LegacyMetadataTests().definition(card)[1]['implemented'],card['oracle_text'])

if __name__=='__main__':unittest.main()
