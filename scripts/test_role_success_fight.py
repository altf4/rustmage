"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

NAME = 'Curse of the Werefox'

class RoleSuccessFightTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']
        cls.card = next((c for c in cls.cards if c['name'] == NAME))

    def test_full_metadata_and_unique_pinned_source_provenance(self):
        m = json.loads((ROOT / 'docs/role-success-fight-provenance.json').read_text())
        self.assertEqual(m['card'], self.card)
        root = Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE', '/home/altf4/Code/mage'))
        if not root.exists():
            self.skipTest('set RUSTMAGE_XMAGE_SOURCE to verify pinned Java provenance')
        self.assertEqual(len({s['java_source'] for s in m['sources']}), len(m['sources']))
        for item in m['sources']:
            b = (root / item['java_source']).read_bytes()
            self.assertEqual(hashlib.sha256(b).hexdigest(), item['java_sha256'])
            self.assertEqual(b, subprocess.check_output(['git', '-C', str(root), 'show', m['java_revision'] + ':' + item['java_source']]))
