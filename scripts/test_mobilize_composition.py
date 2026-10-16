"""Whole-body gates for unsupported combined convoke/mobilize recipes."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('mobilize_union', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)
class MobilizeCompositionTests(unittest.TestCase):
    def test_convoke_and_mobilize_remain_closed_in_both_selector_directions(self):
        snapshot = json.loads((g.ROOT / 'data/standard/cards.json').read_text())
        rows = {c['name']: c for c in snapshot['cards']}
        mobilize = json.loads((g.ROOT / 'docs/mobilize-admissions.json').read_text())['cards']
        convoke = ['Appeal to Eirdu', 'Disruptor of Currents', 'Lofty Dreams', 'Protective Response', 'Sun-Dappled Celebrant', 'Unexpected Assistance', 'Vote Out', 'Winter, Team Player', 'Merrow Skyswimmer']
        cases = [dict(rows[c['name']], oracle_text='Convoke\n' + rows[c['name']]['oracle_text']) for c in mobilize]
        cases += [dict(rows[n], oracle_text=rows[n]['oracle_text'] + '\nMobilize 1') for n in convoke]
        self.assertEqual(len(cases), 17)
        for card in cases:
            with self.subTest(name=card['name'], text=card['oracle_text']):
                one = dict(cards=[card], card_count=1)
                one['cards_sha256'] = hashlib.sha256(json.dumps(one['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                _, report = g.compile_catalogue(one, dict(banned_card_names=[]))
                self.assertFalse(next(c for c in report['cards'] if c['name'] == card['name'])['implemented'])
