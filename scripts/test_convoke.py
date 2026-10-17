"""Whole fixed Convoke bodies, exact target grammar, and reciprocal cost gates."""
import hashlib,importlib.util,json,re,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('convoke_generator',Path(__file__).with_name('generate_catalogue.py'));g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
class ConvokeCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.snapshot=json.loads((g.ROOT/'data/standard/cards.json').read_text());cls.facts={c['name']:c for c in cls.snapshot['cards']};cls.manifest=json.loads((g.ROOT/'docs/convoke-admissions.json').read_text())
 def compile(self,cards):
  s=dict(cards=cards,card_count=len(cards));s['cards_sha256']=hashlib.sha256(json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();return g.compile_catalogue(s,dict(banned_card_names=[]))
 def test_nine_intrinsic_and_adjacent_full_equipment_preserve_every_residual(self):
  self.assertEqual(len(self.manifest['cards']),9);self.assertEqual([r['name'] for r in self.manifest['adjacent']],['Bespoke Bō'])
  names=[r['name'] for r in self.manifest['cards']+self.manifest['adjacent']];source,report=self.compile([self.facts[n] for n in names]);implemented={r['name'] for r in report['cards'] if r['implemented']};self.assertTrue(set(names)<=implemented)
  self.assertEqual(source.count('pub(super) fn has_convoke'),1);body=source.split('fn has_convoke(card: Card) -> bool {')[1].split('}')[0];self.assertEqual(len(re.findall(r'\b[0-9]+\b',body))-1,9)
  for row in self.manifest['cards']+self.manifest['adjacent']:
   self.assertEqual(row['oracle_text'],self.facts[row['name']]['oracle_text']);self.assertEqual(row['mana_cost'],self.facts[row['name']]['mana_cost'])
  for suffix in ['\nDiscover 3.','\nWhen this permanent leaves, copy it.','\nYou may play an additional land on each of your turns.']:
   _,bad=self.compile([dict(self.facts[n],oracle_text=self.facts[n]['oracle_text']+suffix) for n in names]);self.assertFalse(any(c['implemented'] for c in bad['cards'] if c['name'] in names))
 def test_unrestricted_other_and_one_or_two_have_exact_cardinality(self):
  spec,program=g.targeted_program("return up to one other target nonland permanent to its owner's hand.");self.assertEqual(spec,('OtherNonland',0,1));self.assertEqual(program,['ReturnToHand']);self.assertIn('relation: -1',g.rust_targets(spec));self.assertIn('other: true',g.rust_targets(spec))
  self.assertEqual(g.targeted_program('one or two target creatures each get +2/+1 until end of turn.')[0],('Creature',1,2))
  self.assertEqual(g.targeted_program("return up to two other target nonland permanents to their owners' hands."),(('OtherNonland',0,2),['ReturnToHand']))
  for bad in ["return up to three other target nonland permanents to their owners' hands.",'one or three target creatures each get +2/+1 until end of turn.','one or two target creatures each get +2/+1 until your next turn.']:
   self.assertIsNone(g.targeted_program(bad))
 def test_variable_and_composed_costs_never_revive_a_convoke_body(self):
  names=[r['name'] for r in self.manifest['cards']]
  prefixes=['Bargain (You may sacrifice an artifact, enchantment, or token as you cast this spell.)\n','Offspring {1}\n','Kicker {1}\n','As an additional cost to cast this spell, sacrifice a creature.\n','As an additional cost to cast this spell, collect evidence 6.\n']
  cases=[dict(self.facts[n],oracle_text=p+self.facts[n]['oracle_text']) for n in names for p in prefixes]
  cases.extend(dict(self.facts[n],mana_cost=cost) for n in names for cost in ['{X}{W}','{Y}{W}','{W/P}','{4294967296}{W}'])
  for c in cases:
   _,bad=self.compile([c]);self.assertFalse(next(r for r in bad['cards'] if r['name']==c['name'])['implemented'],c['name']+' '+c['oracle_text'])
  declaration=self.facts[names[0]]['oracle_text'].splitlines()[0]+'\n'
  for n in ['Analyze the Pollen','Candy Grapple','Grow from the Ashes','Mind Spring','Alacrian Jaguar']:
   c=self.facts[n];_,bad=self.compile([dict(c,oracle_text=declaration+c['oracle_text'])]);self.assertFalse(next(r for r in bad['cards'] if r['name']==n)['implemented'],n)
 def test_declaration_is_closed_and_unknown_neighbor_bodies_stay_gated(self):
  c=self.facts['Sun-Dappled Celebrant'];declaration=c['oracle_text'].splitlines()[0]
  for text in [c['oracle_text']+'\n'+declaration,c['oracle_text'].replace('of that creature\'s color','of any color'),c['oracle_text'].replace('Each creature','Each permanent')]:
   _,bad=self.compile([dict(c,oracle_text=text)]);self.assertFalse(next(r for r in bad['cards'] if r['name']==c['name'])['implemented'])
  _,bad=self.compile([self.facts[r['name']] for r in self.manifest['gated']]);self.assertFalse(any(c['implemented'] for c in bad['cards'] if c['name'] in {r['name'] for r in self.manifest['gated']}))
if __name__=='__main__':unittest.main()
