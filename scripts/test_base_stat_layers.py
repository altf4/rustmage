import copy,hashlib,json,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
from scripts.catalogue_preservation import canonical,assert_union
NAMES={'Doc Ock, Sinister Scientist','Mirkwood Meditator','Multiply by Zero','Perfected Theory','Quandrix Charm','Snowmelt Stag','Turtle-Duck','Water Wings','Gateway Sneak','Snooping Page'}
PARENT='5d103e7a3172e4742687d3f77f3ed814264b1532'
DELIVERED='4b309a0'
COMPOSED_PARENT='c0c25197ce1fce2b8973c538455d49e2ffa46b23'
class BaseStatTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.all=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'];cls.cards={c['name']:c for c in cls.all if c['name'] in NAMES}
 def compile(self,c):
  digest=hashlib.sha256(json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();rust,coverage=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=digest),dict(banned_card_names=[]));row=next(x for x in coverage['cards'] if x['name']==c['name']);return rust,row['implemented']
 def test_complete_programs_and_every_reserved_name_are_semantic(self):
  for c in self.cards.values():
   original,ok=self.compile(c);self.assertTrue(ok,c['name']);before=canonical(original,{c['oracle_id']:c['name']})
   for name in ['Grizzly Bears','Lightning Bolt','Pacifism','Generic Base Fixture']:
    raw=c['oracle_text'].replace(c['name'],name)
    if c['type_line'].startswith('Legendary '):raw=raw.replace(c['name'].split(',')[0],name)
    x=dict(c,name=name,oracle_id='00000000-0000-0000-0000-000000000320',oracle_text=raw);rust,ok=self.compile(x);self.assertTrue(ok,(c['name'],name));after=canonical(rust,{x['oracle_id']:name})
    expected=before['definitions'][c['name']+'#0'].replace(json.dumps(c['name']),json.dumps(name)).replace(c['oracle_id'],x['oracle_id']);self.assertEqual(after['definitions'][name+'#0'],expected)
    a={fn:body for (fn,key),body in before['helpers'].items() if key==c['name']+'#0'};b={fn:body for (fn,key),body in after['helpers'].items() if key==name+'#0'};self.assertEqual(a,b)
 def test_generic_routes_authenticate_all_erasure_stages(self):
  statements=[('Instant','Target creature has base power and toughness 4/5 until end of turn.'),('Instant','Target creature has base toughness 7 until end of turn.'),('Instant','Target creature has base power -1 until end of turn.'),('Creature — Human',"{2}: Until end of turn, this creature has base power 4 and gains trample."),('Creature — Human','When this creature enters, target creature has base power and toughness 1/1 until end of turn.'),('Creature — Human',"Landfall — Whenever a land you control enters, you may have this creature's base power and toughness become 4/2 until end of turn."),('Creature — Human','During your turn, this creature has base power and toughness 5/2.'),('Creature — Human','As long as there are eight or more cards in your graveyard, this creature has base power and toughness 8/8.'),('Creature — Human',"{2}: This creature can't be blocked this turn."),('Artifact','{2}, {T}: Target creature has base power and toughness 4/4 until end of turn.')]
  for typ,text in statements:
   c=dict(name='Generic Base Fixture',layout='normal',oracle_id='00000000-0000-0000-0000-000000000321',type_line=typ,mana_cost='{2}{U}',power='2',toughness='3',colors=['U'],keywords=[],oracle_text=text);self.assertTrue(self.compile(c)[1],text)
   variants=[text+' (You win the game.)',text+'\n(Draw seven cards.)',text.replace('base','ba(Draw seven cards.)se',1),text.replace('creature','crea(You win the game.)ture',1),text+' ((You win the game.))',text+' Draw seven cards. You win the game.']
   for raw in variants:
    if raw!=text:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],(typ,raw))
 def test_contextual_reminders_cannot_be_stolen_or_extended(self):
  wing=self.cards['Water Wings'];bare=wing['oracle_text'].split(' (')[0];a,ok=self.compile(wing);self.assertTrue(ok);b,ok=self.compile(dict(wing,oracle_text=bare));self.assertTrue(ok);self.assertEqual(canonical(a,{wing['oracle_id']:wing['name']}),canonical(b,{wing['oracle_id']:wing['name']}))
  reminder="(It can't be the target of spells or abilities your opponents control.)"
  for raw in [bare.replace(' and hexproof','')+' '+reminder,bare+' '+reminder+' (Draw seven cards.)',bare+' ((Draw seven cards.))',bare.replace('creature','creature '+reminder,1)]:self.assertFalse(self.compile(dict(wing,oracle_text=raw))[1],raw)
  doc=self.cards['Doc Ock, Sinister Scientist']
  for raw in [doc['oracle_text'].replace('has hexproof','has flying'),doc['oracle_text']+' (Draw seven cards.)',doc['oracle_text'].replace('He can\'t','You can\'t')]:self.assertFalse(self.compile(dict(doc,oracle_text=raw))[1])
 def test_unknown_cost_shape_and_remaining_whole_family_stay_gated(self):
  for c in self.cards.values():
   for suffix in ['\nYou win the game.','\n(Draw seven cards.)']:self.assertFalse(self.compile(dict(c,oracle_text=c['oracle_text']+suffix))[1],c['name'])
   for layout in ['transform','modal_dfc','split']:self.assertFalse(self.compile(dict(c,layout=layout))[1],(c['name'],layout))
  for c in self.all:
   if c['name'] in ['Vincent\'s Limit Break','Eaten by Piranhas','Witness Protection','Atomic Microsizer','Archon of the Wild Rose','Unable to Scream','Aettir and Priwen']:self.assertFalse(self.compile(c)[1],c['name'])
  card=dict(self.cards['Multiply by Zero'])
  for value in [str(2**127),str(-(2**127)-1)]:self.assertFalse(self.compile(dict(card,oracle_text='Target creature has base power '+value+' until end of turn.'))[1])
 def test_historical_ten_body_preservation_and_current_programs(self):
  physical={c['oracle_id']:c['name'] for c in self.all};old=subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True);before=canonical(old,physical);after=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',DELIVERED+':src/generated/catalogue.rs'],text=True),physical);current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  helpers={('base_stat_setter',n+'#0') for n in ['Doc Ock, Sinister Scientist','Snowmelt Stag']}|{('keyword_grants','Doc Ock, Sinister Scientist#0')};proof=assert_union(before,after,changed={n+'#0' for n in NAMES},added=set(),helpers=helpers,aliases={});self.assertEqual(proof['preserved_definitions'],5321);self.assertEqual(proof['prior_helpers'],888)
  for name in NAMES:self.assertEqual(current['definitions'][name+'#0'],after['definitions'][name+'#0'])
  for helper in helpers:self.assertEqual(current['helpers'][helper],after['helpers'][helper])
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed={n+'#0' for n in NAMES},added=set(),helpers=helpers,aliases={})
 def test_actual_composed_parent_changes_only_ten_bodies(self):
  physical={c['oracle_id']:c['name'] for c in self.all};before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',COMPOSED_PARENT+':src/generated/catalogue.rs'],text=True),physical);after=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  helpers={('base_stat_setter',n+'#0') for n in ['Doc Ock, Sinister Scientist','Snowmelt Stag']}|{('keyword_grants','Doc Ock, Sinister Scientist#0')};proof=assert_union(before,after,changed={n+'#0' for n in NAMES},added=set(),helpers=helpers,aliases={});self.assertEqual(proof['preserved_definitions'],5322)
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed={n+'#0' for n in NAMES},added=set(),helpers=helpers,aliases={})
 def test_full_family_metadata_and_complete_pinned_direct_imports(self):
  manifest=json.loads((g.ROOT/'docs/base-stat-admissions.json').read_text());self.assertEqual(len(manifest['cards']),56);self.assertEqual(set(manifest['initial_complete_consumers']),NAMES);allcards={c['name']:c for c in self.all};seen={};records=0
  self.assertEqual(manifest['missing_sources'],["Galion, Elvenking's Butler"])
  for row in manifest['cards']+manifest['auxiliary_complete_consumers']:
   for key,value in allcards[row['name']].items():self.assertEqual(row[key],value,(row['name'],key))
   if 'java_source'not in row:self.assertEqual(row['name'],"Galion, Elvenking's Butler");continue
   for rec in [row,*row['direct_import_sources']]:
    records+=1;path=rec['java_source'];sha=rec['java_sha256']
    if path in seen:self.assertEqual(seen[path],sha);continue
    blob=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',manifest['java_revision']+':'+path]);self.assertEqual(hashlib.sha256(blob).hexdigest(),sha);self.assertEqual((Path('/home/altf4/Code/mage')/path).read_bytes(),blob);seen[path]=sha
  self.assertEqual(len(seen),242);self.assertEqual(records,57+824);self.assertFalse(manifest['unresolved_explicit_internal_imports'])
if __name__=='__main__':unittest.main()
