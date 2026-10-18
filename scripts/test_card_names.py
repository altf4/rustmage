"""Pinned name metadata consistency; no network or card-behavior translation."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class CardNameMetadataTests(unittest.TestCase):
    def test_interchangeable_face_names_and_source_pins(self):
        packed = (ROOT / 'data/interchangeable-card-names.tsv').read_bytes()
        evidence = json.loads((ROOT / 'data/interchangeable-card-names-source.json').read_text())
        pairs = [tuple(line.split('\t')) for line in packed.decode().splitlines()]
        self.assertEqual(pairs, sorted(set(pairs)))
        self.assertEqual(len(pairs), evidence['face_pairs'])
        self.assertEqual(hashlib.sha256(packed).hexdigest(), evidence['packed_sha256'])
        self.assertEqual(pairs, [(r['original_name'], r['oracle_name']) for r in evidence['pairs']])
        self.assertEqual(len({r['oracle_id'] for r in evidence['pairs']}), 29)
        self.assertEqual(evidence['inventory_count'], 30)
        originals, canonical = map(set, zip(*pairs))
        self.assertFalse(originals & canonical, 'comparison keys must not form chains')
        for row in evidence['pairs']:
            self.assertEqual(row['original_set'], 'sld')
            self.assertIn(row['oracle_id'], row['source_url'])
            self.assertRegex(row['source_sha256'], r'^[0-9a-f]{64}$')
        names_file = (ROOT / 'data/card-names.txt').read_bytes()
        names = names_file.decode().splitlines()
        names_source = json.loads((ROOT / 'data/card-names-source.json').read_text())
        self.assertEqual(names, sorted(set(names)))
        self.assertEqual(len(names), names_source['name_count'])
        self.assertEqual(hashlib.sha256(names_file).hexdigest(), names_source['packed_sha256'])
        self.assertTrue((originals | canonical) <= set(names))
        self.assertNotIn('Godzilla, King of the Monsters', names)
        self.assertNotIn('Hawkins National Laboratory // The Upside Down', names)
        self.assertNotIn("Doric, Nature's Warden // Doric, Owlbear Avenger", names)
