"""Chosen-spell timing permissions use exact whole-pair grammar."""
import copy
import unittest
from scripts import test_omen as omen

class CastingTimingCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        omen.OmenCompilerTests.setUpClass.__func__(cls)
    def compile(self, card):
        return omen.OmenCompilerTests.compile(self, card)
    def admitted(self, card):
        return omen.OmenCompilerTests.admitted(self, card)

    name = "Whirlwing Stormbrood // Dynamic Soar"
    grant = "You may cast sorcery spells and Dragon spells as though they had flash."

    def test_timing_table_and_renamed_complete_pair(self):
        for renamed in [False, True]:
            c = copy.deepcopy(self.cards[self.name])
            if renamed:
                c["name"] = "Test Dragon // Test Counters"
                c["card_faces"][0]["name"] = "Test Dragon"
                c["card_faces"][1]["name"] = "Test Counters"
            rust, coverage = self.compile(c)
            self.assertTrue(next(v["implemented"] for v in coverage["cards"] if v["name"] == c["name"]))
            self.assertIn("=> &[CastFilter::Sorcery, CastFilter::Subtype(\"Dragon\")]", rust)

    def test_normal_permanent_unknown_parentheses_and_spell_instructions_are_closed(self):
        front = copy.deepcopy(self.cards[self.name]["card_faces"][0])
        front["layout"] = "normal"
        front["oracle_id"] = self.cards[self.name]["oracle_id"]
        front["colors"] = ["U"]
        front["keywords"] = ["Flash", "Flying"]
        self.assertTrue(self.admitted(front))
        for residual in ["\n(You win the game.)", "\n  (You win the game.)", " (Draw nineteen cards.)"]:
            c = copy.deepcopy(front)
            c["oracle_text"] += residual
            self.assertFalse(self.admitted(c))
        for name in ["Shock", "Quick Study"]:
            c = copy.deepcopy(self.cards[name])
            c["oracle_text"] += "\nYou may cast sorcery spells as though they had flash."
            self.assertFalse(self.admitted(c))

    def test_static_timing_requires_a_compiled_battlefield_source(self):
        for type_line in ["Artifact", "Creature — Human"]:
            c = copy.deepcopy(self.cards["Shock"])
            c.update(type_line=type_line, power="2", toughness="2")
            c["oracle_text"] += "\nYou may cast sorcery spells as though they had flash."
            self.assertFalse(self.admitted(c))

    def test_timing_route_cannot_use_legacy_name_residual_bypasses(self):
        for name in ["Grizzly Bears", "Llanowar Elves"]:
            c=copy.deepcopy(self.cards[self.name]["card_faces"][0])
            c.update(name=name,layout="normal",oracle_id=self.cards[self.name]["oracle_id"],colors=["U"],keywords=["Flash","Flying"])
            self.assertTrue(self.admitted(c))
            for residual in ["\nDraw nineteen cards.","\n(You win the game.)"]:
                bad=copy.deepcopy(c)
                bad["oracle_text"]+=residual
                self.assertFalse(self.admitted(bad))

    def test_split_literal_and_nested_unknown_reminders_cannot_revive_legacy_names(self):
        front=copy.deepcopy(self.cards[self.name]["card_faces"][0])
        front.update(layout="normal",oracle_id=self.cards[self.name]["oracle_id"],colors=["U"],keywords=["Flash","Flying"])
        for name in ["Grizzly Bears","Llanowar Elves"]:
            for old,new in [("as though","as(You win the game.) though"),("cast","ca(Draw a card.)st"),("as though","as((You win the game.)) though"),("as though","as\nthough"),("flash.","fl(Discover 3.)ash.")]:
                c=copy.deepcopy(front);c["name"]=name;c["oracle_text"]=c["oracle_text"].replace(old,new)
                with self.subTest(name=name,fragment=new):self.assertFalse(self.admitted(c))

    def test_unknown_permissions_and_any_unknown_face_text_remain_gated(self):
        changes = [
            "You may cast sorcery spells and Dragon spells as though they had flying.",
            "You may cast sorcery spells or Dragon spells as though they had flash.",
            "You may cast sorcery spells and UnknownType spells as though they had flash.",
            "Each player may cast sorcery spells and Dragon spells as though they had flash.",
            self.grant + " (Draw a card.)",
            self.grant + " Then draw a card.",
            "You may cast sorcery spells and Dragon spells as though they had flash this turn.",
        ]
        for text in changes:
            c = copy.deepcopy(self.cards[self.name])
            c["card_faces"][0]["oracle_text"] = c["card_faces"][0]["oracle_text"].replace(self.grant,text)
            with self.subTest(text=text): self.assertFalse(self.admitted(c))
        for face in [0,1]:
            for residual in ["\nDraw nineteen cards."," (Ignore all costs.)","\nYou may play lands as though they had flash."]:
                c = copy.deepcopy(self.cards[self.name])
                c["card_faces"][face]["oracle_text"] += residual
                with self.subTest(face=face,residual=residual): self.assertFalse(self.admitted(c))

if __name__ == "__main__": unittest.main()
