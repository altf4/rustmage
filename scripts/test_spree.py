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
if __name__=='__main__':unittest.main()
