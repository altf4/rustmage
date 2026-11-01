"""Core Python/FFI contracts; card rules are tested by the native suite."""
import unittest

from rustmage import EngineError, Game, OBSERVATION_SCHEMA


def game():
    return Game([['Forest'] * 7, ['Island'] * 7], seed=1, deck_rules='Freeform')


class BindingContractTests(unittest.TestCase):
    def test_observations_preserve_hidden_information(self):
        with game() as handle:
            own, other = handle.observe('P0'), handle.observe('P1')
            self.assertEqual(own['schema'], OBSERVATION_SCHEMA)
            self.assertEqual(sum(o['owner'] == 'P0' and o['zone'] == 'Hand' for o in own['objects']), 7)
            self.assertFalse(any(o['owner'] == 'P0' for o in other['objects']))

    def test_branch_survives_its_parent_and_restore_rejects_old_indices(self):
        with game() as parent:
            child = parent.clone()
        with child:
            before = child.observe('P0')
            saved = child.snapshot()
            keep = before['actions'].index({'kind': 'Keep'})
            child.step(before, keep)
            child.restore(saved)
            restored = child.observe('P0')
            self.assertEqual(restored['actions'], before['actions'])
            with self.assertRaises(EngineError):
                child.step(before, keep)
            self.assertEqual(child.observe('P0'), restored)
            child.drop_snapshot(saved)

    def test_closed_handle_reports_an_error(self):
        handle = game()
        handle.close()
        with self.assertRaises(EngineError):
            handle.observe('P0')
