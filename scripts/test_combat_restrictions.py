import copy,hashlib,json,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
NAMES={'Mouser Mark III','Pacifism','Technodrome','The Lion-Turtle','Tiger-Dillo'}
class CombatRestrictionTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name'] in NAMES}
 def compile(self,c):
  payload=json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode();rust,cov=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=hashlib.sha256(payload).hexdigest()),dict(banned_card_names=[]));return rust,next(row['implemented'] for row in cov['cards'] if row['name']==c['name'])
 def normalize(self,rust,name):
  rows=[line for line in rust.splitlines() if line.lstrip().startswith('Definition {')];names=[json.loads(re.search(r'name: ("(?:[^"\\]|\\.)*")',row)[1]) for row in rows];row=next(row for row in rows if 'name: '+json.dumps(name)+',' in row);return re.sub(r'Card\((\d+)\)',lambda m:'Card('+json.dumps(names[int(m[1])])+')',row)
 def test_complete_all_five_and_authenticated_names(self):
  for c in self.cards.values():
   rust,ok=self.compile(c);self.assertTrue(ok,c['name']);original_rule=g.extract_combat_restriction(c)[1];self.assertIsNotNone(original_rule)
   for name in ['Reserved Combat Fixture','Lightning Bolt','Grizzly Bears']:
    renamed=dict(c,name=name,oracle_id='00000000-0000-0000-0000-000000003080',oracle_text=c['oracle_text'].replace(c['name'],name));other,ok=self.compile(renamed);self.assertTrue(ok);self.assertEqual(g.extract_combat_restriction(renamed)[1],original_rule)
    self.assertEqual(self.normalize(other,name),self.normalize(rust,c['name']).replace(json.dumps(c['name']),json.dumps(name)).replace(json.dumps(c['oracle_id']),json.dumps(renamed['oracle_id'])))
 def test_untouched_raw_boundaries_and_unsupported_conditions(self):
  for c in self.cards.values():
   for body in [c['oracle_text']+' (You win the game.)',c['oracle_text'].replace("can't attack","can't (Draw seven cards.) attack"),c['oracle_text'].replace("can't attack","can't att(You win the game.)ack"),c['oracle_text']+'\n((Draw seven cards.))',c['oracle_text']+'\nYou win the game.']:
    self.assertFalse(self.compile(dict(c,oracle_text=body))[1],(c['name'],body))
  base=self.cards['Mouser Mark III']
  for condition in ['you control another land','you control another artifact you do not own','you control another artifact. You win the game','you control another artifact and have ten life']:
   self.assertFalse(self.compile(dict(base,oracle_text="This creature can't attack unless "+condition+'.'))[1])
  for typ in ['Instant','Sorcery']:
   self.assertFalse(self.compile(dict(base,type_line=typ,oracle_text="This creature can't attack unless you control another artifact.\nDraw a card."))[1])
  c=self.cards['The Lion-Turtle'];self.assertFalse(self.compile(dict(c,name='Unrelated Name'))[1])
 def test_generic_self_and_aura_routes_all_keep_full_semantics(self):
  base=self.cards['Tiger-Dillo']
  for body in ["This creature can't attack.","This creature can't attack or block.","This creature can't attack unless its power is 6 or greater.","This creature can't attack or block unless there are three or more Lesson cards in your graveyard."]:
   c=dict(base,name='Generic Combat Fixture',oracle_text=body);self.assertTrue(self.compile(c)[1])
   for bad in [body+' (You win the game.)',body.replace("can't","ca(You win the game.)n't"),body+' ((Draw seven cards.))']:self.assertFalse(self.compile(dict(c,oracle_text=bad))[1])
  aura=dict(self.cards['Pacifism'],oracle_text="Flash\nEnchant creature\nEnchanted creature can't attack or block.");self.assertTrue(self.compile(aura)[1])
  for name in ['Cooped Up','Path to Redemption','Uneasy Alliance','Petrify','Sab-Sunen, Luxa Embodied','Patchwork Beastie']:
   c=next(c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name']==name);self.assertFalse(self.compile(c)[1],name)
 def test_exact_five_preserve_every_old_row_and_helper(self):
  from scripts.catalogue_preservation import canonical,assert_union
  old=subprocess.check_output(['git','-C',str(g.ROOT),'show','b39084973f5e4d1d347f99322080c315df2fc7bc:src/generated/catalogue.rs'],text=True);physical={c['oracle_id']:c['name'] for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']};before=canonical(old,physical);delivered=subprocess.check_output(['git','-C',str(g.ROOT),'show','5d103e7a3172e4742687d3f77f3ed814264b1532:src/generated/catalogue.rs'],text=True);after=canonical(delivered,physical);helpers={('combat_restriction',n+'#0') for n in NAMES}|{('aura','Pacifism#0')};report=assert_union(before,after,changed={n+'#0' for n in NAMES},added=set(),helpers=helpers,aliases={});self.assertEqual(report['preserved_definitions'],5326)
  current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  for name in NAMES:self.assertEqual(current['definitions'][name+'#0'],after['definitions'][name+'#0'])
  for key in helpers:self.assertEqual(current['helpers'][key],after['helpers'][key])
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed={n+'#0' for n in NAMES},added=set(),helpers=helpers,aliases={})
 def test_pinned_full_records_and_explicit_imports(self):
  m=json.loads((g.ROOT/'docs/combat-restriction-admissions.json').read_text());self.assertEqual({c['name'] for c in m['cards']},NAMES)
  for row in m['cards']:
   for k in ['name','oracle_id','mana_cost','type_line','oracle_text']:self.assertEqual(row[k],self.cards[row['name']][k])
   for rec in row['sources']:
    b=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',m['java_revision']+':'+rec['java_source']]);self.assertEqual(hashlib.sha256(b).hexdigest(),rec['java_sha256'])
if __name__=='__main__':unittest.main()
