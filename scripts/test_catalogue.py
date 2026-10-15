import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('generate', Path(__file__).with_name('generate_catalogue.py'))
generate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generate)


class CatalogueCompilerTests(unittest.TestCase):
    def card(self, **overrides):
        row = {'layout': 'normal', 'type_line': 'Creature — Cat', 'power': '2', 'toughness': '1',
               'mana_cost': '{W}', 'oracle_text': ''}
        row.update(overrides)
        return row

    def test_exact_creature_template_rejects_other_mechanics(self):
        self.assertEqual(generate.creature_template(self.card()), ('Creature(2, 1)', 0))
        for changes in [dict(oracle_text='Flying\nWhen this creature enters, draw a card.'),
                        dict(type_line='Legendary Creature — Cat'), dict(power='*'),
                        dict(mana_cost='{X}{W}'), dict(mana_cost='{W/P}'), dict(layout='transform'),
                        dict(oracle_text='Flying, first strike')]:
            self.assertIsNone(generate.creature_template(self.card(**changes)), changes)

    def test_keyword_template_requires_every_piece_of_text_to_be_implemented(self):
        self.assertEqual(generate.creature_template(self.card(oracle_text='Flying, vigilance')), ('Creature(2, 1)', 9))
        self.assertEqual(generate.creature_template(self.card(oracle_text='Reach (This creature can block creatures with flying.)')), ('Creature(2, 1)', 2))
        self.assertIsNone(generate.creature_template(self.card(oracle_text='Reach (reminder)\nOther Cats you control get +1/+1.')))

    def test_missing_or_null_rules_text_never_means_vanilla(self):
        row = self.card(oracle_text='When this creature enters, create a token.')
        self.assertIsNone(generate.creature_template(row))
        del row['oracle_text']
        self.assertIsNone(generate.creature_template(row))
        row['oracle_text'] = None
        self.assertIsNone(generate.creature_template(row))
        row['oracle_text'] = ''
        self.assertEqual(generate.creature_template(row), ('Creature(2, 1)', 0))

    def test_cost_parser_does_not_approximate_unimplemented_symbols(self):
        self.assertEqual(generate.fixed_cost('{3}{R}{R}'), ([0, 0, 0, 2, 0], 3))
        for value in ('{C}', '{S}', '{X}', '{2/R}', '{R/P}', '', '{W}{unrecognized}'):
            self.assertIsNone(generate.fixed_cost(value))


if __name__ == '__main__':
    unittest.main()
