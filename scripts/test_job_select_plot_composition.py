"""Keep the parallel Job Select/Plot selectors closed to unaudited composites."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('job_plot_union', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)

class JobPlotCompositionTests(unittest.TestCase):
    def test_recognized_plot_does_not_revive_job_select_composites(self):
        rows = {c['name']: c for c in json.loads((g.ROOT / 'data/standard/cards.json').read_text())['cards']}
        names = ["Bard's Bow", "Dark Knight's Greatsword", "Dragoon's Lance", "Machinist's Arsenal", "Monk's Fist", "Samurai's Katana", "Warrior's Sword"]
        declaration = next(line for line in rows["Djinn of Fool's Fall"]['oracle_text'].splitlines() if line.startswith('Plot '))
        for name in names:
            with self.subTest(name=name):
                base = rows[name]
                self.assertIsNotNone(g.job_select_template(base))
                mutant = dict(base, oracle_text=base['oracle_text'] + '\n' + declaration)
                residual, cost = g.extract_plot(mutant)
                self.assertIsNotNone(cost)
                self.assertEqual(residual['oracle_text'], base['oracle_text'])
                one = dict(cards=[mutant], card_count=1)
                one['cards_sha256'] = hashlib.sha256(json.dumps(one['cards'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                _, report = g.compile_catalogue(one, dict(banned_card_names=[]))
                self.assertFalse(next(c for c in report['cards'] if c['name'] == name)['implemented'])
