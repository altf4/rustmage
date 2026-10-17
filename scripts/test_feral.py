"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

NAME = 'Feral Deathgorger // Dusk Sight'

class FeralProvenanceTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}

    def test_all_feral_provenance_records_match_checked_out_and_pinned_java(self):
        import os, subprocess
        from pathlib import Path
        source_root = os.environ.get('RUSTMAGE_XMAGE_SOURCE')
        if not source_root:
            self.skipTest('Set RUSTMAGE_XMAGE_SOURCE to audit the pinned original Java bytes')
        source_root = Path(source_root)
        m = json.loads((ROOT / 'docs/omen-admissions.json').read_text())
        entry = next((c for c in m['cards'] if c['name'] == NAME))
        self.assertEqual(entry['status'], 'complete_pair_admitted')
        self.assertIsNone(entry['remaining_obligation'])
        self.assertTrue(any((s['java_source'].endswith('/TargetCardInASingleGraveyard.java') for s in entry['sources'])))
        manifest = json.loads((ROOT / 'docs/single-graveyard-admissions.json').read_text())
        sources = {s['java_source']: s for card in manifest['cards'] for s in card['sources']}
        for source in sources.values():
            p = source['java_source']
            expected = source['java_sha256']
            self.assertEqual(hashlib.sha256((source_root / p).read_bytes()).hexdigest(), expected, p)
            blob = subprocess.check_output(['git', '-C', str(source_root), 'show', m['java_revision'] + ':' + p])
            self.assertEqual(hashlib.sha256(blob).hexdigest(), expected, p)
