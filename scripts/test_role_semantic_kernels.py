"""Full Role source inventory retained independently of the removed parser."""
import hashlib,json,os,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]
class RoleProvenanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards={c["name"]:c for c in json.loads((ROOT/"data/standard/cards.json").read_text())["cards"]}
    def test_historical_six_body_union_and_current_programs_preserve_pinned_sources(self):
        names = ['Return Triumphant', 'Twisted Sewer-Witch', 'Syr Armont, the Redeemer', 'Asinine Antics', 'A Tale for the Ages', 'Mystical Tether']
        manifest = json.loads((ROOT / 'docs/role-semantic-admissions.json').read_text())
        self.assertEqual({c['name'] for c in manifest['cards']}, set(names))
        source = xmage_source()
        records = manifest['shared_sources'] + [v for c in manifest['cards'] for v in c['sources']]
        for card in manifest['cards']:
            self.assertEqual(card['oracle_id'], self.cards[card['name']]['oracle_id'])
            self.assertEqual(card['facts'], {k: self.cards[card['name']][k] for k in card['facts']})
        for record in {v['java_source']: v for v in records}.values():
            blob = subprocess.check_output(['git', '-C', str(source), 'show', manifest['java_revision'] + ':' + record['java_source']])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), record['java_sha256'])
            self.assertEqual((source / record['java_source']).read_bytes(), blob)
