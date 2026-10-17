"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]

class HandLandTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}
        cls.audit = json.loads((ROOT / 'docs/hand-land-admissions.json').read_text())

    def test_complete_twelve_body_metadata_and_all_pinned_java_files(self):
        self.assertEqual(len(self.audit['cards']), 12)
        self.assertEqual(self.audit['source_records'], 187)
        self.assertEqual(self.audit['unique_source_count'], 86)
        root = xmage_source()
        for row in self.audit['cards']:
            self.assertEqual(row['facts'], self.cards[row['name']])
        for row in self.audit['unique_sources']:
            b = (root / row['java_source']).read_bytes()
            self.assertEqual(hashlib.sha256(b).hexdigest(), row['java_sha256'])
            self.assertEqual(b, subprocess.check_output(['git', '-C', str(root), 'show', self.audit['java_revision'] + ':' + row['java_source']]))
