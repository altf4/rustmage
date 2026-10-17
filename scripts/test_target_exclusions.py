"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]

NAMES = {'Amazing Acrobatics', 'Coeurl', 'Decoy Ploy', 'Guerrilla Gorilla', 'Overwhelming Surge', 'Rock Soldiers', 'Sterling Keykeeper'}

class ExclusionTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards'] if c['name'] in NAMES}

    def test_all_full_records_and_explicit_direct_imports_are_pinned(self):
        m = json.loads((ROOT / 'docs/target-exclusion-admissions.json').read_text())
        root = xmage_source()
        self.assertEqual({r['name'] for r in m['cards']}, NAMES)
        for row in m['cards']:
            for k in ['name', 'oracle_id', 'mana_cost', 'type_line', 'oracle_text']:
                self.assertEqual(row[k], self.cards[row['name']][k])
            for rec in row['sources']:
                blob = subprocess.check_output(['git', '-C', str(root), 'show', m['java_revision'] + ':' + rec['java_source']])
                self.assertEqual((root / rec['java_source']).read_bytes(), blob)
                self.assertEqual(hashlib.sha256(blob).hexdigest(), rec['java_sha256'])
