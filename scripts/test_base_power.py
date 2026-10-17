"""Complete pinned source inventory for effective base-power consumers."""
import hashlib
import json
from pathlib import Path
import subprocess
import unittest
from scripts import generate_catalogue as g


class BasePowerAuditTests(unittest.TestCase):
    def test_two_complete_physical_bodies_and_direct_source_pins(self):
        audit = json.loads((g.ROOT / 'docs/base-power-admissions.json').read_text())
        facts = {c['name']: c for c in json.loads((g.ROOT / 'data/standard/cards.json').read_text())['cards']}
        expected = {'Kutzil, Malamet Exemplar', 'Sovereign Okinec Ahau'}
        self.assertEqual(set(audit['proposed_complete']), expected)
        self.assertEqual({r['name'] for r in audit['cards']}, expected)
        seen = {}
        records = 0
        for row in audit['cards']:
            self.assertEqual(row['facts'], facts[row['name']])
            for item in [row, *row['direct_import_sources']]:
                records += 1
                path, digest = item['java_source'], item['java_sha256']
                self.assertEqual(seen.setdefault(path, digest), digest)
        self.assertEqual(records, 26)
        self.assertEqual(audit['source_records'], records)
        self.assertEqual(len(seen), 23)
        self.assertEqual(audit['unique_source_count'], len(seen))
        self.assertEqual({r['java_source']: r['java_sha256'] for r in audit['unique_sources']}, seen)
        for path, digest in seen.items():
            blob = subprocess.check_output(['git', '-C', '/home/altf4/Code/mage', 'show', audit['java_revision'] + ':' + path])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), digest)
            self.assertEqual((Path('/home/altf4/Code/mage') / path).read_bytes(), blob)


if __name__ == '__main__':
    unittest.main()
