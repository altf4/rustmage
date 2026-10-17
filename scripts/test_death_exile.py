"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

class DeathExileTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}
        cls.audit = json.loads((ROOT / 'docs/death-exile-admissions.json').read_text())

    def test_entire_pinned_java_source_and_full_metadata_audit(self):
        root = Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE', '/home/altf4/Code/mage'))
        if not root.exists():
            self.skipTest('set RUSTMAGE_XMAGE_SOURCE for complete pinned source audit')
        self.assertEqual(self.audit['source_records'], 242)
        self.assertEqual(len(self.audit['unique_sources']), 88)
        for row in self.audit['cards']:
            self.assertEqual(row['facts'], self.cards[row['name']])
        for row in self.audit['unique_sources']:
            blob = (root / row['java_source']).read_bytes()
            self.assertEqual(hashlib.sha256(blob).hexdigest(), row['java_sha256'])
            self.assertEqual(blob, subprocess.check_output(['git', '-C', str(root), 'show', self.audit['java_revision'] + ':' + row['java_source']]))
