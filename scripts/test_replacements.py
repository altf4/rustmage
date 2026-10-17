"""Closed complete token/counter replacement bodies and residual grammar."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('replacement_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Doubling Season','Exalted Sunborn','Loading Zone','Michelangelo, Weirdness to 11','Yoshimaru, Beloved Companion','Caradora, Heart of Alacria','Quina, Qu Gourmet','Draconic Visitor','Bartolomé del Presidio']
class ReplacementCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.rows={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile(self,c):
  a=[c];s=dict(cards=a,card_count=1,cards_sha256=hashlib.sha256(json.dumps(a,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest());return g.compile_catalogue(s,dict(banned_card_names=[]))
 def admitted(self,c):return next(v['implemented'] for v in self.compile(c)[1]['cards'] if v['name']==c['name'])
 def test_nine_complete_bodies_and_immutable_replacement_tables(self):
  for n in NAMES:
   with self.subTest(name=n):
    rust,report=self.compile(self.rows[n]);self.assertTrue(self.admitted(self.rows[n]));self.assertIn('fn replacement_effects',rust)
    self.assertFalse(self.admitted(dict(self.rows[n],oracle_text=self.rows[n]['oracle_text']+'\nDiscover 3.')))
  self.assertEqual(g.extract_replacements(self.rows['Doubling Season'])[1],['ReplacementSpec::DoubleTokens { effects_only: true }','ReplacementSpec::Counters { filter: ReplacementFilter::Permanent, kind: None, factor: 2, add: 0, effects_only: true }'])
 def test_printed_cost_layout_and_unknown_text_fail_closed_for_every_body(self):
  for n in NAMES:
   for cost in ['{Y}{G}','{W/P}','{4294967296}{G}','{4294967295}{1}','{X}{G}','{2/Z}','{G}junk','']:
    with self.subTest(name=n,cost=cost):self.assertFalse(self.admitted(dict(self.rows[n],mana_cost=cost)))
   self.assertFalse(self.admitted(dict(self.rows[n],layout='transform')),n)
   with self.assertRaises((TypeError,ValueError)):self.admitted(dict(self.rows[n],oracle_text=None))
   self.assertTrue(self.admitted(dict(self.rows[n],mana_cost='{4294967295}{G}')),n)
 def test_replacement_recipient_quantities_conditions_and_optional_clauses_are_not_stripped(self):
  for n in NAMES[:-1]:
   c=self.rows[n];line=next(v for v in c['oracle_text'].splitlines() if v.startswith('If '))
   mutations=[line.replace('you control','an opponent controls'),line.replace('one or more','exactly one'),line.replace('instead.','instead. You gain 1 life.'),line.replace('If ','You may. If '),line.replace('tokens','copies')]
   for m in set(mutations)-{line}:
    with self.subTest(name=n,line=m):self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace(line,m))))
 def test_unknown_composite_cost_and_timing_selectors_remain_gated(self):
  clauses=[self.rows['Analyze the Pollen']['oracle_text'].splitlines()[0],self.rows['Candy Grapple']['oracle_text'].splitlines()[0],'Kicker {1}','Offspring {1}','Convoke','Cycling {1}','Flashback {1}','Saddle 1','Mobilize 1','When this creature enters, it endures 1.']
  for n in NAMES[:-1]:
   for clause in clauses:
    with self.subTest(name=n,clause=clause):self.assertFalse(self.admitted(dict(self.rows[n],oracle_text=clause+'\n'+self.rows[n]['oracle_text'])))
 def test_typed_residual_filters_aliases_and_complete_sacrifice_body(self):
  self.assertIn('subtype: Some("Frog")',g.sacrifice_spec('a Frog')[1]);self.assertIn('kinds: 0',g.sacrifice_spec('a Frog')[1])
  self.assertEqual(g.search_filter('Mount or Vehicle card'),'AnyOf(&[SearchFilter::Subtype("Mount"), SearchFilter::Subtype("Vehicle")])')
  rust,_=self.compile(self.rows['Yoshimaru, Beloved Companion']);self.assertIn('TargetPredicate::Legendary',rust)
  for n in ['Quina, Qu Gourmet','Bartolomé del Presidio']:
   c=self.rows[n];alias=n.split(',')[0] if ',' in n else n
   self.assertTrue(self.admitted(dict(c,name='Audited Permanent',oracle_text=c['oracle_text'].replace(alias,'Audited Permanent'))))
   self.assertFalse(self.admitted(dict(c,oracle_text=c['oracle_text'].replace('counter on ','counter on another '))))
 def test_generic_closure_keeps_more_complex_replacement_neighbors_whole_and_gated(self):
  for n in ['Elspeth, Storm Slayer','Innkeeper’s Talent','Doc Samson','Bard Class','Twitching Doll']:
   if n in self.rows:
    with self.subTest(name=n):self.assertFalse(self.admitted(self.rows[n]))
if __name__=='__main__':unittest.main()
