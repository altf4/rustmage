import concurrent.futures
import json
import unittest
from rustmage import EngineError, Game


class BindingTests(unittest.TestCase):
    def game(self):
        return Game([["Mountain"] * 24 + ["LightningBolt"] * 36] * 2, seed=17, deck_rules="Freeform")

    def test_private_views_clone_restore_and_stale_indices(self):
        with self.game() as game:
            observation = game.observe("P0")
            self.assertEqual(len(observation["objects"]), 7)
            self.assertTrue(all(o["owner"] == "P0" for o in observation["objects"]))
            self.assertEqual(game.observe("P1")["actions"], [])
            saved = game.snapshot()
            with game.clone() as branch:
                keep = next(i for i, a in enumerate(observation["actions"]) if a["kind"] == "Keep")
                branch.step(observation, keep)
                self.assertEqual(game.observe("P0"), observation)
                self.assertEqual(branch.observe("P1")["actor"], "P1")
            game.restore(saved)
            restored = game.observe("P0")
            self.assertGreater(restored.pop("revision"), observation["revision"])
            before = dict(observation)
            before.pop("revision")
            self.assertEqual(restored, before)
            with self.assertRaisesRegex(EngineError, "StaleRevision"):
                game.step(observation, 1)
            game.drop_snapshot(saved)
            with self.assertRaisesRegex(EngineError, "unknown snapshot"):
                game.restore(saved)

    def test_input_and_lifecycle(self):
        with self.assertRaises(EngineError):
            Game([["BlackLotus"] * 60] * 2)
        with self.assertRaises(EngineError):
            Game([["LightningBolt"] * 60] * 2)
        game = self.game()
        with self.assertRaises(EngineError):
            game.request("unknown")
        game.close()
        game.close()
        with self.assertRaisesRegex(EngineError, "closed"):
            game.observe("P0")

    def test_parallel_independent_games(self):
        def run(seed):
            with self.game() as game:
                for _ in range(100):
                    # Observation calls are read-only; independent handles have no global RNG.
                    game.observe("P0")
                return json.dumps(game.observe("P0"), sort_keys=True)
        with concurrent.futures.ThreadPoolExecutor(4) as pool:
            outputs = list(pool.map(run, range(8)))
        self.assertEqual(len(set(outputs)), 1)


if __name__ == "__main__":
    unittest.main()
