"""Complete semantic Role bodies, generic predicates and raw text authentication."""
import copy
import json
import unittest
import hashlib
import os
import subprocess
from pathlib import Path
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata

ENCHANTED = ['A Tale for the Ages', 'Syr Armont, the Redeemer']

class RoleSemanticCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}

    def test_exact_six_body_union_preserves_current_parent_and_pinned_java_sources(self):
        from scripts.catalogue_preservation import canonical, assert_union
        names=['Return Triumphant','Twisted Sewer-Witch','Syr Armont, the Redeemer','Asinine Antics','A Tale for the Ages','Mystical Tether']
        physical={c['oracle_id']:c['name'] for c in self.cards.values()}
        before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show','804f539567d2f50b27c801915a3eb5afd7f8d692:src/generated/catalogue.rs'],text=True),physical)
        after=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
        helpers={('enchanted_modifiers',n+'#0') for n in ['Syr Armont, the Redeemer','A Tale for the Ages']}|{('flash_surcharge',n+'#0') for n in ['Asinine Antics','Mystical Tether']}
        assert_union(before,after,changed={n+'#0' for n in names},added=set(),helpers=helpers,aliases={})
        manifest=json.loads((g.ROOT/'docs/role-semantic-admissions.json').read_text())
        self.assertEqual({c['name'] for c in manifest['cards']},set(names))
        source=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'))
        records=manifest['shared_sources']+[v for c in manifest['cards'] for v in c['sources']]
        for card in manifest['cards']:
            self.assertEqual(card['oracle_id'],self.cards[card['name']]['oracle_id'])
            self.assertEqual(card['facts'],{k:self.cards[card['name']][k] for k in card['facts']})
        for record in {v['java_source']:v for v in records}.values():
            blob=subprocess.check_output(['git','-C',str(source),'show',manifest['java_revision']+':'+record['java_source']])
            self.assertEqual(hashlib.sha256(blob).hexdigest(),record['java_sha256'])
            self.assertEqual((source/record['java_source']).read_bytes(),blob)

    def test_complete_mass_role_body_keeps_token_prefix_and_exact_recipient_filter(self):
        original=self.cards['Twisted Sewer-Witch']
        before,row=metadata.LegacyMetadataTests().definition(original)
        self.assertTrue(row['implemented'])
        self.assertIn('Instruction::CreateRolesMatching',before)
        self.assertIn('subtype: Some("Rat")',before)
        self.assertIn('relation: 0',before)
        renamed=dict(original,name='Independent Mass Role Subject')
        after,row=metadata.LegacyMetadataTests().definition(renamed)
        self.assertTrue(row['implemented'])
        self.assertEqual(before.replace(json.dumps(original['name']),json.dumps(renamed['name'])),after)
        for text in [original['oracle_text']+'\nDiscover 4.',
                     original['oracle_text']+' (Draw seven cards.)',
                     original['oracle_text'].replace('attached to that Rat','attached to that Elf'),
                     original['oracle_text'].replace('loses 1 life','loses 2 life'),
                     original['oracle_text'].replace('for each Rat','for each (Draw seven cards.) Rat')]:
            self.assertFalse(metadata.LegacyMetadataTests().definition(dict(original,oracle_text=text))[1]['implemented'],text)

    def test_paid_flash_timing_is_generic_additional_cost_and_requires_whole_raw_body(self):
        for name in ['Asinine Antics','Mystical Tether']:
            original=self.cards[name]
            compiled,row=metadata.LegacyMetadataTests.compile([original])
            self.assertTrue(next(r for r in row['cards'] if r['name']==name)['implemented'])
            self.assertIn('pub(super) fn flash_surcharge',compiled)
            self.assertIn('Some(ManaCost { colored: [0, 0, 0, 0, 0, 0], generic: 2 })',compiled)
            renamed=dict(original,name='Independent Paid Timing Subject')
            before,_=metadata.LegacyMetadataTests().definition(original)
            after,row=metadata.LegacyMetadataTests().definition(renamed)
            self.assertTrue(row['implemented'])
            self.assertEqual(before.replace(json.dumps(name),json.dumps(renamed['name'])),after)
            for text in [original['oracle_text']+'\nDiscover 4.',
                         original['oracle_text']+' (Draw seven cards.)',
                         original['oracle_text'].replace('had flash','had (Draw seven cards.) flash'),
                         original['oracle_text'].replace('pay {2}','pay {4294967296}'),
                         original['oracle_text'].replace('more to cast it.','more to cast it. (Draw seven cards.)')]:
                self.assertFalse(metadata.LegacyMetadataTests().definition(dict(original,oracle_text=text))[1]['implemented'],text)

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
