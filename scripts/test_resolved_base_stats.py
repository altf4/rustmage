"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]

INITIAL_NAMES = {'Genemorph Imago', 'Moon Girl and Devil Dinosaur', 'Belligerent Yearling'}
NAMES = INITIAL_NAMES | {'PuPu UFO', 'Atomic Microsizer'}

class ResolvedBaseTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.all = json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']
        cls.cards = {c['name']: c for c in cls.all if c['name'] in NAMES}

    def test_complete_pinned_java_and_direct_imports_are_audit_only(self):
        audit = json.loads((ROOT / 'docs/resolved-base-stat-admissions.json').read_text())
        self.assertEqual({r['name'] for r in audit['cards']}, NAMES)
        java = xmage_source()
        facts = {c['name']: c for c in self.all}
        seen = set()
        records = 0
        for row in audit['cards']:
            self.assertEqual(row['oracle_id'], facts[row['name']]['oracle_id'])
            for k, v in row['facts'].items():
                self.assertEqual(v, facts[row['name']][k])
            for source in row['sources']:
                records += 1
                p = source['java_source']
                blob = (java / p).read_bytes()
                self.assertEqual(hashlib.sha256(blob).hexdigest(), source['java_sha256'])
                self.assertEqual(blob, subprocess.check_output(['git', '-C', str(java), 'show', audit['java_revision'] + ':' + p]))
                seen.add(p)
        self.assertEqual((records, len(seen)), (144, 105))
