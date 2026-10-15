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
                        dict(oracle_text='Flying, protection from red')]:
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

    def test_removal_template_requires_exact_text_types_and_cost(self):
        row = self.card(type_line='Instant', mana_cost='{U}',
                        oracle_text="Return target creature to its owner's hand.")
        self.assertEqual(generate.spell_template(row), 'ReturnToHand')
        self.assertEqual(generate.spell_template(dict(row, oracle_text='Destroy target creature.')), 'Destroy')
        for changes in [dict(oracle_text=None), dict(oracle_text=''),
                        dict(oracle_text=row['oracle_text'] + '\nDraw a card.'),
                        dict(type_line='Creature — Wizard'), dict(layout='adventure'),
                        dict(mana_cost='{X}{U}'), dict(type_line='Kindred Instant — Elf')]:
            self.assertIsNone(generate.spell_template(dict(row, **changes)), changes)

    def test_program_statements_preserve_order_and_reject_residual_text(self):
        self.assertEqual(generate.statement_program('Draw two cards. You lose 2 life.'),
                         ['Draw(PlayerSet::Controller, 2)', 'LoseLife(PlayerSet::Controller, 2)'])
        self.assertEqual(generate.statement_program('you gain 1 life and draw a card.'),
                         ['GainLife(PlayerSet::Controller, 1)', 'Draw(PlayerSet::Controller, 1)'])
        self.assertEqual(generate.statement_program('you may draw a card. If you do, discard a card.'),
                         ['May { skip: 2 }', 'Draw(PlayerSet::Controller, 1)', 'Discard(PlayerSet::Controller, 1)'])
        for text in [None, '', 'Draw two cards. Unknown instruction.', 'You gain X life.',
                     'Draw a card if you control a Hero.', 'Draw a card. You gain 2 life',
                     'You gain 999999 life.', 'you may discard a card. If you do, draw a card.']:
            self.assertIsNone(generate.statement_program(text), text)

    def test_trigger_creatures_require_complete_supported_abilities(self):
        card = self.card(oracle_text='Flying\nWhen this creature enters, draw a card.')
        self.assertEqual(generate.trigger_creature_template(card),
                         ('Creature(2, 1)', 1, [('SelfEnters', ['Draw(PlayerSet::Controller, 1)'], 'None')]))
        card['oracle_text'] += '\nWhen this creature dies, you gain 2 life.'
        self.assertEqual(len(generate.trigger_creature_template(card)[2]), 2)
        for change in [dict(oracle_text=None), dict(oracle_text=card['oracle_text'] + '\n{T}: Draw a card.'),
                       dict(oracle_text='When this creature enters, draw a card if you control an Elf.'),
                       dict(oracle_text='When this creature enters, draw a card.\nWard {2}'),
                       dict(mana_cost='{G/U}{G/U}'), dict(layout='transform'),
                       dict(type_line='Legendary Creature — Cat'), dict(toughness='*')]:
            self.assertIsNone(generate.trigger_creature_template(dict(card, **change)), change)

    def test_program_spells_and_trigger_filters_remain_exact(self):
        card = self.card(type_line='Instant', oracle_text='Draw two cards.')
        self.assertEqual(generate.spell_program_template(card), ('None', ['Draw(PlayerSet::Controller, 2)']))
        self.assertIsNone(generate.spell_program_template(dict(card, type_line='Creature — Wizard')))
        self.assertIsNone(generate.spell_program_template(dict(card, oracle_text='Draw two cards. Scry 1.')))
        for prefix, condition in [('Whenever another creature you control enters, ', 'OtherCreatureEntersYou'),
                                  ('Whenever this creature or another creature you control enters, ', 'CreatureEntersYou'),
                                  ('Landfall — Whenever a land you control enters, ', 'LandEntersYou'),
                                  ('Whenever another artifact you control enters, ', 'OtherArtifactEntersYou')]:
            row = self.card(oracle_text=prefix + 'you gain 1 life.')
            self.assertEqual(generate.trigger_creature_template(row)[2][0][0], condition)

    def test_targeted_program_specs_and_optional_effects_are_distinct(self):
        self.assertEqual(generate.targeted_program('you may destroy target artifact or enchantment.'),
                         ('ArtifactEnchantment', ['May { skip: 1 }', 'Destroy']))
        self.assertEqual(generate.targeted_program('tap up to two target creatures an opponent controls.'),
                         (('CreatureOpponent', 0, 2), ['Tap']))
        self.assertEqual(generate.targeted_program('Untap one or two target creatures. They each get +2/+2 until end of turn.'),
                         (('Creature', 1, 2), ['Untap', 'Grow(2)']))
        self.assertEqual(generate.targeted_program('Target creature gets -2/-2 until end of turn. You gain 2 life.'),
                         ('Creature', ['Grow(-2)', 'GainLife(PlayerSet::Controller, 2)']))
        for text in ['Target creature gets +1/+3 until end of turn.', 'Tap target creature. Scry 1.',
                     'Return target creature card from your graveyard to the battlefield with a counter.',
                     'Target player draws a card. Target opponent loses 1 life.',
                     'Tap target creature. It does not untap next turn.']:
            self.assertIsNone(generate.targeted_program(text), text)

    def test_targeted_graveyard_and_player_effects_keep_their_filters(self):
        self.assertEqual(generate.targeted_program('Return target creature card from your graveyard to the battlefield.'),
                         ('CreatureGraveYou', ['ReturnToBattlefield']))
        self.assertEqual(generate.targeted_program('target player draws a card and loses 1 life.'),
                         ('Player', ['Draw(PlayerSet::Targeted, 1)', 'LoseLife(PlayerSet::Targeted, 1)']))
        self.assertEqual(generate.targeted_program('target opponent discards a card.'),
                         ('Opponent', ['Discard(PlayerSet::Targeted, 1)']))
        self.assertIn('min: 0, max: 2', generate.rust_targets(('CreatureOpponent', 0, 2)))

    def test_spell_subtypes_do_not_enable_extra_card_types_or_partial_text(self):
        row = self.card(name='Test', type_line='Instant — Lesson',
                        oracle_text='Untap one or two target creatures. They each get +2/+2 until end of turn.')
        self.assertIsNotNone(generate.spell_program_template(row))
        self.assertIsNone(generate.spell_program_template(dict(row, type_line='Kindred Instant — Elf')))
        self.assertIsNone(generate.spell_program_template(dict(row, oracle_text=row['oracle_text'] + ' Draw a card if you control an Elf.')))


if __name__ == '__main__':
    unittest.main()
