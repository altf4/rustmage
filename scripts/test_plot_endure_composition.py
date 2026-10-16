"""Keep unaudited Plot/Endure whole-body compositions closed in both selectors."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('plot_endure_union', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)
class PlotEndureCompositionTests(unittest.TestCase):
    def test_recognized_plot_and_endure_compositions_do_not_revive_partial_bodies(self):
        rows = {c['name']: c for c in json.loads((g.ROOT / 'data/standard/cards.json').read_text())['cards']}
        plot_names = ['Beastbond Outcaster', 'Blacksnag Buzzard', 'Cunning Coyote', "Djinn of Fool's Fall", 'Irascible Wolverine', 'Loan Shark', 'Slickshot Show-Off', 'Spinewoods Paladin', 'Stagecoach Security', 'Stingerback Terror', 'Demonic Ruckus', 'Rictus Robber', 'Rise of the Varmints']
        endure = json.loads((g.ROOT / 'docs/endure-admissions.json').read_text())['cards']
        declaration = next(line for line in rows["Djinn of Fool's Fall"]['oracle_text'].splitlines() if line.startswith('Plot '))
        cases = [dict(rows[n], oracle_text=rows[n]['oracle_text'] + '\nWhen this creature enters, it endures 1.') for n in plot_names]
        for c in endure:
            mutant = dict(rows[c['name']], oracle_text=declaration + '\n' + rows[c['name']]['oracle_text'])
            self.assertIsNotNone(g.extract_plot(mutant)[1])
            cases.append(mutant)
        self.assertEqual(len(cases), 21)
        for card in cases:
            with self.subTest(name=card['name']):
                one = dict(cards=[card], card_count=1)
                one['cards_sha256'] = hashlib.sha256(json.dumps(one['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                _, report = g.compile_catalogue(one, dict(banned_card_names=[]))
                self.assertFalse(next(c for c in report['cards'] if c['name'] == card['name'])['implemented'])
