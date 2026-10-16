"""Cross-feature whole-body gates in the integrated evidence catalogue."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('evidence_union', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)


class EvidenceCompositionTests(unittest.TestCase):
    def test_scalar_and_bargain_compositions_stay_gated_in_both_directions(self):
        snapshot = json.loads((g.ROOT / 'data/standard/cards.json').read_text())
        by_name = {c['name']: c for c in snapshot['cards']}
        evidence_names = ['Analyze the Pollen', 'Axebane Ferox', 'Bite Down on Crime', 'Crimestopper Sprite', 'Evidence Examiner', 'Extract a Confession', 'Forensic Researcher', 'Sample Collector', 'Surveillance Monitor', 'Vitu-Ghazi Inspector']
        scalar = json.loads((g.ROOT / 'docs/scalar-x-admissions.json').read_text())
        bargain = json.loads((g.ROOT / 'docs/bargain-admissions.json').read_text())
        declaration = by_name['Analyze the Pollen']['oracle_text'].splitlines()[0] + '\n'
        bargain_declaration = by_name['Candy Grapple']['oracle_text'].splitlines()[0] + '\n'
        altered = [dict(by_name[c['name']], oracle_text=declaration + by_name[c['name']]['oracle_text']) for c in scalar['cards'] + bargain['cards']]
        altered += [dict(by_name[name], mana_cost='{X}' + by_name[name]['mana_cost']) for name in evidence_names]
        altered += [dict(by_name[name], oracle_text=bargain_declaration + by_name[name]['oracle_text']) for name in evidence_names]
        self.assertEqual(len(altered), 42)
        for card in altered:
            with self.subTest(name=card['name'], cost=card['mana_cost'], text=card['oracle_text']):
                cards = [card]
                one = dict(cards=cards, card_count=1)
                one['cards_sha256'] = hashlib.sha256(json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                _, report = g.compile_catalogue(one, dict(banned_card_names=[]))
                self.assertFalse(next(c for c in report['cards'] if c['name'] == card['name'])['implemented'])
