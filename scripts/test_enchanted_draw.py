"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]

NAME = "Lord Skitter's Blessing"

class EnchantedDrawProvenanceTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.card = next((c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards'] if c['name'] == NAME))

    def test_complete_pinned_record_and_every_direct_java_import(self):
        manifest = json.loads((ROOT / 'docs/enchanted-draw-admissions.json').read_text())
        row = manifest['cards'][0]
        for key in ['name', 'oracle_id', 'mana_cost', 'type_line', 'oracle_text']:
            self.assertEqual(row[key], self.card[key])
        root = xmage_source()
        self.assertEqual(len(row['sources']), 20)
        for record in row['sources']:
            blob = subprocess.check_output(['git', '-C', str(root), 'show', manifest['java_revision'] + ':' + record['java_source']])
            self.assertEqual((root / record['java_source']).read_bytes(), blob)
            self.assertEqual(hashlib.sha256(blob).hexdigest(), record['java_sha256'])
