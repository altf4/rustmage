"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]

NAMES = {'Fire Magic', 'Ice Magic', 'Restoration Magic', 'Thunder Magic', "Tifa's Limit Break", "Vincent's Limit Break", 'Boros Charm', 'Epic Fight'}

class TieredTests(unittest.TestCase):

    def test_six_physical_bodies_all_eighteen_modes_and_full_java_are_pinned(self):
        manifest = json.loads((ROOT / 'docs/tiered-admissions.json').read_text())
        cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards'] if 'Tiered' in c.get('oracle_text', '')}
        self.assertEqual(len(cards), 6)
        self.assertEqual({r['facts']['name'] for r in manifest['cards']}, set(cards))
        self.assertEqual(sum((len(r['tiers']) for r in manifest['cards'])), 18)
        records = []
        for row in manifest['cards']:
            card = cards[row['facts']['name']]
            self.assertEqual(row['facts'], {k: card.get(k) for k in row['facts']})
            self.assertEqual(card['layout'], 'normal')
            self.assertFalse(card.get('card_faces'))
            self.assertEqual(len(row['tiers']), 3)
            self.assertEqual(card['oracle_text'].splitlines()[0], 'Tiered (Choose one additional cost.)')
            records.extend(row['sources'])
            for tier in row['tiers']:
                self.assertIn('• ' + tier['label'] + ' — ' + tier['additional_cost'] + ' — ' + tier['body'], card['oracle_text'])
        records.extend(manifest['shared_sources'])
        self.assertEqual(len(records), 140)
        self.assertEqual(manifest['tiered_source_records'], 140)
        self.assertEqual(len({r['java_source'] for r in records}), 87)
        for row in manifest['adjacent_complete_bodies']:
            card = next((c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards'] if c['name'] == row['facts']['name']))
            self.assertEqual(card, row['facts'])
            records.extend(row['sources'])
        self.assertEqual(len(records), manifest['source_records'])
        unique = {r['java_source']: r['java_sha256'] for r in records}
        self.assertEqual(len(unique), len(manifest['unique_sources']))
        self.assertEqual(unique, {r['java_source']: r['java_sha256'] for r in manifest['unique_sources']})
        for path, sha in unique.items():
            raw = subprocess.check_output(['git', '-C', str(xmage_source()), 'show', manifest['java_revision'] + ':' + path])
            self.assertEqual(hashlib.sha256(raw).hexdigest(), sha)
