"""Complete three-body resolved-stat lowering; audit files never grant admission."""
import copy,hashlib,json,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
from scripts.catalogue_preservation import canonical,assert_union
NAMES={'Genemorph Imago','Moon Girl and Devil Dinosaur','Belligerent Yearling'}
PARENT='4b309a0'
class ResolvedBaseTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.all=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'];cls.cards={c['name']:c for c in cls.all if c['name']in NAMES}
 def compile(self,c):
  sha=hashlib.sha256(json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();r,v=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=sha),dict(banned_card_names=[]));return r,next(x['implemented']for x in v['cards']if x['name']==c['name'])
 def test_whole_bodies_are_name_independent_and_each_ability_is_present(self):
  for c in self.cards.values():
   rust,ok=self.compile(c);self.assertTrue(ok,c['name'])
   for alias in [c['name'],'Generic Resolved Fixture','Shock','Forest','Grizzly Bears']:
    renamed=dict(c,name=alias,oracle_text=c['oracle_text'].replace(c['name'],alias));r,ok=self.compile(renamed);self.assertTrue(ok,(c['name'],alias))
    self.assertIn('SetResolvedBaseStats'if c['name']!='Moon Girl and Devil Dinosaur'else'SetBaseStats',r)
   if c['name']=='Moon Girl and Devil Dinosaur':
    self.assertIn('ordinal: Some(2)',rust);self.assertIn('OncePerTurn(&TriggerCondition::ArtifactEntersYou)',rust);self.assertIn('GrantKeywords { subject: Subject::Source, keywords: Keywords(256) }',rust)
   if c['name']=='Belligerent Yearling':self.assertIn('May { skip: 1 }',rust);self.assertIn('ResolvedStatValue::EnteringPower',rust)
   if c['name']=='Genemorph Imago':self.assertIn('minimum: 6, yes: 6, no: 3',rust)
 def test_raw_reminders_costs_and_incomplete_bodies_do_not_supply_rules(self):
  for c in self.cards.values():
   raw=c['oracle_text'];mutants=[raw+' (You win the game.)',raw+'\n((Draw seven cards.))',raw.replace('base','ba(Draw seven cards.)se',1),raw.replace('creature',"crea(This creature can't be blocked.)ture",1),raw+'\nDraw seven cards. You win the game.',raw+'\nYou may put a land from your hand onto the battlefield.']
   for alias in ['Generic Resolved Fixture','Shock',c['name']]:
    for text in mutants:
     if text!=raw:self.assertFalse(self.compile(dict(c,name=alias,oracle_text=text.replace(c['name'],alias)))[1],(c['name'],alias,text))
   for cost in ['{X}{U}','{1}{Q}','{4294967296}{U}']:self.assertFalse(self.compile(dict(c,mana_cost=cost))[1],(c['name'],cost))
   for layout in ['split','transform','modal_dfc']:self.assertFalse(self.compile(dict(c,layout=layout))[1],(c['name'],layout))
  c=self.cards['Genemorph Imago']
  for raw in [c['oracle_text'].replace('six','many'),c['oracle_text'].replace('6/6',str(2**127)+'/6')]:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],raw)
  c=self.cards['Belligerent Yearling']
  for raw in [c['oracle_text'].replace("that creature's power","that creature's toughness"),c['oracle_text'].replace('until end of turn','until your next turn')]:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],raw)
 def test_exact_three_against_prototype_foundation_preserve_all_other_definitions(self):
  physical={c['oracle_id']:c['name']for c in self.all};a=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True),physical);b=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  proof=assert_union(a,b,changed={n+'#0'for n in NAMES},added=set(),helpers=set(),aliases={});self.assertEqual(proof['preserved_definitions'],5328)
  mutant=copy.deepcopy(b);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(a,mutant,changed={n+'#0'for n in NAMES},added=set(),helpers=set(),aliases={})
 def test_complete_pinned_java_and_direct_imports_are_audit_only(self):
  audit=json.loads((g.ROOT/'docs/resolved-base-stat-admissions.json').read_text());self.assertEqual({r['name']for r in audit['cards']},NAMES);java=Path('/home/altf4/Code/mage');facts={c['name']:c for c in self.all};seen=set();records=0
  for row in audit['cards']:
   self.assertEqual(row['oracle_id'],facts[row['name']]['oracle_id'])
   for k,v in row['facts'].items():self.assertEqual(v,facts[row['name']][k])
   for source in row['sources']:
    records+=1;p=source['java_source'];blob=(java/p).read_bytes();self.assertEqual(hashlib.sha256(blob).hexdigest(),source['java_sha256']);self.assertEqual(blob,subprocess.check_output(['git','-C',str(java),'show',audit['java_revision']+':'+p]));seen.add(p)
  self.assertEqual((records,len(seen)),(107,89))
if __name__=='__main__':unittest.main()
