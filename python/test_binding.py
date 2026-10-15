import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from rustmage import EngineError, Game


class BindingTests(unittest.TestCase):
    def game(self):
        return Game([["Mountain"] * 24 + ["LightningBolt"] * 36] * 2, seed=17, deck_rules="Freeform")

    def test_private_views_clone_restore_and_stale_indices(self):
        with self.game() as game:
            observation = game.observe("P0")
            coverage = json.loads((Path(__file__).resolve().parents[1] / "data/standard/coverage.json").read_text())
            self.assertEqual(observation["catalogue"], coverage["catalogue_version"])
            self.assertEqual(observation["schema"], 3)
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

    def test_standard_admission_distinguishes_legality_and_rule_coverage(self):
        deck = ["Plains"] * 56 + ["Savannah Lions"] * 4
        with Game([deck, deck], deck_rules="Standard") as game:
            self.assertEqual(game.observe("P0")["players"][0]["library_count"], 53)
        with self.assertRaisesRegex(EngineError, "not Standard legal"):
            Game([["Mountain"] * 56 + ["Lightning Bolt"] * 4] * 2, deck_rules="Standard")
        with self.assertRaisesRegex(EngineError, "rules not implemented"):
            Game([["Hare Apparent"] * 60] * 2, deck_rules="Standard")

    def test_decoded_nul_in_invalid_card_is_an_error_without_aborting_host(self):
        # A regression in this boundary can abort the interpreter, so exercise it
        # in a child and assert normal exit rather than crashing the whole suite.
        result = subprocess.run([sys.executable, "-c", """
from rustmage import EngineError, Game
try:
    Game([["bad\\x00"] * 60] * 2)
except EngineError as error:
    assert "unknown card" in str(error)
else:
    raise AssertionError("invalid card accepted")
"""], env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parent)}, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

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

    def test_complete_selfplay_through_shared_library(self):
        # The complete control loop stays in this process. Actor changes require
        # a fresh observation for the new player, never a union of both hands.
        with self.game() as game:
            observation = game.observe("P0")
            cast_count = 0
            for _ in range(5000):
                if observation["outcome"] is not None:
                    break
                actor = observation["actor"]
                if observation["viewer"] != actor:
                    observation = game.observe(actor)
                opponent = "P1" if actor == "P0" else "P0"

                def score(action):
                    kind = action["kind"]
                    if kind == "Cast":
                        return 80 if action["target"] == {"Player": opponent} else -10
                    return {"Keep": 100, "PlayLand": 90, "TapMana": 20,
                            "Discard": 40, "Concede": -1000, "Mulligan": -100}.get(kind, 0)

                index = max(range(len(observation["actions"])), key=lambda i: score(observation["actions"][i]))
                cast_count += observation["actions"][index]["kind"] == "Cast"
                observation = game.step(observation, index)
            else:
                self.fail("self-play did not end naturally")
            self.assertGreaterEqual(cast_count, 7)
            self.assertIn("Winner", observation["outcome"])
            self.assertEqual(game.observe("P0")["outcome"], game.observe("P1")["outcome"])

    def test_standard_keyword_combat_through_shared_library(self):
        white = ["Plains"] * 28
        for name in ["Savannah Lions", "Healer's Hawk", "Bishop's Soldier", "Brightblade Stoat",
                     "Leonin Skyhunter", "Serra Angel", "Shrike Force", "Iron Giant"]:
            white.extend([name] * 4)
        green = ["Forest"] * 24
        for name in ["Ankle Biter", "Dragon Sniper", "Magnigoth Sentry", "Colossadactyl",
                     "Tajuru Pathwarden", "Galewind Moose", "Gigantosaurus", "Bear Cub", "Jibbirik Omnivore"]:
            green.extend([name] * 4)
        seen_steps = set()
        with Game([white, green], seed=2, deck_rules="Standard") as game:
            obs = game.observe("P0")
            for _ in range(10000):
                if obs["outcome"] is not None:
                    break
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                seen_steps.add(obs["step"])
                def score(action):
                    if action["kind"] == "AssignDamage":
                        self.assertIn("target", action)
                        if obs["damage_assigned"]:
                            self.assertIn("source", obs["damage_assigned"][0])
                        return 50 + action["amount"]
                    if action["kind"] == "TapMana":
                        return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                    return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                            "Cast": 80, "Attack": 60, "Block": 60,
                            "Discard": 50}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                obs = game.step(obs, index)
            else:
                self.fail("Standard keyword game did not end")
            self.assertIn("FirstStrikeDamage", seen_steps)
            self.assertIn("CombatDamage", seen_steps)
            self.assertIn("Winner", obs["outcome"])


if __name__ == "__main__":
    unittest.main()
