"""Full Role source inventory retained independently of the removed parser."""
import hashlib,json,os,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class RoleProvenanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards={c["name"]:c for c in json.loads((ROOT/"data/standard/cards.json").read_text())["cards"]}
    def test_full_role_inventory_matches_snapshot_and_pinned_source_blobs(self):
        import hashlib, os, subprocess
        from pathlib import Path
        manifest = json.loads((ROOT / 'docs/role-admissions.json').read_text())
        expected = {n for n, c in self.cards.items() if 'Role token' in c.get('oracle_text', '') + ' '.join((f.get('oracle_text', '') for f in c.get('card_faces', [])))}
        self.assertEqual({r['name'] for r in manifest['cards']}, expected)
        source = Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE', '/home/altf4/Code/mage'))
        records = manifest['shared_sources'] + [v for c in manifest['cards'] for v in c['sources']]
        for record in {v['java_source']: v for v in records}.values():
            blob = subprocess.check_output(['git', '-C', str(source), 'show', manifest['java_revision'] + ':' + record['java_source']])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), record['java_sha256'])
            self.assertEqual((source / record['java_source']).read_bytes(), blob)
