"""Whole Spree grammar, immutable subset lowering and raw adversarial controls."""
import copy,json,unittest
from scripts import generate_catalogue as g
class SpreeCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.rows=[c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']if c.get('oracle_text','').startswith('Spree ')]
 def test_all_twenty_one_bodies_remain_inventory_and_only_complete_modes_lower(self):
  self.assertEqual(len(self.rows),21)
  self.assertEqual({c['name']for c in self.rows if g.spree_spell_template(c)}, {'Explosive Derailment','Phantom Interference'})
 def test_every_nonempty_subset_costs_order_and_group_rebasing(self):
  for card in self.rows:
   result=g.spree_spell_template(card)
   if not result:continue
   modes,costs,grants=result;self.assertEqual(len(modes),3);self.assertEqual(len(costs),3);self.assertEqual(grants,[])
   self.assertTrue(modes[2][2][0]=='SelectGroup(0)');self.assertIn('SelectGroup(1)',modes[2][2]);self.assertEqual(len(modes[2][1]),2)
   self.assertEqual(modes[2][2],modes[0][2]+[x.replace('SelectGroup(0)','SelectGroup(1)')for x in modes[1][2]])
   extra=[g.spell_cost_options(line.split(' — ')[0][2:])[0]for line in card['oracle_text'].splitlines()[1:]]
   self.assertIn('generic: '+str(sum(c[1]for c in extra)),costs[2])
 def test_raw_header_modes_costs_and_unknown_residuals_never_erase(self):
  for card in self.rows:
   if not g.spree_spell_template(card):continue
   for name in ['Generic complete spree',*[x[0]for x in g.LEGACY]]:
    clone=dict(card,name=name,oracle_text=card['oracle_text'].replace(card['name'],name));self.assertIsNotNone(g.spree_spell_template(clone))
   raw=card['oracle_text']
   for bad in [raw.replace('one or more','zero or more'),raw.replace('additional costs.','additional costs. You win the game.'),raw+'\nDiscover 4.',raw.replace('+ {','• {',1),raw.replace('Spree','(Spree)')]:
    clone=dict(card,oracle_text=bad);self.assertIsNone(g.spree_spell_template(clone));self.assertFalse(g.spree_routes_reminders_valid(clone))
   for line in raw.splitlines():
    self.assertIsNone(g.spree_spell_template(dict(card,oracle_text=raw.replace(line,line+' (Draw seven cards.)'))))
   for cost in ['{X}','{Q}','{W/P}']:
    self.assertIsNone(g.spree_spell_template(dict(card,oracle_text=raw.replace('+ {', '+ '+cost+'{',1))))
 def test_complete_inventory_facts_and_every_pinned_java_byte(self):
  import hashlib,subprocess
  from pathlib import Path
  audit=json.loads((g.ROOT/'docs/spree-admissions.json').read_text())
  self.assertEqual(audit['java_revision'],'492369c7844b8d76479f0393ff9eb6b0b0cf30d7')
  self.assertEqual(audit['physical_cards'],21);self.assertEqual(audit['unique_source_count'],134)
  facts={c['name']:c for c in self.rows}
  self.assertEqual({c['name']:c['facts']for c in audit['cards']},facts)
  for record in audit['cards']:self.assertEqual(record['unresolved_imports'],[])
  seen=set()
  for record in audit['unique_sources']:
   path=record['path'];self.assertNotIn(path,seen);seen.add(path)
   blob=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',audit['java_revision']+':'+path])
   self.assertEqual(len(blob),record['bytes']);self.assertEqual(hashlib.sha256(blob).hexdigest(),record['sha256'])
   self.assertEqual((Path('/home/altf4/Code/mage')/path).read_bytes(),blob)
 def test_strict_actual_parent_two_complete_bodies_and_unrelated_mutation(self):
  import subprocess
  from scripts.catalogue_preservation import canonical,assert_union
  physical={c['oracle_id']:c['name']for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
  before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show','2812c749b63b38d10572bf96baacd527464852de:src/generated/catalogue.rs'],text=True),physical)
  after=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  names={'Explosive Derailment','Phantom Interference'}
  args=dict(changed={n+'#0'for n in names},added={'@token/2b2fa37913ce33dcd173c4f3#0'},helpers={('tiered_costs',n+'#0')for n in names},aliases={'@token/2b2fa37913ce33dcd173c4f3':'@token/2b2fa37913ce33dcd173c4f3#0'})
  self.assertEqual(set(after['tokens'])-set(before['tokens']),{'@token/2b2fa37913ce33dcd173c4f3'})
  self.assertEqual(len(after['tokens']),97)
  for key,value in before['tokens'].items():self.assertEqual(after['tokens'][key],value)
  self.assertIn('subtypes: \"Spirit\", colors: 1',after['tokens']['@token/2b2fa37913ce33dcd173c4f3'])
  after=copy.deepcopy(after);after['tokens']=copy.deepcopy(before['tokens'])
  proof=assert_union(before,after,**args)
  self.assertEqual(proof['preserved_definitions'],5331);self.assertEqual(proof['prior_helpers'],928);self.assertEqual(proof['token_recipes'],96)
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,**args)
if __name__=='__main__':unittest.main()
