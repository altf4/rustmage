"""Do not partially admit unsupported combined fixed Endure/Mobilize bodies."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('endure_mobilize_union', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)
class EndureMobilizeCompositionTests(unittest.TestCase):
    def test_both_whole_body_selectors_reject_combined_trigger_recipes(self):
        rows = {c['name']: c for c in json.loads((g.ROOT / 'data/standard/cards.json').read_text())['cards']}
        mobilize = json.loads((g.ROOT / 'docs/mobilize-admissions.json').read_text())['cards']
        endure = json.loads((g.ROOT / 'docs/endure-admissions.json').read_text())['cards']
        cases = [dict(rows[c['name']], oracle_text=rows[c['name']]['oracle_text'] + '\nWhen this creature enters, it endures 1.') for c in mobilize]
        cases += [dict(rows[c['name']], oracle_text=rows[c['name']]['oracle_text'] + '\nMobilize 1') for c in endure]
        self.assertEqual(len(cases), 16)
        for card in cases:
            with self.subTest(name=card['name']):
                one = dict(cards=[card], card_count=1)
                one['cards_sha256'] = hashlib.sha256(json.dumps(one['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                _, report = g.compile_catalogue(one, dict(banned_card_names=[]))
                self.assertFalse(next(c for c in report['cards'] if c['name'] == card['name'])['implemented'])
