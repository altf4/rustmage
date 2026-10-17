"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

class BasePowerAuditTests(unittest.TestCase):

    def test_three_complete_physical_bodies_and_direct_source_pins(self):
        audit = json.loads((ROOT / 'docs/base-power-admissions.json').read_text())
        facts = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}
        expected = {'Kutzil, Malamet Exemplar', 'Sovereign Okinec Ahau', 'Voice of Victory'}
        self.assertEqual(set(audit['proposed_complete']), expected)
        self.assertEqual({r['name'] for r in audit['cards']}, expected)
        seen = {}
        records = 0
        for row in audit['cards']:
            self.assertEqual(row['facts'], facts[row['name']])
            for item in [row, *row['direct_import_sources']]:
                records += 1
                path, digest = (item['java_source'], item['java_sha256'])
                self.assertEqual(seen.setdefault(path, digest), digest)
        self.assertEqual(records, 35)
        self.assertEqual(audit['source_records'], records)
        self.assertEqual(len(seen), 25)
        self.assertEqual(audit['unique_source_count'], len(seen))
        self.assertEqual({r['java_source']: r['java_sha256'] for r in audit['unique_sources']}, seen)
        for path, digest in seen.items():
            blob = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', audit['java_revision'] + ':' + path])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), digest)
            self.assertEqual((Path('/home/altf4/Code/mage') / path).read_bytes(), blob)
