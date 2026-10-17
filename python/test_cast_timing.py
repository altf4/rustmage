"""Paid chosen-spell timing through ordinary decks and the native public API."""
import unittest
from rustmage import Game, EngineError
import test_adventure

WHIRL = "Whirlwing Stormbrood // Dynamic Soar"
DRAGON = "Dirgur Island Dragon // Skimming Strike"
BILBO = "Bilbo Baggins, Burglar // Take a Glance"
class CastingTimingTests(unittest.TestCase):
    replay = test_adventure.AdventureTests.replay
    denied = test_adventure.AdventureTests.denied
    def test_paid_flash_source_omen_dragon_and_adventure_both_seats(self):
        for p in ["P0", "P1"]:
            with self.subTest(seat=p): self.scenario(p,2640)
    def scenario(self,p,seed,library=None):
        q="P1" if p=="P0" else "P0"
        deck=["Island"]*16+["Forest"]*16+sum(([n]*4 for n in [WHIRL,DRAGON,BILBO,"Bear Cub","Bygone Colossus","Shock","Thrill of Possibility"]),[])
        self.assertEqual(len(deck),60)
        with Game([deck,deck],seed=seed,starting_player=p,deck_rules="Standard",library=library) as g:
            stage="source";source=None;inset=None;dragon=None;bilbo=None;base=None;descriptor=set();attacked=False;frames=0
            for iteration in range(30000):
                o=g.observe("P0")
                if o["outcome"] is not None:
                    self.assertEqual(stage,"finish",(p,seed,stage,iteration))
                    self.assertTrue(attacked)
                    self.assertTrue({(source,"Normal"),(inset,"Omen"),(dragon,"Normal"),(bilbo,"Adventure")} <= descriptor)
                    return frames
                o=g.observe(o["actor"]);actor=o["actor"];objects={v["id"]:v for v in o["objects"]};offturn=o["active"]!=p
                mine=g.observe(p);hand=[v for v in mine["objects"] if v["zone"]=="Hand"]
                ally=next((v for v in objects.values() if v["owner"]==p and v["zone"]=="Battlefield" and v["card"]=="Bear Cub"),None)
                lands=[v for v in objects.values() if v["owner"]==p and v["zone"]=="Battlefield" and v["card"] in ["Island","Forest"] and not v["tapped"]]
                quiet=o["decision"]=="priority" and not o["stack"] and not o["announcing"] and not o["resolving"]
                for d in o["spell_characteristics"]:
                    if d["object"]["card"] in [source,inset,dragon,bilbo]:
                        descriptor.add((d["object"]["card"],d["face"]))
                        if d["object"]["card"]==inset and d["face"]=="Omen":
                            self.assertEqual((d["name"],d["type_line"],d["mana_cost"]),("Dynamic Soar","Sorcery — Omen","{2}{G}"))
                if quiet and stage=="wait_source":
                    self.assertEqual(objects[source]["zone"],"Battlefield")
                    self.assertEqual((objects[source]["power"],objects[source]["toughness"]),(4,3));stage="omen"
                if quiet and stage=="wait_omen":
                    self.assertEqual(ally["counters"]["plus_one"],base["counters"]+3)
                    self.assertEqual(o["players"][int(p[-1])]["library_count"],base["library"]+1)
                    self.assertFalse(any(v["id"]==inset for v in mine["objects"]))
                    self.assertFalse(o["adventures"]);self.assertFalse(o["exile_permissions"]);stage="dragon"
                if quiet and stage=="wait_dragon":
                    self.assertEqual(objects[dragon]["zone"],"Battlefield");stage="adventure"
                if quiet and stage=="wait_adventure":
                    self.assertEqual(objects[bilbo]["zone"],"Exile")
                    self.assertTrue(o["adventures"])
                    if actor==p and offturn:
                        self.denied(g,o,{"kind":"AnnounceExile","card":bilbo})
                        stage="finish"
                if quiet and actor==p and offturn:
                    land=next((v for v in hand if v["card"] in ["Island","Forest"]),None)
                    if land:self.denied(g,o,{"kind":"PlayLand","card":land["id"]})
                    if stage=="source":
                        c=next((v for v in hand if v["card"]==WHIRL),None)
                        if c:self.denied(g,o,{"kind":"AnnounceOmen","card":c["id"]})
                def score(a):
                    k=a["kind"];n=objects.get(a.get("card"),{}).get("card","")
                    if k in ["Concede","Mulligan","Cast","BeginCast","AnnounceWarp","AnnounceFlashback","AnnouncePlot","AnnounceActivate","Activate"]:return -2000
                    if k=="AnnounceCast":
                        if stage=="finish":return 2300 if n in [WHIRL,DRAGON,"Bear Cub","Bygone Colossus"] else -1000
                        if actor==p:
                            if n=="Bear Cub":return 2400 if not ally else -1000
                            if n==WHIRL:return 2600 if stage=="source" and offturn and len(lands)>=8 and ally and sum(v["card"]==WHIRL for v in hand)>=2 else -1000
                            if n==DRAGON:return 2600 if stage=="dragon" and offturn else -1000
                        return -1000
                    if k=="AnnounceOmen":return 2600 if actor==p and stage=="omen" and offturn and n==WHIRL else -1000
                    if k=="AnnounceAdventure":return 2600 if actor==p and stage=="adventure" and offturn and n==BILBO else -1000
                    if k=="AnnounceExile":return -1000
                    if k=="ChooseTarget":return 2700 if ally and a["target"].get("Card")==ally["id"] else 500
                    if k=="PlayLand":return 1800-20*sum(v["owner"]==actor and v["zone"]=="Battlefield" and v["card"]==n for v in objects.values())
                    if k in ["TapMana","ActivateMana","AnnounceMana"]:return 2100 if o["decision"] in ["pay_costs","resolution_mana"] and n in ["Island","Forest"] else -1000
                    if k in ["PayCosts","PutTrigger","FinishTargets","FinishSearch","FinishAttackers","FinishBlockers","Keep","ChooseSpellCost"]:return 2400
                    if k in ["Discard","Bottom"]:
                        reserve = (2 if stage=="source" else 1) if n==WHIRL else (0 if ally else 1) if n=="Bear Cub" else 1
                        if actor==p and n in [WHIRL,DRAGON,BILBO,"Bear Cub"] and sum(v["card"]==n for v in hand)<=reserve:return -1000
                        return 2200 if n in ["Shock","Thrill of Possibility","Bygone Colossus"] else 1000
                    if k=="Attack":return 2800 if stage=="finish" else -1000
                    if k=="Pass":return 0
                    return 1000
                j=max(range(len(o["actions"])),key=lambda j:score(o["actions"][j]));a=o["actions"][j];self.assertGreaterEqual(score(a),0,(stage,o["decision"]))
                if actor==p and a["kind"]=="AnnounceCast":
                    if stage=="source" and objects[a["card"]]["card"]==WHIRL:source=a["card"];stage="wait_source"
                    elif stage=="dragon" and objects[a["card"]]["card"]==DRAGON:dragon=a["card"];stage="wait_dragon"
                if a["kind"]=="AnnounceOmen" and actor==p:inset=a["card"];stage="wait_omen"
                if a["kind"]=="AnnounceAdventure" and actor==p:bilbo=a["card"];stage="wait_adventure"
                if a["kind"]=="PayCosts" and o["announcing"] and o["announcing"]["card"]==inset:
                    base=dict(counters=ally["counters"]["plus_one"],library=o["players"][int(p[-1])]["library_count"])
                if a["kind"]=="Attack" and a["card"]==source:attacked=True
                before=[g.observe(v) for v in ["P0","P1"]]
                with self.assertRaisesRegex(EngineError,"WrongActor"):g.act(dict(o,actor="P1" if actor=="P0" else "P0"),a)
                self.assertEqual(before,[g.observe(v) for v in ["P0","P1"]]);self.replay(g,o,j);frames+=1
            self.fail(("unreached",p,seed,stage))

if __name__=="__main__":unittest.main()
