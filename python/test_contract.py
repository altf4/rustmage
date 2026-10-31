"""Small deterministic checks of the public binding and planeswalker actions."""
import ctypes
import json
from pathlib import Path
import unittest

from rustmage import EngineError, Game, OBSERVATION_SCHEMA


ROOT = Path(__file__).resolve().parents[1]
DECKS = [["Forest"] * 60, ["Island"] * 60]


def game(**config):
    return Game(DECKS, seed=37, **config)


def keep_index(observation):
    return observation["actions"].index({"kind": "Keep"})


def state(observation):
    """Restoring intentionally changes only the observation revision."""
    return {key: value for key, value in observation.items() if key != "revision"}


class BindingContractTests(unittest.TestCase):
    def test_planeswalker_loyalty_through_public_actions(self):
        # The fixed opening has four Plains and three Ajanis. Freeform keeps the
        # public fixture deterministic without a special game-state constructor.
        decks = [["Plains"] * 14 + ["Ajani, Outland Chaperone"] * 6,
                 ["Plains"] * 20]
        with Game(decks, seed=37, deck_rules="Freeform") as handle:
            observation = handle.observe("P0")

            def choose(kind, **fields):
                nonlocal observation
                observation = handle.observe(observation["actor"])
                action = next((action for action in observation["actions"]
                               if action["kind"] == kind
                               and all(action.get(key) == value for key, value in fields.items())), None)
                self.assertIsNotNone(action, (kind, fields, observation["decision"]))
                observation = handle.step(observation, observation["actions"].index(action))

            def advance_until(predicate):
                nonlocal observation
                for _ in range(100):
                    observation = handle.observe(observation["actor"])
                    if predicate(observation):
                        return
                    kinds = {action["kind"] for action in observation["actions"]}
                    for kind in ("Keep", "FinishOpeningHand", "FinishAttackers", "Pass"):
                        if kind in kinds:
                            choose(kind)
                            break
                    else:
                        self.fail(f"unexpected decision: {observation['decision']}")
                self.fail("fixed planeswalker fixture did not reach its decision")

            for turn in (1, 3, 5):
                advance_until(lambda view: view["actor"] == view["active"] == "P0"
                              and view["turn"] == turn and view["step"] == "Main1")
                choose("PlayLand")
            for _ in range(3):
                choose("TapMana")
            walker = next(obj["id"] for obj in observation["objects"]
                          if obj["card"] == "Ajani, Outland Chaperone" and obj["zone"] == "Hand")
            choose("AnnounceCast", card=walker)
            choose("PayCosts")
            advance_until(lambda view: any(obj["id"] == walker and obj["zone"] == "Battlefield"
                                           for obj in view["objects"]))
            before = handle.observe("P0")
            ajani = next(obj for obj in before["objects"] if obj["id"] == walker)
            self.assertEqual(ajani["counters"]["loyalty"], 3)
            action = next(action for action in before["actions"]
                          if action["kind"] == "AnnounceActivate"
                          and action["card"] == walker and action["ability"] == 0)
            snapshot = handle.snapshot()
            with handle.clone() as branch:
                raw = branch.act(before, action)
                observation = handle.step(before, before["actions"].index(action))
                self.assertEqual(state(raw), state(observation))
            choose("PayCosts")
            ajani = next(obj for obj in handle.observe("P0")["objects"] if obj["id"] == walker)
            self.assertEqual(ajani["counters"]["loyalty"], 4)
            advance_until(lambda view: any(obj["card"] == "@token/c48e9e904360e4cee1deeed8"
                                           and obj["zone"] == "Battlefield" for obj in view["objects"]))
            current = handle.observe("P0")
            self.assertFalse(any(action["kind"] == "AnnounceActivate" and action["card"] == walker
                                 for action in current["actions"]))
            handle.restore(snapshot)
            restored = handle.observe("P0")
            ajani = next(obj for obj in restored["objects"] if obj["id"] == walker)
            self.assertEqual(ajani["counters"]["loyalty"], 3)
            self.assertIn(action, restored["actions"])

    def test_owned_branch_survives_parent_close(self):
        parent = game()
        self.addCleanup(parent.close)
        before = parent.observe("P0")
        snapshot = parent.snapshot()
        branch = parent.clone()
        self.addCleanup(branch.close)
        self.assertEqual(branch._lib.rm_abi_version(), 1)
        self.assertEqual(branch.observe("P0"), before)
        # A branch owns its game but does not inherit a parent's checkpoints.
        with self.assertRaisesRegex(EngineError, "unknown snapshot"):
            branch.restore(snapshot)
        parent.close()
        parent.close()
        for operation in (lambda: parent.observe("P0"), parent.clone):
            with self.assertRaisesRegex(EngineError, "closed"):
                operation()
        after = branch.step(before, keep_index(before))
        self.assertEqual(after["actor"], "P1")
        self.assertGreater(after["revision"], before["revision"])

    def test_outside_constructor_round_trip_and_invalid_envelopes(self):
        outside = {"mode": "ConstructedSideboard", "pools": [["Shock"], []]}
        with game(outside=outside) as handle:
            self.assertIn("Shock", json.dumps(handle.observe("P0")))
            self.assertNotIn("Shock", json.dumps(handle.observe("P1")))
            checkpoint = handle.snapshot()
            branch = handle.clone()
            self.addCleanup(branch.close)
            before = state(branch.observe("P0"))
            handle.restore(checkpoint)
            self.assertEqual(state(handle.observe("P0")), before)
            native = handle._lib
            envelope = dict(config=dict(decks=DECKS, seed=1), outside=outside)
            for value in (b"{}", b"{", b"\xff", json.dumps({**envelope, "extra": True}).encode(),
                          json.dumps({**envelope, "outside": {"mode": "bad", "pools": [[], []]}}).encode()):
                with self.subTest(value=value):
                    error = ctypes.c_void_p()
                    pointer = native.rm_new_with_outside(value, ctypes.byref(error))
                    self.assertFalse(pointer)
                    self.assertTrue(error.value)
                    self.assertTrue(handle._take_string(error.value))
            self.assertFalse(native.rm_new_with_outside(b"{}", None))
            self.assertEqual(state(handle.observe("P0")), before)

    def test_native_malformed_inputs_are_errors_and_preserve_live_handle(self):
        with game() as handle:
            native = handle._lib
            before = handle.observe("P0")
            invalid_card = json.dumps({"decks": [["bad\0"] * 60] * 2, "seed": 0}).encode()
            for config in (None, b"\xff", b"{", b"{}", invalid_card):
                with self.subTest(config=config):
                    error = ctypes.c_void_p()
                    pointer = native.rm_new(config, ctypes.byref(error))
                    self.assertFalse(pointer)
                    self.assertTrue(error.value)
                    self.assertTrue(handle._take_string(error.value))
            self.assertFalse(native.rm_new(b"{}", None))
            for pointer, payload in ((handle._handle, b"{"), (handle._handle, b"\xff"),
                                     (handle._handle, None), (None, b"{}")):
                with self.subTest(payload=payload, null_handle=pointer is None):
                    response = native.rm_request(pointer, payload)
                    self.assertTrue(response)
                    self.assertTrue(json.loads(handle._take_string(response))["error"])
                    self.assertEqual(handle.observe("P0"), before)
            self.assertFalse(native.rm_clone(None))
            native.rm_free(None)
            native.rm_string_free(None)

    def test_rejected_requests_are_transactional(self):
        with game() as handle:
            before = handle.observe("P0")
            requests = [
                ("unknown", {}),
                ("observe", {"viewer": "P2"}),
                ("observe", {"viewer": "P0", "extra": True}),
                ("step", {"actor": "P1", "revision": before["revision"],
                          "index": keep_index(before), "viewer": "P0"}),
                ("step", {"actor": "P0", "revision": before["revision"],
                          "index": len(before["actions"]), "viewer": "P0"}),
            ]
            for operation, fields in requests:
                with self.subTest(operation=operation, fields=fields):
                    with self.assertRaises(EngineError):
                        handle.request(operation, **fields)
                    self.assertEqual(handle.observe("P0"), before)
            with self.assertRaisesRegex(EngineError, "IllegalAction"):
                handle.act(before, {"kind": "PlayLand", "card": before["objects"][0]["id"]})
            self.assertEqual(handle.observe("P0"), before)

    def test_deck_admission_checks_size_copies_legality_and_implementation(self):
        with Game([["Plains"] * 56 + ["Savannah Lions"] * 4] * 2,
                  deck_rules="Standard") as handle:
            self.assertEqual(handle.observe("P0")["players"][0]["hand_count"], 7)
        for deck, rules, error in [
            (["Forest"] * 59, "Constructed", "60 cards"),
            (["Forest"] * 55 + ["Savannah Lions"] * 5, "Standard", "more than 4"),
            (["Mountain"] * 56 + ["Lightning Bolt"] * 4, "Standard", "not Standard legal"),
            (["no such card"] * 60, "Freeform", "unknown card"),
        ]:
            with self.subTest(rules=rules, error=error), self.assertRaisesRegex(EngineError, error):
                Game([deck, deck], deck_rules=rules)
        # The same supported off-pool card is allowed under the explicit Freeform policy.
        with Game([["Lightning Bolt"] * 7] * 2, deck_rules="Freeform") as handle:
            self.assertEqual(len(handle.observe("P0")["objects"]), 7)
        coverage = json.loads((ROOT / "data/standard/coverage.json").read_text())
        unsupported = next((row["name"] for row in coverage["cards"]
                            if row["standard_legal"] and not row["implemented"]), None)
        if unsupported is not None:
            with self.assertRaisesRegex(EngineError, "rules not implemented"):
                Game([["Forest"] * 56 + [unsupported] * 4] * 2, deck_rules="Standard")

    def test_private_views_and_schema_are_public_contracts(self):
        with game() as handle:
            for viewer, own_card in (("P0", "Forest"), ("P1", "Island")):
                with self.subTest(viewer=viewer):
                    observation = handle.observe(viewer)
                    self.assertEqual(observation["schema"], OBSERVATION_SCHEMA)
                    self.assertEqual(observation["viewer"], viewer)
                    self.assertEqual(len(observation["objects"]), 7)
                    self.assertTrue(all((item["zone"], item["owner"], item["card"])
                                        == ("Hand", viewer, own_card) for item in observation["objects"]))
                    self.assertEqual([(item["hand_count"], item["library_count"])
                                      for item in observation["players"]], [(7, 53), (7, 53)])
                    self.assertEqual(observation["known_hand"], [])
                    if viewer != observation["actor"]:
                        self.assertEqual(observation["actions"], [])
                    else:
                        self.assertIn({"kind": "Keep"}, observation["actions"])

    def test_raw_and_indexed_actions_have_the_same_owned_result(self):
        with game() as parent, parent.clone() as raw, parent.clone() as indexed:
            before = parent.observe("P0")
            index = keep_index(before)
            expected = raw.act(before, before["actions"][index])
            self.assertEqual(indexed.step(before, index), expected)
            self.assertEqual(parent.observe("P0"), before)

    def test_snapshot_restore_replays_and_rejects_stale_actions(self):
        with game() as handle:
            before = handle.observe("P0")
            snapshot = handle.snapshot()
            expected = handle.step(before, keep_index(before))
            handle.restore(snapshot)
            restored = handle.observe("P0")
            self.assertEqual(state(restored), state(before))
            self.assertGreater(restored["revision"], expected["revision"])
            for operation in (lambda: handle.step(before, keep_index(before)),
                              lambda: handle.act(before, {"kind": "Keep"})):
                with self.assertRaisesRegex(EngineError, "StaleRevision"):
                    operation()
                self.assertEqual(handle.observe("P0"), restored)
            self.assertEqual(state(handle.step(restored, keep_index(restored))), state(expected))
            handle.drop_snapshot(snapshot)
            after = handle.observe("P1")
            for operation in (lambda: handle.restore(snapshot), lambda: handle.drop_snapshot(snapshot)):
                with self.assertRaisesRegex(EngineError, "unknown snapshot"):
                    operation()
                self.assertEqual(handle.observe("P1"), after)

    def test_fixed_seed_and_starting_player_reproduce_decisions(self):
        with game(starting_player="P1") as first, game(starting_player="P1") as second:
            before = first.observe("P1")
            self.assertEqual(before, second.observe("P1"))
            self.assertEqual(before["actor"], "P1")
            self.assertEqual(first.observe("P0")["actions"], [])
            self.assertEqual(first.act(before, {"kind": "Keep"}),
                             second.step(before, keep_index(before)))


if __name__ == "__main__":
    unittest.main()
