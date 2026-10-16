"""Collect-evidence complete bodies, exact cost grammar and honest admission gates."""
import importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('evidence_gen',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
NAMES=['Analyze the Pollen','Axebane Ferox','Bite Down on Crime','Crimestopper Sprite','Evidence Examiner','Extract a Confession','Forensic Researcher','Sample Collector','Surveillance Monitor','Vitu-Ghazi Inspector']
class EvidenceCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text());cls.cards={c['name']:c for c in cls.snapshot['cards']}
 def test_ten_whole_bodies_and_unknown_tails(self):
  for name in NAMES:
   c,e=g.extract_evidence(self.cards[name]);self.assertIsNotNone(g.fixed_evidence_body(c,e),name)
   for suffix in ['\nDiscover 3.','\nGain control of target permanent.','\nCreate a token copy of this creature.']:
    self.assertIsNone(g.fixed_evidence_body(dict(c,oracle_text=c['oracle_text']+suffix),e),name)
 def test_optional_declaration_is_exact_unique_and_thresholds_agree(self):
  c=self.cards['Bite Down on Crime'];line,body=c['oracle_text'].split('\n',1)
  for text in [line+'\n'+line+'\n'+body,line.replace('To collect evidence 6','To collect evidence 5')+'\n'+body,line.replace('mana value 6','mana value 7')+'\n'+body,line.replace('you may','you must')+'\n'+body]:
   self.assertIsNone(g.extract_evidence(dict(c,oracle_text=text))[1])
  for value in [0,5,7,65536]:
   p,e=g.extract_evidence(dict(c,oracle_text=c['oracle_text'].replace('evidence 6','evidence '+str(value)).replace('value 6','value '+str(value))))
   if e is not None:self.assertIsNone(g.fixed_evidence_body(p,e))
 def test_activated_evidence_cost_cannot_hide_composite_payments_or_grave_mana(self):
  self.assertIsNotNone(g.activation_cost('{T}, Collect evidence 3'))
  for text in ['{T}, Collect evidence 3, Collect evidence 3','{T}, Collect evidence 3, Sacrifice this creature','Exile this card from your graveyard, Collect evidence 3','{T}, Collect evidence 3, Discard a card','{T}, Collect evidence X']:
   self.assertIsNone(g.activation_cost(text),text)
 def test_fixed_spell_bodies_require_valid_bounded_costs(self):
  for name in ['Analyze the Pollen','Bite Down on Crime','Extract a Confession']:
   original=self.cards[name];c,e=g.extract_evidence(original);self.assertIsNotNone(g.fixed_evidence_body(c,e))
   for cost in ['{Y}{G}','{W/P}','{4294967296}{G}','{X}{G}','{0/X}','{U/2}']:
    self.assertIsNone(g.fixed_evidence_body(dict(c,mana_cost=cost),e),(name,cost))
   for cost in ['{G}','{0}','{2}{G}']:
    self.assertIsNotNone(g.fixed_evidence_body(dict(c,mana_cost=cost),e),(name,cost))
 def test_whole_catalogue_preserves_gates_and_single_search_replacement(self):
  c,e=g.extract_evidence(self.cards['Analyze the Pollen']);program=g.fixed_evidence_body(c,e)[3][1];self.assertEqual(len(program),4);self.assertEqual(program[0],'IfEvidence { skip: 2 }');self.assertEqual(program[2],'Skip(1)');self.assertTrue(program[1].startswith('Search('));self.assertTrue(program[3].startswith('Search('))
  _,coverage=g.compile_catalogue(self.snapshot,json.loads((g.ROOT/'data/targets/standard-2026-10-15.json').read_text()));supported={r['name'] for r in coverage['cards'] if r['implemented']};self.assertTrue(set(NAMES)<=supported)
  for name in ['Cryptex','Conspiracy Unraveler','Deadly Cover-Up','Behind the Mask','Hedge Whisperer','Tenth District Hero','Lamplight Phoenix','Izoni, Center of the Web','Incinerator of the Guilty']:
   self.assertNotIn(name,supported)
if __name__=='__main__':unittest.main()
