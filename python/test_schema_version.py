"""Current contract expectations use the same source as Rust's wire version."""
import ast
import os
from pathlib import Path
import unittest
from rustmage import Game, OBSERVATION_SCHEMA

ROOT = Path(__file__).resolve().parents[1]

class SchemaVersionTests(unittest.TestCase):
    def test_native_observation_matches_shared_version(self):
        self.assertEqual(OBSERVATION_SCHEMA, int((ROOT / "data/observation-schema.txt").read_text()))
        self.assertGreater(OBSERVATION_SCHEMA, 0)
        deck = ["Forest"] * 60
        with Game([deck, deck], library=os.environ.get("RUSTMAGE_SCHEMA_LIBRARY")) as game:
            for player in ("P0", "P1"):
                self.assertEqual(game.observe(player)["schema"], OBSERVATION_SCHEMA)

    def test_current_schema_assertions_do_not_repeat_a_numeric_version(self):
        def pairs(actual, expected):
            if isinstance(actual, (ast.Tuple, ast.List)) and isinstance(expected, (ast.Tuple, ast.List)):
                for left, right in zip(actual.elts, expected.elts):
                    yield from pairs(left, right)
            else:
                yield actual, expected
        for directory in ("python", "examples"):
            for path in (ROOT / directory).glob("*.py"):
                tree = ast.parse(path.read_text(), filename=str(path))
                for node in ast.walk(tree):
                    if not (isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
                            and node.func.attr == "assertEqual" and len(node.args) == 2):
                        continue
                    for actual, expected in pairs(*node.args):
                        if (isinstance(actual, ast.Subscript) and isinstance(actual.slice, ast.Constant)
                                and actual.slice.value == "schema"):
                            self.assertFalse(isinstance(expected, ast.Constant) and type(expected.value) is int,
                                             f"{path.relative_to(ROOT)}:{node.lineno}: import OBSERVATION_SCHEMA")

if __name__ == "__main__":
    unittest.main()
