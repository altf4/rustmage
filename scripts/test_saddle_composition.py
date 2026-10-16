"""Cross-feature whole-body gates in the integrated saddle catalogue."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('saddle_union', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)


class SaddleCompositionTests(unittest.TestCase):
    def test_evidence_bargain_and_scalar_saddle_compositions_stay_gated(self):
        snapshot = json.loads((g.ROOT / 'data/standard/cards.json').read_text())
        by_name = {c['name']: c for c in snapshot['cards']}
        saddle = json.loads((g.ROOT / 'docs/saddle-admissions.json').read_text())['cards']
        evidence = json.loads((g.ROOT / 'docs/evidence-admissions.json').read_text())['cards']
        evidence_declaration = by_name['Analyze the Pollen']['oracle_text'].splitlines()[0] + '\n'
        bargain_declaration = by_name['Candy Grapple']['oracle_text'].splitlines()[0] + '\n'
        altered = []
        for row in saddle:
            card = by_name[row['name']]
            altered.extend([
                dict(card, oracle_text=evidence_declaration + card['oracle_text']),
                dict(card, oracle_text=bargain_declaration + card['oracle_text']),
                dict(card, mana_cost='{X}' + card['mana_cost']),
            ])
        altered.extend(dict(by_name[row['name']], oracle_text=by_name[row['name']]['oracle_text'] + '\nSaddle 1') for row in evidence)
        self.assertEqual(len(altered), 67)
        for card in altered:
            with self.subTest(name=card['name'], cost=card['mana_cost'], text=card['oracle_text']):
                cards = [card]
                one = dict(cards=cards, card_count=1)
                one['cards_sha256'] = hashlib.sha256(json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                _, report = g.compile_catalogue(one, dict(banned_card_names=[]))
                self.assertFalse(next(c for c in report['cards'] if c['name'] == card['name'])['implemented'])
