"""Complete printed ability-removal family inventory; never an admission whitelist."""
import hashlib
import json
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class AbilityRemovalInventoryTests(unittest.TestCase):
    def test_complete_physical_family_and_full_pinned_source_records(self):
        manifest = json.loads((ROOT / 'docs/ability-removal-admissions.json').read_text())
        cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']
                 if re.search(r'loses? all abilities', c.get('oracle_text', ''))}
        self.assertEqual(len(cards), 24)
        self.assertEqual({r['facts']['name'] for r in manifest['cards']}, set(cards))
        records = []
        for row in manifest['cards']:
            self.assertEqual(row['facts'], cards[row['facts']['name']])
            self.assertIn(row['status'], ('inventoried_priority_not_implemented', 'gated_dependency'))
            self.assertTrue(row['required_semantics'])
            self.assertTrue(any('/cards/' in r['java_source'] for r in row['sources']))
            records.extend(row['sources'])
        self.assertEqual(len(records), manifest['source_records'])
        self.assertEqual(len(records), 410)
        unique = {r['java_source']: r['java_sha256'] for r in records}
        self.assertEqual(len(unique), 114)
        self.assertEqual(unique, {r['java_source']: r['java_sha256'] for r in manifest['unique_sources']})
        for source, sha in unique.items():
            raw = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show',
                                           manifest['java_revision'] + ':' + source])
            self.assertEqual(hashlib.sha256(raw).hexdigest(), sha, source)


if __name__ == '__main__':
    unittest.main()
