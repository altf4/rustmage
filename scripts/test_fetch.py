import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('fetch', Path(__file__).with_name('fetch_standard.py'))
fetch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fetch)


class FetchValidationTests(unittest.TestCase):
    def test_normal_text_is_required_but_explicit_empty_text_is_valid(self):
        row = {'name': 'Test Creature', 'layout': 'normal'}
        with self.assertRaisesRegex(ValueError, 'missing explicit rules text'):
            fetch.validate_rules_text(row)
        row['oracle_text'] = None
        with self.assertRaises(ValueError):
            fetch.validate_rules_text(row)
        row['oracle_text'] = ''
        fetch.validate_rules_text(row)

    def test_multiface_cards_require_every_face_text(self):
        row = {'layout': 'transform', 'card_faces': [{'name': 'Front', 'oracle_text': 'Flying'}, {'name': 'Back'}]}
        with self.assertRaisesRegex(ValueError, 'card face'):
            fetch.validate_rules_text(row)
        row['card_faces'][1]['oracle_text'] = ''
        fetch.validate_rules_text(row)

    def test_entire_frozen_snapshot_passes_text_validation(self):
        snapshot = Path(__file__).resolve().parents[1] / 'data/standard/cards.json'
        for row in json.loads(snapshot.read_text())['cards']:
            fetch.validate_rules_text(row)


if __name__ == '__main__':
    unittest.main()
