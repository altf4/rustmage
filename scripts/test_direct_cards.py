"""Metadata linkage and complete-source pins; no English behavior compiler."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from scripts import link_catalogue as linker


class DirectCardTests(unittest.TestCase):
    def test_all_typed_definitions_have_stable_ids_and_set_paths(self):
        metadata, records, version = linker.load()
        self.assertEqual(len(metadata['cards']), len(records))
        self.assertEqual([c['index'] for c in metadata['cards']], list(range(len(records))))
        self.assertEqual(len(version), 64)
        for card in metadata['cards']:
            record = records[card['index']]
            self.assertEqual(Path(record['relative']).parts[2], card['set'])
            self.assertEqual(record['catalogue_id'], card['index'])
            self.assertIn('typed_lift', record)
        outputs = linker.render(metadata, records, version)
        for path, value in outputs.items():
            self.assertEqual(path.read_text(), value, str(path))

    def test_pinned_facts_duplicates_paths_and_completeness_are_checked(self):
        metadata, records, _ = linker.load()
        card = next(c for c in metadata['cards'] if c['facts_sha256'])
        record = records[card['index']]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'data/standard').mkdir(parents=True)
            (root/'data/catalogue-identities.json').write_text(json.dumps(dict(metadata, cards=[card])))
            (root/'data/standard/cards.json').write_bytes((linker.ROOT/'data/standard/cards.json').read_bytes())
            path = root/'src/card_definitions'/card['path']
            path.parent.mkdir(parents=True)
            path.write_text('pub const RULES: CardRules = CardRules::EMPTY;')
            payload = {k:v for k,v in record.items() if k not in ['source','relative']}
            path.with_suffix('.json').write_text(json.dumps(payload))
            (root/'src/card_definitions/mod.rs').write_text('// module')
            linker.load(root)
            for field, value in [('complete', 'true'), ('facts_sha256', 'wrong'), ('name', 'Wrong identity')]:
                path.with_suffix('.json').write_text(json.dumps(dict(payload, **{field:value})))
                with self.assertRaises(ValueError): linker.load(root)
            path.with_suffix('.json').write_text(json.dumps(payload))
            duplicate = path.with_name('duplicate.json')
            duplicate.write_text(json.dumps(payload))
            with self.assertRaises(ValueError): linker.load(root)
            duplicate.unlink()
            renamed = path.with_name('renamed.json')
            path.with_suffix('.json').rename(renamed)
            with self.assertRaises(ValueError): linker.load(root)

    def test_emitted_physical_metadata_is_pinned_per_card(self):
        import copy
        metadata, records, _ = linker.load()
        card = next(c for c in metadata['cards'] if c['name'] == 'Llanowar Elves')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'data/standard').mkdir(parents=True)
            (root/'data/standard/cards.json').write_bytes((linker.ROOT/'data/standard/cards.json').read_bytes())
            path = root/'src/card_definitions'/card['path']
            path.parent.mkdir(parents=True)
            path.write_text('pub const RULES: CardRules = CardRules::EMPTY;')
            record = {k:v for k,v in records[card['index']].items() if k not in ['source','relative']}
            path.with_suffix('.json').write_text(json.dumps(record))
            (root/'src/card_definitions/mod.rs').write_text('// module')
            def check(value):
                (root/'data/catalogue-identities.json').write_text(json.dumps(value))
                return linker.load(root)
            minimal = dict(metadata, cards=[card])
            check(minimal)
            for field,value in [('mana_cost','{9}'),('generic',9),('colored',[0]*6),
                                ('type_line','Artifact'),('kind',16),('mana_color',None),
                                ('legendary',True),('standard_legal',False)]:
                changed=copy.deepcopy(minimal)
                changed['cards'][0][field]=value
                with self.assertRaisesRegex(ValueError,'pinned physical metadata changed'):
                    check(changed)
            changed=copy.deepcopy(minimal)
            changed['colors'][card['index']] ^= 1
            with self.assertRaisesRegex(ValueError,'pinned physical metadata changed'):
                check(changed)

    def test_authored_java_provenance(self):
        _, records, _ = linker.load()
        authored = [r for r in records.values() if 'java_source' in r]
        self.assertGreaterEqual(len(authored), 8)
        pins = [(r['java_revision'], r['java_source'], r['java_sha256']) for r in authored]
        # Reusable effects/targets are provenance too, including records whose
        # own implementation was mechanically lifted rather than newly authored.
        pins.extend((pin['revision'], pin['path'], pin['sha256'])
                    for record in records.values()
                    for pin in record.get('xmage_dependencies', []))
        for revision, path, digest in sorted(set(pins)):
            raw = subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',revision+':'+path])
            self.assertEqual(hashlib.sha256(raw).hexdigest(), digest, path)

if __name__ == '__main__': unittest.main()
