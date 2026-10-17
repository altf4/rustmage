"""Whole-body Plot admission, bounded costs and closed selector composition."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('plot_generate', Path(__file__).with_name('generate_catalogue.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)
NAMES = ["Beastbond Outcaster", "Blacksnag Buzzard", "Cunning Coyote", "Djinn of Fool's Fall", "Irascible Wolverine", "Loan Shark", "Slickshot Show-Off", "Spinewoods Paladin", "Stagecoach Security", "Stingerback Terror", "Demonic Ruckus", "Rictus Robber", "Rise of the Varmints"]
class PlotCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text())
  cls.rows={c['name']:c for c in cls.snapshot['cards']}
 def compile(self,card):
  cards=[card]
  snapshot=dict(cards=cards,card_count=1,cards_sha256=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest())
  return g.compile_catalogue(snapshot,dict(banned_card_names=[]))
 def admitted(self,card):return next(c for c in self.compile(card)[1]['cards'] if c['name']==card['name'])['implemented']
 def test_all_thirteen_and_whole_residual_boundary(self):
  for name in NAMES:
   with self.subTest(name=name):
    self.assertTrue(self.admitted(self.rows[name]))
    self.assertFalse(self.admitted(dict(self.rows[name],oracle_text=self.rows[name]['oracle_text']+'\nDiscover 3.')))
  free = self.rows['Freestrider Commando']
  self.assertTrue(self.admitted(free))
  self.assertFalse(self.admitted(dict(free, oracle_text=free['oracle_text']+'\nDiscover 3.')))
  for card in self.snapshot['cards']:
   if any(l.startswith('Plot ') for l in card.get('oracle_text','').splitlines()) and card['name'] not in NAMES+['Freestrider Commando']:
    self.assertFalse(self.admitted(card),card['name'])
 def test_costs_reminder_duplicates_and_selector_revival(self):
  base=self.rows["Djinn of Fool's Fall"]
  for cost in ['{X}','{Q}','{4294967296}','{4294967295}{1}','{W/P}','{2/U}']:
   text=base['oracle_text'].replace('{3}{U}',cost)
   self.assertFalse(self.admitted(dict(base,oracle_text=text)),cost)
  line=next(l for l in base['oracle_text'].splitlines() if l.startswith('Plot '))
  for text in [base['oracle_text']+'\n'+line,base['oracle_text'].replace('Plot only as a sorcery.','Plot any time.'),base['oracle_text'].replace('You may pay {3}{U}','You may pay {2}{U}')]:
   self.assertFalse(self.admitted(dict(base,oracle_text=text)))
  for declaration in [next(l for l in self.rows['Appeal to Eirdu']['oracle_text'].splitlines() if l.startswith('Convoke ')),next(l for l in self.rows['Candy Grapple']['oracle_text'].splitlines() if l.startswith('Bargain ')),'Kicker {1}','Offspring {1}','Flashback {1}','Cycling {1}','Mobilize 1','Saddle 1']:
   mutant=dict(base,oracle_text=base['oracle_text']+'\n'+declaration)
   if declaration.startswith('Convoke '):self.assertTrue(g.extract_convoke(mutant)[1])
   if declaration.startswith('Bargain '):self.assertTrue(g.extract_bargain(mutant)[1])
   self.assertFalse(self.admitted(mutant),declaration)
  for cost in ['{X}{U}','{Q}','{4294967296}','{4294967295}{1}','{W}junk']:
   self.assertFalse(self.admitted(dict(base,mana_cost=cost)),cost)
  self.assertTrue(self.admitted(dict(base,mana_cost='{4294967295}{U}')))
 def test_cost_metadata_changes_catalogue_fingerprint(self):
  base=self.rows["Djinn of Fool's Fall"]
  changed=dict(base,oracle_text=base['oracle_text'].replace('{3}{U}','{2}{U}'))
  a,ar=self.compile(base);b,br=self.compile(changed)
  self.assertTrue(self.admitted(changed))
  self.assertNotEqual(ar['catalogue_version'],br['catalogue_version'])
  self.assertNotEqual(a,b)
