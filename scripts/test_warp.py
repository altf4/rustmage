"""Closed whole Warp bodies, bounded alternatives and complete shared residuals."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('warp_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Bygone Colossus','Eusocial Engineering','Germinating Wurm','Knight Luminary','Red Tiger Mechan','Sinister Cryologist','Starbreach Whale','Weftblade Enhancer','Drix Fatemaker','Mechanozoa','Memorial Team Leader','Nova Hellkite']
class WarpCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.rows={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  a=[c];s=dict(cards=a,card_count=1,cards_sha256=hashlib.sha256(json.dumps(a,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest());return g.compile_catalogue(s,dict(banned_card_names=[]))
 def admitted(self,c):return next(v['implemented'] for v in self.compile(c)[1]['cards'] if v['name']==c['name'])
 def test_twelve_warp_and_two_collateral_bodies_with_complete_residuals(self):
  for n in NAMES+['Runaway Boulder','Waylaying Pirates']:
   with self.subTest(name=n):self.assertTrue(self.admitted(self.rows[n]));self.assertFalse(self.admitted(dict(self.rows[n],oracle_text=self.rows[n]['oracle_text']+'\nDiscover 3.')))
  gated=[c for c in self.rows.values() if 'Warp' in c.get('keywords',[]) and c['name']not in NAMES];self.assertEqual(len(gated),20)
  for c in gated:self.assertFalse(self.admitted(c),c['name'])
 def test_warp_costs_full_declaration_and_later_selectors_stay_closed(self):
  extra=[next(l for l in self.rows['Appeal to Eirdu']['oracle_text'].splitlines() if l.startswith('Convoke ')),self.rows['Candy Grapple']['oracle_text'].splitlines()[0],next(l for l in self.rows["Djinn of Fool's Fall"]['oracle_text'].splitlines() if l.startswith('Plot ')),'Kicker {1}','Offspring {1}','Flashback {1}','Cycling {1}','Saddle 1','Mobilize 1','When this creature enters, it endures 1.','As an additional cost to cast this spell, collect evidence 6.']
  count=0
  for n in NAMES:
   c=self.rows[n];line=next(l for l in c['oracle_text'].splitlines() if l.startswith('Warp '));cost=line.split(' ',2)[1]
   for bad in ['{X}','{Y}','{W/P}','{2/U}','{W/U}','{4294967296}','{4294967295}{1}']:
    self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace(line,line.replace(cost,bad)))),(n,bad));count+=1
   for addition in extra:
    mutant=dict(c,oracle_text=c['oracle_text']+'\n'+addition)
    if addition.startswith('Convoke '):self.assertTrue(g.extract_convoke(mutant)[1])
    if addition.startswith('Bargain '):self.assertTrue(g.extract_bargain(mutant)[1])
    if addition.startswith('Plot '):self.assertIsNotNone(g.extract_plot(mutant)[1])
    self.assertFalse(self.admitted(mutant),(n,addition));count+=1
   for mana in ['{X}{G}','{Q}','{W/P}','{4294967296}','{G}junk']:
    self.assertFalse(self.admitted(dict(c,mana_cost=mana)),(n,mana));count+=1
   self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text']+'\n'+line)));count+=1
  self.assertEqual(count,288)
 def test_warp_fingerprint_and_fixed_bounds_are_executable_metadata(self):
  c=self.rows['Bygone Colossus'];changed=dict(c,oracle_text=c['oracle_text'].replace('Warp {3}','Warp {4}'));a,ar=self.compile(c);b,br=self.compile(changed);self.assertTrue(self.admitted(changed));self.assertNotEqual(ar['catalogue_version'],br['catalogue_version']);self.assertNotEqual(a,b)
  self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace('for its warp cost','for free'))));self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace('on a later turn','this turn'))))
 def test_shared_residuals_keep_turn_recipient_and_copy_boundaries(self):
  for n,old,new in [('Memorial Team Leader','During your turn','During each turn'),('Drix Fatemaker','+1/+1 counter on it has trample','-1/-1 counter on it has trample'),('Mechanozoa','artifact or creature an opponent controls','artifact or land an opponent controls')]:
   self.assertFalse(self.admitted(dict(self.rows[n],oracle_text=self.rows[n]['oracle_text'].replace(old,new))))
  c=self.rows['Drix Fatemaker'];self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text']+'\nOffspring {1}')))
  for n in NAMES:
   for layout in ['transform','modal_dfc','adventure']:self.assertFalse(self.admitted(dict(self.rows[n],layout=layout)))
