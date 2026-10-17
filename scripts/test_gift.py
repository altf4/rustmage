"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

NAMES = {'Blooming Blast', 'Consumed by Greed', 'Crumb and Get It', 'Dewdrop Cure', 'Into the Flood Maw', "Long River's Pull", 'Longstalk Brawl', 'Mind Spiral', 'Nocturnal Hunger', 'Peerless Recycling', "Sazacap's Brew", 'Scrapshooter', 'Wear Down', "Bilbo's Gambit", "Dawn's Truce", 'Valley Rally', 'Wildfire Howl'}

class GiftTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']

    def test_all23_full_java_bodies_and_direct_imports_are_pinned(self):
        manifest = json.loads((ROOT / 'docs/gift-admissions.json').read_text())
        gifts = {c['name']: c for c in self.cards if c.get('oracle_text', '').startswith('Gift ')}
        self.assertEqual(len(gifts), 23)
        self.assertEqual({c['name'] for c in manifest['cards']}, set(gifts))
        root = Path('/home/altf4/Code/mage')
        unique = {}
        for card in manifest['cards']:
            self.assertEqual(card['facts'], {k: v for k, v in gifts[card['name']].items() if k in ['mana_cost', 'type_line', 'oracle_text', 'power', 'toughness', 'colors', 'keywords', 'layout']})
            for source in card['sources']:
                unique[source['java_source']] = source['java_sha256']
        unique.update({s['java_source']: s['java_sha256'] for s in manifest['shared_sources']})
        for path, sha in unique.items():
            raw = (root / path).read_bytes()
            self.assertEqual(hashlib.sha256(raw).hexdigest(), sha)
            self.assertEqual(raw, subprocess.check_output(['git', '-C', str(root), 'show', manifest['java_revision'] + ':' + path]))
