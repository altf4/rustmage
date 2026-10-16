"""Closed complete-body bargain compilation and honest admission boundaries."""
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('bargain_generate',Path(__file__).with_name('generate_catalogue.py'))
g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
class BargainCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text());cls.facts={c['name']:c for c in cls.snapshot['cards']};cls.manifest=json.loads((g.ROOT/'docs/bargain-admissions.json').read_text())
 def test_complete_ten_and_mutated_residuals_gate(self):
  self.assertEqual(len(self.manifest['cards']),10)
  for row in self.manifest['cards']:
   card=self.facts[row['name']];self.assertEqual(card['oracle_text'],row['oracle_text']);body,found=g.extract_bargain(card);self.assertTrue(found);self.assertIsNotNone(g.fixed_bargain_body(body))
   for suffix in ['\nDiscover 3.','\nCreate a token copy of this creature.','\nGain control of target permanent.']:
    self.assertIsNone(g.fixed_bargain_body(dict(body,oracle_text=body['oracle_text']+suffix)))
 def test_exact_once_only_declaration(self):
  c=self.facts['Candy Grapple'];line,body=c['oracle_text'].split('\n',1)
  for text in [line+'\n'+line+'\n'+body,line.replace('artifact, enchantment, or token','creature')+'\n'+body,'Bargain\n'+body,line.replace('You may','You must')+'\n'+body]:
   self.assertFalse(g.extract_bargain(dict(c,oracle_text=text))[1])
 def test_replacements_are_one_operation_and_one_window(self):
  for name in ['Candy Grapple',"Kellan's Lightblades",'Farsight Ritual']:
   body,_=g.extract_bargain(self.facts[name]);program=g.fixed_bargain_body(body)[3][1]
   self.assertEqual(len(program),4);self.assertEqual(program[0],'IfBargained { skip: 2 }');self.assertEqual(program[2],'Skip(1)')
  p=g.fixed_bargain_body(g.extract_bargain(self.facts['Farsight Ritual'])[0])[3][1]
  self.assertIn('look: 8',p[1]);self.assertIn('look: 4',p[3]);self.assertIn('Fixed(2)',p[1]);self.assertIn('BottomRandom',p[1])
 def test_admissions_and_unimplemented_residuals(self):
  source,coverage=g.compile_catalogue(self.snapshot,json.loads((g.ROOT/'data/targets/standard-2026-10-15.json').read_text()))
  names={r['name'] for r in self.manifest['cards']};supported={r['name'] for r in coverage['cards'] if r['implemented']}
  self.assertTrue(names<=supported);self.assertFalse({r['name'] for r in self.manifest['gated_cards']}&supported)
  self.assertIn('pub(super) fn bargain_reduction',source);self.assertIn('artifact_enchantment_or_token: false',source)
if __name__=='__main__':unittest.main()
