"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

class LiveBaseStatTests(unittest.TestCase):

    def test_four_complete_metadata_records_and_all_direct_import_sources(self):
        audit = json.loads((ROOT / 'docs/live-base-stat-admissions.json').read_text())
        cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}
        expected = {'Maha, Its Feathers Night', 'Archon of the Wild Rose', 'Aettir and Priwen', 'Burden of Proof'}
        self.assertEqual(set(audit['proposed_complete']), expected)
        self.assertEqual({r['name'] for r in audit['cards']}, expected)
        records = 0
        seen = {}
        for row in audit['cards']:
            self.assertEqual(row['facts'], cards[row['name']])
            for rec in [row, *row['direct_import_sources']]:
                records += 1
                p = rec['java_source']
                h = rec['java_sha256']
                self.assertEqual(seen.setdefault(p, h), h)
        self.assertEqual(records, 60)
        self.assertEqual(audit['source_records'], records)
        self.assertEqual(audit['unique_source_count'], len(seen))
        self.assertEqual({r['java_source']: r['java_sha256'] for r in audit['unique_sources']}, seen)
        for p, h in seen.items():
            blob = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', audit['java_revision'] + ':' + p])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), h)
            self.assertEqual((Path('/home/altf4/Code/mage') / p).read_bytes(), blob)
        self.assertFalse(audit['unresolved_explicit_internal_imports'])
