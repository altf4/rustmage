"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

class UntappedEntryTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}

    def test_pinned_full_java_audit(self):
        audit = json.loads((ROOT / 'docs/untapped-entry-admissions.json').read_text())
        self.assertEqual(audit['source_records'], 88)
        self.assertEqual(audit['unique_sources'], 52)
        self.assertEqual([c['name'] for c in audit['cards'] if c['current_standard']], ['Spelunking', 'The Wandering Minstrel'])
        import hashlib
        for record in [s for c in audit['cards'] for s in c['sources']] + audit['additional_sources']:
            raw = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', audit['java_revision'] + ':' + record['java_source']])
            self.assertEqual(hashlib.sha256(raw).hexdigest(), record['java_sha256'])
