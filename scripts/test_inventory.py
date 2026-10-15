import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('inventory', Path(__file__).with_name('inventory_standard.py'))
inventory = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inventory)


class InventoryTests(unittest.TestCase):
    def test_comments_do_not_admit_missing_or_digital_cards(self):
        source = r'''
        // cards.add(new SetCardInfo("Missing", 1, Rarity.RARE, mage.cards.m.Missing.class));
        /* cards.add(new SetCardInfo("Digital", 2, Rarity.RARE, mage.cards.d.Digital.class)); */
        cards.add(new SetCardInfo("Real // Name", "123a", Rarity.COMMON, mage.cards.r.Real.class));
        '''
        self.assertEqual(inventory.set_entries(source), [('Real // Name', '123a', 'COMMON', 'mage.cards.r.Real')])

    def test_comment_markers_and_escapes_inside_strings_are_preserved(self):
        source = r'''cards.add(new SetCardInfo("Bob's \"//\" Card", 12, Rarity.RARE, mage.cards.b.Bob.class));'''
        self.assertEqual(inventory.set_entries(source)[0][0], 'Bob\'s "//" Card')

    def test_unknown_declarations_fail_instead_of_losing_cards(self):
        with self.assertRaisesRegex(ValueError, 'unparsed'):
            inventory.set_entries('new SetCardInfo("Missing", 1, Rarity.RARE, null)')

    def test_authoritative_and_source_spellings_reconcile_without_changing_output_names(self):
        self.assertEqual(inventory.name_key('Araña, Heart of the Spider'), inventory.name_key('Arana, Heart of the Spider'))
        self.assertEqual(inventory.name_key('With Great Power . . .'), inventory.name_key('With Great Power...'))


if __name__ == '__main__':
    unittest.main()
