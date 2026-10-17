"""Closed complete Vehicle bodies preserve printed identity and cost semantics."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('crew_generator',Path(__file__).with_name('generate_catalogue.py'))
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=["Adventurer's Airship","Air Response Unit","Boosted Sloop","Broadcast Rambler","Burner Rocket","Careening Mine Cart","Clamorous Ironclad","Cultivator's Caravan","Debris Beetle","Dependable Quinjet","Fire Nation Warship","Haunted Hellride","Hulldrifter","Lumbering Worldwagon","Magitek Armor","Ripclaw Wrangler","S.H.I.E.L.D. Helicarrier","Skybox Ferry","Spotcycle Scouter","Strixhaven Skycoach","Thopter Fabricator","Tundra Tank","Turtle Blimp","Veloheart Bike"]
class CrewCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def implemented(self,c):
  cards=[c];snapshot=dict(cards=cards,card_count=1,cards_sha256=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest())
  rust,report=g.compile_catalogue(snapshot,dict(banned_card_names=[]))
  return next(x for x in report['cards'] if x['name']==c['name'])['implemented']
 def test_all_complete_bodies_with_printed_artifact_identity(self):
  for name in NAMES:
   with self.subTest(name=name):
    self.assertTrue(self.implemented(self.cards[name]))
    self.assertEqual(g.permanent_type(self.cards[name]),'Artifact')
  for name in ['Cargo Ship','Midnight Mangler','Luxurious Locomotive','Subterranean Schooner','Turtle Van','Thunderous Velocipede','Unidentified Hovership']:
   self.assertFalse(self.implemented(self.cards[name]),name)
 def test_declaration_and_body_are_closed(self):
  base=self.cards['Air Response Unit'];line=next(x for x in base['oracle_text'].splitlines() if x.startswith('Crew '))
  for bad in ['Crew X','Crew -1','Crew 65536','Crew 1\nCrew 2','Crew 1 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)','Crew 1 Activate only once each turn.']:
   self.assertFalse(self.implemented(dict(base,oracle_text=base['oracle_text'].replace(line,bad))),bad)
  for n in [0,1,65535]:
   self.assertTrue(self.implemented(dict(base,oracle_text=base['oracle_text'].replace(line,f'Crew {n}'))))
  for name in NAMES:
   self.assertFalse(self.implemented(dict(self.cards[name],oracle_text=self.cards[name]['oracle_text']+'\nDiscover 3.')),name)
  world=self.cards['Lumbering Worldwagon']
  self.assertFalse(self.implemented(dict(world,oracle_text='Crew 4')))
 def test_other_mechanics_and_external_vehicle_nouns_are_not_erased(self):
  base=self.cards['Air Response Unit']
  clauses=[self.cards['Candy Grapple']['oracle_text'].splitlines()[0],self.cards['Appeal to Eirdu']['oracle_text'].splitlines()[0],self.cards['Slickshot Show-Off']['oracle_text'].splitlines()[-1], 'Kicker {1}','Offspring {1}','Flashback {1}','Mobilize 1','Saddle 1','Whenever another Vehicle attacks, draw a card.','Vehicles you control get +1/+1.']
  for clause in clauses:
   self.assertFalse(self.implemented(dict(base,oracle_text=base['oracle_text']+'\n'+clause)),clause)
 def test_cost_bounds_and_full_cycling(self):
  base=self.cards['Air Response Unit']
  for cost in ['{X}','{Q}','{4294967296}','{4294967295}{1}','{2/Z}','{W/P}','','{W}junk']:
   self.assertFalse(self.implemented(dict(base,mana_cost=cost)),cost)
  self.assertTrue(self.implemented(dict(base,mana_cost='{4294967295}{W}')))
  for name in ['Clamorous Ironclad','Skybox Ferry']:
   self.assertTrue(self.implemented(self.cards[name]))
   self.assertFalse(self.implemented(dict(self.cards[name],oracle_text=self.cards[name]['oracle_text']+'\nWhen you cycle this card, discover 3.')))
if __name__=='__main__': unittest.main()
