"""Pinned complete source inventory for live static layer effects (issue331)."""
import hashlib,json,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g

class LiveBaseStatTests(unittest.TestCase):
 def test_four_complete_metadata_records_and_all_direct_import_sources(self):
  audit=json.loads((g.ROOT/'docs/live-base-stat-admissions.json').read_text());cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']};expected={'Maha, Its Feathers Night','Archon of the Wild Rose','Aettir and Priwen','Burden of Proof'}
  self.assertEqual(set(audit['proposed_complete']),expected);self.assertEqual({r['name'] for r in audit['cards']},expected);records=0;seen={}
  for row in audit['cards']:
   self.assertEqual(row['facts'],cards[row['name']])
   for rec in [row,*row['direct_import_sources']]:
    records+=1;p=rec['java_source'];h=rec['java_sha256'];self.assertEqual(seen.setdefault(p,h),h)
  self.assertEqual(records,60);self.assertEqual(audit['source_records'],records);self.assertEqual(audit['unique_source_count'],len(seen));self.assertEqual({r['java_source']:r['java_sha256'] for r in audit['unique_sources']},seen)
  for p,h in seen.items():
   blob=subprocess.check_output(['git','-C','/home/altf4/Code/mage','show',audit['java_revision']+':'+p]);self.assertEqual(hashlib.sha256(blob).hexdigest(),h);self.assertEqual((Path('/home/altf4/Code/mage')/p).read_bytes(),blob)
  self.assertFalse(audit['unresolved_explicit_internal_imports'])


class LiveBaseCompilerTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.all=json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'];cls.names={'Maha, Its Feathers Night','Archon of the Wild Rose','Aettir and Priwen','Burden of Proof'};cls.cards=[c for c in cls.all if c['name']in cls.names]
 def compile(self,c):
  sha=hashlib.sha256(json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();rust,cov=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=sha),dict(banned_card_names=[]));return rust,next(x['implemented']for x in cov['cards']if x['name']==c['name'])
 def test_all_four_whole_bodies_are_semantic_and_reject_every_unknown_line(self):
  for c in self.cards:
   self.assertTrue(self.compile(c)[1],c['name'])
   for name in ['Generic live source','Grizzly Bears','Pacifism','__XMAGE_MODAL_ACTIVATION_0__']:
    self.assertTrue(self.compile(dict(c,name=name,oracle_text=c['oracle_text'].replace(c['name'],name)))[1],(c['name'],name))
   lines=c['oracle_text'].splitlines()
   for i,line in enumerate(lines):
    for bad in [line+' (You win the game.)',line+' ((Draw seven cards.))','(Lose the game.) '+line,line+'\nYou win the game.']:
     edited=lines[:];edited[i]=bad;self.assertFalse(self.compile(dict(c,oracle_text='\n'.join(edited)))[1],(c['name'],bad))
   for cost in ['{X}','{Z}','{999999999999999999999999999999}']:self.assertFalse(self.compile(dict(c,mana_cost=cost))[1],(c['name'],cost))
   for layout in ['transform','split','modal_dfc']:self.assertFalse(self.compile(dict(c,layout=layout))[1])
 def test_live_clauses_never_discard_partial_bodies_or_change_layer_semantics(self):
  for c in self.cards:
   for word in set(c['oracle_text'].split()):
    if len(word)<3:continue
    raw=c['oracle_text'].replace(word,word[:1]+'(You win the game.)'+word[1:],1);self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],(c['name'],raw))
   for raw in [c['oracle_text']+'\nDraw seven cards.',c['oracle_text']+'\nThis creature has base power and toughness 3/3.']:
    # A supported independent self-setter may compose; an unknown residual may not.
    if 'Draw seven' in raw:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1])
  c=next(c for c in self.cards if c['name']=='Maha, Its Feathers Night')
  for raw in [c['oracle_text'].replace('toughness 1','toughness '+str(2**127)),c['oracle_text']+'\nCreatures you control have base power and toughness 4/4.',c['oracle_text'].replace('toughness 1.','toughness 1 and have changeling.'),c['oracle_text'].replace('toughness 1.','toughness 1 and have flash.')]:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1],raw)
 def test_strict_actual_foundation_four_body_union(self):
  import copy
  from scripts.catalogue_preservation import canonical,assert_union
  physical={c['oracle_id']:c['name']for c in self.all};parent='19e4602b436b7aa8c9708f28f87fe288985c9377';old=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',parent+':src/generated/catalogue.rs'],text=True),physical);new=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show','58e1830ba68a1e72de1862473f46791deca8df9e:src/generated/catalogue.rs'],text=True),physical);current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  helpers={('live_static_effects',n+'#0')for n in self.names}|{('equipment','Aettir and Priwen#0'),('aura','Burden of Proof#0')}
  proof=assert_union(old,new,changed={n+'#0'for n in self.names},added=set(),helpers=helpers,aliases={});self.assertEqual(proof['preserved_definitions'],5328)
  for name in self.names:self.assertEqual(current['definitions'][name+'#0'],new['definitions'][name+'#0'])
  for helper in helpers:self.assertEqual(current['helpers'][helper],new['helpers'][helper])
  mutant=copy.deepcopy(new);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(old,mutant,changed={n+'#0'for n in self.names},added=set(),helpers=helpers,aliases={})

 def test_actual_merged_parent_preserves_every_other_program(self):
  import copy
  from scripts.catalogue_preservation import canonical,assert_union
  parent='ba3752a456908e0bee67a86e88864b14ba1312df';physical={c['oracle_id']:c['name']for c in self.all}
  old=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',parent+':src/generated/catalogue.rs'],text=True),physical);new=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  helpers={('live_static_effects',n+'#0')for n in self.names}|{('equipment','Aettir and Priwen#0'),('aura','Burden of Proof#0')};args=dict(changed={n+'#0'for n in self.names},added=set(),helpers=helpers,aliases={})
  proof=assert_union(old,new,**args);self.assertEqual(proof['preserved_definitions'],5328);self.assertEqual(proof['prior_helpers'],912)
  mutant=copy.deepcopy(new);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(old,mutant,**args)

if __name__=='__main__':unittest.main()
