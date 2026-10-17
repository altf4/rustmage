import copy,hashlib,json,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
NAMES={'Aetherize',"River's Rebuke"}
class BattlefieldReturnTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name'] in NAMES}
 def compile(self,c):
  payload=json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode();rust,cov=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=hashlib.sha256(payload).hexdigest()),dict(banned_card_names=[]));return rust,next(row['implemented'] for row in cov['cards'] if row['name']==c['name'])
 def definition(self,rust,name):
  rows=[line for line in rust.splitlines() if line.lstrip().startswith('Definition {')];names=[json.loads(re.search(r'name: ("(?:[^"\\]|\\.)*")',row)[1]) for row in rows];row=next(row for row in rows if 'name: '+json.dumps(name)+',' in row);return re.sub(r'Card\((\d+)\)',lambda m:'Card('+json.dumps(names[int(m[1])])+')',row)
 def test_whole_programs_and_reserved_names(self):
  for c in self.cards.values():
   original,ok=self.compile(c);self.assertTrue(ok)
   for name in ['Reserved Return Fixture','Lightning Bolt','Grizzly Bears','Pacifism']:
    renamed=dict(c,name=name,oracle_id='00000000-0000-0000-0000-000000003060');rust,ok=self.compile(renamed);self.assertTrue(ok)
    expected=self.definition(original,c['name']).replace(json.dumps(c['name']),json.dumps(name)).replace(json.dumps(c['oracle_id']),json.dumps(renamed['oracle_id']))
    self.assertEqual(self.definition(rust,name),expected)
 def test_all_new_generic_routes_authenticate_raw_rules(self):
  forms=["Return all attacking creatures to their owner's hand.","Return all creatures to their owners' hands.","Return all nonland permanents target player controls to their owner's hand."]
  wrappers=[('Instant','{2}{U}',lambda t:t),('Creature — Human','{2}{U}',lambda t:'When this creature enters, '+t[0].lower()+t[1:]),('Artifact','{2}',lambda t:'{2}, {T}: '+t),('Enchantment','{2}{U}',lambda t:'When this enchantment enters, '+t[0].lower()+t[1:])]
  for body in forms:
   for typ,cost,wrap in wrappers:
    card=dict(name='Generic Return Fixture',oracle_id='00000000-0000-0000-0000-000000003061',layout='normal',mana_cost=cost,type_line=typ,oracle_text=wrap(body),power='2',toughness='2',colors=['U'],keywords=[])
    self.assertTrue(self.compile(card)[1],(typ,body))
    for mutation in [body+' (You win the game.)',body.replace('Return','Return (Draw seven cards.)'),body.replace('creatures','crea(You win the game.)tures'),body+' ((You win the game.))',body+'\n(Draw seven cards.)']:
     if mutation==body:continue
     self.assertFalse(self.compile(dict(card,oracle_text=wrap(mutation)))[1],(typ,mutation))
 def test_incomplete_other_consumers_stay_gated(self):
  all_cards=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']
  for c in all_cards:
   if c['name'] in ['Faerie Slumber Party','Season of Weaving','Turtles in Time','Summon: Leviathan','The Eagles Are Coming!']:self.assertFalse(self.compile(c)[1],c['name'])
  for c in self.cards.values():
   for suffix in [' Draw seven cards. You win the game.','\nYou win the game.']:self.assertFalse(self.compile(dict(c,oracle_text=c['oracle_text']+suffix))[1])
 def test_historical_return_delivery_and_current_complete_programs(self):
  from scripts.catalogue_preservation import canonical,assert_union
  old=subprocess.check_output(['git','-C',str(g.ROOT),'show','c0232954838cf71203a5be7c0e8fc80a66f6bf26:src/generated/catalogue.rs'],text=True);physical={c['oracle_id']:c['name'] for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']};before=canonical(old,physical);after=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show','8087b8b1cc6f7f43e5c87eeec5e2e2f83adbc4f9:src/generated/catalogue.rs'],text=True),physical);report=assert_union(before,after,changed={n+'#0' for n in NAMES},added=set(),helpers=set(),aliases={});self.assertEqual(report['preserved_definitions'],5329)
  current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  for name in NAMES:self.assertEqual(current['definitions'][name+'#0'],after['definitions'][name+'#0'])
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed={n+'#0' for n in NAMES},added=set(),helpers=set(),aliases={})
 def test_full_pinned_sources_and_direct_imports(self):
  m=json.loads((g.ROOT/'docs/battlefield-return-admissions.json').read_text());self.assertEqual({c['name'] for c in m['cards']},NAMES)
  for row in m['cards']:
   for k in ['name','oracle_id','mana_cost','type_line','oracle_text']:self.assertEqual(row[k],self.cards[row['name']][k])
   for rec in row['sources']:
    blob=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',m['java_revision']+':'+rec['java_source']]);self.assertEqual(hashlib.sha256(blob).hexdigest(),rec['java_sha256'])
if __name__=='__main__':unittest.main()
