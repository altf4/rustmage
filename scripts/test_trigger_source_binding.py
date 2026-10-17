"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

class TriggerSourceBindingProvenanceTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.card = next((c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards'] if c['name'] == "Morcant's Loyalist"))

    def test_all_direct_sources_equal_checkout_and_pinned_git_blobs(self):
        manifest = json.loads((ROOT / 'docs/trigger-source-binding-admissions.json').read_text())
        self.assertEqual(len(manifest['cards']), 1)
        row = manifest['cards'][0]
        for key in ['name', 'oracle_id', 'mana_cost', 'type_line', 'oracle_text', 'power', 'toughness']:
            self.assertEqual(row[key], self.card[key])
        root = Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE', '/home/altf4/Code/mage'))
        for record in row['sources']:
            blob = subprocess.check_output(['git', '-C', str(root), 'show', manifest['java_revision'] + ':' + record['java_source']])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), record['java_sha256'])
            self.assertEqual((root / record['java_source']).read_bytes(), blob)
