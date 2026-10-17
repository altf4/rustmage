"""Actual paid Adventure faces and normal physical-card play through the public API."""
import json
import unittest
from pathlib import Path
from rustmage import Game, EngineError

FACTS = {c["name"]: c for c in json.loads((Path(__file__).resolve().parents[1] / "data/standard/cards.json").read_text())["cards"]}
NAMES = [
 "Bilbo Baggins, Burglar // Take a Glance", "Frolicking Familiar // Blow Off Steam",
 "Galvanic Giant // Storm Reading", "Gingerbread Hunter // Puny Snack",
 "Glóin the Mighty // Easy Pickings", "Gollum, Silent Slinker // Meager Meal",
 "Grabby Giant // That's Mine", "Hollow Scavenger // Bakery Raid",
 "Imodane's Recruiter // Train Troops", "Midgar, City of Mako // Reactor Raid",
 "Minecart Daredevil // Ride the Rails", "Obyra's Attendants // Desperate Parry",
 "Ratcatcher Trainee // Pest Problem", "Shrouded Shepherd // Cleave Shadows",
 "Smaug, the Great Calamity // Spew Flame", "Stormkeld Vanguard // Bear Down",
 "Tempest Hart // Scan the Clouds", "Threadbind Clique // Rip the Seams",
 "Two-Headed Hunter // Twice the Rage"]
BASICS = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
PREFERRED = ["Island", "Mountain", "Island", "Swamp", "Mountain", "Swamp", "Mountain", "Forest", "Mountain", "Swamp", "Mountain", "Island", "Mountain", "Swamp", "Mountain", "Forest", "Forest", "Plains", "Mountain"]

