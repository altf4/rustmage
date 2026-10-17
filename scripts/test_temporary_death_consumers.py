"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

NAMES = {"Galuf's Final Act", 'Rabid Attack', 'Perigee Beckoner', 'Presumed Dead', 'Person of Interest', "It Doesn't Add Up", 'Absolving Lammasu', 'Reasonable Doubt', 'Convenient Target'}

class TemporaryDeathConsumerTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}

    def test_full_inventory_source_pin_and_actual_card_facts(self):
        m = json.loads((ROOT / 'docs/temporary-death-consumers.json').read_text())
        self.assertEqual(len(m['cards']), 24)
        self.assertEqual(m['java_revision'], '492369c7844b8d76479f0393ff9eb6b0b0cf30d7')
        seen = set()
        for row in m['cards']:
            for key in ['name', 'oracle_id', 'mana_cost', 'type_line', 'oracle_text']:
                self.assertEqual(row[key], self.cards[row['name']][key])
            for source in row['sources']:
                path = source['java_source']
                if path in seen:
                    continue
                b = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', m['java_revision'] + ':' + path])
                self.assertEqual(hashlib.sha256(b).hexdigest(), source['java_sha256'])
                seen.add(path)
        self.assertEqual(len(seen), 140)
