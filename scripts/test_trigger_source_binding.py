"""Whole Elf-lord recovery body and exact public-source binding provenance."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata


class TriggerSourceBindingCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.card = next(c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name']=="Morcant's Loyalist")

    def admitted(self, card):
        rust, coverage = metadata.LegacyMetadataTests.compile([card])
        row = next(c for c in coverage['cards'] if c['name']==card['name'])
        return rust, row['implemented']

    def test_complete_body_and_reserved_renamed_semantics(self):
        original, admitted = self.admitted(self.card)
        self.assertTrue(admitted)
        self.assertIn('TriggerCondition::SelfDies', original)
        self.assertIn('other: true', original)
        self.assertIn('TargetPredicate::Subtype("Elf")', original)
        self.assertIn('Instruction::ReturnToHand', original)
        def body(rust, name):
            rows=[line for line in rust.splitlines() if line.lstrip().startswith('Definition {')]
            index=next(i for i,line in enumerate(rows) if 'name: '+json.dumps(name)+',' in line)
            modifiers=rust.split('fn static_modifiers(',1)[1].split('\n}',1)[0]
            arm=next(line.strip().split(' => ',1)[1] for line in modifiers.splitlines() if line.strip().startswith(str(index)+' => '))
            return rows[index],arm
        original_body, original_modifier=body(original,self.card['name'])
        for name, *_ in g.LEGACY:
            changed = dict(self.card, name=name)
            rust, admitted = self.admitted(changed)
            self.assertTrue(admitted, name)
            changed_body, changed_modifier=body(rust,name)
            # Reusing a fixture name removes its fallback row and shifts IDs;
            # compare the entire supplied definition and complete lord arm.
            self.assertEqual(original_body.replace(json.dumps(self.card['name']),json.dumps(name)),changed_body,name)
            self.assertEqual(original_modifier,changed_modifier,name)

    def test_raw_residuals_and_unbounded_or_unknown_subtypes_stay_closed(self):
        text = self.card['oracle_text']
        for line in range(2):
            for extra in [' Discover 4.', ' (Draw seven cards.)', ' ((Draw seven cards.))']:
                lines=text.splitlines();lines[line]+=extra
                self.assertFalse(self.admitted(dict(self.card,oracle_text='\n'.join(lines)))[1], (line, extra))
        for old,new in [('another target','an(other) target'),('another target','another (Draw seven cards.) target'),('Elf card','UnknownSubtype card'),('to your hand.','to your hand. You win the game.'),('Other Elves you control get +1/+1.','Other Elves you control get +1/+1. (Discover 8.)')]:
            changed=text.replace(old,new)
            self.assertNotEqual(text, changed)
            self.assertFalse(self.admitted(dict(self.card,oracle_text=changed))[1], (old,new))

    def test_all_direct_sources_equal_checkout_and_pinned_git_blobs(self):
        manifest=json.loads((g.ROOT/'docs/trigger-source-binding-admissions.json').read_text())
        self.assertEqual(len(manifest['cards']),1)
        row=manifest['cards'][0]
        for key in ['name','oracle_id','mana_cost','type_line','oracle_text','power','toughness']:
            self.assertEqual(row[key],self.card[key])
        root=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'))
        for record in row['sources']:
            blob=subprocess.check_output(['git','-C',str(root),'show',manifest['java_revision']+':'+record['java_source']])
            self.assertEqual(hashlib.sha256(blob).hexdigest(),record['java_sha256'])
            self.assertEqual((root/record['java_source']).read_bytes(),blob)


if __name__=='__main__': unittest.main()
