"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

NAMES = {'Mouser Mark III', 'Pacifism', 'Technodrome', 'The Lion-Turtle', 'Tiger-Dillo'}

class CombatRestrictionTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards'] if c['name'] in NAMES}

    def test_pinned_full_records_and_explicit_imports(self):
        m = json.loads((ROOT / 'docs/combat-restriction-admissions.json').read_text())
        self.assertEqual({c['name'] for c in m['cards']}, NAMES)
        for row in m['cards']:
            for k in ['name', 'oracle_id', 'mana_cost', 'type_line', 'oracle_text']:
                self.assertEqual(row[k], self.cards[row['name']][k])
            for rec in row['sources']:
                b = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', m['java_revision'] + ':' + rec['java_source']])
                self.assertEqual(hashlib.sha256(b).hexdigest(), rec['java_sha256'])