class AdventureTests(unittest.TestCase):
 def replay(self, g, o, j):
  actor = o["actor"]; other = "P1" if actor == "P0" else "P0"
  snapshot = g.snapshot(); parent = g.clone(); child = parent.clone(); parent.close()
  with child:
   expected = child.step(child.observe(actor), j); other_expected = child.observe(other)
  g.step(o, j); g.restore(snapshot)
  with self.assertRaisesRegex(EngineError, "StaleRevision"): g.step(o, j)
  actual = g.step(g.observe(actor), j); expected["revision"] = actual["revision"]
  self.assertEqual(actual, expected)
  other_actual = g.observe(other); other_expected["revision"] = other_actual["revision"]
  self.assertEqual(other_actual, other_expected); g.drop_snapshot(snapshot)
  return actual

 def denied(self, g, o, action):
  before = [g.observe(v) for v in ["P0", "P1"]]
  with self.assertRaises(EngineError): g.act(o, action)
  self.assertEqual(before, [g.observe(v) for v in ["P0", "P1"]])

 def test_all_nineteen_paid_insets_then_same_physical_card_normal_play_both_seats(self):
  for i in range(19):
   for player in ["P0", "P1"]:
    with self.subTest(card=NAMES[i], player=player): self.scenario(i, player, True)

 def test_all_nineteen_paid_normal_faces_both_seats(self):
  for i in range(19):
   for player in ["P0", "P1"]:
    with self.subTest(card=NAMES[i], player=player): self.scenario(i, player, False)

 def test_optional_creature_target_and_sacrifice_declines_still_resolve_both_seats(self):
  for i in [5,9]:
   for player in ["P0","P1"]:
    with self.subTest(card=NAMES[i],player=player): self.scenario(i,player,True,skip=True)

 def test_every_inset_alias_is_rejected_as_a_separate_deck_card(self):
  for name in NAMES:
   inset=FACTS[name]["card_faces"][1]["name"]
   with self.subTest(inset=inset):
    with self.assertRaises(EngineError):
     with Game([[inset]*60,["Forest"]*60],seed=1,deck_rules="Freeform"): pass

 def scenario(self, i, player, adventure, skip=False):
  name = NAMES[i]; pi = int(player[1]); enemy = "P1" if player == "P0" else "P0"
  own = sum(([b] * 9 for b in BASICS), []) + [PREFERRED[i]] * 24 + [name] * 20 + ["Bear Cub"] * 16 + ["Candy Trail"] * 12
  foe = ["Forest"] * 48 + ["Bear Cub"] * 20 + ["Candy Trail"] * 12
  with Game([own, foe] if player == "P0" else [foe, own], seed=2540+i, starting_player=player, deck_rules="Freeform") as g:
   source = None; stage = "setup"; base = None; exile_ref = None; saw_descriptor = False
   for iteration in range(30000):
    o = g.observe("P0"); self.assertIsNone(o["outcome"], (name, player, adventure, stage, iteration))
    o = g.observe(o["actor"]); actor = o["actor"]; objects = {v["id"]: v for v in o["objects"]}
    mine = [v for v in objects.values() if v["owner"] == player and v["zone"] == "Battlefield"]
    theirs = [v for v in objects.values() if v["owner"] == enemy and v["zone"] == "Battlefield"]
    ally = next((v for v in mine if v["card"] == "Bear Cub"), None)
    target = next((v for v in theirs if v["card"] == ("Candy Trail" if i == 15 else "Bear Cub")), None)
    lands = [v for v in mine if v["card"] in BASICS and not v["tapped"]]
    quiet = o["decision"] == "priority" and not o["stack"] and not o["announcing"] and not o["resolving"]
    if stage == "inset_effect" and quiet:
     self.assertTrue(saw_descriptor)
     self.assertEqual(objects[source]["zone"], "Exile")
     permission = next(p for p in o["adventures"] if p["object"]["card"] == source)
     self.assertEqual(permission["player"], player); self.assertEqual(permission["card"], name)
     exile_ref = permission["object"]; self.assertEqual(exile_ref, objects[source]["reference"])
     self.assertEqual(o["adventures"], g.observe(enemy)["adventures"])
     self.denied(g, o, {"kind": "AnnounceAdventure", "card": source})
     self.denied(g, o, {"kind": "AnnounceCast", "card": source})
     if i == 0: self.assertEqual(o["players"][pi]["library_count"], base["library"])
     if i == 1: self.assertEqual(target["damage"], base["damage"] + 1)
     if i in [2,16]: self.assertEqual(o["players"][pi]["library_count"], base["library"] - (4 if i == 2 else 2))
     if i in [3,14,17]: self.assertEqual(objects[base["target"]]["zone"], "Graveyard")
     if i == 4: self.assertEqual(target["damage"], base["damage"]+1)
     if i == 5:
      self.assertEqual(ally["counters"]["plus_one"], base["counter"]+(0 if skip else 1))
      self.assertEqual(o["players"][pi]["life"], base["life"]+2)
     if i in [6,7,8,12]:
      subtype = {6:"Treasure",7:"Food",8:"Knight",12:"Rat"}[i]
      tokens = [v for v in mine if v.get("token") and v["token"]["subtypes"] == subtype]
      self.assertEqual(len(tokens), 2 if i in [8,12] else 1)
      for token in tokens:
       self.assertEqual(token["token"]["colors"], {6:0,7:0,8:1,12:4}[i])
       if i in [8,12]: self.assertEqual((token["power"],token["toughness"]), (2,2) if i==8 else (1,1))
     if i == 9: self.assertEqual(o["players"][pi]["library_count"],base["library"]-(0 if skip else 2))
     if i == 10: self.assertEqual((ally["power"], ally["toughness"]), (base["power"]+2,base["toughness"]+1))
     if i == 11: self.assertEqual(target["power"], base["enemy_power"]-4)
     if i == 13: self.assertEqual((target["power"],target["toughness"]),(base["enemy_power"]-1,1))
     if i == 15: self.assertEqual(objects[base["target"]]["zone"], "Graveyard")
     if i == 18: self.assertTrue(ally["keywords"] & 1024)
     stage = "normal_from_exile"
    if stage == "normal_effect" and quiet:
     body = objects[source]; self.assertEqual(body["zone"], "Battlefield")
     self.assertEqual(body["card"], name)
     front = FACTS[name]["card_faces"][0]
     if i != 9:
      self.assertEqual((body["power"], body["toughness"]), (int(front["power"])+(1 if i==8 else 0), int(front["toughness"])))
      for keyword,bit in [("Flying",1),("Reach",2),("Trample",256),("Menace",32768)]:
       if front["oracle_text"].startswith(keyword): self.assertTrue(body["keywords"] & bit)
     self.assertFalse(any(v["object"]["card"] == source for v in o["adventures"]))
     if exile_ref: self.assertEqual(body["reference"]["generation"],exile_ref["generation"]+(1 if i==9 else 2))
     if i == 0: self.assertEqual(o["players"][pi]["library_count"], base["library"]-1)
     if i == 3: self.assertTrue(any(v.get("token") and v["token"]["subtypes"] == "Food" for v in mine))
     if i == 8:
      self.assertEqual(body["power"],3); self.assertTrue(body["keywords"] & 4)
      self.assertEqual(ally["power"],base["power"]+1)
     if i == 9: self.assertTrue(body["tapped"])
     if i == 13: self.assertEqual(ally["power"],base["power"]+2)
     return
    ready = len(lands)>=8 and ally and target and (not adventure or i!=17 or target["tapped"])
    for descriptor in o["spell_characteristics"]:
     if source is not None and descriptor["object"]["card"] == source:
      face = FACTS[name]["card_faces"][1 if stage == "inset_effect" else 0]
      self.assertEqual(descriptor["physical_card"],name); self.assertEqual(descriptor["name"],face["name"])
      self.assertEqual(descriptor["type_line"],face["type_line"]); self.assertEqual(descriptor["mana_cost"],face["mana_cost"])
      if stage == "inset_effect":
       saw_descriptor = True; self.assertEqual(descriptor["face"],"Adventure")
       item = o["announcing"] or next((x for x in o["stack"] if x["card"]==source),None) or o["resolving"]
       self.assertEqual(item["source_card"],face["name"]); self.assertEqual(item["cast"]["method"],"Normal")
       self.assertEqual(item["cast"]["face"],"Adventure"); self.assertEqual(item["cast"]["physical_card"],name)
    def score(a):
     k=a["kind"]
     if k in ["Cast","BeginCast","AnnounceFlashback","AnnounceWarp","AnnouncePlot","AnnounceActivate","Activate","Concede"]: return -2000
     if k == "AnnounceAdventure": return 2500 if actor==player and adventure and stage=="setup" and objects[a["card"]]["card"]==name and ready else -1000
     if k == "AnnounceExile": return 2500 if actor==player and stage=="normal_from_exile" and a["card"]==source else -1000
     if k == "AnnounceCast":
      n=objects[a["card"]]["card"]
      if actor==player:
       if n==name: return 2500 if not adventure and stage=="setup" and ready else -1000
       return 2000 if n=="Bear Cub" and not ally else -1000
      return 2000 if n=="Bear Cub" and not any(v["card"]==n for v in theirs) or n=="Candy Trail" and i==15 and not target else -1000
     if k == "PlayLand":
      n=objects[a["card"]]["card"]
      if n==name: return 2600 if actor==player and (stage=="normal_from_exile" and a["card"]==source or not adventure and stage=="setup" and ready) else -1000
      count=sum(v["owner"]==actor and v["zone"]=="Battlefield" and v["card"]==n for v in objects.values())
      return 1800-count*20
     if k == "ChooseTarget":
      t=a["target"]
      if "Player" in t: return (2600 if t["Player"]==player else 2000) if i==5 else 200
      if skip and i==5 and stage=="inset_effect": return -1000
      desired=ally if i in [5,10,18] or stage=="normal_effect" else target
      return 2600 if desired and t.get("Card")==desired["id"] else 500
     if k == "Attack": return 2800 if actor==enemy and adventure and i==17 and stage=="setup" and len(lands)>=7 and a["card"]==target["id"] else -1000
     if k in ["TapMana","ActivateMana","AnnounceMana"]: return 2100 if o["decision"] in ["pay_costs","resolution_mana"] and objects[a["card"]]["card"] in BASICS else -1000
     if k in ["PayCosts","PutTrigger","FinishTargets","FinishAttackers","FinishBlockers","Keep","ChooseSpellCost"]: return 2400
     if k in ["Discard","Bottom"]: return -100 if objects[a["card"]]["card"]==name else 2200
     if k == "DeclineEffectPayment": return 2700 if skip else -1000
     if k == "Sacrifice": return -1000 if skip else 2400 if a.get("object",{}).get("card")==ally["id"] else 1000
     if k == "ChooseOptional": return 2500 if a["accept"] != skip else 0
     if k == "Pass": return 0
     return 1000
    j=max(range(len(o["actions"])),key=lambda x:score(o["actions"][x])); a=o["actions"][j]; ann=o["announcing"]
    if a["kind"] in ["AnnounceAdventure","AnnounceExile","AnnounceCast","PlayLand"] and actor==player and objects[a["card"]]["card"]==name:
     source=a["card"]; stage="inset_effect" if a["kind"]=="AnnounceAdventure" else "normal_effect"
     self.denied(g,dict(o,actor=enemy),a)
     if a["kind"]=="PlayLand": base={"library":o["players"][pi]["library_count"],"power":ally["power"]}
    if a["kind"]=="PayCosts" and ann and ann["kind"]=="Spell" and ann["card"]==source:
     base={"library":o["players"][pi]["library_count"],"life":o["players"][pi]["life"],"counter":ally["counters"]["plus_one"],"power":ally["power"],"toughness":ally["toughness"],"enemy_power":target["power"] if target else 0,"damage":target["damage"] if target else 0,"target":target["id"] if target else None}
    self.replay(g,o,j)
   self.fail(("no completed Adventure scenario",name,player,adventure,stage))
