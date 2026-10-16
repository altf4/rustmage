"""Closed celebration history predicates and complete ten-body admissions."""
import importlib.util,json,hashlib
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('celebration_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
class CelebrationCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text());cls.target=json.loads((g.ROOT/'data/targets/standard-2026-10-15.json').read_text());cls.manifest=json.loads((g.ROOT/'docs/celebration-admissions.json').read_text());cls.facts={r['name']:r for r in cls.snapshot['cards']}
 def compile(self,cards):
  one=dict(cards=cards,card_count=len(cards));one['cards_sha256']=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();return g.compile_catalogue(one,dict(banned_card_names=[]))
 def test_all_ten_complete_bodies_and_unknown_residuals(self):
  self.assertEqual(len(self.manifest['cards']),10)
  names={r['name'] for r in self.manifest['cards']};source,coverage=g.compile_catalogue(self.snapshot,self.target);supported={r['name'] for r in coverage['cards'] if r['implemented']};self.assertTrue(names<=supported);self.assertNotIn('Goddric, Cloaked Reveler',supported)
  for row in self.manifest['cards']:self.assertEqual(row['oracle_text'],self.facts[row['name']]['oracle_text'])
  for suffix in ['\nDiscover 3.','\nWhen this creature dies, copy it.','\nYou may play an additional land on each of your turns.']:
   cards=[dict(self.facts[n],oracle_text=self.facts[n]['oracle_text']+suffix) for n in names];_,report=self.compile(cards);self.assertFalse(any(r['implemented'] for r in report['cards'] if r['name'] in names))
 def test_history_condition_is_exact_and_typed_everywhere(self):
  condition='two or more nonland permanents entered the battlefield under your control this turn'
  self.assertIn('NonlandEntriesThisTurn',g.stat_threshold_condition(condition));self.assertIn('minimum: 2',g.stat_threshold_condition(condition));self.assertIn('minimum: Some(2)',g.step_guard(condition))
  for bad in [condition.replace('nonland','nontoken'),condition.replace('your control','any control'),condition.replace('this turn','last turn'),condition+' or you gained life this turn']:
   self.assertIsNone(g.stat_threshold_condition(bad));self.assertIsNone(g.step_guard(bad))
 def test_optional_attachment_and_whole_equip_and_reduction(self):
  self.assertEqual(g.targeted_program('attach this Equipment to up to one target creature you control.'),(('CreatureYou',0,1),['AttachSource']))
  for bad in ['attach this Equipment to up to two target creatures you control.','attach this Equipment to up to one target creature.','attach this Equipment to up to one target creature you control. Draw a card if you control a Dragon.']:
   self.assertIsNone(g.targeted_program(bad))
  eq=g.equipment_template(self.facts['Bespoke Battlegarb']);self.assertEqual(eq[-3:],(2,0,0));self.assertEqual(len(eq[3]),1);self.assertIn('min: 0',g.rust_targets(eq[2][0][2]))
  body,reductions=g.extract_cost_reductions(self.facts['Raging Battle Mouse']);self.assertEqual(len(reductions),1);self.assertIn('StepValue::SpellsCast',reductions[0]);self.assertIn('minimum: Some(1), maximum: Some(1)',reductions[0]);self.assertIsNotNone(g.trigger_creature_template(body))
 def test_printed_costs_and_scalar_composites_stay_closed(self):
  names={r['name'] for r in self.manifest['cards']}
  for cost in ['{X}{R}','{Y}{W}','{W/P}','{4294967296}{R}','{4294967295}{1}{R}']:
   _,report=self.compile([dict(self.facts[n],mana_cost=cost) for n in names]);self.assertFalse(any(r['implemented'] for r in report['cards'] if r['name'] in names))
if __name__=='__main__':unittest.main()
