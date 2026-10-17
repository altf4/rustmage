"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]

NAMES = {'Doc Ock, Sinister Scientist', 'Mirkwood Meditator', 'Multiply by Zero', 'Perfected Theory', 'Quandrix Charm', 'Snowmelt Stag', 'Turtle-Duck', 'Water Wings', 'Gateway Sneak', 'Snooping Page'}

class BaseStatTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.all = json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']
        cls.cards = {c['name']: c for c in cls.all if c['name'] in NAMES}

    def test_full_family_metadata_and_complete_pinned_direct_imports(self):
        manifest = json.loads((ROOT / 'docs/base-stat-admissions.json').read_text())
        self.assertEqual(len(manifest['cards']), 56)
        self.assertEqual(set(manifest['initial_complete_consumers']), NAMES)
        allcards = {c['name']: c for c in self.all}
        seen = {}
        records = 0
        self.assertEqual(manifest['missing_sources'], ["Galion, Elvenking's Butler"])
        for row in manifest['cards'] + manifest['auxiliary_complete_consumers']:
            for key, value in allcards[row['name']].items():
                self.assertEqual(row[key], value, (row['name'], key))
            if 'java_source' not in row:
                self.assertEqual(row['name'], "Galion, Elvenking's Butler")
                continue
            for rec in [row, *row['direct_import_sources']]:
                records += 1
                path = rec['java_source']
                sha = rec['java_sha256']
                if path in seen:
                    self.assertEqual(seen[path], sha)
                    continue
                blob = subprocess.check_output(['git', '-C', str(xmage_source()), 'show', manifest['java_revision'] + ':' + path])
                self.assertEqual(hashlib.sha256(blob).hexdigest(), sha)
                self.assertEqual((xmage_source() / path).read_bytes(), blob)
                seen[path] = sha
        self.assertEqual(len(seen), 242)
        self.assertEqual(records, 57 + 824)
        self.assertFalse(manifest['unresolved_explicit_internal_imports'])
