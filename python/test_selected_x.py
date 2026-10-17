"""Chosen Omen X, mass effects and discard Ward through paid Standard games."""
import unittest
import json
from pathlib import Path
from rustmage import Game, EngineError
import test_adventure

NAME = "Scavenger Regent // Exude Toxin"
BASICS = ("Swamp", "Forest", "Mountain")

class SelectedXTests(unittest.TestCase):
    replay = test_adventure.AdventureTests.replay
    denied = test_adventure.AdventureTests.denied

    def test_paid_normal_ward_and_omen_zero_through_three_both_seats(self):
        for p in ("P0", "P1"):
            for x in range(4):
                for pay in (False, True):
                    with self.subTest(seat=p, x=x, ward=pay):
                        self.scenario(p, x, pay, 2700+x)

    def test_internal_inset_cannot_be_a_separate_deck_card(self):
        with self.assertRaises(EngineError):
            with Game([["Exude Toxin"]*60, ["Swamp"]*60], deck_rules="Freeform"):
                pass

    def scenario(self, p, x, pay, seed, library=None):
        q = "P1" if p == "P0" else "P0"
        deck = ["Swamp"]*14+["Forest"]*8+["Mountain"]*10
        deck += sum(([n]*4 for n in (NAME, "Bear Cub", "Three Tree Mascot", "Chrome Companion", "Shock", "Bygone Colossus", "Thrill of Possibility")), [])
        self.assertEqual(len(deck), 60)
        with Game([deck, deck], seed=seed, starting_player=p, deck_rules="Standard", library=library) as g:
            self.assertEqual(g.observe(p)["catalogue"],json.loads((Path(__file__).resolve().parents[1]/"data/standard/coverage.json").read_text())["catalogue_version"])
            self.assertEqual(g.observe(p)["schema"],103)
            stage="setup"; source=inset=shock=resource=None; base=None
            frames=0; ward_seen=False; x_seen=False; descriptor=False; attacked=False
            for iteration in range(30000):
                o=g.observe("P0")
                if o["outcome"] is not None:
                    self.assertEqual(stage,"finish",(p,x,pay,seed,stage,iteration))
                    self.assertTrue(ward_seen and x_seen and descriptor and attacked)
                    return frames
                o=g.observe(o["actor"]); actor=o["actor"]
                ob={v["id"]:v for v in o["objects"]}
                mine=g.observe(p); hands={v:g.observe(v)["objects"] for v in (p,q)}
                hands={v:[c for c in cards if c["zone"]=="Hand"] for v,cards in hands.items()}
                field=lambda owner,name:next((v for v in ob.values() if v["owner"]==owner and v["card"]==name and v["zone"]=="Battlefield"),None)
                bears={v:field(v,"Bear Cub") for v in (p,q)}
                mascots={v:field(v,"Three Tree Mascot") for v in (p,q)}
                dragons={v:field(v,NAME) for v in (p,q)}
                lands=[v for v in ob.values() if v["owner"]==p and v["zone"]=="Battlefield" and v["card"] in BASICS and not v["tapped"]]
                quiet=o["decision"]=="priority" and not o["stack"] and not o["announcing"] and not o["resolving"]
                if stage=="setup" and all(bears.values()) and all(mascots.values()) and dragons[q]:stage="normal"
                if quiet and stage=="wait_normal":
                    self.assertEqual(ob[source]["zone"],"Battlefield")
                    self.assertEqual((ob[source]["power"],ob[source]["toughness"]),(4,4))
                    stage="ward"
                if quiet and stage=="wait_ward":
                    self.assertTrue(ward_seen)
                    self.assertEqual(ob[shock]["zone"],"Graveyard")
                    self.assertEqual(ob[source]["damage"],2 if pay else 0)
                    if pay:self.assertEqual(ob[resource]["zone"],"Graveyard")
                    stage="omen"
                if quiet and stage=="wait_omen":
                    self.assertTrue(x_seen and descriptor)
                    self.assertEqual(o["players"][int(p[-1])]["library_count"],base["library"]+1)
                    self.assertFalse(any(v["id"]==inset for v in mine["objects"]))
                    self.assertFalse(o["adventures"]);self.assertFalse(o["exile_permissions"])
                    for card,old in base["creatures"].items():
                        now=ob[card]
                        if old["card"]=="Bear Cub":
                            if x>=2:self.assertEqual(now["zone"],"Graveyard")
                            else:self.assertEqual((now["power"],now["toughness"]),(old["power"]-x,old["toughness"]-x))
                        else:
                            self.assertEqual(now["zone"],"Battlefield")
                            self.assertEqual((now["power"],now["toughness"]),(old["power"],old["toughness"]))
                    stage="finish"
                for d in o["spell_characteristics"]:
                    if d["object"]["card"]==inset and d["face"]=="Omen":
                        self.assertEqual((d["name"],d["type_line"],d["mana_cost"]),("Exude Toxin","Sorcery — Omen","{X}{B}{B}"))
                        if o["announcing"] and o["announcing"]["x"] is not None:
                            self.assertEqual(d["mana_value"],2+x);descriptor=True
                if o["x_choice"]:
                    self.assertEqual(actor,p);self.assertIsNone(o["payment_request"])
                    self.assertIsNone(g.observe(q)["x_choice"]);self.assertFalse(g.observe(q)["actions"])
                    intervals=o["x_choice"]["intervals"]
                    self.assertTrue(any(r["minimum"]<=x<=r["maximum"] for r in intervals))
                    self.denied(g,o,{"kind":"ChooseX","value":intervals[-1]["maximum"]+1})
                    action={"kind":"ChooseX","value":x};x_seen=True
                    # Concrete X values need not appear among representative actions.
                    before=[g.observe(v) for v in (p,q)];saved=g.snapshot()
                    with self.assertRaisesRegex(EngineError,"WrongActor"):g.act(dict(o,actor=q),action)
                    self.assertEqual(before,[g.observe(v) for v in (p,q)])
                    with g.clone() as parent:
                        child=parent.clone()
                    with child:
                        child.act(child.observe(p),action)
                        g.act(o,action)
                        self.assertEqual([g.observe(v) for v in (p,q)],[child.observe(v) for v in (p,q)])
                        expected=[g.observe(v) for v in (p,q)]
                    g.restore(saved)
                    with self.assertRaisesRegex(EngineError,"StaleRevision"):g.act(o,action)
                    g.act(g.observe(p),action)
                    actual=[g.observe(v) for v in (p,q)]
                    for a,b in zip(actual,expected):a["revision"]=b["revision"]
                    self.assertEqual(actual,expected);g.drop_snapshot(saved);frames+=1;continue
                ward=o["effect_choice"] is not None
                if ward:
                    self.assertEqual(actor,q);self.assertIsNone(g.observe(p)["effect_choice"])
                    self.assertEqual(stage,"wait_ward");ward_seen=True
                def score(a):
                    k=a["kind"]; n=ob.get(a.get("card"),{}).get("card","")
                    if k in ("Concede","Mulligan","Cast","BeginCast","AnnounceWarp","AnnounceFlashback","AnnouncePlot","AnnounceActivate","Activate"):return -2000
                    if k=="AnnounceCast":
                        if stage=="finish":return 2300 if n in (NAME,"Bear Cub","Three Tree Mascot","Bygone Colossus") else -1000
                        if n=="Bear Cub":return 2400 if not bears[actor] else -1000
                        if n=="Three Tree Mascot":return 2400 if not mascots[actor] else -1000
                        if n==NAME:
                            if actor==q:return 2400 if not dragons[q] else -1000
                            return 2600 if stage=="normal" and len(lands)>=8 and sum(v["card"]==NAME for v in hands[p])>=2 else -1000
                        if n=="Shock":return 2600 if actor==q and stage=="ward" and len(hands[q])>=2 else -1000
                        return -1000
                    if k=="AnnounceOmen":return 2700 if actor==p and stage=="omen" and n==NAME and len(lands)>=x+2 and sum(v["card"]=="Swamp" for v in lands)>=2 else -1000
                    if k in ("AnnounceAdventure","AnnounceExile"):return -1000
                    if k=="ChooseTarget":return 2800 if a["target"].get("Card")==source else 500
                    if k=="PlayLand":return 1800-20*sum(v["owner"]==actor and v["zone"]=="Battlefield" and v["card"]==n for v in ob.values())
                    if k in ("TapMana","ActivateMana","AnnounceMana"):return 2100 if o["decision"] in ("pay_costs","resolution_mana") else -1000
                    if k in ("PayCosts","PutTrigger","FinishTargets","FinishSearch","FinishAttackers","FinishBlockers","Keep","ChooseSpellCost"):return 2400
                    if k=="DeclineEffectPayment":return -1000 if pay else 2800
                    if k in ("Discard","Bottom"):
                        if ward:return 2800 if pay else -1000
                        reserve=0
                        if stage!="finish":
                            if n==NAME:reserve=(2 if stage in ("setup","normal") else 1) if actor==p else (0 if dragons[q] else 1)
                            elif n=="Shock" and actor==q:reserve=1 if stage in ("setup","normal","wait_normal","ward") else 0
                            elif n=="Bear Cub":reserve=0 if bears[actor] else 1
                            elif n=="Three Tree Mascot":reserve=0 if mascots[actor] else 1
                        if sum(v["card"]==n for v in hands[actor])<=reserve:return -1000
                        return 2200 if n in ("Chrome Companion","Thrill of Possibility","Bygone Colossus") else 1000
                    if k=="Attack":return 2800 if stage=="finish" else -1000
                    if k=="Pass":return 0
                    return 1000
                j=max(range(len(o["actions"])),key=lambda i:score(o["actions"][i]));a=o["actions"][j]
                self.assertGreaterEqual(score(a),0,(stage,o["decision"],o["actions"]))
                if a["kind"]=="AnnounceCast":
                    if actor==p and stage=="normal" and ob[a["card"]]["card"]==NAME:source=a["card"];stage="wait_normal"
                    if actor==q and stage=="ward" and ob[a["card"]]["card"]=="Shock":shock=a["card"];stage="wait_ward"
                if a["kind"]=="AnnounceOmen" and actor==p:inset=a["card"];stage="wait_omen"
                if ward and a["kind"]=="Discard":resource=a["card"]
                if a["kind"]=="PayCosts" and o["announcing"] and o["announcing"]["card"]==inset:
                    self.assertEqual((o["payment_request"]["required"],o["payment_request"]["generic"]),([0,0,2,0,0,0],x))
                    self.assertEqual(sum(a["payment"]),x+2)
                    base={"library":o["players"][int(p[-1])]["library_count"],"creatures":{c["id"]:c for c in [*bears.values(),*mascots.values(),*dragons.values()]}}
                if a["kind"]=="Attack" and a["card"]==source:attacked=True
                before=[g.observe(v) for v in (p,q)]
                with self.assertRaisesRegex(EngineError,"WrongActor"):g.act(dict(o,actor="P1" if actor=="P0" else "P0"),a)
                self.assertEqual(before,[g.observe(v) for v in (p,q)])
                self.replay(g,o,j);frames+=1
            self.fail(("unreached",p,x,pay,seed,stage))

if __name__=="__main__":unittest.main()
