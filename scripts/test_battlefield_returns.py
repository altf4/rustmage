"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

NAMES = {'Aetherize', "River's Rebuke"}

class BattlefieldReturnTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards'] if c['name'] in NAMES}

    def test_full_pinned_sources_and_direct_imports(self):
        m = json.loads((ROOT / 'docs/battlefield-return-admissions.json').read_text())
        self.assertEqual({c['name'] for c in m['cards']}, NAMES)
        for row in m['cards']:
            for k in ['name', 'oracle_id', 'mana_cost', 'type_line', 'oracle_text']:
                self.assertEqual(row[k], self.cards[row['name']][k])
            for rec in row['sources']:
                blob = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', m['java_revision'] + ':' + rec['java_source']])
                self.assertEqual(hashlib.sha256(blob).hexdigest(), rec['java_sha256'])
