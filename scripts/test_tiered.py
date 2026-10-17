"""Full physical Tiered inventory; never an execution whitelist."""
import copy,hashlib,json,subprocess,unittest
from scripts.catalogue_preservation import canonical,assert_union
from scripts import generate_catalogue as g
class TieredTests(unittest.TestCase):
 def test_six_physical_bodies_all_eighteen_modes_and_full_java_are_pinned(self):
  manifest=json.loads((g.ROOT/'docs/tiered-admissions.json').read_text());cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']if 'Tiered' in c.get('oracle_text','')}
  self.assertEqual(len(cards),6);self.assertEqual({r['facts']['name']for r in manifest['cards']},set(cards));self.assertEqual(sum(len(r['tiers'])for r in manifest['cards']),18)
  records=[]
  for row in manifest['cards']:
   card=cards[row['facts']['name']];self.assertEqual(row['facts'],{k:card.get(k)for k in row['facts']});self.assertEqual(card['layout'],'normal');self.assertFalse(card.get('card_faces'));self.assertEqual(len(row['tiers']),3);self.assertEqual(card['oracle_text'].splitlines()[0],'Tiered (Choose one additional cost.)');records.extend(row['sources'])
   for tier in row['tiers']:self.assertIn('• '+tier['label']+' — '+tier['additional_cost']+' — '+tier['body'],card['oracle_text'])
  records.extend(manifest['shared_sources']);self.assertEqual(len(records),140);self.assertEqual(manifest['tiered_source_records'],140)
  self.assertEqual(len({r['java_source'] for r in records}),87)
  for row in manifest['adjacent_complete_bodies']:
   card=next(c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name']==row['facts']['name']);self.assertEqual(card,row['facts']);records.extend(row['sources'])
  self.assertEqual(len(records),manifest['source_records'])
  unique={r['java_source']:r['java_sha256']for r in records};self.assertEqual(len(unique),len(manifest['unique_sources']));self.assertEqual(unique,{r['java_source']:r['java_sha256']for r in manifest['unique_sources']})
  for path,sha in unique.items():
   raw=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',manifest['java_revision']+':'+path]);self.assertEqual(hashlib.sha256(raw).hexdigest(),sha)

NAMES={'Fire Magic','Ice Magic','Restoration Magic','Thunder Magic',"Tifa's Limit Break","Vincent's Limit Break",'Boros Charm','Epic Fight'}
PARENT='17d100bc39114551fdd677b215535622b2a11393'
COMPOSED_PARENT='ba3752a456908e0bee67a86e88864b14ba1312df'
DELIVERED='8b80e47'
class TieredCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.all=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'];cls.cards={c['name']:c for c in cls.all if c['name']in NAMES}
 def compile(self,c):
  sha=hashlib.sha256(json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();rust,coverage=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=sha),dict(banned_card_names=[]));row=next(x for x in coverage['cards']if x['name']==c['name']);return rust,row['implemented']
 def test_all_eight_bodies_and_every_mode_are_semantic(self):
  for c in self.cards.values():
   rust,ok=self.compile(c);self.assertTrue(ok,c['name']);old=canonical(rust,{c['oracle_id']:c['name']})
   for name in ['Generic Tiered Fixture','Grizzly Bears','Lightning Bolt','Pacifism']:
    x=dict(c,name=name,oracle_id='00000000-0000-0000-0000-000000000329',oracle_text=c['oracle_text'].replace(c['name'],name));newrust,ok=self.compile(x);self.assertTrue(ok,(c['name'],name));new=canonical(newrust,{x['oracle_id']:name});expected=old['definitions'][c['name']+'#0'].replace(c['name'],name).replace(c['oracle_id'],x['oracle_id']);self.assertEqual(expected,new['definitions'][name+'#0']);self.assertEqual({fn:body for(fn,key),body in old['helpers'].items()if key==c['name']+'#0'},{fn:body for(fn,key),body in new['helpers'].items()if key==name+'#0'})
   if c['name']not in ['Boros Charm','Epic Fight']:
    parsed=g.tiered_spell_template(c);self.assertIsNotNone(parsed);self.assertEqual(len(parsed[0]),3);self.assertEqual(len(parsed[1]),3)
    bare=dict(c,oracle_text=c['oracle_text'].replace('Tiered (Choose one additional cost.)','Tiered'));rust2,ok=self.compile(bare);self.assertTrue(ok);a=canonical(rust2,{c['oracle_id']:c['name']});self.assertEqual(a,old)
 def test_all_new_raw_routes_reject_unrecognized_infix_nested_and_borrowed_reminders(self):
  for c in self.cards.values():
   text=c['oracle_text'];variants=[text+' (You win the game.)',text+'\n((Draw seven cards.))',text.replace('creature','crea(You win the game.)ture',1),text.replace('permanent','perma(Draw seven cards.)nent',1),text+'\nYou win the game.',text.replace(' — ',' (You win the game.) — ',1)]
   if text.startswith('Tiered'):
    variants += [text.replace('Choose one additional cost.','Choose one additional cost. You win the game.'),text.replace('{0}','{X}',1),text.replace('{0}','{B/P}',1),text.replace('Tiered','Ti(You win the game.)ered',1),text.replace('Tiered','Tiered (You win the game.)',1),text.replace('• ','Unknown instruction\n• ',1),text.replace('Tiered','Flying (This creature can\'t be blocked except by creatures with flying or reach.)\nTiered',1)]
   for raw in variants:
    if raw!=text:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],(c['name'],raw))
   for layout in ['transform','split','modal_dfc']:self.assertFalse(self.compile(dict(c,layout=layout))[1])
   for cost in ['{Z}','{999999999999999999999999999999}','{X}']:self.assertFalse(self.compile(dict(c,mana_cost=cost))[1],(c['name'],cost))
  generic=["Double target creature's power and toughness until end of turn.","Triple target creature's power and toughness until end of turn.","Target creature's owner shuffles it into their library.",'Target permanent gains hexproof and indestructible until end of turn.','Permanents you control gain indestructible until end of turn.']
  for text in generic:
   c=dict(self.cards['Thunder Magic'],name='Independent generic fixture',oracle_text=text);self.assertTrue(self.compile(c)[1],text)
   for raw in [text+' (You win the game.)',text+' ((You win the game.))',text.replace('target','tar(Draw seven cards.)get',1),text.replace('permanents','perma(You win the game.)nents',1),text+'\n(Choose one additional cost.)']:
    if raw!=text:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],raw)
 def test_tiered_requires_complete_every_branch_and_common_clause(self):
  for c in self.cards.values():
   if not c['oracle_text'].startswith('Tiered'):continue
   for index,line in enumerate(c['oracle_text'].splitlines()):
    if line.startswith('• '):
     lines=c['oracle_text'].splitlines();lines[index]=line+' You win the game.';self.assertFalse(self.compile(dict(c,oracle_text='\n'.join(lines)))[1],(c['name'],index))
  v=self.cards["Vincent's Limit Break"]
  for old,new in [('tapped','untapped'),("owner's","controller's"),('end of turn','your next turn'),('you control','an opponent controls'),('3/2.','3/2. Draw a card.'),('3/2.',str(2**127)+'/2.')]:self.assertFalse(self.compile(dict(v,oracle_text=v['oracle_text'].replace(old,new,1)))[1],(old,new))
 def test_authenticated_gift_body_preserved_without_borrowing_its_reminder(self):
  c=next(c for c in self.all if c['name']=="Dawn's Truce");self.assertTrue(self.compile(c)[1])
  for raw in [c['oracle_text']+' (You win the game.)',c['oracle_text'].replace('they draw a card','they draw seven cards'),c['oracle_text'].replace('hexproof','hex(You win the game.)proof',1)]:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],raw)
 def test_historical_eight_body_union_and_current_owned_programs(self):
  physical={c['oracle_id']:c['name']for c in self.all};before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True),physical);after=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',DELIVERED+':src/generated/catalogue.rs'],text=True),physical);current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  for name in NAMES:self.assertEqual(current['definitions'][name+'#0'],after['definitions'][name+'#0'])
  helpers={('tiered_costs',n+'#0')for n in NAMES if n not in ['Boros Charm','Epic Fight']};proof=assert_union(before,after,changed={n+'#0'for n in NAMES},added=set(),helpers=helpers,aliases={});self.assertEqual(proof['preserved_definitions'],5324)
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed={n+'#0'for n in NAMES},added=set(),helpers=helpers,aliases={})
 def test_actual_schema98_parent_has_exactly_eight_complete_body_changes(self):
  physical={c['oracle_id']:c['name']for c in self.all};before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',COMPOSED_PARENT+':src/generated/catalogue.rs'],text=True),physical);after=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show','61a7d0cef4b10591958cea4391c553db4c2c07ff:src/generated/catalogue.rs'],text=True),physical);current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical);helpers={('tiered_costs',n+'#0')for n in NAMES if n not in ['Boros Charm','Epic Fight']};proof=assert_union(before,after,changed={n+'#0'for n in NAMES},added=set(),helpers=helpers,aliases={});self.assertEqual(proof['preserved_definitions'],5324);self.assertEqual(proof['prior_helpers'],912)
  for name in NAMES:self.assertEqual(current['definitions'][name+'#0'],after['definitions'][name+'#0'])
  for helper in helpers:self.assertEqual(current['helpers'][helper],after['helpers'][helper])
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed={n+'#0'for n in NAMES},added=set(),helpers=helpers,aliases={})
if __name__=='__main__':unittest.main()

